#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! `ekushe`: Windows System Tray IME, Global Phonetic/Fixed-Layout Hook,
//! Floating Candidate + Sign/Conjunct Hints Overlay, and Ekushe Studio Server.

mod glass_popup;
mod secure_field;
mod selection;
mod setup_wizard;
mod studio_http;
mod studio_ui;
mod update_check;

use std::env;
use std::io::{Read, Write};
use std::mem::{size_of, zeroed};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::Instant;

use ekushe_core::{
    config::{Config, InputLanguage, SwitchHotkey, TypingMode},
    conjuncts::search_signs_and_conjuncts,
    convert::{banglish_to_bangla, bijoy_to_unicode, unicode_to_bijoy},
    layout::LayoutId,
    Ekushe, SessionOutput,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, ClientToScreen, GetMonitorInfoW, MonitorFromPoint, ValidateRect,
    MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, SendInput, ToUnicodeEx, INPUT, INPUT_0,
    INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK, VK_CONTROL, VK_ESCAPE,
    VK_F12, VK_LSHIFT, VK_MENU, VK_RETURN, VK_RSHIFT, VK_SHIFT, VK_SPACE, VK_TAB,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CallNextHookEx, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    DestroyWindow, DispatchMessageW, FindWindowW, GetAncestor, GetClassNameW, GetCursorPos,
    GetForegroundWindow, GetGUIThreadInfo, GetMessageW, GetSystemMetrics, GetWindowLongW,
    GetWindowThreadProcessId, KillTimer, LoadCursorW, LoadIconW, LoadImageW, PeekMessageW,
    PostMessageW, PostQuitMessage, PostThreadMessageW, RegisterClassW, SetCursor,
    SetForegroundWindow, SetMenuDefaultItem, SetTimer, SetWindowPos, SetWindowsHookExW, ShowWindow,
    TrackPopupMenu, TranslateMessage, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT, GUITHREADINFO,
    GWL_STYLE, HHOOK, HICON, HWND_TOPMOST, IDC_HAND, IDI_APPLICATION, IMAGE_ICON, KBDLLHOOKSTRUCT,
    LR_LOADFROMFILE, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, MSLLHOOKSTRUCT,
    SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_APP, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_NULL, WM_PAINT, WM_RBUTTONUP, WM_SETCURSOR,
    WM_SYSKEYDOWN, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

/// Embedded 256x256 PNG logo for the Studio UI (`/logo.png`)
static LOGO_PNG_BYTES: &[u8] = include_bytes!("../../../assets/icons/logo_256.png");

/// Bundled Bangladeshi & Bengali OpenType Fonts (100% offline, compiled into binary)
static FONT_KALPURUSH: &[u8] = include_bytes!("../../../assets/fonts/Kalpurush.ttf");
static FONT_SOLAIMANLIPI: &[u8] = include_bytes!("../../../assets/fonts/SolaimanLipi.ttf");
static FONT_SIYAMRUPALI: &[u8] = include_bytes!("../../../assets/fonts/SiyamRupali.ttf");
static FONT_HINDSILIGURI_REG: &[u8] =
    include_bytes!("../../../assets/fonts/HindSiliguri-Regular.ttf");
static FONT_HINDSILIGURI_BOLD: &[u8] =
    include_bytes!("../../../assets/fonts/HindSiliguri-Bold.ttf");

/// Magic extra info tag so our low-level keyboard hook never intercepts our own `SendInput` events.
pub(crate) const EKUSHE_EXTRA_INFO: usize = 0xE21B_0945;

const WM_TRAYICON: u32 = WM_APP + 1;
const TIMER_HIDE_BADGE: usize = 101;

// Tray Context Menu Command IDs
const CMD_TOGGLE_LANG: usize = 1001;
const CMD_OPEN_STUDIO: usize = 1002;
const CMD_STUDIO_SIGNS: usize = 1003;
const CMD_STUDIO_SPELL: usize = 1004;
const CMD_STUDIO_CONVERT: usize = 1005;
const CMD_STUDIO_SETTINGS: usize = 1006;
const CMD_STUDIO_PLAYGROUND: usize = 1007;
const CMD_STUDIO_XRAY: usize = 1008;
const CMD_STUDIO_FIX: usize = 1009;
const CMD_HOTKEY_DBL_SHIFT: usize = 1010;
const CMD_HOTKEY_CTRL_SPACE: usize = 1011;
const CMD_HOTKEY_F12: usize = 1012;
const CMD_HOTKEY_CTRL_ALT_B: usize = 1013;
const CMD_MODE_PHONETIC_FIRST: usize = 1020;
const CMD_MODE_SMART: usize = 1021;
const CMD_MODE_PHONETIC_ONLY: usize = 1022;
const CMD_LAYOUT_AVRO: usize = 1030;
const CMD_LAYOUT_PROBHAT: usize = 1031;
const CMD_LAYOUT_JATIYA: usize = 1032;
const CMD_LAYOUT_BORNONA: usize = 1033;
const CMD_LAYOUT_MUNIR: usize = 1034;
const CMD_CHECK_UPDATE: usize = 1096;
const CMD_UPDATE_DOWNLOAD: usize = 1097;
const CMD_REPORT: usize = 1098;
const CMD_EXIT: usize = 1099;

struct AppSharedState {
    ekushe: Ekushe,
    last_output: SessionOutput,
    /// Number of UTF-16 code units currently injected into the active editor for the live preedit
    injected_utf16_len: usize,
    /// Whether Shift was pressed down without any other key pressed while it was held
    shift_clean_down: bool,
    /// Monotonic start instant for millisecond timestamps
    start_instant: Instant,
    /// Local loopback port where Ekushe Studio is listening
    studio_port: u16,
    /// Overlay popup window handle
    overlay_hwnd: isize,
    /// Tray host window handle
    tray_hwnd: isize,
    /// Cached Bangla tray icon handle
    icon_bangla: isize,
    /// Cached English tray icon handle
    icon_english: isize,
    /// Transient status banner text (e.g. "বাং · বাংলা সক্রিয়" or "EN · English Keyboard")
    badge_banner: Option<String>,
    /// Interactive bounding boxes for Candidate Pills in the floating overlay (`(rect, candidate_index)`)
    cand_hit_rects: Vec<(RECT, usize)>,
    /// Interactive bounding boxes for Sign/Conjunct Hint Chips in the floating overlay (`(rect, roman_key_hint)`)
    hint_hit_rects: Vec<(RECT, String)>,
    /// Interactive bounding box for the "📖 সহায়িকা" helper button in the floating overlay header
    helper_btn_rect: RECT,
    /// Interactive bounding box for the "🔽 সংক্ষিপ্ত / 🔼 যুক্তবর্ণ" Compact/Expanded popup toggle button
    compact_btn_rect: RECT,
    /// Translucent opacity percentage (`70..=100`, default `92`) for the floating popup bar
    popup_opacity_pct: u8,
    /// The focused control the current composition was typed into (0 = none)
    comp_focus: isize,
    /// Foreground window seen by the last keystroke (per-app language switching)
    last_fg: isize,
    /// `learned_version` last written to disk
    saved_learned_version: u64,
}

static STATE: Lazy<Mutex<AppSharedState>> = Lazy::new(|| {
    let mut cfg = Config::load_from(&Config::default_config_path()).unwrap_or_default();
    if cfg.candidate_font_family.is_empty() || cfg.candidate_font_family == "Nirmala UI" {
        cfg.candidate_font_family = "Kalpurush".to_string();
    }
    // Ensure Bangla digits are enabled in Ekushe Bangla mode so English mode = 0-9 and Ekushe mode = ০-৯
    cfg.bangla_digits = true;
    let start = cfg.start_language;
    let mut ek = Ekushe::new(cfg);
    // English when Windows starts unless the user chose Bangla; double-tap Shift switches
    if ek.language() != start {
        ek.toggle_language();
    }
    // Words the user picked in earlier sessions
    let _ = ek
        .suggestions_engine_mut()
        .load_user_dict(&learned_words_path());
    let saved_learned_version = ek.suggestions_engine().learned_version();
    Mutex::new(AppSharedState {
        ekushe: ek,
        last_output: SessionOutput::default(),
        injected_utf16_len: 0,
        shift_clean_down: false,
        start_instant: Instant::now(),
        studio_port: 0,
        overlay_hwnd: 0,
        tray_hwnd: 0,
        icon_bangla: 0,
        icon_english: 0,
        badge_banner: None,
        cand_hit_rects: Vec::new(),
        hint_hit_rects: Vec::new(),
        helper_btn_rect: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        compact_btn_rect: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        popup_opacity_pct: 85,
        comp_focus: 0,
        last_fg: 0,
        saved_learned_version,
    })
});

static mut KEYBOARD_HOOK: HHOOK = ptr::null_mut();
/// The thread that owns the low-level hooks and does nothing else (see `start_hook_thread`)
static HOOK_THREAD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static mut MOUSE_HOOK: HHOOK = ptr::null_mut();
/// Set by the mouse hook when the user clicks outside the candidate popup: the caret may have moved,
/// so the half-typed word must be forgotten (never backspaced at the new position).
static COMPOSITION_STALE: AtomicBool = AtomicBool::new(false);
static OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);
static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
const WM_EKUSHE_FORGET: u32 = WM_APP + 2;
/// Another `ekushe.exe` started by the same user asks the running one to open Studio (only the
/// running app can make Studio's one-time launch codes).
const WM_EKUSHE_OPEN_STUDIO: u32 = WM_APP + 3;
/// The convert-selection hotkey was pressed (the keyboard hook asks; the work runs elsewhere)
const WM_EKUSHE_CONVERT_SELECTION: u32 = WM_APP + 4;
/// Show a badge from another thread (`wparam`: which one, see `selection_badge`)
const WM_EKUSHE_SELECTION_DONE: u32 = WM_APP + 5;
/// The keyboard hook asks the tray thread to redraw the candidate popup (`refresh_candidate_overlay`)
/// once the hook has returned. Drawing never happens inside the hook: Windows removes a low-level
/// hook that takes longer than `LowLevelHooksTimeout` (about 300 ms), silently, and typing in Bangla
/// would just stop.
const WM_EKUSHE_REFRESH_OVERLAY: u32 = WM_APP + 6;
/// Show the badge in `STATE.badge_banner` (posted by the hook for the same reason)
const WM_EKUSHE_SHOW_BADGE: u32 = WM_APP + 7;
/// Posted to the hook thread: re-install the hooks (see `TIMER_REHOOK`)
const WM_EKUSHE_REHOOK: u32 = WM_APP + 40;
/// Update the tray icon for the current language (posted by the hook; `Shell_NotifyIconW` talks to
/// Explorer and can block)
const WM_EKUSHE_UPDATE_TRAY_ICON: u32 = WM_APP + 8;
const TIMER_SAVE_LEARNED: usize = 102;
/// Re-install the low-level hooks every 30 s: if Windows ever dropped one (hook timeout, see
/// `WM_EKUSHE_REFRESH_OVERLAY`), typing recovers within half a minute instead of never.
const TIMER_REHOOK: usize = 103;
const REHOOK_INTERVAL_MS: u32 = 30_000;
const ES_PASSWORD_STYLE: i32 = 0x0020;

