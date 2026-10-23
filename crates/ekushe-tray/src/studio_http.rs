//! The safe parts of Studio's little web server, shared by the Windows app (this crate) and by Studio
//! on macOS and Linux (`crates/ekushe-ffi`, which includes this file).
//!
//! - **Bounded requests**: one request may take at most [`REQUEST_DEADLINE`] in total, its header at
//!   most [`MAX_HEADER`] bytes and its body at most [`MAX_BODY`]; bad or oversized requests get an error
//!   instead of being processed. A slow client cannot hold the server: each connection has its own
//!   thread, and at most [`MAX_CONNECTIONS`] are served at once.
//! - **Only the user who opened Studio gets its key.** The page that carries the API key is served
//!   only for a fresh, single-use launch code, made when Ekushe opens Studio for you; the page keeps
//!   the key in the tab's session storage, so reloading works. Without a code, `GET /` still serves
//!   the page but with no key, and it asks you to open Studio from Ekushe's menu.
//!
//! Only the standard library is used.

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Largest request header accepted
pub const MAX_HEADER: usize = 16 * 1024;
/// Largest request body accepted (text pasted into the converter or spell checker)
pub const MAX_BODY: usize = 1 << 20;
/// Time allowed for a whole request to arrive, however it trickles in
pub const REQUEST_DEADLINE: Duration = Duration::from_secs(5);
/// Time allowed for writing a response
pub const WRITE_DEADLINE: Duration = Duration::from_secs(5);
/// Connections served at the same time; more get "503 busy" straight away
pub const MAX_CONNECTIONS: usize = 8;
/// How long a launch code stays valid
pub const CODE_LIFETIME: Duration = Duration::from_secs(120);

/// Why a request was not read
#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    /// The client closed the connection, or sent nothing before the deadline: answer nothing
    Closed,
    /// Answer with this HTTP status line (e.g. `"413 Payload Too Large"`) and close
    Status(&'static str),
}

/// Read one HTTP request (header and body) within the limits above.
pub fn read_request(s: &mut TcpStream) -> Result<String, ReadError> {
    let start = Instant::now();
    let _ = s.set_write_timeout(Some(WRITE_DEADLINE));
    let mut data: Vec<u8> = Vec::with_capacity(4096);
    let mut buf = [0u8; 8192];
    let mut head_end: Option<usize> = None;
    let mut body_len = 0usize;
    loop {
        let Some(left) = REQUEST_DEADLINE.checked_sub(start.elapsed()) else {
            return Err(if data.is_empty() {
                ReadError::Closed
            } else {
                ReadError::Status("408 Request Timeout")
            });
        };
        let _ = s.set_read_timeout(Some(left.max(Duration::from_millis(1))));
        let before = data.len();
        match s.read(&mut buf) {
            Ok(0) => {
                return Err(if data.is_empty() {
                    ReadError::Closed
                } else {
                    ReadError::Status("400 Bad Request")
                });
            }
            Ok(n) => data.extend_from_slice(&buf[..n]),
            Err(e)
                if matches!(
                    e.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                continue
            }
            Err(_) => return Err(ReadError::Closed),
        }
        if head_end.is_none() {
            // Look for the end of the header only in the new bytes (and the 3 before them)
            let from = before.saturating_sub(3);
            if let Some(p) = data[from..].windows(4).position(|w| w == b"\r\n\r\n") {
                let end = from + p + 4;
                if end > MAX_HEADER {
                    return Err(ReadError::Status("431 Request Header Fields Too Large"));
                }
                body_len = content_length(&data[..end])?;
                head_end = Some(end);
            } else if data.len() > MAX_HEADER {
                return Err(ReadError::Status("431 Request Header Fields Too Large"));
            }
        }
        if let Some(end) = head_end {
            if data.len() >= end + body_len {
                data.truncate(end + body_len);
                return Ok(String::from_utf8_lossy(&data).into_owned());
            }
        }
    }
}

/// The body length a header announces. Chunked bodies, conflicting or malformed lengths, and
/// bodies over [`MAX_BODY`] are refused.
fn content_length(head: &[u8]) -> Result<usize, ReadError> {
    let head = String::from_utf8_lossy(head);
    let mut len: Option<usize> = None;
    for line in head.lines().skip(1) {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let k = k.trim();
        if k.eq_ignore_ascii_case("transfer-encoding") {
            return Err(ReadError::Status("501 Not Implemented"));
        }
        if k.eq_ignore_ascii_case("content-length") {
            let v = v.trim();
            if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ReadError::Status("400 Bad Request"));
            }
            let n: usize = v
                .parse()
                .map_err(|_| ReadError::Status("413 Payload Too Large"))?;
            if len.is_some_and(|l| l != n) {
                return Err(ReadError::Status("400 Bad Request"));
            }
            len = Some(n);
        }
    }
    let n = len.unwrap_or(0);
    if n > MAX_BODY {
        return Err(ReadError::Status("413 Payload Too Large"));
    }
    Ok(n)
}

