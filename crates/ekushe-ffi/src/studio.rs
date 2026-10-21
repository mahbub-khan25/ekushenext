//! Studio (writing pad, conjunct guide, spell check, converter, settings) served on 127.0.0.1 from
//! inside the input-method process, working on the shared engine: settings saved here apply to
//! typing straight away and are written to `config.toml`.
//!
//! Same protections as the Windows tray's Studio: the Host must be loopback, a foreign Origin is
//! refused, and every `/api/*` call needs the per-run token embedded in the served page.

#[path = "../../ekushe-tray/src/studio_ui.rs"]
mod studio_ui;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::sync::OnceLock;

use ekushe_core::config::{Config, InputLanguage, SwitchHotkey, TypingMode};
use ekushe_core::conjuncts::search_signs_and_conjuncts;
use ekushe_core::convert::{banglish_to_bangla, bijoy_to_unicode, unicode_to_bijoy};
use ekushe_core::layout::LayoutId;
use ekushe_core::phonetic::PhoneticEngine;

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
    static T: OnceLock<String> = OnceLock::new();
    T.get_or_init(|| {
        use std::hash::{BuildHasher, Hasher};
        let mut out = String::new();
        for i in 0..2u64 {
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u64(i);
            h.write_u128(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0),
            );
            out.push_str(&format!("{:016x}", h.finish()));
        }
        out
    })
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
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = handle(stream, port);
        }
    });
    port
}

/// Run Studio in the foreground on `port` (for `examples/studio_preview.rs`).
pub fn serve_blocking(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let port = listener.local_addr()?.port();
    PORT.store(port, Ordering::SeqCst);
    for stream in listener.incoming().flatten() {
        let _ = handle(stream, port);
    }
    Ok(())
}

fn read_request(s: &mut TcpStream) -> String {
    let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(800)));
    let mut data = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        match s.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                if let Some(end) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&data[..end]).to_ascii_lowercase();
                    let len = head
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:"))
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    if data.len() >= end + 4 + len || data.len() > 1 << 20 {
                        break;
                    }
                }
            }
        }
    }
    String::from_utf8_lossy(&data).into_owned()
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

fn handle(mut s: TcpStream, port: u16) -> std::io::Result<()> {
    let req = read_request(&mut s);
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
            let html = studio_ui::STUDIO_HTML.replace("__EKUSHE_TOKEN__", token());
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
            let (c, h) = SHARED.lock().ekushe.suggestions_engine().suggest(
                &q,
                &PhoneticEngine::new(),
                false,
                true,
                9,
            );
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
        let page = http(
            port,
            format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
        );
        assert!(
            page.starts_with("HTTP/1.1 200")
                && page.contains("tab-signs")
                && page.contains(token())
        );
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