unsafe fn register_embedded_bangla_fonts() {
    for font_bytes in [
        FONT_KALPURUSH,
        FONT_SOLAIMANLIPI,
        FONT_SIYAMRUPALI,
        FONT_HINDSILIGURI_REG,
        FONT_HINDSILIGURI_BOLD,
    ] {
        let num_fonts: u32 = 0;
        let _ = AddFontMemResourceEx(
            font_bytes.as_ptr() as *const _,
            font_bytes.len() as u32,
            ptr::null(),
            &num_fonts,
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    unsafe {
        let _ = windows_sys::Win32::UI::WindowsAndMessaging::SetProcessDPIAware();
        // Register bundled Bangladeshi fonts (Kalpurush, SolaimanLipi, Siyam Rupali, Hind Siliguri) in GDI memory
        register_embedded_bangla_fonts();

        // CI / design check: draw the popup and badge to PNG files and exit
        if let Some(i) = args.iter().position(|a| a == "--render-popup") {
            let dir = PathBuf::from(
                args.get(i + 1)
                    .map(String::as_str)
                    .unwrap_or("popup-previews"),
            );
            match glass_popup::render_previews(&dir) {
                Ok(files) => {
                    for f in files {
                        println!("{}", f.display());
                    }
                    return;
                }
                Err(e) => {
                    eprintln!("render failed: {e}");
                    std::process::exit(1);
                }
            }
        }

        if exe_name.contains("uninstall") || args.iter().any(|a| a == "--uninstall") {
            if let Err(e) = setup_wizard::perform_uninstall() {
                eprintln!("Ekushe uninstall failed: {e}");
                std::process::exit(1);
            }
            return;
        }

        if args.iter().any(|a| a == "--auto-install") {
            let launch = !args.iter().any(|a| a == "--no-launch");
            if let Err(e) = setup_wizard::perform_full_install(true, true, launch) {
                eprintln!("Ekushe install failed: {e}");
                std::process::exit(1);
            }
            return;
        }

        if exe_name.contains("setup")
            || exe_name.contains("install")
            || args.iter().any(|a| a == "--installer")
        {
            setup_wizard::run_gui_installer_window();
            return;
        }
    }

    // Automatically ensure icons, Bangladeshi fonts, and default configuration are extracted and active
    setup_wizard::ensure_self_configured();

    let open_studio_on_launch = !args
        .iter()
        .any(|a| a == "--minimized" || a == "--tray-only");

    // If an instance of Ekushe is already running in the System Tray, reuse it and open Studio if requested!
    if !args.iter().any(|a| a == "--self-test") {
        if let Some(existing_port) = check_existing_studio_port() {
            if open_studio_on_launch || args.iter().any(|a| a == "--open-studio") {
                // The running app opens Studio (FindWindow only sees this user's desktop)
                let host =
                    unsafe { FindWindowW(wide_null("EkusheTrayHostClass").as_ptr(), ptr::null()) };
                if host.is_null() {
                    open_studio_plain(existing_port);
                } else {
                    unsafe { PostMessageW(host, WM_EKUSHE_OPEN_STUDIO, 0, 0) };
                }
            }
            return;
        }
    }

    // Start local loopback Studio HTTP server on a fixed or ephemeral localhost port
    let port = start_studio_server();
    {
        let mut st = STATE.lock();
        st.studio_port = port;
    }

    if args.iter().any(|a| a == "--self-test") {
        run_self_test(port);
        return;
    }

    unsafe {
        let hinstance = GetModuleHandleW(ptr::null());

        // Load our custom tray icons from assets/icons/
        let (icon_bn, icon_en) = load_tray_icons();
        {
            let mut st = STATE.lock();
            st.icon_bangla = icon_bn as isize;
            st.icon_english = icon_en as isize;
        }

        // Register hidden tray host window class and floating candidate overlay window class
        let tray_class_name = wide_null("EkusheTrayHostClass");
        let wc_tray = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(tray_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: icon_bn,
            hCursor: ptr::null_mut(),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: tray_class_name.as_ptr(),
        };
        RegisterClassW(&wc_tray);

        let overlay_class_name = wide_null("EkusheOverlayPopupClass");
        let wc_overlay = WNDCLASSW {
            style: 0x0002_0000, // CS_DROPSHADOW for native Windows floating shadow
            lpfnWndProc: Some(overlay_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: icon_bn,
            hCursor: LoadCursorW(ptr::null_mut(), IDC_HAND),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: overlay_class_name.as_ptr(),
        };
        RegisterClassW(&wc_overlay);

        let tray_hwnd = CreateWindowExW(
            0,
            tray_class_name.as_ptr(),
            wide_null("Ekushe Next Tray Host").as_ptr(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        );

        // Per-monitor DPI aware, so the popup is drawn at the display's real resolution
        let overlay_hwnd = glass_popup::with_dpi_awareness(|| {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED,
                overlay_class_name.as_ptr(),
                wide_null("Ekushe Next Candidate Overlay").as_ptr(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance,
                ptr::null(),
            )
        });

        {
            let mut st = STATE.lock();
            st.tray_hwnd = tray_hwnd as isize;
            st.overlay_hwnd = overlay_hwnd as isize;
        }
        TRAY_HWND.store(tray_hwnd as isize, Ordering::SeqCst);
        OVERLAY_HWND.store(overlay_hwnd as isize, Ordering::SeqCst);

        // Add icon to Windows System Tray
        add_or_update_tray_icon(tray_hwnd, true);
        update_check::start_background(tray_hwnd as isize, || {
            STATE.lock().ekushe.config().check_for_updates
        });

        // Install the global low-level keyboard hook, and the mouse hook (mouse clicks can move the
        // caret; the mouse hook makes sure a half-typed word is then forgotten), on their own thread
        start_hook_thread();
        // Save learned words every 15 s when something new was learned (and again on exit)
        SetTimer(tray_hwnd, TIMER_SAVE_LEARNED, 15_000, None);
        SetTimer(tray_hwnd, TIMER_REHOOK, REHOOK_INTERVAL_MS, None);
        // Watches for password boxes in browsers and Store apps
        secure_field::start();

        // Show initial floating badge ("বাং · একুশে বাংলা সক্রিয়") and optionally open Studio
        show_mode_badge("বাং · বাংলায় লিখছেন — দুবার Shift চাপলে English");
        if open_studio_on_launch {
            open_studio_window(port);
        }

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        stop_hook_thread();
        save_learned_words_if_changed();
        remove_tray_icon(tray_hwnd);
    }
}

/// The low-level hooks get a thread of their own that only waits for keys. Windows runs a low-level
/// hook on the thread that installed it, between that thread's messages: on the tray thread a key
/// had to wait whenever the popup was being drawn, and a key that waits longer than
/// `LowLevelHooksTimeout` goes straight to the app, unconverted, while the hook still rewrites the
/// word around it (a Latin "e" inside a Bangla word, letters typed twice). Here nothing delays it.
unsafe fn start_hook_thread() {
    std::thread::spawn(|| unsafe {
        use windows_sys::Win32::System::Threading::GetCurrentThreadId;
        let mut msg: MSG = zeroed();
        // make sure this thread has a message queue before anyone posts to it
        PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, 0);
        HOOK_THREAD.store(GetCurrentThreadId(), Ordering::SeqCst);
        install_hooks();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_EKUSHE_REHOOK {
                install_hooks();
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        remove_hooks();
    });
}

/// Ask the hook thread to re-install its hooks (in case Windows dropped one)
unsafe fn rehook() {
    let tid = HOOK_THREAD.load(Ordering::SeqCst);
    if tid != 0 {
        PostThreadMessageW(tid, WM_EKUSHE_REHOOK, 0, 0);
    }
}

/// Remove the hooks (on their own thread) and end that thread
unsafe fn stop_hook_thread() {
    let tid = HOOK_THREAD.swap(0, Ordering::SeqCst);
    if tid != 0 {
        PostThreadMessageW(tid, 0x0012, 0, 0); // WM_QUIT
                                               // give it a moment to unhook before the process exits
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Install (or re-install) the low-level keyboard and mouse hooks. Runs on the hook thread, which
/// owns them. Re-installing a live hook is harmless: the old one is removed first.
unsafe fn install_hooks() {
    remove_hooks();
    let hinstance = GetModuleHandleW(ptr::null());
    KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0);
    MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0);
}

unsafe fn remove_hooks() {
    if !KEYBOARD_HOOK.is_null() {
        UnhookWindowsHookEx(KEYBOARD_HOOK);
        KEYBOARD_HOOK = ptr::null_mut();
    }
    if !MOUSE_HOOK.is_null() {
        UnhookWindowsHookEx(MOUSE_HOOK);
        MOUSE_HOOK = ptr::null_mut();
    }
}

/// From the keyboard hook: have the tray thread redraw (or hide) the candidate popup after the hook
/// returns. See `WM_EKUSHE_REFRESH_OVERLAY`.
fn request_overlay_refresh() {
    let tray = TRAY_HWND.load(Ordering::SeqCst);
    if tray != 0 {
        unsafe { PostMessageW(tray as HWND, WM_EKUSHE_REFRESH_OVERLAY, 0, 0) };
    }
}

/// From the keyboard hook: show `banner` once the hook has returned.
fn request_badge(banner: &str) {
    STATE.lock().badge_banner = Some(banner.to_string());
    let tray = TRAY_HWND.load(Ordering::SeqCst);
    if tray != 0 {
        unsafe { PostMessageW(tray as HWND, WM_EKUSHE_SHOW_BADGE, 0, 0) };
    }
}

/// From the keyboard hook: update the tray icon for the current language once the hook has returned.
fn request_tray_icon_update() {
    let tray = TRAY_HWND.load(Ordering::SeqCst);
    if tray != 0 {
        unsafe { PostMessageW(tray as HWND, WM_EKUSHE_UPDATE_TRAY_ICON, 0, 0) };
    }
}

/// Self-test mode for automated verification of all System Tray right-click URLs & HTTP Studio endpoints.
fn run_self_test(port: u16) {
    println!("Running Ekushe Tray + Studio comprehensive self-test on 127.0.0.1:{port}...");

    let http_get = |uri: &str| -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect studio server");
        let req = format!(
            "GET {uri} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Ekushe-Token: {}\r\nConnection: close\r\n\r\n",
            studio_token()
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        String::from_utf8_lossy(&resp).into_owned()
    };

    let http_post = |uri: &str, json_body: &str| -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect studio server");
        let req = format!(
            "POST {uri} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nX-Ekushe-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            studio_token(),
            json_body.len(),
            json_body
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        String::from_utf8_lossy(&resp).into_owned()
    };

    // 1. Verify all System Tray Right-Click Studio Tab URLs return 200 OK + Studio HTML
    for uri in [
        "/",
        "/?tab=playground",
        "/?tab=xray",
        "/?tab=signs",
        "/?tab=spell",
        "/?tab=converter",
        "/?tab=settings",
    ] {
        let r = http_get(uri);
        assert!(r.starts_with("HTTP/1.1 200 OK"), "Failed GET {uri}");
        assert!(
            r.contains("tab-signs") && r.contains("tab-xray") && r.contains("tab-spell"),
            "Missing HTML panels on {uri}"
        );
        println!("✓ Right-Click Studio URL verified: GET {uri} -> 200 OK");
    }

    // 1b. The Studio refuses other websites: no token, a foreign Host, a foreign Origin
    let raw = |req: String| -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect studio server");
        s.write_all(req.as_bytes()).unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        String::from_utf8_lossy(&resp).into_owned()
    };
    for bad in [
        format!("GET /api/state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
        format!("GET / HTTP/1.1\r\nHost: evil.example:{port}\r\nConnection: close\r\n\r\n"),
        format!(
            "POST /api/toggle HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://evil.example\r\nX-Ekushe-Token: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            studio_token()
        ),
    ] {
        assert!(raw(bad).starts_with("HTTP/1.1 403"), "Studio accepted a foreign request");
    }
    println!("✓ Studio rejects requests without the token, foreign Host and foreign Origin");

    // 1c. The page only carries the key for a fresh one-time launch code
    assert!(
        !http_get("/").contains(studio_token()),
        "the plain page must not carry the key"
    );
    let launch = studio_http::launch_url(port, Some("signs"));
    let launch_path = &launch[launch.find("/?").unwrap()..];
    assert!(http_get(launch_path).contains(studio_token()));
    assert!(
        !http_get(launch_path).contains(studio_token()),
        "a launch code must work once"
    );
    println!("✓ Studio key only with a one-time launch code");

    // 1d. A slow client does not block Studio; an oversized request is refused
    let mut slow = TcpStream::connect(("127.0.0.1", port)).expect("connect studio server");
    let _ = slow.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n");
    let t0 = std::time::Instant::now();
    assert!(http_get("/api/state").contains("\"language\""));
    assert!(
        t0.elapsed() < std::time::Duration::from_secs(2),
        "a slow client blocked Studio"
    );
    drop(slow);
    let big = format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Pad: {}\r\n\r\n",
        "a".repeat(studio_http::MAX_HEADER + 100)
    );
    assert!(
        raw(big).starts_with("HTTP/1.1 431"),
        "oversized request accepted"
    );
    println!("✓ Studio stays responsive with a slow client and refuses oversized requests");

    // 2. Verify /api/state
    let r_state = http_get("/api/state");
    assert!(r_state.contains("\"language\""));
    assert!(r_state.contains("Kalpurush"));
    println!("✓ /api/state verified");

    // 3. Verify /api/suggest (Phonetic + Connected Letters + Sign Hints)
    let r_sug = http_get("/api/suggest?q=swadhin");
    assert!(r_sug.contains("স্বাধীন"));
    println!("✓ /api/suggest?q=swadhin verified (স্বাধীন)");

    // 4. Verify /api/signs (Connected Letters & Signs Guide)
    let r_signs = http_get("/api/signs?q=sw");
    assert!(r_signs.contains("স্ব"));
    println!("✓ /api/signs?q=sw verified (স্ব)");

    // 5. Verify POST /api/spell (Bangla Spell Checker)
    let r_spell = http_post("/api/spell", r#"{"text":"আমি বাংলাদেস এ বাস করি"}"#);
    assert!(r_spell.contains("বাংলাদেশ"));
    println!("✓ POST /api/spell verified (বাংলাদেস -> বাংলাদেশ)");

    // 6. Verify POST /api/convert (Bijoy <-> Unicode & Banglish)
    let r_conv1 = http_post(
        "/api/convert",
        r#"{"mode":"bijoy2uni","text":"Avwg evsjvq Mvb MvB|"}"#,
    );
    assert!(r_conv1.contains("আমি বাংলায় গান গাই।"));
    let r_conv2 = http_post(
        "/api/convert",
        r#"{"mode":"banglish","text":"ami banglay gan gai"}"#,
    );
    assert!(r_conv2.contains("আমি বাংলায় গান গাই"));
    println!("✓ POST /api/convert verified (Bijoy <-> Unicode & Banglish)");

    // 7. Verify POST /api/settings
    let r_set = http_post(
        "/api/settings",
        r#"{"hotkey":"DoubleTapShift","typing_mode":"PhoneticFirst","active_layout":"AvroPhonetic","font_family":"Kalpurush"}"#,
    );
    assert!(r_set.contains("\"ok\":true"));
    println!("✓ POST /api/settings verified");

    // 8. "A word was wrong", backup and restore
    assert!(http_get("/api/recent").contains("\"recent\""));
    let r_wrong = http_post(
        "/api/wrong",
        r#"{"roman":"nothere","wrong":"x","right":"y"}"#,
    );
    assert!(r_wrong.contains("not a recent word"));
    let backup = http_get("/api/backup");
    assert!(backup.contains("\"ekushe_backup\": 1") && backup.contains("config.toml"));
    let body = &backup[backup.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0)..];
    assert!(http_post("/api/restore", body).contains("\"ok\":true"));
    assert!(http_post("/api/restore", "{}").contains("\"ok\":false"));
    assert!(http_get("/api/state").contains(update_check::current_version()));
    println!("✓ recent words, backup and restore verified");

    println!("ALL EKUSHE STUDIO & SYSTEM TRAY ENDPOINTS PASSED 100%!");
}

/// Load `tray_bangla.ico` and `tray_english.ico` from `assets/icons/` relative to executable or workspace.
unsafe fn load_tray_icons() -> (HICON, HICON) {
    let candidates = [
        PathBuf::from("assets/icons"),
        env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("assets/icons")))
            .unwrap_or_default(),
        env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .and_then(|d| d.parent())
                    .and_then(|d| d.parent())
                    .map(|d| d.join("assets/icons"))
            })
            .unwrap_or_default(),
    ];

    for dir in &candidates {
        let bn_path = dir.join("tray_bangla.ico");
        let en_path = dir.join("tray_english.ico");
        if bn_path.exists() && en_path.exists() {
            let bn_w = wide_null(&bn_path.to_string_lossy());
            let en_w = wide_null(&en_path.to_string_lossy());
            let h_bn = LoadImageW(
                ptr::null_mut(),
                bn_w.as_ptr(),
                IMAGE_ICON,
                32,
                32,
                LR_LOADFROMFILE,
            ) as HICON;
            let h_en = LoadImageW(
                ptr::null_mut(),
                en_w.as_ptr(),
                IMAGE_ICON,
                32,
                32,
                LR_LOADFROMFILE,
            ) as HICON;
            if !h_bn.is_null() && !h_en.is_null() {
                return (h_bn, h_en);
            }
        }
    }

    let fallback = LoadIconW(ptr::null_mut(), IDI_APPLICATION);
    (fallback, fallback)
}

