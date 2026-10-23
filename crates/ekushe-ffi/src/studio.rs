//! Studio (writing pad, conjunct guide, spell check, converter, settings) served on 127.0.0.1 from
//! inside the input-method process, working on the shared engine: settings saved here apply to
//! typing straight away and are written to `config.toml`.
//!
//! Same protections as the Windows tray's Studio: the Host must be loopback, a foreign Origin is
//! refused, every `/api/*` call needs the per-run key, and the page only carries that key when it is
//! opened with a fresh one-time launch code ([`launch_url`]). Requests are bounded in size and time,
//! and a slow client cannot hold the server (see `studio_http.rs`).

#[path = "../../ekushe-tray/src/studio_http.rs"]
mod studio_http;
#[path = "../../ekushe-tray/src/studio_ui.rs"]
mod studio_ui;

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};

use ekushe_core::config::{Config, InputLanguage, SwitchHotkey, TypingMode};
use ekushe_core::conjuncts::search_signs_and_conjuncts;
use ekushe_core::convert::{banglish_to_bangla, bijoy_to_unicode, unicode_to_bijoy};
use ekushe_core::layout::LayoutId;

use crate::SHARED;

static LOGO: &[u8] = include_bytes!("../../../assets/icons/logo_256.png");
static FONTS: &[(&str, &[u8])] = &[
    (
        "Kalpurush.ttf",
        include_bytes!("../../../assets/fonts/Kalpurush.ttf"),
    ),
    (
        "SolaimanLipi.ttf",
        include_bytes!("../../../assets/fonts/SolaimanLipi.ttf"),
    ),
    (
        "SiyamRupali.ttf",
        include_bytes!("../../../assets/fonts/SiyamRupali.ttf"),
    ),
    (
        "HindSiliguri-Regular.ttf",
        include_bytes!("../../../assets/fonts/HindSiliguri-Regular.ttf"),
    ),
    (
        "HindSiliguri-Bold.ttf",
        include_bytes!("../../../assets/fonts/HindSiliguri-Bold.ttf"),
    ),
];

static PORT: AtomicU16 = AtomicU16::new(0);
/// When the last request arrived (seconds since the Unix epoch), for apps that quit when idle
static LAST_REQUEST: AtomicU64 = AtomicU64::new(0);

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Seconds since Studio last served a request (or since it started).
pub fn idle_seconds() -> u64 {
    now_secs().saturating_sub(LAST_REQUEST.load(Ordering::Relaxed))
}
/// Popup transparency setting (kept for the Studio slider; the macOS candidate window is drawn by
/// the system)
static OPACITY: AtomicU64 = AtomicU64::new(92);

fn token() -> &'static str {
    studio_http::token()
}

