#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! `ekushe`: Windows System Tray IME, Global Phonetic/Fixed-Layout Hook,
//! Floating Candidate + Sign/Conjunct Hints Overlay, and Ekushe Studio Server.

mod setup_wizard;
mod studio_ui;

use std::env;
use std::io::{Read, Write};
use std::mem::{size_of, zeroed};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::ptr;
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

use windows_sys::Win32::Foundation::{
    COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows_sys::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateFontW, CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetTextExtentPoint32W, InvalidateRect, RoundRect, SelectObject, SetBkMode,
    SetTextColor, TextOutW, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HDC,
    PAINTSTRUCT, PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, ToUnicode, GetKeyboardState, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK, VK_CONTROL, VK_ESCAPE, VK_F12,
    VK_LSHIFT, VK_MENU, VK_RETURN, VK_RSHIFT, VK_SHIFT, VK_SPACE, VK_TAB,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CallNextHookEx, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    DestroyWindow, DispatchMessageW, GetClassNameW, GetClientRect, GetCursorPos,
    GetForegroundWindow, GetMessageW, GetSystemMetrics, KillTimer, LoadCursorW, LoadIconW,
    LoadImageW, PostMessageW, PostQuitMessage, RegisterClassW, SetCursor, SetForegroundWindow,
    SetLayeredWindowAttributes, SetMenuDefaultItem, SetTimer, SetWindowPos, SetWindowsHookExW,
    ShowWindow, TrackPopupMenu, TranslateMessage, UnhookWindowsHookEx, CW_USEDEFAULT, HHOOK, HICON,
    HWND_TOPMOST, IDC_HAND, IDI_APPLICATION, IMAGE_ICON, KBDLLHOOKSTRUCT, LR_LOADFROMFILE,
    LWA_ALPHA, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, SM_CXSCREEN, SM_CYSCREEN,
    SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, WH_KEYBOARD_LL, WM_APP, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
    WM_NULL, WM_PAINT, WM_RBUTTONUP, WM_SETCURSOR, WM_SYSKEYDOWN, WM_TIMER, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

/// Embedded 256x256 PNG logo for the Studio UI (`/logo.png`)
static LOGO_PNG_BYTES: &[u8] = include_bytes!("../../../assets/icons/logo_256.png");

/// Bundled Bangladeshi & Bengali OpenType Fonts (100% offline, compiled into binary)
static FONT_KALPURUSH: &[u8] = include_bytes!("../../../assets/fonts/Kalpurush.ttf");
static FONT_SOLAIMANLIPI: &[u8] = include_bytes!("../../../assets/fonts/SolaimanLipi.ttf");
static FONT_SIYAMRUPALI: &[u8] = include_bytes!("../../../assets/fonts/SiyamRupali.ttf");
static FONT_HINDSILIGURI_REG: &[u8] = include_bytes!("../../../assets/fonts/HindSiliguri-Regular.ttf");
static FONT_HINDSILIGURI_BOLD: &[u8] = include_bytes!("../../../assets/fonts/HindSiliguri-Bold.ttf");

/// Magic extra info tag so our low-level keyboard hook never intercepts our own `SendInput` events.
const EKUSHE_EXTRA_INFO: usize = 0xE21B_0945;

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
}

static STATE: Lazy<Mutex<AppSharedState>> = Lazy::new(|| {
    let mut cfg = Config::load_from(&Config::default_config_path()).unwrap_or_default();
    if cfg.candidate_font_family.is_empty() || cfg.candidate_font_family == "Nirmala UI" {
        cfg.candidate_font_family = "Kalpurush".to_string();
    }
    // Ensure Bangla digits are enabled in Ekushe Bangla mode so English mode = 0-9 and Ekushe mode = ০-৯
    cfg.bangla_digits = true;
    let ek = Ekushe::new(cfg);
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
        helper_btn_rect: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        compact_btn_rect: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        popup_opacity_pct: 92,
    })
});

static mut KEYBOARD_HOOK: HHOOK = ptr::null_mut();