unsafe fn add_or_update_tray_icon(hwnd: HWND, is_add: bool) {
    let st = STATE.lock();
    let is_bangla = st.ekushe.language() == InputLanguage::Bangla;
    let hicon = if is_bangla {
        st.icon_bangla as HICON
    } else {
        st.icon_english as HICON
    };
    let hotkey_str = st.ekushe.config().switch_hotkey.display_name();
    let tip = if is_bangla {
        format!("একুশে নেক্সট বাংলা সক্রিয় (বাং) — Switch: {hotkey_str}")
    } else {
        format!("Ekushe Next: Normal Keyboard (EN) — Switch: {hotkey_str}")
    };
    drop(st);

    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = hicon;

    let tip_w = wide_null(&tip);
    for (i, &ch) in tip_w.iter().take(127).enumerate() {
        nid.szTip[i] = ch;
    }

    if is_add {
        nid.uFlags |= NIF_INFO;
        let info_title = wide_null("একুশে নেক্সট (Ekushe Next Bangla Keyboard)");
        let info_body =
            wide_null("সিস্টেম ট্রে-তে সক্রিয় আছে! দুইবার Shift চেপে বাংলা ⇄ ইংরেজি পরিবর্তন করুন।");
        for (i, &c) in info_title.iter().take(63).enumerate() {
            nid.szInfoTitle[i] = c;
        }
        for (i, &c) in info_body.iter().take(255).enumerate() {
            nid.szInfo[i] = c;
        }
        Shell_NotifyIconW(NIM_ADD, &nid);
    } else {
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

/// Where the popup goes: under the text caret when the application reports one, else the mouse.
unsafe fn popup_anchor() -> POINT {
    let fg = GetForegroundWindow();
    if !fg.is_null() {
        let tid = GetWindowThreadProcessId(fg, ptr::null_mut());
        let mut gti: GUITHREADINFO = zeroed();
        gti.cbSize = size_of::<GUITHREADINFO>() as u32;
        if GetGUIThreadInfo(tid, &mut gti) != 0
            && !gti.hwndCaret.is_null()
            && gti.rcCaret.bottom > gti.rcCaret.top
        {
            let mut pt = POINT {
                x: gti.rcCaret.left,
                y: gti.rcCaret.bottom,
            };
            if ClientToScreen(gti.hwndCaret, &mut pt) != 0 {
                return POINT {
                    x: pt.x - 12,
                    y: pt.y - 16,
                };
            }
        }
    }
    let mut pt = POINT { x: 400, y: 400 };
    GetCursorPos(&mut pt);
    pt
}

/// Keep a `w`×`h` popup anchored at `pt` inside that monitor's work area (flip above if needed).
unsafe fn place_popup(pt: POINT, w: i32, h: i32, s: f32) -> (i32, i32) {
    let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    let mut mi: MONITORINFO = zeroed();
    mi.cbSize = size_of::<MONITORINFO>() as u32;
    let work = if GetMonitorInfoW(mon, &mut mi) != 0 {
        mi.rcWork
    } else {
        RECT {
            left: 0,
            top: 0,
            right: GetSystemMetrics(SM_CXSCREEN),
            bottom: GetSystemMetrics(SM_CYSCREEN),
        }
    };
    let below = pt.y + (24.0 * s) as i32;
    let y = if below + h > work.bottom - 8 {
        (pt.y - h - (12.0 * s) as i32).max(work.top + 8)
    } else {
        below
    };
    let x =
        (pt.x + (12.0 * s) as i32).clamp(work.left + 8, (work.right - w - 8).max(work.left + 8));
    (x, y)
}

/// Draw `content` as frosted glass and show it next to the caret, without taking focus.
unsafe fn present_overlay(content: glass_popup::Content) {
    let (hwnd, font, opacity) = {
        let st = STATE.lock();
        (
            st.overlay_hwnd as HWND,
            st.ekushe.config().candidate_font_family.clone(),
            st.popup_opacity_pct,
        )
    };
    if hwnd.is_null() {
        return;
    }
    glass_popup::with_dpi_awareness(|| {
        let anchor = popup_anchor();
        let s = glass_popup::scale_at(anchor);
        let look = glass_popup::windows_look();
        let blur = look.transparency && opacity < 100;
        let alpha = glass_popup::glass_alpha(opacity, blur);
        let Some((canvas, hits)) = glass_popup::render(&content, &font, s, look.dark, alpha) else {
            return;
        };
        {
            let mut st = STATE.lock();
            st.cand_hit_rects = hits.candidates;
            st.hint_hit_rects = hits.hints;
            st.helper_btn_rect = hits.helper;
            st.compact_btn_rect = hits.compact;
        }
        let (x, y) = place_popup(anchor, canvas.w, canvas.h, s);
        canvas.present(hwnd, x, y);
        glass_popup::set_backdrop(
            hwnd,
            blur,
            look.dark,
            canvas.w,
            canvas.h,
            glass_popup::radius(s),
        );
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    });
}

/// Show a short message (language switched, a setting changed) next to the caret for 1.6 s.
/// The badge shown after converting a selection
fn selection_badge(code: usize) -> &'static str {
    match code {
        0 => "বাং · বিজয় থেকে ইউনিকোডে বদলানো হয়েছে",
        1 => "বাং · বাংলিশ থেকে বাংলায় বদলানো হয়েছে",
        2 => "বাং · আগে লেখাটি বেছে নিন (select the text first)",
        3 => "বাং · বদলানোর মতো কিছু নেই — লেখাটি আগেই বাংলা",
        4 => "বাং · লেখাটি অনেক লম্বা — স্টুডিওর রূপান্তর ব্যবহার করুন",
        _ => "বাং · এই উইন্ডোর লেখা বদলানো গেল না",
    }
}

unsafe fn show_mode_badge(banner: &str) {
    let overlay_hwnd = {
        let mut st = STATE.lock();
        st.badge_banner = Some(banner.to_string());
        st.overlay_hwnd as HWND
    };
    if overlay_hwnd.is_null() {
        return;
    }
    present_overlay(glass_popup::Content::Badge(banner.to_string()));
    SetTimer(overlay_hwnd, TIMER_HIDE_BADGE, 1600, None);
}

unsafe fn refresh_candidate_overlay() {
    let (overlay_hwnd, content) = {
        let mut st = STATE.lock();
        st.badge_banner = None;
        let out = &st.last_output;
        let has = !out.preedit.is_empty() || !out.candidates.is_empty();
        let content = has.then(|| glass_popup::Content::Candidates {
            roman: out.roman_buffer.clone(),
            candidates: out.candidates.iter().map(|c| c.text.clone()).collect(),
            selected: out.selected_index,
            hints: out
                .sign_hints
                .iter()
                .map(|h| (h.result_bangla.clone(), h.key_hint.clone()))
                .collect(),
            // Tips (how to type a word on offer directly) lead the hint row: they are hints whose
            // result is one of the candidates
            tips: out
                .sign_hints
                .iter()
                .take_while(|h| out.candidates.iter().any(|c| c.text == h.result_bangla))
                .count(),
            compact: st.ekushe.config().compact_popup,
        });
        (st.overlay_hwnd as HWND, content)
    };
    if overlay_hwnd.is_null() {
        return;
    }
    match content {
        Some(c) => {
            KillTimer(overlay_hwnd, TIMER_HIDE_BADGE);
            present_overlay(c);
        }
        None => {
            ShowWindow(overlay_hwnd, SW_HIDE);
        }
    }
}

/// The control that has keyboard focus in the foreground window (null if unknown).
unsafe fn focused_control() -> HWND {
    let fg = GetForegroundWindow();
    if fg.is_null() {
        return ptr::null_mut();
    }
    let tid = GetWindowThreadProcessId(fg, ptr::null_mut());
    let mut gti: GUITHREADINFO = zeroed();
    gti.cbSize = size_of::<GUITHREADINFO>() as u32;
    if GetGUIThreadInfo(tid, &mut gti) != 0 && !gti.hwndFocus.is_null() {
        gti.hwndFocus
    } else {
        fg
    }
}

/// True for standard Windows password boxes (Edit / RichEdit with `ES_PASSWORD`). Password fields in
/// browsers and Store apps are found by `secure_field` (UI Automation).
unsafe fn is_password_field(h: HWND) -> bool {
    if h.is_null() {
        return false;
    }
    let mut cls_buf = [0u16; 64];
    let len = GetClassNameW(h, cls_buf.as_mut_ptr(), cls_buf.len() as i32);
    if len <= 0 {
        return false;
    }
    let cls = String::from_utf16_lossy(&cls_buf[..len as usize]).to_ascii_lowercase();
    (cls.contains("edit")) && (GetWindowLongW(h, GWL_STYLE) & ES_PASSWORD_STYLE) != 0
}

/// Forget the current composition without touching the text (the caret may be somewhere else now).
fn forget_composition(st: &mut AppSharedState) -> bool {
    let had = st.injected_utf16_len > 0 || !st.last_output.preedit.is_empty();
    st.ekushe.reset_composition();
    st.last_output = SessionOutput::default();
    st.injected_utf16_len = 0;
    st.comp_focus = 0;
    LAST_TYPED.lock().clear();
    had
}

/// `%APPDATA%\Ekushe\learned.tsv`: the words this user picked (roman → Bangla, with counts).
fn learned_words_path() -> PathBuf {
    Config::default_dir().join("learned.tsv")
}

/// `%APPDATA%\Ekushe\wrong-words.tsv`: words the user reported as wrong, with the word they meant
/// (`roman \t wrong \t right \t previous word`). Kept on this computer only; the user can copy it
/// into a problem report from Studio.
fn wrong_words_path() -> PathBuf {
    Config::default_dir().join("wrong-words.tsv")
}

/// Most corrections kept in the file (oldest are dropped).
const WRONG_WORDS_KEPT: usize = 500;

fn append_wrong_word(line: &str) -> std::io::Result<()> {
    let path = wrong_words_path();
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<&str> = old.lines().filter(|l| !l.trim().is_empty()).collect();
    lines.push(line);
    let skip = lines.len().saturating_sub(WRONG_WORDS_KEPT);
    let mut out = lines[skip..].join("\n");
    out.push('\n');
    ekushe_core::suggest::write_atomic(&path, &out)
}

/// Write learned words to disk if anything new was learned. The file is written outside the shared
/// lock (the keyboard hook needs it) and atomically (temporary file + rename).
fn save_learned_words_if_changed() {
    let pending = {
        let mut st = STATE.lock();
        let v = st.ekushe.suggestions_engine().learned_version();
        if v == st.saved_learned_version {
            None
        } else {
            st.saved_learned_version = v;
            Some(st.ekushe.suggestions_engine().user_dict_tsv())
        }
    };
    if let Some(tsv) = pending {
        let _ = ekushe_core::suggest::write_atomic(&learned_words_path(), &tsv);
    }
}

/// Lower-case executable file name of the process that owns `hwnd` (e.g. `winword.exe`).
unsafe fn exe_name_of(hwnd: HWND) -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == 0 {
        return None;
    }
    let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if h.is_null() {
        return None;
    }
    let mut buf = [0u16; 512];
    let mut len = buf.len() as u32;
    let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len) != 0;
    CloseHandle(h);
    if !ok {
        return None;
    }
    let full = String::from_utf16_lossy(&buf[..len as usize]);
    full.rsplit(['\\', '/']).next().map(|s| s.to_lowercase())
}

