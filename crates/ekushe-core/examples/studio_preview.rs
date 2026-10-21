//! Run Ekushe Studio on any computer (macOS, Linux, Windows) without the tray app, to work on its
//! design. Uses the real engine, dictionary and Studio page; settings are kept in memory only.
//!
//! ```sh
//! cargo run -p ekushe-core --example studio_preview --features embedded-dict
//! # then open http://127.0.0.1:45219/
//! ```

#[path = "../../ekushe-tray/src/studio_ui.rs"]
mod studio_ui;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use ekushe_core::config::{Config, InputLanguage};
use ekushe_core::conjuncts::search_signs_and_conjuncts;
use ekushe_core::convert::{banglish_to_bangla, bijoy_to_unicode, unicode_to_bijoy};
use ekushe_core::phonetic::PhoneticEngine;
use ekushe_core::Ekushe;

fn main() {
    let port: u16 = std::env::var("EKUSHE_PREVIEW_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(45219);
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind 127.0.0.1");
    println!("Ekushe Studio preview: http://127.0.0.1:{port}/");
    let mut ek = Ekushe::new(Config::default());
    let mut opacity: u64 = 92;
    for stream in listener.incoming().flatten() {
        let _ = handle(stream, &mut ek, &mut opacity);
    }
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
                    if data.len() >= end + 4 + len {
                        break;
                    }
                }
            }
        }
    }
    String::from_utf8_lossy(&data).into_owned()
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
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let h = std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("00");
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
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    s.write_all(body)
}

fn handle(mut s: TcpStream, ek: &mut Ekushe, opacity: &mut u64) -> std::io::Result<()> {
    let req = read_request(&mut s);
    let mut first = req.lines().next().unwrap_or("").split_whitespace();
    let (method, path) = (first.next().unwrap_or(""), first.next().unwrap_or("/"));
    let clean = path.split('?').next().unwrap_or("/");
    let body = req.split_once("\r\n\r\n").map(|x| x.1).unwrap_or("");
    let json = |v: serde_json::Value| v.to_string().into_bytes();
    let ct = "application/json; charset=utf-8";
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    match (method, clean) {
        ("GET", "/") => {
            let html = studio_ui::STUDIO_HTML.replace("__EKUSHE_TOKEN__", "preview");
            respond(
                &mut s,
                "200 OK",
                "text/html; charset=utf-8",
                html.as_bytes(),
            )
        }
        ("GET", "/logo.png") | ("GET", "/favicon.ico") => {
            let b = std::fs::read(root.join("assets/icons/logo_256.png")).unwrap_or_default();
            respond(&mut s, "200 OK", "image/png", &b)
        }
        ("GET", p) if p.starts_with("/fonts/") => {
            let name = p.trim_start_matches("/fonts/").replace("..", "");
            match std::fs::read(root.join("assets/fonts").join(name)) {
                Ok(b) => respond(&mut s, "200 OK", "font/ttf", &b),
                Err(_) => respond(&mut s, "404 Not Found", "text/plain", b"no font"),
            }
        }
        ("GET", "/api/state") => {
            let cfg = ek.config();
            let mut snippets: Vec<_> = ek.suggestions_engine().snippets().iter().collect();
            snippets.sort();
            let v = serde_json::json!({
                "language": if ek.language() == InputLanguage::Bangla { "Bangla" } else { "English" },
                "hotkey_id": "DoubleTapShift",
                "hotkey_display": cfg.switch_hotkey.display_name(),
                "typing_mode": format!("{:?}", cfg.typing_mode),
                "active_layout": format!("{:?}", cfg.active_layout),
                "font_family": cfg.candidate_font_family,
                "popup_opacity": *opacity,
                "compact_popup": cfg.compact_popup,
                "auto_english_in_terminals": cfg.auto_english_in_terminals,
                "snippets": snippets,
            });
            respond(&mut s, "200 OK", ct, &json(v))
        }
        ("POST", "/api/toggle") => {
            let l = ek.toggle_language();
            respond(
                &mut s,
                "200 OK",
                ct,
                &json(serde_json::json!({ "language": format!("{l:?}") })),
            )
        }
        ("GET", "/api/suggest") => {
            let q = query(path, "q");
            let (c, h) =
                ek.suggestions_engine()
                    .suggest(&q, &PhoneticEngine::new(), false, true, 9);
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
            let issues = ek
                .suggestions_engine()
                .spell_checker()
                .check_text(p["text"].as_str().unwrap_or(""));
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
                *opacity = o.clamp(70, 100);
            }
            let mut cfg = ek.config().clone();
            if let Some(f) = p["font_family"].as_str() {
                cfg.candidate_font_family = f.to_string();
            }
            if let Some(c) = p["compact_popup"].as_bool() {
                cfg.compact_popup = c;
            }
            if let Some(t) = p["auto_english_in_terminals"].as_bool() {
                cfg.auto_english_in_terminals = t;
            }
            if let (Some(k), Some(v)) = (
                p["snippet_trigger"].as_str(),
                p["snippet_expansion"].as_str(),
            ) {
                cfg.custom_snippets.insert(k.to_string(), v.to_string());
            }
            ek.set_config(cfg);
            respond(&mut s, "200 OK", ct, b"{\"ok\":true}")
        }
        _ => respond(&mut s, "404 Not Found", "text/plain", b"Not Found"),
    }
}