unsafe fn register_embedded_bangla_fonts() {
    for font_bytes in [
        FONT_KALPURUSH,
        FONT_SOLAIMANLIPI,
        FONT_SIYAMRUPALI,
        FONT_HINDSILIGURI_REG,
        FONT_HINDSILIGURI_BOLD,
    ] {
        let mut num_fonts: u32 = 0;
        let _ = AddFontMemResourceEx(
            font_bytes.as_ptr() as *const _,
            font_bytes.len() as u32,
            ptr::null(),
            &mut num_fonts,
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

        if exe_name.contains("uninstall") || args.iter().any(|a| a == "--uninstall") {
            let _ = setup_wizard::perform_uninstall();
            return;
        }

        if args.iter().any(|a| a == "--auto-install") {
            let launch = !args.iter().any(|a| a == "--no-launch");
            let _ = setup_wizard::perform_full_install(true, true, launch);
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

    let open_studio_on_launch = !args.iter().any(|a| a == "--minimized" || a == "--tray-only");

    // If an instance of Ekushe is already running in the System Tray, reuse it and open Studio if requested!
    if !args.iter().any(|a| a == "--self-test") {
        if let Some(existing_port) = check_existing_studio_port() {
            if open_studio_on_launch || args.iter().any(|a| a == "--open-studio") {
                open_studio_window_tab(existing_port, None);
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

        let overlay_hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED,
            overlay_class_name.as_ptr(),
            wide_null("Ekushe Next Candidate Overlay").as_ptr(),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            688,
            132,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        );

        // Apply hardware-composited Frosted Glass Alpha Transparency (92% = 235/255)
        let _ = SetLayeredWindowAttributes(overlay_hwnd, 0, 235, LWA_ALPHA);

        // Enable Windows 11 hardware anti-aliased rounded corners (DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_ROUND = 2)
        // and Acrylic transient backdrop hint (DWMWA_SYSTEMBACKDROP_TYPE = 38, DWMSBT_TRANSIENTWINDOW = 3)
        let corner_pref: u32 = 2;
        let _ = DwmSetWindowAttribute(
            overlay_hwnd,
            33,
            &corner_pref as *const u32 as *const _,
            size_of::<u32>() as u32,
        );
        let backdrop_type: u32 = 3;
        let _ = DwmSetWindowAttribute(
            overlay_hwnd,
            38,
            &backdrop_type as *const u32 as *const _,
            size_of::<u32>() as u32,
        );

        {
            let mut st = STATE.lock();
            st.tray_hwnd = tray_hwnd as isize;
            st.overlay_hwnd = overlay_hwnd as isize;
        }

        // Add icon to Windows System Tray
        add_or_update_tray_icon(tray_hwnd, true);

        // Install global low-level keyboard hook
        KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0);

        // Show initial floating badge ("বাং · একুশে বাংলা সক্রিয়") and optionally open Studio
        show_mode_badge("বাং · একুশে বাংলা সক্রিয় (Tap Shift×2 to switch)");
        if open_studio_on_launch {
            open_studio_window(port);
        }

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if !KEYBOARD_HOOK.is_null() {
            UnhookWindowsHookEx(KEYBOARD_HOOK);
        }
        remove_tray_icon(tray_hwnd);
    }
}

/// Self-test mode for automated verification of all System Tray right-click URLs & HTTP Studio endpoints.
fn run_self_test(port: u16) {
    println!("Running Ekushe Tray + Studio comprehensive self-test on 127.0.0.1:{port}...");

    let http_get = |uri: &str| -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect studio server");
        let req = format!("GET {uri} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        s.write_all(req.as_bytes()).unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        String::from_utf8_lossy(&resp).into_owned()
    };

    let http_post = |uri: &str, json_body: &str| -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect studio server");
        let req = format!(
            "POST {uri} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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
    let r_conv1 = http_post("/api/convert", r#"{"mode":"bijoy2uni","text":"Avwg evsjvq Mvb MvB|"}"#);
    assert!(r_conv1.contains("আমি বাংলায় গান গাই।"));
    let r_conv2 = http_post("/api/convert", r#"{"mode":"banglish","text":"ami banglay gan gai"}"#);
    assert!(r_conv2.contains("আমি বাংলায় গান গাই"));
    println!("✓ POST /api/convert verified (Bijoy <-> Unicode & Banglish)");

    // 7. Verify POST /api/settings
    let r_set = http_post(
        "/api/settings",
        r#"{"hotkey":"DoubleTapShift","typing_mode":"PhoneticFirst","active_layout":"AvroPhonetic","font_family":"Kalpurush"}"#,
    );
    assert!(r_set.contains("\"ok\":true"));
    println!("✓ POST /api/settings verified");

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
        let info_body = wide_null(
            "সিস্টেম ট্রে-তে সক্রিয় আছে! দুইবার Shift চেপে বাংলা ⇄ ইংরেজি পরিবর্তন করুন।",
        );
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

/// Clamp popup position `(x, y)` so the floating bar never overflows the right or bottom screen edges.
unsafe fn clamp_popup_pos(raw_x: i32, raw_y: i32, w: i32, h: i32) -> (i32, i32) {
    let screen_w = GetSystemMetrics(SM_CXSCREEN).max(800);
    let screen_h = GetSystemMetrics(SM_CYSCREEN).max(600);
    let x = raw_x.clamp(12, (screen_w - w - 16).max(12));
    let y = if raw_y + h > screen_h - 48 {
        (raw_y - h - 28).max(12)
    } else {
        raw_y.max(12)
    };
    (x, y)
}

/// Show a floating mode switch badge or candidate overlay near the current cursor position.
unsafe fn show_mode_badge(banner: &str) {
    let overlay_hwnd = {
        let mut st = STATE.lock();
        st.badge_banner = Some(banner.to_string());
        st.cand_hit_rects.clear();
        st.hint_hit_rects.clear();
        st.overlay_hwnd as HWND
    };
    if overlay_hwnd.is_null() {
        return;
    }

    let mut pt = POINT { x: 400, y: 400 };
    GetCursorPos(&mut pt);
    let (x, y) = clamp_popup_pos(pt.x + 14, pt.y + 22, 436, 52);
    SetWindowPos(
        overlay_hwnd,
        HWND_TOPMOST,
        x,
        y,
        436,
        52,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
    InvalidateRect(overlay_hwnd, ptr::null(), 1);
    SetTimer(overlay_hwnd, TIMER_HIDE_BADGE, 1600, None);
}

unsafe fn refresh_candidate_overlay() {
    let (overlay_hwnd, has_content, has_hints) = {
        let mut st = STATE.lock();
        st.badge_banner = None;
        let has = !st.last_output.preedit.is_empty() || !st.last_output.candidates.is_empty();
        let hints = !st.last_output.sign_hints.is_empty();
        (st.overlay_hwnd as HWND, has, hints)
    };
    if overlay_hwnd.is_null() {
        return;
    }

    if !has_content {
        ShowWindow(overlay_hwnd, SW_HIDE);
        return;
    }

    KillTimer(overlay_hwnd, TIMER_HIDE_BADGE);
    let mut pt = POINT { x: 400, y: 400 };
    GetCursorPos(&mut pt);
    let pop_w = 688;
    let pop_h = if has_hints { 130 } else { 90 };
    let (x, y) = clamp_popup_pos(pt.x + 14, pt.y + 24, pop_w, pop_h);
    SetWindowPos(
        overlay_hwnd,
        HWND_TOPMOST,
        x,
        y,
        pop_w,
        pop_h,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
    ShowWindow(overlay_hwnd, SW_SHOWNOACTIVATE);
    InvalidateRect(overlay_hwnd, ptr::null(), 1);
}

/// Fast, zero-privilege check for whether the active foreground window is a Terminal / Console app
/// (`Windows Terminal`, `Command Prompt`, `PowerShell`, `Git Bash / mintty`, `PuTTY`).
unsafe fn is_foreground_terminal_window() -> bool {
    let fg = GetForegroundWindow();
    if fg.is_null() {
        return false;
    }
    let mut cls_buf = [0u16; 96];
    let len = GetClassNameW(fg, cls_buf.as_mut_ptr(), cls_buf.len() as i32);
    if len <= 0 {
        return false;
    }
    let cls = String::from_utf16_lossy(&cls_buf[..len as usize]);
    matches!(
        cls.as_str(),
        "CASCADIA_HOSTING_WINDOW_CLASS"
            | "ConsoleWindowClass"
            | "mintty"
            | "PuTTY"
            | "VirtualConsoleClass"
            | "org.wezfurlong.wezterm"
    )
}

/// Low-level keyboard hook (`WH_KEYBOARD_LL`) that powers:
/// 1. One-key toggle (`Double-Tap Shift`, `Ctrl+Space`, `F12`, `Ctrl+Alt+B`)
/// 2. Per-App Terminal Smart Guard (auto-English in PowerShell / cmd / Windows Terminal)
/// 3. `Ctrl+Backspace` to flip the last committed word between Bangla ⇄ English
/// 4. Live phonetic & fixed-layout Bangla typing in any Windows app (with Bangla digits ০-৯ in Ekushe mode and 0-9 in English mode)
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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
                if let Some(new_lang) = st.ekushe.on_shift_tap(now_ms) {
                    st.injected_utf16_len = 0;
                    st.last_output = SessionOutput::default();
                    let tray_hwnd = st.tray_hwnd as HWND;
                    drop(st);
                    add_or_update_tray_icon(tray_hwnd, false);
                    let msg = match new_lang {
                        InputLanguage::Bangla => "বাং · একুশে বাংলা সক্রিয় (Numbers: ০-৯)",
                        InputLanguage::English => "EN · ইংরেজি কীবোর্ড সক্রিয় (Numbers: 0-9)",
                    };
                    show_mode_badge(msg);
                    return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
                }
            }
        }
        return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
    }

    if is_keydown {
        let mut st = STATE.lock();
        st.shift_clean_down = false;

        let switch_triggered = match &st.ekushe.config().switch_hotkey {
            SwitchHotkey::CtrlSpace => ctrl_down && !alt_down && vk == VK_SPACE,
            SwitchHotkey::F12 => vk == VK_F12,
            SwitchHotkey::CtrlAltB => ctrl_down && alt_down && vk == b'B' as u16,
            _ => false,
        };

        if switch_triggered {
            let new_lang = st.ekushe.toggle_language();
            st.injected_utf16_len = 0;
            st.last_output = SessionOutput::default();
            let tray_hwnd = st.tray_hwnd as HWND;
            drop(st);
            add_or_update_tray_icon(tray_hwnd, false);
            let msg = match new_lang {
                InputLanguage::Bangla => "বাং · একুশে বাংলা সক্রিয় (Numbers: ০-৯)",
                InputLanguage::English => "EN · ইংরেজি কীবোর্ড সক্রিয় (Numbers: 0-9)",
            };
            show_mode_badge(msg);
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

        // If English mode is active, or Terminal Auto-Pause is active, or Ctrl/Alt shortcuts are pressed, pass straight through
        if st.ekushe.language() == InputLanguage::English
            || (ctrl_down && !alt_down)
            || (st.ekushe.config().auto_english_in_terminals && is_foreground_terminal_window())
        {
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // If navigation keys (PageUp/PageDown/End/Home/Left/Up/Right/Down: 0x21..=0x28) are pressed while composing,
        // keep the injected text intact and close the composition buffer cleanly.
        if (0x21..=0x28).contains(&vk) && st.injected_utf16_len > 0 {
            st.ekushe.reset_composition();
            st.last_output = SessionOutput::default();
            st.injected_utf16_len = 0;
            drop(st);
            refresh_candidate_overlay();
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // Handle Escape: cancel current composition popup
        if vk == VK_ESCAPE && !st.last_output.preedit.is_empty() {
            st.ekushe.reset_composition();
            st.last_output = SessionOutput::default();
            st.injected_utf16_len = 0;
            drop(st);
            refresh_candidate_overlay();
            return 1;
        }

        // Handle Tab: cycle through candidates in the popup!
        if vk == VK_TAB && st.last_output.candidates.len() > 1 {
            let next_idx = (st.last_output.selected_index + 1) % st.last_output.candidates.len();
            st.last_output.selected_index = next_idx;
            let cand_text = st.last_output.candidates[next_idx].text.clone();
            let prev_len = st.injected_utf16_len;
            st.injected_utf16_len = cand_text.encode_utf16().count();
            drop(st);
            replace_active_text(prev_len, &cand_text);
            refresh_candidate_overlay();
            return 1;
        }

        // Handle Backspace while composing
        if vk == VK_BACK {
            let prev_len = st.injected_utf16_len;
            if prev_len > 0 {
                let out = st.ekushe.backspace();
                let new_text = out.preedit.clone();
                st.injected_utf16_len = new_text.encode_utf16().count();
                st.last_output = out;
                drop(st);
                replace_active_text(prev_len, &new_text);
                refresh_candidate_overlay();
                return 1;
            }
            return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
        }

        // Handle Space or Enter when composing
        if (vk == VK_SPACE || vk == VK_RETURN) && st.injected_utf16_len > 0 {
            let sel = st.last_output.selected_index;
            let trailing = if vk == VK_SPACE { Some(' ') } else { None };
            let prev_len = st.injected_utf16_len;
            let out = st.ekushe.select_candidate(sel, trailing);
            st.injected_utf16_len = 0;
            let committed = out.committed.clone().unwrap_or_default();
            st.last_output = SessionOutput::default();
            drop(st);
            replace_active_text(prev_len, &committed);
            refresh_candidate_overlay();
            if vk == VK_RETURN {
                return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
            }
            return 1;
        }

        // Translate virtual key to printable ASCII/Unicode character (including Numpad 0-9!)
        if let Some(ch) = vk_to_char(vk, kb.scanCode, shift_down) {
            if !ch.is_control() {
                let prev_len = st.injected_utf16_len;
                let altgr = ctrl_down && alt_down;
                let out = st.ekushe.push_char(ch, altgr);
                if let Some(committed) = &out.committed {
                    st.injected_utf16_len = 0;
                    let to_send = committed.clone();
                    st.last_output = SessionOutput::default();
                    drop(st);
                    replace_active_text(prev_len, &to_send);
                    refresh_candidate_overlay();
                    return 1;
                } else {
                    let preedit = out.preedit.clone();
                    st.injected_utf16_len = preedit.encode_utf16().count();
                    st.last_output = out;
                    drop(st);
                    replace_active_text(prev_len, &preedit);
                    refresh_candidate_overlay();
                    return 1;
                }
            }
        }
    }

    CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
}

unsafe fn vk_to_char(vk: u16, scan_code: u32, shift: bool) -> Option<char> {
    // Support Numpad 0..=9 (VK_NUMPAD0 = 0x60 .. VK_NUMPAD9 = 0x69) directly so Numpad also types বাংলা ০-৯ in Ekushe mode!
    if !shift && (0x60..=0x69).contains(&vk) {
        return char::from_u32(('0' as u32) + (vk as u32 - 0x60));
    }
    let mut kbd_state = [0u8; 256];
    GetKeyboardState(kbd_state.as_mut_ptr());
    if shift {
        kbd_state[VK_SHIFT as usize] = 0x80;
    }
    let mut buf = [0u16; 4];
    let rc = ToUnicode(
        vk as u32,
        scan_code,
        kbd_state.as_ptr(),
        buf.as_mut_ptr(),
        buf.len() as i32,
        0,
    );
    if rc == 1 {
        char::from_u32(buf[0] as u32)
    } else {
        None
    }
}

/// Atomically send `backspaces` followed by `new_text` as UTF-16 `KEYEVENTF_UNICODE` events.
unsafe fn replace_active_text(backspaces: usize, new_text: &str) {
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

    SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        size_of::<INPUT>() as i32,
    );
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
                        InputLanguage::Bangla => "বাং · একুশে বাংলা সক্রিয় (Bangla Active)",
                        InputLanguage::English => "EN · সাধারণ ইংরেজি কীবোর্ড (English Default)",
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
        wide_null("🔤 যুক্তবর্ণ ও কার-ফলা সহায়িকা (Signs & যুক্তবর্ণ Guide)").as_ptr(),
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
        (
            TypingMode::Smart,
            "মোড: Smart (Default)",
            CMD_MODE_SMART,
        ),
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
                InputLanguage::Bangla => "বাং · একুশে বাংলা সক্রিয় (Bangla Active)",
                InputLanguage::English => "EN · সাধারণ ইংরেজি কীবোর্ড (English Default)",
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
            show_mode_badge(&format!("✓ হটকি নির্বাচিত: {hk_name}"));
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
            show_mode_badge(&format!("✓ টাইপিং মোড: {label}"));
        }
        CMD_LAYOUT_AVRO
        | CMD_LAYOUT_PROBHAT
        | CMD_LAYOUT_JATIYA
        | CMD_LAYOUT_BORNONA
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
            show_mode_badge(&format!("✓ কীবোর্ড লেআউট: {lay_name}"));
        }
        CMD_EXIT => {
            DestroyWindow(hwnd);
        }
        _ => {}
    }
}

unsafe fn draw_rounded_pill(
    hdc: HDC,
    r: RECT,
    radius: i32,
    fill_color: COLORREF,
    border_color: COLORREF,
    border_width: i32,
) {
    let brush = CreateSolidBrush(fill_color);
    let pen = CreatePen(PS_SOLID, border_width, border_color);
    let old_b = SelectObject(hdc, brush as _);
    let old_p = SelectObject(hdc, pen as _);
    RoundRect(hdc, r.left, r.top, r.right, r.bottom, radius, radius);
    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
    DeleteObject(brush as _);
    DeleteObject(pen as _);
}

unsafe fn measure_wide_text(hdc: HDC, w: &[u16]) -> (i32, i32) {
    if w.is_empty() {
        return (0, 0);
    }
    let mut sz: SIZE = zeroed();
    GetTextExtentPoint32W(hdc, w.as_ptr(), w.len() as i32, &mut sz);
    (sz.cx, sz.cy)
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
                        st.last_output.sign_hints.clear();
                    } else {
                        let roman = st.last_output.roman_buffer.clone();
                        let pre = st.last_output.preedit.clone();
                        st.last_output.sign_hints = ekushe_core::conjuncts::contextual_sign_hints(&roman, &pre);
                    }
                }
                refresh_candidate_overlay();
                return 0;
            }

            if let Some(cand_idx) = clicked_cand {
                let mut st = STATE.lock();
                let prev_len = st.injected_utf16_len;
                let out = st.ekushe.select_candidate(cand_idx, Some(' '));
                st.injected_utf16_len = 0;
                let committed = out.committed.clone().unwrap_or_default();
                st.last_output = SessionOutput::default();
                drop(st);
                replace_active_text(prev_len, &committed);
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
                st.injected_utf16_len = preedit.encode_utf16().count();
                st.last_output = last_out;
                drop(st);
                replace_active_text(prev_len, &preedit);
                refresh_candidate_overlay();
                return 0;
            }

            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc_screen = BeginPaint(hwnd, &mut ps);

            let mut client_rc: RECT = zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let w_px = (client_rc.right - client_rc.left).max(1);
            let h_px = (client_rc.bottom - client_rc.top).max(1);

            // Offscreen double-buffering for zero-flicker 2026 HUD rendering
            let hdc = CreateCompatibleDC(hdc_screen);
            let hbm = CreateCompatibleBitmap(hdc_screen, w_px, h_px);
            let old_bm = SelectObject(hdc, hbm as _);

            // Modern 2026 Light-Mode Palette (COLORREF = 0x00BBGGRR)
            let c_surface: COLORREF = 0x00FF_FFFF;         // Pure Alabaster Card #FFFFFF
            let c_header_strip: COLORREF = 0x00F6_FAF8;    // Soft Mint-Slate Header Strip #F8FAF6
            let c_outer_border: COLORREF = 0x00CE_DFD7;    // Delicate Sage Border #D7DFCE -> #D7E5DE (0x00DEE5D7)
            let c_emerald_brand: COLORREF = 0x004E_6E0D;   // Bangladeshi Emerald #0D6E4E
            let c_emerald_dark: COLORREF = 0x003B_5409;    // Deep Emerald #09543B
            let c_crimson: COLORREF = 0x002B_1EC8;         // Bangladeshi Crimson #C81E2B
            let c_crimson_bg: COLORREF = 0x00F2_F0FD;      // Soft Rose Pill #FDF0F2
            let c_crimson_border: COLORREF = 0x00D5_CEFA;  // Soft Rose Border #FACED5
            let c_pill_bg: COLORREF = 0x00F6_F9F4;         // Soft Mint Pill #F4F9F6
            let c_pill_border: COLORREF = 0x00DC_E8D6;     // Soft Mint Border #D6E8DC
            let c_chip_bg: COLORREF = 0x00FA_FBF8;         // Hint Chip Surface #F8FBFA
            let c_keycap_bg: COLORREF = 0x00EE_F3EF;       // Keyboard Keycap Badge #EFF3EE
            let c_text_main: COLORREF = 0x001E_280F;       // Deep Forest Ink #0F281E
            let c_text_muted: COLORREF = 0x0064_7558;      // Slate-Mint Muted #587564
            let c_white: COLORREF = 0x00FF_FFFF;           // Crisp White #FFFFFF

            // Fill base card & outer border
            let bg_brush = CreateSolidBrush(c_surface);
            FillRect(hdc, &client_rc, bg_brush);
            DeleteObject(bg_brush as _);

            draw_rounded_pill(
                hdc,
                RECT { left: 0, top: 0, right: w_px, bottom: h_px },
                16,
                c_surface,
                c_outer_border,
                1,
            );

            SetBkMode(hdc, TRANSPARENT as i32);

            let mut st = STATE.lock();
            st.cand_hit_rects.clear();
            st.hint_hit_rects.clear();
            st.helper_btn_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            st.compact_btn_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            let is_compact = st.ekushe.config().compact_popup;

            let chosen_font = if st.ekushe.config().candidate_font_family.is_empty() {
                "Kalpurush".to_string()
            } else {
                st.ekushe.config().candidate_font_family.clone()
            };
            let font_name_w = wide_null(&chosen_font);
            let ui_font_w = wide_null("Segoe UI");

            let font_bn_bold = CreateFontW(
                -20, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, font_name_w.as_ptr(),
            );
            let font_bn_med = CreateFontW(
                -19, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, font_name_w.as_ptr(),
            );
            let font_bn_small = CreateFontW(
                -16, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, font_name_w.as_ptr(),
            );
            let font_ui_small = CreateFontW(
                -13, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, ui_font_w.as_ptr(),
            );

            let old_font = SelectObject(hdc, font_bn_bold as _);

            if let Some(banner) = st.badge_banner.clone() {
                // ── Mode Switch Toast Pill (বাং / EN) ───────────────────────────
                let is_en = banner.starts_with("EN");
                let badge_color = if is_en { 0x0052_4233 } else { c_emerald_brand };
                let badge_label = if is_en { "EN" } else { "বাং" };

                draw_rounded_pill(
                    hdc,
                    RECT { left: 10, top: 9, right: 56, bottom: h_px - 9 },
                    12,
                    badge_color,
                    badge_color,
                    1,
                );
                SetTextColor(hdc, c_white);
                let mut r_badge = RECT { left: 10, top: 9, right: 56, bottom: h_px - 9 };
                let bw = wide_no_null(badge_label);
                DrawTextW(
                    hdc,
                    bw.as_ptr(),
                    bw.len() as i32,
                    &mut r_badge,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                SelectObject(hdc, font_bn_med as _);
                SetTextColor(hdc, c_text_main);
                let mut r_txt = RECT { left: 68, top: 6, right: w_px - 12, bottom: h_px - 6 };
                let tw = wide_no_null(&banner);
                DrawTextW(
                    hdc,
                    tw.as_ptr(),
                    tw.len() as i32,
                    &mut r_txt,
                    DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
            } else {
                // ════════════════════════════════════════════════════════════════
                // TIER 1: Sleek Top Header Bar (y: 4..36)
                // ════════════════════════════════════════════════════════════════
                draw_rounded_pill(
                    hdc,
                    RECT { left: 4, top: 4, right: w_px - 4, bottom: 36 },
                    10,
                    c_header_strip,
                    c_header_strip,
                    1,
                );

                // 1a. Left "বাং" Emerald Pill Badge
                draw_rounded_pill(
                    hdc,
                    RECT { left: 10, top: 7, right: 48, bottom: 33 },
                    9,
                    c_emerald_brand,
                    c_emerald_brand,
                    1,
                );
                SelectObject(hdc, font_bn_small as _);
                SetTextColor(hdc, c_white);
                let mut r_bn_pill = RECT { left: 10, top: 7, right: 48, bottom: 33 };
                let bn_lbl = wide_no_null("বাং");
                DrawTextW(
                    hdc,
                    bn_lbl.as_ptr(),
                    bn_lbl.len() as i32,
                    &mut r_bn_pill,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                // 1b. Roman Buffer Keycap Pill (e.g. " g ")
                let roman_str = st.last_output.roman_buffer.clone();
                let roman_w = wide_no_null(&roman_str);
                SelectObject(hdc, font_ui_small as _);
                let (rw_px, _) = measure_wide_text(hdc, &roman_w);
                let rom_pill_w = (rw_px + 18).max(28);
                let r_rom_pill = RECT {
                    left: 56,
                    top: 7,
                    right: 56 + rom_pill_w,
                    bottom: 33,
                };
                draw_rounded_pill(hdc, r_rom_pill, 8, c_crimson_bg, c_crimson_border, 1);
                SetTextColor(hdc, c_crimson);
                let mut r_rom_txt = r_rom_pill;
                DrawTextW(
                    hdc,
                    roman_w.as_ptr(),
                    roman_w.len() as i32,
                    &mut r_rom_txt,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                // 1c. Arrow + Live Bangla Preedit Preview
                let arrow_x = r_rom_pill.right + 8;
                SetTextColor(hdc, c_text_muted);
                let arr_w = wide_no_null("→");
                TextOutW(hdc, arrow_x, 11, arr_w.as_ptr(), arr_w.len() as i32);

                SelectObject(hdc, font_bn_bold as _);
                SetTextColor(hdc, c_emerald_dark);
                let pre_w = wide_no_null(&st.last_output.preedit);
                TextOutW(hdc, arrow_x + 18, 7, pre_w.as_ptr(), pre_w.len() as i32);

                // 1d. Right-side Clickable "📖 সহায়িকা" + "🔽 সংক্ষিপ্ত / 🔼 যুক্তবর্ণ" Toggle + Keycap Hints
                let r_help_btn = RECT {
                    left: w_px - 94,
                    top: 7,
                    right: w_px - 8,
                    bottom: 33,
                };
                st.helper_btn_rect = r_help_btn;
                draw_rounded_pill(hdc, r_help_btn, 8, c_pill_bg, c_pill_border, 1);
                SelectObject(hdc, font_bn_small as _);
                SetTextColor(hdc, c_emerald_brand);
                let mut r_help_txt = r_help_btn;
                let help_lbl = wide_no_null("📖 সহায়িকা");
                DrawTextW(
                    hdc,
                    help_lbl.as_ptr(),
                    help_lbl.len() as i32,
                    &mut r_help_txt,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                let r_comp_btn = RECT {
                    left: w_px - 188,
                    top: 7,
                    right: w_px - 100,
                    bottom: 33,
                };
                st.compact_btn_rect = r_comp_btn;
                draw_rounded_pill(hdc, r_comp_btn, 8, c_pill_bg, c_pill_border, 1);
                SelectObject(hdc, font_bn_small as _);
                SetTextColor(hdc, c_emerald_dark);
                let mut r_comp_txt = r_comp_btn;
                let comp_lbl = if is_compact {
                    wide_no_null("🔼 যুক্তবর্ণ")
                } else {
                    wide_no_null("🔽 সংক্ষিপ্ত")
                };
                DrawTextW(
                    hdc,
                    comp_lbl.as_ptr(),
                    comp_lbl.len() as i32,
                    &mut r_comp_txt,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                // Keycap Hints: [Tab ↹] পরের শব্দ · [Space] নির্বাচন
                let r_key_hints = RECT {
                    left: w_px - 392,
                    top: 7,
                    right: w_px - 194,
                    bottom: 33,
                };
                draw_rounded_pill(hdc, r_key_hints, 8, c_keycap_bg, c_pill_border, 1);
                SelectObject(hdc, font_bn_small as _);
                SetTextColor(hdc, c_text_muted);
                let mut r_kh_txt = r_key_hints;
                let kh_lbl = wide_no_null("Tab ↹ পরের · Space নির্বাচন");
                DrawTextW(
                    hdc,
                    kh_lbl.as_ptr(),
                    kh_lbl.len() as i32,
                    &mut r_kh_txt,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                // ════════════════════════════════════════════════════════════════
                // TIER 2: Interactive Word Suggestion Pills (y: 42..80)
                // ════════════════════════════════════════════════════════════════
                let mut cur_x = 10;
                let pill_top = 42;
                let pill_bot = 80;
                let sel_idx = st.last_output.selected_index;
                let cands_snapshot: Vec<(usize, String)> = st
                    .last_output
                    .candidates
                    .iter()
                    .take(6)
                    .enumerate()
                    .map(|(i, c)| (i, c.text.clone()))
                    .collect();

                for (i, cand_text) in cands_snapshot {
                    let is_sel = i == sel_idx;
                    let label_str = format!("{}  {}", i + 1, cand_text);
                    let lw = wide_no_null(&label_str);

                    SelectObject(
                        hdc,
                        if is_sel { font_bn_bold as _ } else { font_bn_med as _ },
                    );
                    let (tw, _) = measure_wide_text(hdc, &lw);
                    let pill_w = (tw + 26).max(58);

                    if cur_x + pill_w > w_px - 10 {
                        break;
                    }

                    let r_pill = RECT {
                        left: cur_x,
                        top: pill_top,
                        right: cur_x + pill_w,
                        bottom: pill_bot,
                    };

                    if is_sel {
                        draw_rounded_pill(
                            hdc,
                            r_pill,
                            11,
                            c_emerald_brand,
                            c_emerald_dark,
                            1,
                        );
                        SetTextColor(hdc, c_white);
                    } else {
                        draw_rounded_pill(
                            hdc,
                            r_pill,
                            11,
                            c_pill_bg,
                            c_pill_border,
                            1,
                        );
                        SetTextColor(hdc, c_text_main);
                    }

                    let mut r_txt = r_pill;
                    DrawTextW(
                        hdc,
                        lw.as_ptr(),
                        lw.len() as i32,
                        &mut r_txt,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );

                    st.cand_hit_rects.push((r_pill, i));
                    cur_x += pill_w + 8;
                }

                // ════════════════════════════════════════════════════════════════
                // TIER 3: "যুক্তবর্ণ / কার" Interactive 2-Tone Mini-Chips (y: 88..122)
                // ════════════════════════════════════════════════════════════════
                if !st.last_output.sign_hints.is_empty() {
                    let chip_top = 88;
                    let chip_bot = 120;

                    // Section label pill on left
                    let r_sec = RECT {
                        left: 10,
                        top: chip_top,
                        right: 106,
                        bottom: chip_bot,
                    };
                    draw_rounded_pill(hdc, r_sec, 8, c_keycap_bg, c_pill_border, 1);
                    SelectObject(hdc, font_bn_small as _);
                    SetTextColor(hdc, c_emerald_dark);
                    let mut r_sec_txt = r_sec;
                    let sec_w = wide_no_null("যুক্তবর্ণ / কার");
                    DrawTextW(
                        hdc,
                        sec_w.as_ptr(),
                        sec_w.len() as i32,
                        &mut r_sec_txt,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );

                    let mut h_x = 114;
                    let hints_snapshot: Vec<(String, String)> = st
                        .last_output
                        .sign_hints
                        .iter()
                        .take(10)
                        .map(|h| (h.result_bangla.clone(), h.key_hint.clone()))
                        .collect();

                    for (res_bn, key_rom) in hints_snapshot {
                        let bn_w = wide_no_null(&res_bn);
                        let rom_w = wide_no_null(&key_rom);

                        SelectObject(hdc, font_bn_small as _);
                        let (bw_px, _) = measure_wide_text(hdc, &bn_w);
                        SelectObject(hdc, font_ui_small as _);
                        let (kw_px, _) = measure_wide_text(hdc, &rom_w);

                        let chip_w = (bw_px + kw_px + 24).max(56);
                        // Stop cleanly before overflowing the right border — never cut off mid-chip!
                        if h_x + chip_w > w_px - 10 {
                            break;
                        }

                        let r_chip = RECT {
                            left: h_x,
                            top: chip_top,
                            right: h_x + chip_w,
                            bottom: chip_bot,
                        };
                        draw_rounded_pill(hdc, r_chip, 9, c_chip_bg, c_pill_border, 1);

                        // Left half: Bangla connected letter / kar in Bold Emerald
                        SelectObject(hdc, font_bn_small as _);
                        SetTextColor(hdc, c_emerald_dark);
                        let mut r_bn_part = RECT {
                            left: h_x + 7,
                            top: chip_top,
                            right: h_x + 9 + bw_px,
                            bottom: chip_bot,
                        };
                        DrawTextW(
                            hdc,
                            bn_w.as_ptr(),
                            bn_w.len() as i32,
                            &mut r_bn_part,
                            DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                        );

                        // Right half: Roman key shortcut in Crimson
                        SelectObject(hdc, font_ui_small as _);
                        SetTextColor(hdc, c_crimson);
                        let mut r_rom_part = RECT {
                            left: h_x + 13 + bw_px,
                            top: chip_top,
                            right: h_x + chip_w - 5,
                            bottom: chip_bot,
                        };
                        DrawTextW(
                            hdc,
                            rom_w.as_ptr(),
                            rom_w.len() as i32,
                            &mut r_rom_part,
                            DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                        );

                        st.hint_hit_rects.push((r_chip, key_rom));
                        h_x += chip_w + 6;
                    }
                }
            }
            drop(st);

            // Blit offscreen buffer to screen cleanly
            BitBlt(hdc_screen, 0, 0, w_px, h_px, hdc, 0, 0, SRCCOPY);

            SelectObject(hdc, old_font);
            SelectObject(hdc, old_bm);
            DeleteObject(font_bn_bold as _);
            DeleteObject(font_bn_med as _);
            DeleteObject(font_bn_small as _);
            DeleteObject(font_ui_small as _);
            DeleteObject(hbm as _);
            DeleteDC(hdc);

            EndPaint(hwnd, &ps);
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

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = handle_http_client(stream);
        }
    });

    port
}

fn read_full_http_request(stream: &mut TcpStream) -> std::io::Result<String> {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(800)));
    let mut data: Vec<u8> = Vec::with_capacity(8192);
    let mut chunk = [0u8; 4096];
    let mut header_end: Option<usize> = None;
    let mut content_len: usize = 0;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&chunk[..n]);
                if header_end.is_none() {
                    if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                        let end = pos + 4;
                        header_end = Some(end);
                        let hdr_str = String::from_utf8_lossy(&data[..end]);
                        for line in hdr_str.lines() {
                            let lower = line.to_ascii_lowercase();
                            if let Some(rest) = lower.strip_prefix("content-length:") {
                                content_len = rest.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }
                if let Some(end) = header_end {
                    if data.len() >= end + content_len {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }

    Ok(String::from_utf8_lossy(&data).into_owned())
}

fn handle_http_client(mut stream: TcpStream) -> std::io::Result<()> {
    let req = read_full_http_request(&mut stream)?;
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

    if method == "GET" && (clean_path == "/" || clean_path.starts_with("/index")) {
        return write_http_response(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            studio_ui::STUDIO_HTML.as_bytes(),
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
            "snippets": st.ekushe.suggestions_engine().snippets(),
        })
        .to_string();
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
        let st = STATE.lock();
        let phon = ekushe_core::phonetic::PhoneticEngine::new();
        let (cands, hints) = st.ekushe.suggestions_engine().suggest(&q, &phon, false, true, 9);
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
        let st = STATE.lock();
        let issues = st.ekushe.suggestions_engine().spell_checker().check_text(text);
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
        let mode = parsed.get("mode").and_then(|v| v.as_str()).unwrap_or("bijoy2uni");
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

    if method == "POST" && clean_path == "/api/settings" {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let hk_str = parsed.get("hotkey").and_then(|v| v.as_str()).unwrap_or("DoubleTapShift");
        let mode_str = parsed.get("typing_mode").and_then(|v| v.as_str()).unwrap_or("Smart");
        let lay_str = parsed.get("active_layout").and_then(|v| v.as_str()).unwrap_or("AvroPhonetic");
        let font_str = parsed.get("font_family").and_then(|v| v.as_str()).unwrap_or("Kalpurush");
        let opacity_opt = parsed.get("popup_opacity").and_then(|v| v.as_u64()).map(|n| (n as u8).clamp(70, 100));
        let compact_opt = parsed.get("compact_popup").and_then(|v| v.as_bool());
        let term_opt = parsed.get("auto_english_in_terminals").and_then(|v| v.as_bool());
        let snip_trig = parsed.get("snippet_trigger").and_then(|v| v.as_str()).unwrap_or("").trim();
        let snip_exp = parsed.get("snippet_expansion").and_then(|v| v.as_str()).unwrap_or("").trim();

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
            if !snip_trig.is_empty() && !snip_exp.is_empty() {
                cfg.custom_snippets.insert(snip_trig.to_string(), snip_exp.to_string());
            }
            let _ = cfg.save_to(&Config::default_config_path());
            st.ekushe.set_config(cfg);
            (st.tray_hwnd, st.overlay_hwnd, st.popup_opacity_pct)
        };
        unsafe {
            if tray_hwnd != 0 {
                add_or_update_tray_icon(tray_hwnd as HWND, false);
            }
            if overlay_hwnd != 0 {
                let alpha = ((opacity_pct as u32 * 255) / 100).clamp(178, 255) as u8;
                let _ = SetLayeredWindowAttributes(overlay_hwnd as HWND, 0, alpha, LWA_ALPHA);
            }
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
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
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

pub fn open_studio_window_tab(port: u16, tab: Option<&str>) {
    let url = match tab {
        Some(t) => format!("http://127.0.0.1:{port}/?tab={t}"),
        None => format!("http://127.0.0.1:{port}/"),
    };
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
    let _ = Command::new("cmd").args(["/C", "start", "", &url]).spawn();
}

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_no_null(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