/// Low-level mouse hook: a click anywhere except the candidate popup ends the composition.
unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    const WM_LBUTTONDOWN_: usize = 0x0201;
    const WM_RBUTTONDOWN_: usize = 0x0204;
    const WM_MBUTTONDOWN_: usize = 0x0207;
    const WM_XBUTTONDOWN_: usize = 0x020B;
    if code >= 0
        && matches!(
            wparam,
            WM_LBUTTONDOWN_ | WM_RBUTTONDOWN_ | WM_MBUTTONDOWN_ | WM_XBUTTONDOWN_
        )
    {
        let ms = &*(lparam as *const MSLLHOOKSTRUCT);
        let target = WindowFromPoint(ms.pt);
        let root = if target.is_null() {
            target
        } else {
            GetAncestor(target, GA_ROOT)
        };
        let overlay = OVERLAY_HWND.load(Ordering::SeqCst);
        if overlay == 0 || (target as isize != overlay && root as isize != overlay) {
            COMPOSITION_STALE.store(true, Ordering::SeqCst);
            let tray = TRAY_HWND.load(Ordering::SeqCst);
            if tray != 0 {
                PostMessageW(tray as HWND, WM_EKUSHE_FORGET, 0, 0);
            }
        }
    }
    CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
}

/// Low-level keyboard hook (`WH_KEYBOARD_LL`) that powers:
/// 1. One-key toggle (`Double-Tap Shift`, `Ctrl+Space`, `F12`, `Ctrl+Alt+B`)
/// 2. Per-App Terminal Smart Guard (auto-English in PowerShell / cmd / Windows Terminal)
/// 3. `Ctrl+Backspace` to flip the last committed word between Bangla ⇄ English
/// 4. Live phonetic & fixed-layout Bangla typing in any Windows app (with Bangla digits ০-৯ in Ekushe mode and 0-9 in English mode)
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let r = keyboard_hook_inner(code, wparam, lparam);
    let ours = code >= 0 && (*(lparam as *const KBDLLHOOKSTRUCT)).dwExtraInfo == EKUSHE_EXTRA_INFO;
    // A key that reached the app unconverted changes the text before the caret: what was typed last
    // is no longer there to compare with (see `replace_active_text`)
    if r != 1 && code >= 0 && !ours && (wparam as u32 == WM_KEYDOWN || wparam as u32 == 0x0104) {
        let vk = (*(lparam as *const KBDLLHOOKSTRUCT)).vkCode;
        // Shift, Ctrl, Alt, the Windows keys and Caps Lock type nothing
        if !matches!(vk, 0x10..=0x12 | 0x14 | 0x5B | 0x5C | 0xA0..=0xA5) {
            LAST_TYPED.lock().clear();
        }
    }
    r
}