/// Start the server on a background thread (once). Returns its port, or 0 if no port was free.
pub fn start() -> u16 {
    let p = PORT.load(Ordering::SeqCst);
    if p != 0 {
        return p;
    }
    let Ok(listener) =
        TcpListener::bind(("127.0.0.1", 45210)).or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
    else {
        return 0;
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    LAST_REQUEST.store(now_secs(), Ordering::Relaxed);
    if PORT
        .compare_exchange(0, port, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return PORT.load(Ordering::SeqCst); // another thread won
    }
    std::thread::spawn(move || studio_http::serve(listener, move |s| handle_conn(s, port)));
    port
}

/// Run Studio in the foreground on `port` (for `examples/studio_preview.rs`).
pub fn serve_blocking(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let port = listener.local_addr()?.port();
    PORT.store(port, Ordering::SeqCst);
    println!("Open {}", launch_url(None).unwrap_or_default());
    studio_http::serve(listener, move |s| handle_conn(s, port));
    Ok(())
}

/// An address that opens Studio for the person running this process (`tab`: e.g. `"settings"`).
///
/// On macOS and Linux other accounts can see the command line of a browser, so the one-time launch
/// code is not put there: it goes in a small page in a folder only this user can read, and the
/// returned `file://` address opens that page, which forwards to Studio.
pub fn launch_url(tab: Option<&str>) -> Option<String> {
    let port = start();
    if port == 0 {
        return None;
    }
    let url = studio_http::launch_url(port, tab);
    #[cfg(unix)]
    {
        if let Some(file) = private_launch_page(&url) {
            return Some(file);
        }
    }
    Some(url)
}

#[cfg(unix)]
fn private_launch_page(url: &str) -> Option<String> {
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_dir())
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".cache")))?;
    let dir = base.join("ekushe-studio");
    let _ = std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir);
    let meta = std::fs::symlink_metadata(&dir).ok()?;
    if !meta.is_dir() {
        return None;
    }
    let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    // Old launch pages are useless (their codes expire) — tidy them away
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let old = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.elapsed().ok());
            if old.is_some_and(|d| d.as_secs() > 300) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let path = dir.join(format!("open-{}.html", studio_http::random_hex(8)));
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Ekushe Studio</title>\
         <meta http-equiv=\"refresh\" content=\"0;url={url}\">\
         <p><a href=\"{url}\">Ekushe Studio</a></p>"
    );
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .ok()?;
    f.write_all(html.as_bytes()).ok()?;
    // Percent-encode the path (a home folder can contain spaces)
    let enc: String = path
        .to_string_lossy()
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'-' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    Some(format!("file://{enc}"))
}

fn header<'a>(req: &'a str, name: &str) -> Option<&'a str> {
    let head = &req[..req.find("\r\n\r\n").unwrap_or(req.len())];
    head.lines().skip(1).find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim())
    })
}

fn allowed(req: &str, method: &str, path: &str, port: u16) -> bool {
    let hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    match header(req, "Host") {
        Some(h) if hosts.iter().any(|a| a.eq_ignore_ascii_case(h)) => {}
        _ => return false,
    }
    if let Some(o) = header(req, "Origin") {
        if !hosts
            .iter()
            .any(|a| o.eq_ignore_ascii_case(&format!("http://{a}")))
        {
            return false;
        }
    }
    if path.starts_with("/api/") {
        return header(req, "X-Ekushe-Token") == Some(token());
    }
    method == "GET"
}

fn query(path: &str, key: &str) -> String {
    let q = path.split_once('?').map(|x| x.1).unwrap_or("");
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return percent_decode(v);
            }
        }
    }
    String::new()
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let h = std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("3F");
                out.push(u8::from_str_radix(h, 16).unwrap_or(b'?'));
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn respond(s: &mut TcpStream, status: &str, ctype: &str, body: &[u8]) -> std::io::Result<()> {
    write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    s.write_all(body)
}

fn handle_conn(mut s: TcpStream, port: u16) {
    match studio_http::read_request(&mut s) {
        Ok(req) => {
            let _ = handle(s, &req, port);
        }
        Err(studio_http::ReadError::Status(st)) => studio_http::respond_error(&mut s, st),
        Err(studio_http::ReadError::Closed) => {}
    }
}