/// Answer an error status (plain text) and close.
pub fn respond_error(s: &mut TcpStream, status: &str) {
    let body = status.as_bytes();
    let _ = s.set_write_timeout(Some(WRITE_DEADLINE));
    let _ = write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = s.write_all(body);
    // Read (and drop) a little of what the client is still sending, so closing does not reset the
    // connection before the client has read the answer
    let _ = s.shutdown(std::net::Shutdown::Write);
    let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
    let mut sink = [0u8; 8192];
    let mut drained = 0usize;
    while drained < 4 * MAX_HEADER {
        match s.read(&mut sink) {
            Ok(0) | Err(_) => break,
            Ok(n) => drained += n,
        }
    }
}

/// Accept connections forever, each on its own thread, at most [`MAX_CONNECTIONS`] at once.
pub fn serve<F>(listener: TcpListener, handler: F)
where
    F: Fn(TcpStream) + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    let active = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming().flatten() {
        if active.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            active.fetch_sub(1, Ordering::SeqCst);
            let mut s = stream;
            respond_error(&mut s, "503 Service Unavailable");
            continue;
        }
        let (h, a) = (handler.clone(), active.clone());
        let spawned = std::thread::Builder::new()
            .name("ekushe-studio-http".into())
            .spawn(move || {
                struct Done(Arc<AtomicUsize>);
                impl Drop for Done {
                    fn drop(&mut self) {
                        self.0.fetch_sub(1, Ordering::SeqCst);
                    }
                }
                let _done = Done(a);
                h(stream);
            });
        if spawned.is_err() {
            active.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

// ───────────────────────── keys ─────────────────────────

/// `n` random bytes as hex, from the operating system's random source.
pub fn random_hex(n: usize) -> String {
    let mut bytes = vec![0u8; n];
    #[cfg(unix)]
    {
        if std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut bytes))
            .is_err()
        {
            fill_from_std(&mut bytes);
        }
    }
    #[cfg(not(unix))]
    fill_from_std(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The standard library's hasher keys come from the OS random source (BCryptGenRandom / ProcessPrng
/// on Windows); hashing a counter with fresh keys gives unpredictable bytes.
fn fill_from_std(out: &mut [u8]) {
    use std::hash::{BuildHasher, Hasher};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    for chunk in out.chunks_mut(8) {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_usize(COUNTER.fetch_add(1, Ordering::Relaxed));
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        );
        let v = h.finish().to_le_bytes();
        chunk.copy_from_slice(&v[..chunk.len()]);
    }
}

/// The key every `/api/*` request must carry (`X-Ekushe-Token`), random for each run.
pub fn token() -> &'static str {
    static T: OnceLock<String> = OnceLock::new();
    T.get_or_init(|| random_hex(24))
}

fn codes() -> &'static Mutex<Vec<(String, Instant)>> {
    static C: OnceLock<Mutex<Vec<(String, Instant)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(Vec::new()))
}

/// A fresh single-use launch code. Ekushe opens Studio at `/?code=<this>`.
pub fn new_launch_code() -> String {
    let code = random_hex(16);
    let mut c = codes().lock().unwrap_or_else(|e| e.into_inner());
    c.retain(|(_, t)| t.elapsed() < CODE_LIFETIME);
    if c.len() >= 16 {
        c.remove(0);
    }
    c.push((code.clone(), Instant::now()));
    code
}

/// Use up a launch code: true if it was valid and unused.
pub fn take_launch_code(code: &str) -> bool {
    if code.len() != 32 {
        return false;
    }
    let mut c = codes().lock().unwrap_or_else(|e| e.into_inner());
    c.retain(|(_, t)| t.elapsed() < CODE_LIFETIME);
    match c
        .iter()
        .position(|(k, _)| same(k.as_bytes(), code.as_bytes()))
    {
        Some(i) => {
            c.remove(i);
            true
        }
        None => false,
    }
}