unsafe fn keyboard_hook_inner(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
    }

    let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
    if kb.dwExtraInfo == EKUSHE_EXTRA_INFO {
        return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
    }

    let vk = kb.vkCode as u16;
    let is_keydown = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
    let is_keyup = wparam == WM_KEYUP as usize;

    let ctrl_down = (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
    let alt_down = (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0;
    let shift_down = (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0;
    // VK_LWIN = 0x5B, VK_RWIN = 0x5C
    let win_down = (GetAsyncKeyState(0x5B) as u16 & 0x8000) != 0
        || (GetAsyncKeyState(0x5C) as u16 & 0x8000) != 0;

    // 1. Handle Switch Hotkey (Double-Tap Shift, Ctrl+Space, F12, Ctrl+Alt+B)
    let is_shift_key = vk == VK_SHIFT || vk == VK_LSHIFT || vk == VK_RSHIFT;
    if is_shift_key {
        let mut st = STATE.lock();
        if st.ekushe.config().switch_hotkey == SwitchHotkey::DoubleTapShift {
            if is_keydown {
                st.shift_clean_down = !ctrl_down && !alt_down;
            } else if is_keyup && st.shift_clean_down {
                st.shift_clean_down = false;
                let now_ms = st.start_instant.elapsed().as_millis() as u64;
                let pending = pending_word(&st);
                if let Some(new_lang) = st.ekushe.on_shift_tap(now_ms) {
                    st.injected_utf16_len = 0;
                    st.last_output = SessionOutput::default();
                    drop(st);
                    if let Some(w) = pending {
                        replace_active_text(0, &w);
                    }
                    request_tray_icon_update();
                    let msg = match new_lang {
                        InputLanguage::Bangla => "বাং · বাংলায় লিখছেন",
                        InputLanguage::English => "EN · English typing",
                    };
                    request_badge(msg);
                    return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
                }
            }
        }
        return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
    }

    if is_keydown {
        let mut st = STATE.lock();
        st.shift_clean_down = false;

        // The caret may have moved (mouse click) or focus changed (another field or window) since the
        // half-typed word was injected: forget it rather than backspacing text somewhere else.
        let focus = focused_control();
        let stale = COMPOSITION_STALE.swap(false, Ordering::SeqCst);
        if st.injected_utf16_len > 0 && (stale || st.comp_focus != focus as isize) {
            forget_composition(&mut st);
            drop(st);
            request_overlay_refresh();
            st = STATE.lock();
        } else if stale || st.comp_focus != focus as isize {
            // The last word is no longer right before the caret: Backspace must not take it back
            st.ekushe.forget_undo();
        }
        // Per-app language: switching to another application restores the language last used there
        // (or its configured default, e.g. terminals → English)
        let fg = GetForegroundWindow();
        if fg as isize != st.last_fg {
            st.last_fg = fg as isize;
            if let Some(exe) = exe_name_of(fg) {
                let before = st.ekushe.language();
                let after = st.ekushe.on_app_focus(&exe);
                if after != before {
                    forget_composition(&mut st);
                    drop(st);
                    request_tray_icon_update();
                    request_overlay_refresh();
                    st = STATE.lock();
                }
            }
        }
        st.comp_focus = focus as isize;
        // Enter, arrows, Delete, Tab, Home/End, shortcuts and English-mode letters reach the
        // application without passing through the engine: after them, the last word is no longer
        // right before the caret, so Backspace must not take it back. (Letters and Space in Bangla
        // go to the engine, which handles this itself; modifier keys alone change nothing, and
        // Ctrl+Backspace still flips the last word.)
        let modifier_only = matches!(vk, 0x10 | 0x11 | 0x12 | 0x14 | 0x5B | 0x5C | 0xA0..=0xA5);
        if vk != VK_BACK && !modifier_only && st.injected_utf16_len == 0 && st.ekushe.has_undo() {
            let to_engine = st.ekushe.language() == InputLanguage::Bangla
                && !ctrl_down
                && !alt_down
                && !win_down
                && vk_to_char(vk, kb.scanCode, shift_down, false).is_some_and(|c| !c.is_control());
            if !to_engine {
                st.ekushe.forget_undo();
            }
        }

        let switch_triggered = match &st.ekushe.config().switch_hotkey {
            SwitchHotkey::CtrlSpace => ctrl_down && !alt_down && vk == VK_SPACE,
            SwitchHotkey::F12 => vk == VK_F12,
            SwitchHotkey::CtrlAltB => ctrl_down && alt_down && vk == b'B' as u16,
            _ => false,
        };

        if switch_triggered {
            let pending = pending_word(&st);
            let new_lang = st.ekushe.toggle_language();
            st.injected_utf16_len = 0;
            st.last_output = SessionOutput::default();
            drop(st);
            if let Some(w) = pending {
                replace_active_text(0, &w);
            }
            request_tray_icon_update();
            let msg = match new_lang {
                InputLanguage::Bangla => "বাং · বাংলায় লিখছেন",
                InputLanguage::English => "EN · English typing",
            };
            request_badge(msg);
            return 1;
        }

        // Password boxes (classic ones, and those in browsers and Store apps that UI Automation
        // reports): type plain characters, no preview, nothing learned. The language hotkey above
        // still works here.
        secure_field::set_active(st.ekushe.language() == InputLanguage::Bangla);
        let root = GetAncestor(focus, GA_ROOT) as isize;
        if is_password_field(focus) || secure_field::is_secret(root) {
            if st.injected_utf16_len > 0 || !st.last_output.preedit.is_empty() {
                forget_composition(&mut st);
                drop(st);
                request_overlay_refresh();
            }
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // Convert the selected text (Bijoy or Banglish) in any app, in either language
        let convert_hotkey = selection::Hotkey::parse(&st.ekushe.config().convert_selection_hotkey);
        if convert_hotkey
            .is_some_and(|hk| hk.matches(vk, ctrl_down, alt_down, shift_down, win_down))
        {
            if st.injected_utf16_len > 0 || !st.last_output.preedit.is_empty() {
                forget_composition(&mut st);
            }
            let tray = st.tray_hwnd as HWND;
            drop(st);
            request_overlay_refresh();
            PostMessageW(tray, WM_EKUSHE_CONVERT_SELECTION, 0, 0);
            return 1;
        }

        // Ctrl+Backspace in Bangla mode: Flip the last committed word between Bangla ⇄ English!
        if st.ekushe.language() == InputLanguage::Bangla
            && ctrl_down
            && !alt_down
            && vk == VK_BACK
            && st.injected_utf16_len == 0
        {
            if let Some((del_len, replacement)) = st.ekushe.flip_last_committed_word() {
                drop(st);
                replace_active_text(del_len, &replacement);
                return 1;
            }
        }

        // Keyboard shortcuts belong to the application: Ctrl+…, Alt+… (menus) and Win+…; AltGr
        // (Ctrl+Alt) still types. A half-typed word stays as it is on screen and is closed.
        let is_win_key = vk == 0x5B || vk == 0x5C;
        let shortcut = (ctrl_down != alt_down) || win_down || is_win_key;
        if shortcut {
            if st.injected_utf16_len > 0 {
                let pending = pending_word(&st);
                forget_composition(&mut st);
                drop(st);
                request_overlay_refresh();
                if let Some(w) = pending {
                    // the word first, then the key itself (it would otherwise reach the app first)
                    replace_then_key(0, &w, vk, kb.flags);
                    return 1;
                }
            }
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }
        // English mode types plain characters. (Terminals start in English through the per-app
        // language; Bangla still works there after switching.)
        if st.ekushe.language() == InputLanguage::English {
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // If navigation keys (PageUp/PageDown/End/Home/Left/Up/Right/Down: 0x21..=0x28) are pressed while composing,
        // keep the injected text intact and close the composition buffer cleanly.
        if (0x21..=0x28).contains(&vk) && st.injected_utf16_len > 0 {
            let pending = pending_word(&st);
            st.ekushe.reset_composition();
            st.last_output = SessionOutput::default();
            st.injected_utf16_len = 0;
            drop(st);
            request_overlay_refresh();
            if let Some(w) = pending {
                replace_then_key(0, &w, vk, kb.flags);
                return 1;
            }
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // Handle Escape: cancel current composition popup
        if vk == VK_ESCAPE && !st.last_output.preedit.is_empty() {
            st.ekushe.reset_composition();
            st.last_output = SessionOutput::default();
            st.injected_utf16_len = 0;
            drop(st);
            request_overlay_refresh();
            return 1;
        }

        // Handle Tab: cycle through candidates in the popup!
        if vk == VK_TAB && st.last_output.candidates.len() > 1 {
            let next_idx = (st.last_output.selected_index + 1) % st.last_output.candidates.len();
            st.last_output.selected_index = next_idx;
            let cand_text = st.last_output.candidates[next_idx].text.clone();
            let prev_len = st.injected_utf16_len;
            let live = st.ekushe.config().type_while_composing;
            st.injected_utf16_len = cand_text.encode_utf16().count();
            drop(st);
            show_word(live, prev_len, &cand_text);
            request_overlay_refresh();
            return 1;
        }

        // Handle Backspace while composing
        if vk == VK_BACK {
            let prev_len = st.injected_utf16_len;
            // Right after a word was committed with a space: reopen it as the word being typed, with
            // its suggestions (another Bangla word, or the English as typed), so a wrong word can be
            // fixed without typing it again. Backspace again deletes a letter.
            if prev_len == 0 && st.ekushe.config().backspace_undo {
                if let Some(out) = st.ekushe.reopen_last_word() {
                    let preedit = out.preedit.clone();
                    let delete = out.delete_before;
                    let live = st.ekushe.config().type_while_composing;
                    st.injected_utf16_len = preedit.encode_utf16().count();
                    st.last_output = out;
                    drop(st);
                    // the word and its space leave the app; it is now the word being typed
                    replace_active_text(delete, if live { &preedit } else { "" });
                    request_overlay_refresh();
                    return 1;
                }
            }
            if prev_len > 0 {
                let out = st.ekushe.backspace();
                let new_text = out.preedit.clone();
                let live = st.ekushe.config().type_while_composing;
                st.injected_utf16_len = new_text.encode_utf16().count();
                st.last_output = out;
                drop(st);
                show_word(live, prev_len, &new_text);
                request_overlay_refresh();
                return 1;
            }
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // Handle Space or Enter when composing
        if (vk == VK_SPACE || vk == VK_RETURN) && st.injected_utf16_len > 0 {
            let sel = st.last_output.selected_index;
            let trailing = if vk == VK_SPACE { Some(' ') } else { None };
            let prev_len = st.injected_utf16_len;
            let live = st.ekushe.config().type_while_composing;
            let out = st.ekushe.select_candidate(sel, trailing);
            st.injected_utf16_len = 0;
            let committed = out.committed.clone().unwrap_or_default();
            st.last_output = SessionOutput::default();
            drop(st);
            let typed_len = if live { prev_len } else { 0 };
            if vk == VK_RETURN {
                // the word, then Enter, in that order (a passed-on Enter would arrive first)
                replace_then_key(typed_len, &committed, VK_RETURN, kb.flags);
            } else {
                replace_active_text(typed_len, &committed);
            }
            request_overlay_refresh();
            return 1;
        }

        // Translate virtual key to printable ASCII/Unicode character (including Numpad 0-9!)
        if let Some(ch) = vk_to_char(vk, kb.scanCode, shift_down, ctrl_down && alt_down) {
            if !ch.is_control() {
                let prev_len = st.injected_utf16_len;
                let live = st.ekushe.config().type_while_composing;
                let altgr = ctrl_down && alt_down;
                let out = st.ekushe.push_char(ch, altgr);
                if let Some(committed) = &out.committed {
                    st.injected_utf16_len = 0;
                    let to_send = committed.clone();
                    st.last_output = SessionOutput::default();
                    drop(st);
                    replace_active_text(if live { prev_len } else { 0 }, &to_send);
                    request_overlay_refresh();
                    return 1;
                } else {
                    let preedit = out.preedit.clone();
                    st.injected_utf16_len = preedit.encode_utf16().count();
                    st.last_output = out;
                    drop(st);
                    show_word(live, prev_len, &preedit);
                    request_overlay_refresh();
                    return 1;
                }
            }
        }
    }

    CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
}

unsafe fn vk_to_char(vk: u16, scan_code: u32, shift: bool, altgr: bool) -> Option<char> {
    // Support Numpad 0..=9 (VK_NUMPAD0 = 0x60 .. VK_NUMPAD9 = 0x69) directly so Numpad also types বাংলা ০-৯ in Ekushe mode!
    if !shift && (0x60..=0x69).contains(&vk) {
        return char::from_u32(('0' as u32) + (vk as u32 - 0x60));
    }
    // GetKeyboardState() describes this hook thread's queue, not the key the user just pressed, so the
    // modifier state is built from the physical keys instead. Caps Lock is a toggle: GetKeyState's low bit.
    let mut kbd_state = [0u8; 256];
    if shift {
        kbd_state[VK_SHIFT as usize] = 0x80;
    }
    if altgr {
        kbd_state[VK_CONTROL as usize] = 0x80;
        kbd_state[VK_MENU as usize] = 0x80;
    }
    if (GetKeyState(0x14) & 1) != 0 {
        kbd_state[0x14] = 0x01; // VK_CAPITAL toggled
    }
    // Translate with the layout of the window the user is typing into (the hook thread's may differ)
    let fg_thread = GetWindowThreadProcessId(GetForegroundWindow(), ptr::null_mut());
    let layout = GetKeyboardLayout(fg_thread);
    let mut buf = [0u16; 4];
    // Flag 0x4: do not change the keyboard state, so dead keys typed in the application still work
    let rc = ToUnicodeEx(
        vk as u32,
        scan_code,
        kbd_state.as_ptr(),
        buf.as_mut_ptr(),
        buf.len() as i32,
        0x4,
        layout,
    );
    if rc == 1 {
        char::from_u32(buf[0] as u32)
    } else {
        None
    }
}

/// The word being typed changed: with `type_while_composing` it is retyped in the app, otherwise
/// it shows only in the suggestion bar until it is placed.
unsafe fn show_word(live: bool, prev_len: usize, text: &str) {
    if live {
        replace_active_text(prev_len, text);
    }
}

/// The word being typed, when it exists only in the suggestion bar (not yet typed into the app):
/// what a key that closes it (a shortcut, an arrow, switching language) must type first
fn pending_word(st: &AppSharedState) -> Option<String> {
    let w = &st.last_output.preedit;
    (!st.ekushe.config().type_while_composing && st.injected_utf16_len > 0 && !w.is_empty())
        .then(|| w.clone())
}

/// Like `replace_active_text`, then the key `vk` itself, in one batch so the app gets them in that
/// order (the key the user pressed is held back by the hook; the key-up that follows is theirs)
unsafe fn replace_then_key(backspaces: usize, text: &str, vk: u16, kb_flags: u32) {
    replace_active_text(backspaces, text);
    // LLKHF_EXTENDED (arrows, Home, End, Page keys, the right Enter) keeps its meaning
    let ext = if kb_flags & 1 != 0 { 0x0001 } else { 0 };
    let key = [make_vk_input(vk, ext)];
    SendInput(1, key.as_ptr(), size_of::<INPUT>() as i32);
}

/// What the last `replace_active_text` typed (to send only the part of a word that changed)
static LAST_TYPED: Mutex<String> = Mutex::new(String::new());

/// Atomically send `backspaces` followed by `new_text` as UTF-16 `KEYEVENTF_UNICODE` events.
///
/// When the text being replaced is what was typed last (the word being typed), only the part after
/// what the two have in common is deleted and retyped: `স্বপ্ন` → `স্বপ্নপুরী` adds `পুরী` instead of
/// deleting six letters and typing ten. Fewer keys reach the app, and fewer Backspaces land inside a
/// conjunct, where apps differ in how much one Backspace deletes.
unsafe fn replace_active_text(backspaces: usize, new_text: &str) {
    let (backspaces, new_text) = {
        let mut last = LAST_TYPED.lock();
        let old = std::mem::replace(&mut *last, new_text.to_string());
        if backspaces > 0 && backspaces == old.encode_utf16().count() {
            let keep: usize = old
                .chars()
                .zip(new_text.chars())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a.len_utf8())
                .sum();
            let kept16 = old[..keep].encode_utf16().count();
            (backspaces - kept16, new_text[keep..].to_string())
        } else {
            (backspaces, new_text.to_string())
        }
    };
    let new_text = new_text.as_str();
    let utf16: Vec<u16> = new_text.encode_utf16().collect();
    let total_events = (backspaces * 2) + (utf16.len() * 2);
    if total_events == 0 {
        return;
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(total_events);

    for _ in 0..backspaces {
        inputs.push(make_vk_input(VK_BACK, 0));
        inputs.push(make_vk_input(VK_BACK, KEYEVENTF_KEYUP));
    }

    for &unit in &utf16 {
        inputs.push(make_unicode_input(unit, 0));
        inputs.push(make_unicode_input(unit, KEYEVENTF_KEYUP));
    }

    let sent = SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        size_of::<INPUT>() as i32,
    );
    if sent as usize != inputs.len() {
        // Windows blocked some or all keystrokes (e.g. the target runs as administrator). What reached
        // the app is unknown, so drop the composition instead of backspacing blindly next time.
        let mut st = STATE.lock();
        forget_composition(&mut st);
        drop(st);
        request_badge("এই উইন্ডোতে লেখা পাঠানো যায়নি — Windows আটকে দিয়েছে");
    }
}

unsafe fn make_vk_input(vk: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: EKUSHE_EXTRA_INFO,
            },
        },
    }
}

unsafe fn make_unicode_input(scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: scan,
                dwFlags: KEYEVENTF_UNICODE | flags,
                time: 0,
                dwExtraInfo: EKUSHE_EXTRA_INFO,
            },
        },
    }
}