fn handle(mut s: TcpStream, req: &str, port: u16) -> std::io::Result<()> {
    let req = req.to_string();
    LAST_REQUEST.store(now_secs(), Ordering::Relaxed);
    let mut first = req.lines().next().unwrap_or("").split_whitespace();
    let (method, path) = (first.next().unwrap_or(""), first.next().unwrap_or("/"));
    let clean = path.split('?').next().unwrap_or("/");
    let body = req.split_once("\r\n\r\n").map(|x| x.1).unwrap_or("");
    if !allowed(&req, method, clean, port) {
        return respond(
            &mut s,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            b"Forbidden",
        );
    }
    let ct = "application/json; charset=utf-8";
    let json = |v: serde_json::Value| v.to_string().into_bytes();

    match (method, clean) {
        ("GET", "/") | ("GET", "/index.html") => {
            let html = studio_http::studio_page(studio_ui::STUDIO_HTML, path);
            respond(
                &mut s,
                "200 OK",
                "text/html; charset=utf-8",
                html.as_bytes(),
            )
        }
        ("GET", "/logo.png") | ("GET", "/favicon.ico") => {
            respond(&mut s, "200 OK", "image/png", LOGO)
        }
        ("GET", p) if p.starts_with("/fonts/") => {
            let name = &p["/fonts/".len()..];
            match FONTS.iter().find(|(n, _)| *n == name) {
                Some((_, bytes)) => respond(&mut s, "200 OK", "font/ttf", bytes),
                None => respond(&mut s, "404 Not Found", "text/plain", b"no such font"),
            }
        }
        ("GET", "/api/state") => {
            let v = {
                let sh = SHARED.lock();
                let cfg = sh.ekushe.config();
                let mut snippets: Vec<_> =
                    sh.ekushe.suggestions_engine().snippets().iter().collect();
                snippets.sort();
                serde_json::json!({
                    "language": if sh.ekushe.language() == InputLanguage::Bangla { "Bangla" } else { "English" },
                    "hotkey_id": match cfg.switch_hotkey {
                        SwitchHotkey::CtrlSpace => "CtrlSpace",
                        SwitchHotkey::F12 => "F12",
                        SwitchHotkey::CtrlAltB => "CtrlAltB",
                        _ => "DoubleTapShift",
                    },
                    "hotkey_display": cfg.switch_hotkey.display_name(),
                    "typing_mode": format!("{:?}", cfg.typing_mode),
                    "active_layout": format!("{:?}", cfg.active_layout),
                    "font_family": cfg.candidate_font_family,
                    "popup_opacity": OPACITY.load(Ordering::Relaxed),
                    "compact_popup": cfg.compact_popup,
                    "auto_english_in_terminals": cfg.auto_english_in_terminals,
                    "snippets": snippets,
                })
            };
            respond(&mut s, "200 OK", ct, &json(v))
        }
        ("POST", "/api/toggle") => {
            let l = SHARED.lock().ekushe.toggle_language();
            respond(
                &mut s,
                "200 OK",
                ct,
                &json(serde_json::json!({ "language": format!("{l:?}") })),
            )
        }
        ("GET", "/api/suggest") => {
            let q = query(path, "q");
            let (c, h) = SHARED.lock().ekushe.suggest_roman(&q);
            respond(
                &mut s,
                "200 OK",
                ct,
                &json(serde_json::json!({ "candidates": c, "sign_hints": h })),
            )
        }
        ("GET", "/api/signs") => {
            let v = serde_json::to_vec(&search_signs_and_conjuncts(&query(path, "q")))
                .unwrap_or_default();
            respond(&mut s, "200 OK", ct, &v)
        }
        ("POST", "/api/spell") => {
            let p: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            // Check a copy, outside the lock, so a long document never stalls typing
            let checker = SHARED
                .lock()
                .ekushe
                .suggestions_engine()
                .spell_checker()
                .clone();
            let issues = checker.check_text(p["text"].as_str().unwrap_or(""));
            respond(
                &mut s,
                "200 OK",
                ct,
                &serde_json::to_vec(&issues).unwrap_or_default(),
            )
        }
        ("POST", "/api/convert") => {
            let p: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let t = p["text"].as_str().unwrap_or("");
            let r = match p["mode"].as_str().unwrap_or("") {
                "uni2bijoy" => unicode_to_bijoy(t),
                "banglish" => banglish_to_bangla(t),
                _ => bijoy_to_unicode(t),
            };
            respond(
                &mut s,
                "200 OK",
                ct,
                &json(serde_json::json!({ "result": r })),
            )
        }
        ("POST", "/api/settings") => {
            let p: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            if let Some(o) = p["popup_opacity"].as_u64() {
                OPACITY.store(o.clamp(70, 100), Ordering::Relaxed);
            }
            let cfg = {
                let mut sh = SHARED.lock();
                let mut cfg = sh.ekushe.config().clone();
                if let Some(h) = p["hotkey"].as_str() {
                    cfg.switch_hotkey = match h {
                        "CtrlSpace" => SwitchHotkey::CtrlSpace,
                        "F12" => SwitchHotkey::F12,
                        "CtrlAltB" => SwitchHotkey::CtrlAltB,
                        _ => SwitchHotkey::DoubleTapShift,
                    };
                }
                if let Some(m) = p["typing_mode"].as_str() {
                    cfg.typing_mode = match m {
                        "PhoneticFirst" => TypingMode::PhoneticFirst,
                        "PhoneticOnly" => TypingMode::PhoneticOnly,
                        _ => TypingMode::Smart,
                    };
                }
                if let Some(l) = p["active_layout"].as_str() {
                    cfg.active_layout = match l {
                        "Probhat" => LayoutId::Probhat,
                        "NationalJatiya" => LayoutId::NationalJatiya,
                        "Bornona" => LayoutId::Bornona,
                        "MunirOptima" => LayoutId::MunirOptima,
                        _ => LayoutId::AvroPhonetic,
                    };
                }
                if let Some(f) = p["font_family"].as_str() {
                    cfg.candidate_font_family = f.to_string();
                }
                if let Some(c) = p["compact_popup"].as_bool() {
                    cfg.compact_popup = c;
                }
                if let Some(t) = p["auto_english_in_terminals"].as_bool() {
                    cfg.auto_english_in_terminals = t;
                }
                let trig = p["snippet_trigger"].as_str().unwrap_or("").trim();
                let exp = p["snippet_expansion"].as_str().unwrap_or("").trim();
                if !trig.is_empty() && !exp.is_empty() {
                    cfg.custom_snippets
                        .insert(trig.to_string(), exp.to_string());
                }
                sh.ekushe.set_config(cfg.clone());
                cfg
            };
            // Written outside the lock
            let ok = cfg.save_to(&Config::default_config_path()).is_ok();
            respond(
                &mut s,
                if ok {
                    "200 OK"
                } else {
                    "500 Internal Server Error"
                },
                ct,
                if ok {
                    b"{\"ok\":true}"
                } else {
                    b"{\"ok\":false}"
                },
            )
        }
        _ => respond(&mut s, "404 Not Found", "text/plain", b"Not Found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn http(port: u16, req: String) -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.write_all(req.as_bytes()).unwrap();
        let mut out = Vec::new();
        s.read_to_end(&mut out).unwrap();
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn studio_serves_page_and_guards_api() {
        let port = start();
        assert_ne!(port, 0);
        // Without a launch code the page has no key; with one it has, once
        let page = http(
            port,
            format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
        );
        assert!(
            page.starts_with("HTTP/1.1 200")
                && page.contains("tab-signs")
                && !page.contains(token())
        );
        let url = studio_http::launch_url(port, Some("signs"));
        let path = &url[url.find("/?").unwrap()..];
        let launch =
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
        assert!(http(port, launch.clone()).contains(token()));
        assert!(
            !http(port, launch).contains(token()),
            "a launch code works once"
        );
        #[cfg(unix)]
        {
            let file = launch_url(None).unwrap();
            let p = file.strip_prefix("file://").unwrap().replace("%20", " ");
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
            let _ = std::fs::remove_file(&p);
        }
        let no_token = http(
            port,
            format!(
                "GET /api/state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(no_token.starts_with("HTTP/1.1 403"));
        let rebinding = http(
            port,
            format!("GET / HTTP/1.1\r\nHost: evil.example:{port}\r\nConnection: close\r\n\r\n"),
        );
        assert!(rebinding.starts_with("HTTP/1.1 403"));
        let ok = http(
            port,
            format!("GET /api/suggest?q=amar HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Ekushe-Token: {}\r\nConnection: close\r\n\r\n", token()),
        );
        assert!(ok.starts_with("HTTP/1.1 200") && ok.contains("candidates"));
    }
}
