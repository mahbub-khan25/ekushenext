//! `ekushe-studio`: start Ekushe Studio (writing pad, conjunct guide, spell check, Bijoy converter,
//! settings) on 127.0.0.1 and open it in the default browser. Quits by itself after 30 minutes
//! without use.
//!
//! `--settings` opens the Settings tab; `--no-browser` only prints the address.

use std::time::Duration;

const IDLE_QUIT_SECS: u64 = 30 * 60;

fn open_in_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(target_os = "macos"))]
    let opener = "xdg-open";
    if std::process::Command::new(opener).arg(url).spawn().is_err() {
        eprintln!("Open this address in your browser: {url}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("ekushe-studio {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let tab = args.iter().any(|a| a == "--settings").then_some("settings");
    // A fresh one-time address for this user (a private page that forwards to Studio)
    let Some(url) = ekushe_ffi::studio::launch_url(tab) else {
        eprintln!("Ekushe Studio: no free port on 127.0.0.1");
        std::process::exit(1);
    };
    if args.iter().any(|a| a == "--no-browser") {
        println!("Ekushe Studio: {url}");
    } else {
        open_in_browser(&url);
    }
    loop {
        std::thread::sleep(Duration::from_secs(30));
        if ekushe_ffi::studio::idle_seconds() > IDLE_QUIT_SECS {
            ekushe_ffi::save_learned_now();
            return;
        }
    }
}