/// Window procedure for the System Tray Host Window.
unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER if wparam == TIMER_SAVE_LEARNED => {
            save_learned_words_if_changed();
            0
        }
        WM_TIMER if wparam == TIMER_REHOOK => {
            rehook();
            0
        }
        WM_EKUSHE_REFRESH_OVERLAY => {
            refresh_candidate_overlay();
            0
        }
        WM_EKUSHE_SHOW_BADGE => {
            let banner = STATE.lock().badge_banner.clone();
            if let Some(b) = banner {
                show_mode_badge(&b);
            }
            0
        }
        WM_EKUSHE_UPDATE_TRAY_ICON => {
            add_or_update_tray_icon(hwnd, false);
            0
        }
        WM_EKUSHE_OPEN_STUDIO => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, None);
            0
        }
        WM_EKUSHE_CONVERT_SELECTION => {
            let tray = hwnd as isize;
            selection::start(
                |w| STATE.lock().ekushe.banglish_word(w),
                move |outcome| {
                    let code = match outcome {
                        selection::Outcome::Converted(
                            ekushe_core::convert::SelectionKind::BijoyToUnicode,
                        ) => 0,
                        selection::Outcome::Converted(_) => 1,
                        selection::Outcome::NothingSelected => 2,
                        selection::Outcome::NothingToConvert => 3,
                        selection::Outcome::TooLong => 4,
                        selection::Outcome::Failed => 5,
                    };
                    unsafe { PostMessageW(tray as HWND, WM_EKUSHE_SELECTION_DONE, code, 0) };
                },
            );
            0
        }
        WM_EKUSHE_SELECTION_DONE => {
            show_mode_badge(selection_badge(wparam));
            0
        }
        WM_EKUSHE_FORGET => {
            if COMPOSITION_STALE.swap(false, Ordering::SeqCst) {
                let had = {
                    let mut st = STATE.lock();
                    forget_composition(&mut st)
                };
                if had {
                    refresh_candidate_overlay();
                }
            }
            0
        }
        WM_TRAYICON => {
            let event = (lparam & 0xFFFF) as u32;
            match event {
                WM_LBUTTONUP => {
                    // Single click on tray icon toggles Bangla <-> English immediately!
                    let new_lang = {
                        let mut st = STATE.lock();
                        st.injected_utf16_len = 0;
                        st.last_output = SessionOutput::default();
                        st.ekushe.toggle_language()
                    };
                    add_or_update_tray_icon(hwnd, false);
                    let banner = match new_lang {
                        InputLanguage::Bangla => "বাং · বাংলায় লিখছেন",
                        InputLanguage::English => "EN · English typing",
                    };
                    show_mode_badge(banner);
                }
                WM_LBUTTONDBLCLK => {
                    let port = STATE.lock().studio_port;
                    open_studio_window_tab(port, None);
                }
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    show_tray_context_menu(hwnd);
                }
                // The "new version" notice was clicked
                windows_sys::Win32::UI::Shell::NIN_BALLOONUSERCLICK
                    if update_check::AVAILABLE.lock().is_some() =>
                {
                    open_in_browser(update_check::DOWNLOAD_PAGE);
                }
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            let cmd = wparam & 0xFFFF;
            handle_tray_command(hwnd, cmd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_context_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    let st = STATE.lock();
    let is_bn = st.ekushe.language() == InputLanguage::Bangla;
    let cfg = st.ekushe.config().clone();
    drop(st);

    // 1. Primary Bold Action at Top: Open Ekushe Studio + Direct Studio Tool Shortcuts
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_OPEN_STUDIO,
        wide_null("✨ একুশে নেক্সট স্টুডিও খুলুন (Open Ekushe Next Studio)").as_ptr(),
    );
    let _ = SetMenuDefaultItem(menu, CMD_OPEN_STUDIO as u32, 0);

    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_PLAYGROUND,
        wide_null("⌨️ স্মার্ট বাংলা রাইটিং ও যুক্তবর্ণ প্যাড (Smart Writing Pad)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_XRAY,
        wide_null("🔬 শব্দ এক্স-রে ও ২-ক্লিক যুক্তবর্ণ নির্মাতা (Word X-Ray & Conjunct Builder)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_SIGNS,
        wide_null("🔤 যুক্তবর্ণ ও চিহ্নের সহায়িকা (+  ,,  ^ …)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_SPELL,
        wide_null("✔ বাংলা বানান পরীক্ষক (Bangla Spell Checker)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_CONVERT,
        wide_null("🔄 বিজয় ⇄ ইউনিকোড কনভার্টার (Bijoy / Unicode Converter)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_FIX,
        wide_null("✏️ ভুল শব্দ ঠিক করুন (A word was wrong)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STUDIO_SETTINGS,
        wide_null("⚙️ সেটিংস, ফন্ট ও স্বচ্ছতা (Settings, Fonts & Opacity)").as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

    // 2. Language Toggle (Bangla <-> English)
    let toggle_label = if is_bn {
        format!(
            "✓ বাংলা সক্রিয় (Bangla Active) — Switch: {}",
            cfg.switch_hotkey.display_name()
        )
    } else {
        format!(
            "বাংলা চালু করুন (Currently EN) — Switch: {}",
            cfg.switch_hotkey.display_name()
        )
    };
    AppendMenuW(
        menu,
        MF_STRING | if is_bn { MF_CHECKED } else { MF_UNCHECKED },
        CMD_TOGGLE_LANG,
        wide_null(&toggle_label).as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

    // 3. Keyboard Layouts
    for (id, cmd_id) in [
        (LayoutId::AvroPhonetic, CMD_LAYOUT_AVRO),
        (LayoutId::Probhat, CMD_LAYOUT_PROBHAT),
        (LayoutId::NationalJatiya, CMD_LAYOUT_JATIYA),
        (LayoutId::Bornona, CMD_LAYOUT_BORNONA),
        (LayoutId::MunirOptima, CMD_LAYOUT_MUNIR),
    ] {
        let flag = if cfg.active_layout == id {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        AppendMenuW(
            menu,
            MF_STRING | flag,
            cmd_id,
            wide_null(&format!("লেআউট: {}", id.display_name())).as_ptr(),
        );
    }
    AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

    // 4. Typing Modes
    for (m, label, cmd_id) in [
        (
            TypingMode::PhoneticFirst,
            "মোড: Phonetic-First",
            CMD_MODE_PHONETIC_FIRST,
        ),
        (TypingMode::Smart, "মোড: Smart (Default)", CMD_MODE_SMART),
        (
            TypingMode::PhoneticOnly,
            "মোড: Phonetic-Only (No Popup)",
            CMD_MODE_PHONETIC_ONLY,
        ),
    ] {
        let flag = if cfg.typing_mode == m {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        AppendMenuW(menu, MF_STRING | flag, cmd_id, wide_null(label).as_ptr());
    }
    AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

    // 5. Switch Hotkey Presets
    for (hk, cmd_id) in [
        (SwitchHotkey::DoubleTapShift, CMD_HOTKEY_DBL_SHIFT),
        (SwitchHotkey::CtrlSpace, CMD_HOTKEY_CTRL_SPACE),
        (SwitchHotkey::F12, CMD_HOTKEY_F12),
        (SwitchHotkey::CtrlAltB, CMD_HOTKEY_CTRL_ALT_B),
    ] {
        let flag = if cfg.switch_hotkey == hk {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        AppendMenuW(
            menu,
            MF_STRING | flag,
            cmd_id,
            wide_null(&format!("হটকি: {}", hk.display_name())).as_ptr(),
        );
    }

    AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());
    if let Some(tag) = update_check::AVAILABLE.lock().clone() {
        AppendMenuW(
            menu,
            MF_STRING,
            CMD_UPDATE_DOWNLOAD,
            wide_null(&format!(
                "⬇️ নতুন সংস্করণ {tag} এসেছে — ডাউনলোড করুন (Update available)"
            ))
            .as_ptr(),
        );
    }
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_CHECK_UPDATE,
        wide_null(&format!(
            "🔄 হালনাগাদ দেখুন (Check for updates) — {}",
            update_check::current_version()
        ))
        .as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_REPORT,
        wide_null("🐞 সমস্যা জানান (Report a problem)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_EXIT,
        wide_null("প্রস্থান করুন (Exit Ekushe Next)").as_ptr(),
    );

    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    SetForegroundWindow(hwnd);
    let selected = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
        pt.x,
        pt.y,
        0,
        hwnd,
        ptr::null(),
    );
    let _ = PostMessageW(hwnd, WM_NULL, 0, 0);
    DestroyMenu(menu);

    if selected > 0 {
        handle_tray_command(hwnd, selected as usize);
    }
}

unsafe fn handle_tray_command(hwnd: HWND, cmd: usize) {
    match cmd {
        CMD_TOGGLE_LANG => {
            let new_lang = STATE.lock().ekushe.toggle_language();
            add_or_update_tray_icon(hwnd, false);
            let msg = match new_lang {
                InputLanguage::Bangla => "বাং · বাংলায় লিখছেন",
                InputLanguage::English => "EN · English typing",
            };
            show_mode_badge(msg);
        }
        CMD_OPEN_STUDIO => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, None);
        }
        CMD_STUDIO_PLAYGROUND => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("playground"));
        }
        CMD_STUDIO_XRAY => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("xray"));
        }
        CMD_STUDIO_SIGNS => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("signs"));
        }
        CMD_STUDIO_SPELL => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("spell"));
        }
        CMD_STUDIO_CONVERT => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("converter"));
        }
        CMD_STUDIO_SETTINGS => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("settings"));
        }
        CMD_STUDIO_FIX => {
            let port = STATE.lock().studio_port;
            open_studio_window_tab(port, Some("fix"));
        }
        CMD_HOTKEY_DBL_SHIFT | CMD_HOTKEY_CTRL_SPACE | CMD_HOTKEY_F12 | CMD_HOTKEY_CTRL_ALT_B => {
            let hk = match cmd {
                CMD_HOTKEY_CTRL_SPACE => SwitchHotkey::CtrlSpace,
                CMD_HOTKEY_F12 => SwitchHotkey::F12,
                CMD_HOTKEY_CTRL_ALT_B => SwitchHotkey::CtrlAltB,
                _ => SwitchHotkey::DoubleTapShift,
            };
            let hk_name = hk.display_name().to_string();
            let mut st = STATE.lock();
            let mut cfg = st.ekushe.config().clone();
            cfg.switch_hotkey = hk;
            let _ = cfg.save_to(&Config::default_config_path());
            st.ekushe.set_config(cfg);
            drop(st);
            add_or_update_tray_icon(hwnd, false);
            show_mode_badge(&format!("বদলানোর কী: {hk_name}"));
        }
        CMD_MODE_PHONETIC_FIRST | CMD_MODE_SMART | CMD_MODE_PHONETIC_ONLY => {
            let (m, label) = match cmd {
                CMD_MODE_PHONETIC_FIRST => (TypingMode::PhoneticFirst, "Phonetic-First"),
                CMD_MODE_PHONETIC_ONLY => (TypingMode::PhoneticOnly, "Phonetic-Only"),
                _ => (TypingMode::Smart, "Smart (Default)"),
            };
            let mut st = STATE.lock();
            let mut cfg = st.ekushe.config().clone();
            cfg.typing_mode = m;
            let _ = cfg.save_to(&Config::default_config_path());
            st.ekushe.set_config(cfg);
            drop(st);
            show_mode_badge(&format!("টাইপিং মোড: {label}"));
        }
        CMD_LAYOUT_AVRO | CMD_LAYOUT_PROBHAT | CMD_LAYOUT_JATIYA | CMD_LAYOUT_BORNONA
        | CMD_LAYOUT_MUNIR => {
            let lay = match cmd {
                CMD_LAYOUT_PROBHAT => LayoutId::Probhat,
                CMD_LAYOUT_JATIYA => LayoutId::NationalJatiya,
                CMD_LAYOUT_BORNONA => LayoutId::Bornona,
                CMD_LAYOUT_MUNIR => LayoutId::MunirOptima,
                _ => LayoutId::AvroPhonetic,
            };
            let lay_name = lay.display_name().to_string();
            let mut st = STATE.lock();
            let mut cfg = st.ekushe.config().clone();
            cfg.active_layout = lay;
            let _ = cfg.save_to(&Config::default_config_path());
            st.ekushe.set_config(cfg);
            drop(st);
            show_mode_badge(&format!("কীবোর্ড লেআউট: {lay_name}"));
        }
        CMD_CHECK_UPDATE => update_check::check_now_and_tell(),
        CMD_UPDATE_DOWNLOAD => open_in_browser(update_check::DOWNLOAD_PAGE),
        CMD_REPORT => {
            let url = ekushe_core::report_url(&windows_name(), STATE.lock().ekushe.config());
            open_in_browser(&url);
        }
        CMD_EXIT => {
            DestroyWindow(hwnd);
        }
        _ => {}
    }
}

