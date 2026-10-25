//! `ekushe-studio`: start Ekushe Studio (writing pad, conjunct guide, spell check, Bijoy converter,
//! settings) on 127.0.0.1. In a browser tab it quits by itself after 30 minutes without use.
//!
//! On Linux it opens in its own window (`ekushe-studio-window`, GTK + WebKitGTK, on any desktop)
//! and ends when the window is closed; without WebKitGTK it opens in the browser.
//!
//! `--settings` opens the Settings tab; `--browser` uses the browser even when the window is
//! available; `--no-browser` only prints the address.

use std::time::Duration;

const IDLE_QUIT_SECS: u64 = 30 * 60;

/// Linux: Studio's own window (`ekushe-studio-window`, GTK + WebKitGTK), installed next to the
/// keyboard engine. Returns `false` when it isn't installed or can't show a window here, so the
/// caller opens the browser instead. Blocks until the window is closed.
#[cfg(target_os = "linux")]
fn run_in_window(url: &str, tab: Option<&str>) -> bool {
    let exe = std::env::current_exe().ok();
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os("EKUSHE_STUDIO_WINDOW") {
        candidates.push(p.into());
    }
    if let Some(dir) = exe.as_ref().and_then(|e| e.parent()) {
        candidates.push(dir.join("../libexec/ekushe/ekushe-studio-window"));
        candidates.push(dir.join("ekushe-studio-window"));
    }
    candidates.push("/usr/libexec/ekushe/ekushe-studio-window".into());
    let Some(window) = candidates.into_iter().find(|p| p.is_file()) else {
        return false;
    };
    match std::process::Command::new(window)
        .arg(url)
        .arg(tab.unwrap_or(""))
        .status()
    {
        // 3: no display; 127: WebKitGTK (or GTK) isn't installed
        Ok(st) => !matches!(st.code(), Some(3) | Some(127)),
        Err(_) => false,
    }
}

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
    #[cfg(target_os = "linux")]
    ekushe_ffi::studio::keyboard::remove_old_cosmic_env();
    if args.iter().any(|a| a == "--version") {
        println!("ekushe-studio {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    #[cfg(target_os = "linux")]
    if args.iter().any(|a| a == "--add-keyboard") {
        // Add Ekushe to this desktop's keyboards (GNOME, KDE Plasma, Xfce, Cinnamon, MATE, ...)
        use ekushe_ffi::studio::keyboard;
        match keyboard::status() {
            None => {
                println!("No desktop session found: add Ekushe Bangla in your keyboard settings.")
            }
            Some(st) if st.added => println!(
                "Ekushe Bangla is already one of your keyboards ({}).",
                st.desktop
            ),
            Some(st) if !st.can_add => println!("{}", st.steps),
            Some(_) => {
                let r = keyboard::add();
                println!("{}", r.message);
                if !r.ok {
                    std::process::exit(1);
                }
            }
        }
        return;
    }
    let tab = args.iter().any(|a| a == "--settings").then_some("settings");
    // A fresh one-time address (only the first page opened with it gets Studio's key)
    let Some(url) = ekushe_ffi::studio::launch_url(tab) else {
        eprintln!("Ekushe Studio: no free port on 127.0.0.1");
        std::process::exit(1);
    };
    if args.iter().any(|a| a == "--no-browser") {
        println!("Ekushe Studio: {url}");
    } else {
        // Its own window when it can (closing it ends Studio), otherwise a browser tab
        #[cfg(target_os = "linux")]
        if !args.iter().any(|a| a == "--browser") && run_in_window(&url, tab) {
            ekushe_ffi::save_learned_now();
            return;
        }
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