/// Compare without stopping at the first difference.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Studio's page for a `GET /` request: with the API key when the query carries a valid launch code,
/// without it otherwise (the page then uses the key it already has in the tab, or asks you to open
/// Studio from Ekushe's menu).
pub fn studio_page(html: &str, path_and_query: &str) -> String {
    let query = path_and_query.split_once('?').map(|x| x.1).unwrap_or("");
    let code = query
        .split('&')
        .find_map(|kv| kv.strip_prefix("code="))
        .unwrap_or("");
    let key = if !code.is_empty() && take_launch_code(code) {
        token()
    } else {
        ""
    };
    html.replace("__EKUSHE_TOKEN__", key)
}

/// The address that opens Studio with a fresh launch code, e.g. `http://127.0.0.1:45210/?code=…&tab=signs`.
pub fn launch_url(port: u16, tab: Option<&str>) -> String {
    let code = new_launch_code();
    match tab {
        Some(t) if !t.is_empty() && t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') => {
            format!("http://127.0.0.1:{port}/?code={code}&tab={t}")
        }
        _ => format!("http://127.0.0.1:{port}/?code={code}"),
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use std::net::TcpListener;

    fn server() -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            serve(l, |mut s| match read_request(&mut s) {
                Ok(req) => {
                    let body = format!("{}", req.len());
                    let _ = write!(
                        s,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                }
                Err(ReadError::Status(st)) => respond_error(&mut s, st),
                Err(ReadError::Closed) => {}
            })
        });
        port
    }

    fn ask(port: u16, raw: &[u8]) -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(8))).unwrap();
        s.write_all(raw).unwrap();
        let mut out = String::new();
        let _ = s.read_to_string(&mut out);
        out
    }

    #[test]
    fn a_slow_client_does_not_block_others() {
        let port = server();
        let mut slow = TcpStream::connect(("127.0.0.1", port)).unwrap();
        slow.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n").unwrap();
        let t = std::thread::spawn(move || {
            for _ in 0..10 {
                std::thread::sleep(Duration::from_millis(200));
                if slow.write_all(b"X").is_err() {
                    break;
                }
            }
        });
        let started = Instant::now();
        let r = ask(port, b"GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 200"), "{r}");
        assert!(started.elapsed() < Duration::from_secs(1));
        t.join().unwrap();
    }

    #[test]
    fn a_trickling_request_is_cut_off_at_the_deadline() {
        let port = server();
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let started = Instant::now();
        let _ = s.write_all(b"GET / HTTP/1.1\r\n");
        let mut out = Vec::new();
        while started.elapsed() < Duration::from_secs(9) {
            if s.write_all(b"X").is_err() {
                break;
            }
            let mut b = [0u8; 256];
            match s.read(&mut b) {
                Ok(0) => break,
                Ok(n) => {
                    out.extend_from_slice(&b[..n]);
                    break;
                }
                Err(_) => {}
            }
        }
        assert!(
            String::from_utf8_lossy(&out).contains("408"),
            "{:?}",
            String::from_utf8_lossy(&out)
        );
        assert!(started.elapsed() < Duration::from_secs(7));
    }

    #[test]
    fn oversized_and_malformed_requests_are_refused() {
        let port = server();
        let big = format!(
            "GET / HTTP/1.1\r\nHost: x\r\nX-Big: {}\r\n\r\n",
            "a".repeat(MAX_HEADER + 10)
        );
        assert!(ask(port, big.as_bytes()).contains("431"));
        let r = ask(
            port,
            format!(
                "POST /api/spell HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
                MAX_BODY + 1
            )
            .as_bytes(),
        );
        assert!(r.contains("413"), "{r}");
        assert!(ask(
            port,
            b"POST / HTTP/1.1\r\nContent-Length: 5\r\nContent-Length: 6\r\n\r\nhello"
        )
        .contains("400"));
        assert!(ask(port, b"POST / HTTP/1.1\r\nContent-Length: -1\r\n\r\n").contains("400"));
        assert!(ask(
            port,
            b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"
        )
        .contains("501"));
        // A complete small request still works, and a body is read exactly
        let r = ask(
            port,
            b"POST /api/spell HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello",
        );
        assert!(r.starts_with("HTTP/1.1 200"), "{r}");
    }

    #[test]
    fn launch_codes_work_once() {
        let html = "const EKUSHE_TOKEN = \"__EKUSHE_TOKEN__\";";
        let url = launch_url(1, Some("signs"));
        let path = &url[url.find("/?").unwrap()..];
        assert!(studio_page(html, path).contains(token()));
        assert!(
            !studio_page(html, path).contains(token()),
            "a code must work only once"
        );
        assert!(!studio_page(html, "/").contains(token()));
        assert!(!studio_page(html, "/?code=00000000000000000000000000000000").contains(token()));
        assert_eq!(token().len(), 48);
        assert_ne!(random_hex(16), random_hex(16));
    }
}