#[inline]
fn pt_inside(r: &RECT, x: i32, y: i32) -> bool {
    x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
}

/// Window procedure for the Modern 2026 Floating Smart Pill Bar (Candidates + Connected-Letter Chips + Mode Toast).
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    const MA_NOACTIVATE: isize = 3;

    match msg {
        WM_MOUSEACTIVATE => MA_NOACTIVATE,
        WM_SETCURSOR => {
            SetCursor(LoadCursorW(ptr::null_mut(), IDC_HAND));
            1
        }
        WM_TIMER => {
            if wparam == TIMER_HIDE_BADGE {
                KillTimer(hwnd, TIMER_HIDE_BADGE);
                ShowWindow(hwnd, SW_HIDE);
            }
            0
        }
        WM_LBUTTONDOWN => {
            let mx = (lparam as u32 & 0xFFFF) as i16 as i32;
            let my = ((lparam as u32 >> 16) & 0xFFFF) as i16 as i32;

            let mut clicked_cand: Option<usize> = None;
            let mut clicked_hint: Option<String> = None;
            let mut clicked_helper = false;
            let mut clicked_compact = false;
            let studio_port;

            {
                let st = STATE.lock();
                studio_port = st.studio_port;
                if pt_inside(&st.helper_btn_rect, mx, my) {
                    clicked_helper = true;
                } else if pt_inside(&st.compact_btn_rect, mx, my) {
                    clicked_compact = true;
                } else {
                    for (r, idx) in &st.cand_hit_rects {
                        if pt_inside(r, mx, my) {
                            clicked_cand = Some(*idx);
                            break;
                        }
                    }
                    if clicked_cand.is_none() {
                        for (r, hint_key) in &st.hint_hit_rects {
                            if pt_inside(r, mx, my) {
                                clicked_hint = Some(hint_key.clone());
                                break;
                            }
                        }
                    }
                }
            }

            if clicked_helper {
                open_studio_window_tab(studio_port, Some("signs"));
                return 0;
            }

            if clicked_compact {
                {
                    let mut st = STATE.lock();
                    let mut cfg = st.ekushe.config().clone();
                    cfg.compact_popup = !cfg.compact_popup;
                    let _ = cfg.save_to(&Config::default_config_path());
                    st.ekushe.set_config(cfg.clone());
                    if cfg.compact_popup {
                        // keep only the tips for words on offer (they show in the compact popup too)
                        let out = &mut st.last_output;
                        let cands: Vec<String> =
                            out.candidates.iter().map(|c| c.text.clone()).collect();
                        out.sign_hints.retain(|h| cands.contains(&h.result_bangla));
                    } else {
                        let roman = st.last_output.roman_buffer.clone();
                        let pre = st.last_output.preedit.clone();
                        st.last_output.sign_hints =
                            ekushe_core::conjuncts::contextual_sign_hints(&roman, &pre);
                    }
                }
                refresh_candidate_overlay();
                return 0;
            }

            if let Some(cand_idx) = clicked_cand {
                let mut st = STATE.lock();
                let prev_len = st.injected_utf16_len;
                let live = st.ekushe.config().type_while_composing;
                let out = st.ekushe.select_candidate(cand_idx, Some(' '));
                st.injected_utf16_len = 0;
                let committed = out.committed.clone().unwrap_or_default();
                st.last_output = SessionOutput::default();
                drop(st);
                replace_active_text(if live { prev_len } else { 0 }, &committed);
                refresh_candidate_overlay();
                return 0;
            }

            if let Some(hint_roman) = clicked_hint {
                let mut st = STATE.lock();
                let prev_len = st.injected_utf16_len;
                st.ekushe.reset_composition();
                let mut last_out = SessionOutput::default();
                for ch in hint_roman.chars() {
                    last_out = st.ekushe.push_char(ch, false);
                }
                let preedit = last_out.preedit.clone();
                let live = st.ekushe.config().type_while_composing;
                st.injected_utf16_len = preedit.encode_utf16().count();
                st.last_output = last_out;
                drop(st);
                show_word(live, prev_len, &preedit);
                refresh_candidate_overlay();
                return 0;
            }

            0
        }
        WM_PAINT => {
            // Content arrives through UpdateLayeredWindow; nothing to paint here
            ValidateRect(hwnd, ptr::null());
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Check if an existing Ekushe Studio HTTP server is already listening on localhost.
fn check_existing_studio_port() -> Option<u16> {
    let port_file = setup_wizard::install_root_dir().join("studio_port.txt");
    let text = std::fs::read_to_string(port_file).ok()?;
    let port: u16 = text.trim().parse().ok()?;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    if TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(120)).is_ok() {
        Some(port)
    } else {
        None
    }
}

/// Start the embedded local loopback HTTP server for Ekushe Studio.
fn start_studio_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:45210")
        .or_else(|_| TcpListener::bind("127.0.0.1:0"))
        .expect("bind localhost studio port");
    let port = listener.local_addr().unwrap().port();

    let port_file = setup_wizard::install_root_dir().join("studio_port.txt");
    let _ = std::fs::write(port_file, port.to_string());

    // One thread per connection (at most a few at once), every request bounded in size and time
    std::thread::spawn(move || {
        studio_http::serve(listener, |mut stream| {
            match studio_http::read_request(&mut stream) {
                Ok(req) => {
                    let _ = handle_http_client(stream, &req);
                }
                Err(studio_http::ReadError::Status(st)) => {
                    studio_http::respond_error(&mut stream, st)
                }
                Err(studio_http::ReadError::Closed) => {}
            }
        })
    });

    port
}

/// Key the Studio page sends with every `/api/*` request. Other web pages open in the browser can
/// reach 127.0.0.1 too; without the key they cannot read settings or change them. The page only
/// carries it when Ekushe opened it with a one-time launch code (see `studio_http.rs`).
fn studio_token() -> &'static str {
    studio_http::token()
}

/// Value of an HTTP header (case-insensitive name), trimmed.
fn http_header<'a>(req: &'a str, name: &str) -> Option<&'a str> {
    let head = &req[..req.find("\r\n\r\n").unwrap_or(req.len())];
    head.lines().skip(1).find_map(|line| {
        let (k, v) = line.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim())
    })
}

/// Only pages served by this Studio may talk to it: the Host must be loopback (blocks DNS rebinding),
/// a browser-sent Origin must be this server, and API calls must carry the page's token.
fn studio_request_allowed(req: &str, method: &str, clean_path: &str, port: u16) -> bool {
    let allowed_hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    match http_header(req, "Host") {
        Some(h) if allowed_hosts.iter().any(|a| a.eq_ignore_ascii_case(h)) => {}
        _ => return false,
    }
    if let Some(origin) = http_header(req, "Origin") {
        let ok = allowed_hosts
            .iter()
            .any(|a| origin.eq_ignore_ascii_case(&format!("http://{a}")));
        if !ok {
            return false;
        }
    }
    if clean_path.starts_with("/api/") {
        return http_header(req, "X-Ekushe-Token") == Some(studio_token());
    }
    // Everything else is read-only page content
    method == "GET"
}

fn handle_http_client(mut stream: TcpStream, req: &str) -> std::io::Result<()> {
    let req = req.to_string();
    if req.is_empty() {
        return Ok(());
    }
    let first_line = req.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0];
    let path = parts[1];
    let clean_path = path.split('?').next().unwrap_or("/");
    let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
    let body = &req[body_start..];

    let port = stream.local_addr().map(|a| a.port()).unwrap_or(0);
    if !studio_request_allowed(&req, method, clean_path, port) {
        return write_http_response(
            &mut stream,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            b"Forbidden",
        );
    }

    if method == "GET" && (clean_path == "/" || clean_path.starts_with("/index")) {
        let html = studio_http::studio_page(studio_ui::STUDIO_HTML, path);
        return write_http_response(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            html.as_bytes(),
        );
    }

    if method == "GET" && (clean_path == "/logo.png" || clean_path == "/favicon.ico") {
        return write_http_response(&mut stream, "200 OK", "image/png", LOGO_PNG_BYTES);
    }

    // Serve bundled Bangladeshi & Bengali OpenType TTF fonts directly from memory
    if method == "GET" && clean_path == "/fonts/Kalpurush.ttf" {
        return write_http_response(&mut stream, "200 OK", "font/ttf", FONT_KALPURUSH);
    }
    if method == "GET" && clean_path == "/fonts/SolaimanLipi.ttf" {
        return write_http_response(&mut stream, "200 OK", "font/ttf", FONT_SOLAIMANLIPI);
    }
    if method == "GET" && clean_path == "/fonts/SiyamRupali.ttf" {
        return write_http_response(&mut stream, "200 OK", "font/ttf", FONT_SIYAMRUPALI);
    }
    if method == "GET" && clean_path == "/fonts/HindSiliguri-Regular.ttf" {
        return write_http_response(&mut stream, "200 OK", "font/ttf", FONT_HINDSILIGURI_REG);
    }
    if method == "GET" && clean_path == "/fonts/HindSiliguri-Bold.ttf" {
        return write_http_response(&mut stream, "200 OK", "font/ttf", FONT_HINDSILIGURI_BOLD);
    }

    if method == "GET" && clean_path == "/api/state" {
        let st = STATE.lock();
        let cfg = st.ekushe.config();
        let lang = match st.ekushe.language() {
            InputLanguage::Bangla => "Bangla",
            InputLanguage::English => "English",
        };
        let hk_id = match cfg.switch_hotkey {
            SwitchHotkey::DoubleTapShift => "DoubleTapShift",
            SwitchHotkey::CtrlSpace => "CtrlSpace",
            SwitchHotkey::F12 => "F12",
            SwitchHotkey::CtrlAltB => "CtrlAltB",
            SwitchHotkey::Custom(_) => "DoubleTapShift",
        };
        let mode_str = format!("{:?}", cfg.typing_mode);
        let lay_str = format!("{:?}", cfg.active_layout);
        // Studio expects a list of [trigger, expansion] pairs
        let mut snippet_pairs: Vec<(&String, &String)> =
            st.ekushe.suggestions_engine().snippets().iter().collect();
        snippet_pairs.sort();
        let json = serde_json::json!({
            "language": lang,
            "hotkey_id": hk_id,
            "hotkey_display": cfg.switch_hotkey.display_name(),
            "typing_mode": mode_str,
            "active_layout": lay_str,
            "font_family": cfg.candidate_font_family,
            "popup_opacity": st.popup_opacity_pct,
            "compact_popup": cfg.compact_popup,
            "auto_english_in_terminals": cfg.auto_english_in_terminals,
            "check_for_updates": cfg.check_for_updates,
            "start_bangla": cfg.start_language == InputLanguage::Bangla,
            "app_version": update_check::current_version(),
            "snippets": snippet_pairs,
            "report_url": ekushe_core::report_url(&windows_name(), cfg),
        })
        .to_string();
        drop(st); // never hold the keyboard hook's lock while writing to the socket
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "POST" && clean_path == "/api/toggle" {
        let (new_lang, tray_hwnd) = {
            let mut st = STATE.lock();
            let nl = st.ekushe.toggle_language();
            (nl, st.tray_hwnd)
        };
        unsafe {
            if tray_hwnd != 0 {
                add_or_update_tray_icon(tray_hwnd as HWND, false);
            }
        }
        let json = serde_json::json!({ "language": format!("{new_lang:?}") }).to_string();
        return write_http_response(&mut stream, "200 OK", "application/json", json.as_bytes());
    }

    if method == "GET" && clean_path == "/api/suggest" {
        let q = extract_query_param(path, "q");
        // Hold the shared lock only while ranking (~1 ms), never while writing to the socket:
        // the keyboard hook needs the same lock on every keystroke.
        let (cands, hints) = STATE.lock().ekushe.suggest_roman(&q);
        let json = serde_json::json!({
            "candidates": cands,
            "sign_hints": hints,
        })
        .to_string();
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "GET" && clean_path == "/api/signs" {
        let q = extract_query_param(path, "q");
        let entries = search_signs_and_conjuncts(&q);
        let json = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "POST" && clean_path == "/api/spell" {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let text = parsed.get("text").and_then(|v| v.as_str()).unwrap_or("");
        // Spell-check a whole document on a copy of the checker, outside the shared lock, so a long
        // check can never stall typing (the keyboard hook needs that lock on every keystroke).
        let checker = STATE
            .lock()
            .ekushe
            .suggestions_engine()
            .spell_checker()
            .clone();
        let issues = checker.check_text(text);
        let json = serde_json::to_string(&issues).unwrap_or_else(|_| "[]".to_string());
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "POST" && clean_path == "/api/convert" {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let mode = parsed
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("bijoy2uni");
        let text = parsed.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let result = match mode {
            "uni2bijoy" => unicode_to_bijoy(text),
            "banglish" => banglish_to_bangla(text),
            _ => bijoy_to_unicode(text),
        };
        let json = serde_json::json!({ "result": result }).to_string();
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "GET" && clean_path == "/api/recent" {
        // The last words Ekushe placed (newest first), kept in memory only, so a wrong one can be fixed.
        let recent = STATE.lock().ekushe.recent_words();
        let reported = std::fs::read_to_string(wrong_words_path()).unwrap_or_default();
        let json = serde_json::json!({ "recent": recent, "reported": reported }).to_string();
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "POST" && clean_path == "/api/recent/clear" {
        STATE.lock().ekushe.clear_recent_words();
        return write_http_response(&mut stream, "200 OK", "application/json", b"{}");
    }

    if method == "POST" && clean_path == "/api/wrong" {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let field = |k: &str| {
            parsed
                .get(k)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let (roman, wrong, mut right) = (field("roman"), field("wrong"), field("right"));
        let fixed = {
            let mut st = STATE.lock();
            // The right word may be typed in roman letters too: take Ekushe's first suggestion for it.
            if !right
                .chars()
                .any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
                && !right.is_empty()
            {
                if let Some(c) = st.ekushe.suggest_roman(&right).0.first() {
                    right = c.text.clone();
                }
            }
            st.ekushe.correct_recent_word(&roman, &wrong, &right)
        };
        let Some(fixed) = fixed else {
            return write_http_response(
                &mut stream,
                "404 Not Found",
                "application/json",
                br#"{"error":"not a recent word"}"#,
            );
        };
        save_learned_words_if_changed();
        let saved = append_wrong_word(&fixed.tsv_line()).is_ok();
        let json = serde_json::json!({ "fixed": fixed, "saved": saved }).to_string();
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.as_bytes(),
        );
    }

    if method == "GET" && clean_path == "/api/backup" {
        // Settings, snippets, learned and reported words in one file the user can keep or move
        save_learned_words_if_changed();
        let today = ekushe_core::dates::now()
            .map(|t| format!("{:04}-{:02}-{:02}", t.year, t.month, t.day))
            .unwrap_or_default();
        let backup = ekushe_core::backup::create(&Config::default_dir(), &today);
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            backup.as_bytes(),
        );
    }

    if method == "POST" && clean_path == "/api/restore" {
        save_learned_words_if_changed();
        let json = match ekushe_core::backup::restore(&Config::default_dir(), body) {
            Ok(done) => {
                let tray_hwnd = {
                    let mut st = STATE.lock();
                    if let Ok(cfg) = Config::load_from(&Config::default_config_path()) {
                        st.ekushe.set_config(cfg);
                    }
                    let _ = st
                        .ekushe
                        .suggestions_engine_mut()
                        .load_user_dict(&learned_words_path());
                    st.saved_learned_version = st.ekushe.suggestions_engine().learned_version();
                    st.tray_hwnd
                };
                unsafe {
                    if tray_hwnd != 0 {
                        add_or_update_tray_icon(tray_hwnd as HWND, false);
                    }
                }
                serde_json::json!({ "ok": true, "restored": done })
            }
            Err(e) => serde_json::json!({ "ok": false, "error": e }),
        };
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            json.to_string().as_bytes(),
        );
    }

    if method == "POST" && clean_path == "/api/settings" {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let hk_str = parsed
            .get("hotkey")
            .and_then(|v| v.as_str())
            .unwrap_or("DoubleTapShift");
        let mode_str = parsed
            .get("typing_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("Smart");
        let lay_str = parsed
            .get("active_layout")
            .and_then(|v| v.as_str())
            .unwrap_or("AvroPhonetic");
        let font_str = parsed
            .get("font_family")
            .and_then(|v| v.as_str())
            .unwrap_or("Kalpurush");
        let opacity_opt = parsed
            .get("popup_opacity")
            .and_then(|v| v.as_u64())
            .map(|n| (n as u8).clamp(70, 100));
        let compact_opt = parsed.get("compact_popup").and_then(|v| v.as_bool());
        let term_opt = parsed
            .get("auto_english_in_terminals")
            .and_then(|v| v.as_bool());
        let update_opt = parsed.get("check_for_updates").and_then(|v| v.as_bool());
        let start_bn_opt = parsed.get("start_bangla").and_then(|v| v.as_bool());
        // `snippet_key` / `snippet_value` are accepted too (older Studio pages sent those names)
        let field = |a: &str, b: &str| {
            parsed
                .get(a)
                .or_else(|| parsed.get(b))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let snip_trig = field("snippet_trigger", "snippet_key");
        let snip_exp = field("snippet_expansion", "snippet_value");

        let (tray_hwnd, overlay_hwnd, opacity_pct) = {
            let mut st = STATE.lock();
            if let Some(op) = opacity_opt {
                st.popup_opacity_pct = op;
            }
            let mut cfg = st.ekushe.config().clone();
            cfg.switch_hotkey = match hk_str {
                "CtrlSpace" => SwitchHotkey::CtrlSpace,
                "F12" => SwitchHotkey::F12,
                "CtrlAltB" => SwitchHotkey::CtrlAltB,
                _ => SwitchHotkey::DoubleTapShift,
            };
            cfg.typing_mode = match mode_str {
                "PhoneticFirst" => TypingMode::PhoneticFirst,
                "PhoneticOnly" => TypingMode::PhoneticOnly,
                _ => TypingMode::Smart,
            };
            cfg.active_layout = match lay_str {
                "Probhat" => LayoutId::Probhat,
                "NationalJatiya" => LayoutId::NationalJatiya,
                "Bornona" => LayoutId::Bornona,
                "MunirOptima" => LayoutId::MunirOptima,
                _ => LayoutId::AvroPhonetic,
            };
            cfg.candidate_font_family = font_str.to_string();
            if let Some(cp) = compact_opt {
                cfg.compact_popup = cp;
            }
            if let Some(tm) = term_opt {
                cfg.auto_english_in_terminals = tm;
            }
            if let Some(u) = update_opt {
                cfg.check_for_updates = u;
            }
            if let Some(b) = start_bn_opt {
                cfg.start_language = if b {
                    InputLanguage::Bangla
                } else {
                    InputLanguage::English
                };
            }
            if !snip_trig.is_empty() && !snip_exp.is_empty() {
                cfg.custom_snippets
                    .insert(snip_trig.to_string(), snip_exp.to_string());
            }
            let _ = cfg.save_to(&Config::default_config_path());
            st.ekushe.set_config(cfg);
            (st.tray_hwnd, st.overlay_hwnd, st.popup_opacity_pct)
        };
        unsafe {
            if tray_hwnd != 0 {
                add_or_update_tray_icon(tray_hwnd as HWND, false);
            }
            // The popup picks up the new opacity the next time it is drawn
            let _ = (overlay_hwnd, opacity_pct);
        }
        return write_http_response(&mut stream, "200 OK", "application/json", b"{\"ok\":true}");
    }

    write_http_response(&mut stream, "404 Not Found", "text/plain", b"Not Found")
}

fn write_http_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn extract_query_param(path: &str, key: &str) -> String {
    let Some(q_idx) = path.find('?') else {
        return String::new();
    };
    let query = &path[q_idx + 1..];
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next() == Some(key) {
            let raw = kv.next().unwrap_or("");
            return url_decode(raw);
        }
    }
    String::new()
}

fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(hex) = std::str::from_utf8(&b[i + 1..i + 3]) {
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    bytes.push(val);
                    i += 3;
                    continue;
                }
            }
        } else if b[i] == b'+' {
            bytes.push(b' ');
            i += 1;
            continue;
        }
        bytes.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Launch Ekushe Studio in a native frameless Edge/Chrome `--app` window (or default browser fallback).
pub fn open_studio_window(port: u16) {
    open_studio_window_tab(port, None);
}

/// Open Studio in this app's server, with a fresh one-time launch code so the page gets its key.
pub fn open_studio_window_tab(port: u16, tab: Option<&str>) {
    open_url_in_app_window(&studio_http::launch_url(port, tab));
}

/// Open Studio's address without a launch code (it then asks to be opened from the tray menu).
fn open_studio_plain(port: u16) {
    open_url_in_app_window(&format!("http://127.0.0.1:{port}/"));
}

fn open_url_in_app_window(url: &str) {
    let url = url.to_string();
    let edge_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for edge in &edge_paths {
        if std::path::Path::new(edge).exists() {
            let _ = Command::new(edge)
                .arg(format!("--app={url}"))
                .arg("--window-size=1180,780")
                .spawn();
            return;
        }
    }
    // The default browser, through the shell
    unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            ptr::null_mut(),
            wide_null("open").as_ptr(),
            wide_null(&url).as_ptr(),
            ptr::null(),
            ptr::null(),
            1, // SW_SHOWNORMAL
        );
    }
}

/// A web page in the default browser.
pub(crate) fn open_in_browser(url: &str) {
    unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            ptr::null_mut(),
            wide_null("open").as_ptr(),
            wide_null(url).as_ptr(),
            ptr::null(),
            ptr::null(),
            1, // SW_SHOWNORMAL
        );
    }
}

/// "Windows 11 (build 26100)" or "Windows 10 (build 19045)", for problem reports.
pub(crate) fn windows_name() -> String {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let mut buf = [0u16; 32];
    let mut len = (buf.len() * 2) as u32;
    let ok = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            wide_null(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion").as_ptr(),
            wide_null("CurrentBuildNumber").as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut len,
        )
    } == 0;
    let build: u32 = if ok {
        String::from_utf16_lossy(&buf[..(len as usize / 2).saturating_sub(1)])
            .trim()
            .parse()
            .unwrap_or(0)
    } else {
        0
    };
    match build {
        0 => "Windows".into(),
        b if b >= 22000 => format!("Windows 11 (build {b})"),
        b => format!("Windows 10 (build {b})"),
    }
}

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
