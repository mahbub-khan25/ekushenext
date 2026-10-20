//! Self-contained Windows GUI Installer (`Ekushe-Setup.exe`) & Zero-Command Auto-Configurator.
//!
//! Features a clean, modern Light Mode Bento-grid design with pure white cards (`#FFFFFF`) on a
//! soft alabaster-mint canvas (`#F3F7F5`), deep emerald (`#0E7452`) & crimson (`#D91E2E`) accents,
//! and High-DPI ClearType dark-on-light Bangla typography.

use std::env;
use std::fs;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr;

use ekushe_core::config::Config;

use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows_sys::Win32::Graphics::Gdi::{
    AddFontResourceW, BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
    CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect,
    InvalidateRect, RoundRect, ScreenToClient, SelectObject, SetBkMode, SetTextColor, DT_CENTER,
    DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, PAINTSTRUCT, PS_SOLID,
    SRCCOPY, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, DrawIconEx, GetClientRect,
    GetCursorPos, GetMessageW, GetSystemMetrics, LoadCursorW, LoadIconW, LoadImageW, MessageBoxW,
    PostQuitMessage, RegisterClassW, SendMessageTimeoutW, SetCursor, SetLayeredWindowAttributes,
    SetProcessDPIAware, ShowWindow, TranslateMessage, DI_NORMAL, HICON, HWND_BROADCAST, IDC_ARROW,
    IDC_HAND, IMAGE_ICON, LR_LOADFROMFILE, LWA_ALPHA, MB_OK, MSG, SMTO_ABORTIFHUNG, SM_CXSCREEN,
    SM_CYSCREEN, SW_SHOW, WM_DESTROY, WM_FONTCHANGE, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_PAINT,
    WM_SETCURSOR, WNDCLASSW, WS_CAPTION, WS_EX_LAYERED, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
};

use crate::{
    wide_null, FONT_HINDSILIGURI_BOLD, FONT_HINDSILIGURI_REG, FONT_KALPURUSH, FONT_SIYAMRUPALI,
    FONT_SOLAIMANLIPI, LOGO_PNG_BYTES,
};

static ICON_EKUSHE_ICO: &[u8] = include_bytes!("../../../assets/icons/ekushe.ico");
static ICON_TRAY_BANGLA_ICO: &[u8] = include_bytes!("../../../assets/icons/tray_bangla.ico");
static ICON_TRAY_ENGLISH_ICO: &[u8] = include_bytes!("../../../assets/icons/tray_english.ico");

static mut OPT_CREATE_SHORTCUTS: bool = true;
static mut OPT_AUTO_STARTUP: bool = true;
static mut IS_DARK_THEME: bool = false; // Default: Light Mode!
static mut WIZARD_STEP: i32 = 0; // 0 = Setup Screen, 1 = Installation Success Screen, 2 = Uninstalled Screen
static mut HOVER_ZONE: i32 = 0; // 0=none, 1=chk1, 2=chk2, 3=install/open_studio, 4=uninstall, 5=close/done, 6=theme
static mut APP_ICON_HANDLE: HICON = ptr::null_mut();

// Hit-test rectangles in client coordinates (840 x 536 client area)
const RECT_BTN_THEME: RECT = RECT {
    left: 652,
    top: 32,
    right: 790,
    bottom: 64,
};
const RECT_CHK_SHORTCUT: RECT = RECT {
    left: 32,
    top: 322,
    right: 808,
    bottom: 370,
};
const RECT_CHK_STARTUP: RECT = RECT {
    left: 32,
    top: 380,
    right: 808,
    bottom: 428,
};
const RECT_BTN_INSTALL: RECT = RECT {
    left: 32,
    top: 454,
    right: 450,
    bottom: 506,
};
const RECT_BTN_UNINSTALL: RECT = RECT {
    left: 466,
    top: 454,
    right: 668,
    bottom: 506,
};
const RECT_BTN_CLOSE: RECT = RECT {
    left: 684,
    top: 454,
    right: 808,
    bottom: 506,
};
const RECT_BTN_SUCCESS_STUDIO: RECT = RECT {
    left: 32,
    top: 454,
    right: 492,
    bottom: 506,
};
const RECT_BTN_SUCCESS_DONE: RECT = RECT {
    left: 508,
    top: 454,
    right: 808,
    bottom: 506,
};

/// Returns `%LOCALAPPDATA%\Programs\Ekushe`.
pub fn install_root_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(r"C:\Ekushe"))
        .join("Programs")
        .join("Ekushe")
}

/// Silently ensures that whenever `ekushe.exe` runs on any PC, its icons, Bangladeshi fonts,
/// and default configuration (`Kalpurush` font + `Double-Tap Shift`) are extracted and active.
pub fn ensure_self_configured() {
    let root = install_root_dir();
    let icons_dir = root.join("assets").join("icons");
    let fonts_dir = root.join("assets").join("fonts");

    let _ = fs::create_dir_all(&icons_dir);
    let _ = fs::create_dir_all(&fonts_dir);

    // Extract embedded icons if missing
    write_if_missing(&root.join("logo.png"), LOGO_PNG_BYTES);
    write_if_missing(&icons_dir.join("ekushe.ico"), ICON_EKUSHE_ICO);
    write_if_missing(&icons_dir.join("tray_bangla.ico"), ICON_TRAY_BANGLA_ICO);
    write_if_missing(&icons_dir.join("tray_english.ico"), ICON_TRAY_ENGLISH_ICO);

    // Extract & register bundled Bangladeshi fonts
    let font_files: &[(&str, &str, &[u8])] = &[
        ("Kalpurush (TrueType)", "Kalpurush.ttf", FONT_KALPURUSH),
        (
            "SolaimanLipi (TrueType)",
            "SolaimanLipi.ttf",
            FONT_SOLAIMANLIPI,
        ),
        (
            "Siyam Rupali (TrueType)",
            "SiyamRupali.ttf",
            FONT_SIYAMRUPALI,
        ),
        (
            "Hind Siliguri (TrueType)",
            "HindSiliguri-Regular.ttf",
            FONT_HINDSILIGURI_REG,
        ),
        (
            "Hind Siliguri Bold (TrueType)",
            "HindSiliguri-Bold.ttf",
            FONT_HINDSILIGURI_BOLD,
        ),
    ];

    let user_win_fonts = dirs::data_local_dir()
        .unwrap_or_else(|| root.clone())
        .join("Microsoft")
        .join("Windows")
        .join("Fonts");
    let _ = fs::create_dir_all(&user_win_fonts);

    for (reg_name, filename, bytes) in font_files {
        let app_font_path = fonts_dir.join(filename);
        write_if_missing(&app_font_path, bytes);

        let win_font_path = user_win_fonts.join(filename);
        write_if_missing(&win_font_path, bytes);

        unsafe {
            let wpath = wide_null(&win_font_path.to_string_lossy());
            let _ = AddFontResourceW(wpath.as_ptr());
            let _ = set_hkcu_reg_sz(
                r"Software\Microsoft\Windows NT\CurrentVersion\Fonts",
                reg_name,
                &win_font_path.to_string_lossy(),
            );
        }
    }

    // Ensure default config.toml exists with Kalpurush Bangladeshi font
    let cfg_path = Config::default_config_path();
    if !cfg_path.exists() {
        let cfg = Config::default();
        let _ = cfg.save_to(&cfg_path);
    }
}

/// Performs a complete, zero-command installation of Ekushe onto the user's Windows PC.
pub fn perform_full_install(
    create_desktop_shortcut: bool,
    enable_startup: bool,
    launch_after: bool,
) -> Result<PathBuf, String> {
    let root = install_root_dir();
    let icons_dir = root.join("assets").join("icons");
    let fonts_dir = root.join("assets").join("fonts");

    fs::create_dir_all(&icons_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&fonts_dir).map_err(|e| e.to_string())?;

    // Write icons & logo
    let _ = fs::write(root.join("logo.png"), LOGO_PNG_BYTES);
    let _ = fs::write(icons_dir.join("ekushe.ico"), ICON_EKUSHE_ICO);
    let _ = fs::write(icons_dir.join("tray_bangla.ico"), ICON_TRAY_BANGLA_ICO);
    let _ = fs::write(icons_dir.join("tray_english.ico"), ICON_TRAY_ENGLISH_ICO);

    // Write & register all Bangladeshi fonts
    ensure_self_configured();

    // Copy current executable to %LOCALAPPDATA%\Programs\Ekushe\ekushe.exe & Uninstall-Ekushe.exe
    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let target_exe = root.join("ekushe.exe");
    let uninstall_exe = root.join("Uninstall-Ekushe.exe");

    if current_exe != target_exe {
        stop_running_ekushe();
        copy_checked(&current_exe, &target_exe)?;
    }
    copy_checked(&current_exe, &uninstall_exe)?;

    // Also copy sibling ekushe_tsf.dll / ekushe-cli.exe if present next to installer
    if let Some(src_dir) = current_exe.parent() {
        for name in ["ekushe_tsf.dll", "ekushe-cli.exe", "ekushe_ffi.dll"] {
            let candidate = src_dir.join(name);
            if candidate.exists() && candidate != root.join(name) {
                let _ = fs::copy(&candidate, root.join(name));
            }
        }
    }

    // Register TSF COM Server in HKCU if ekushe_tsf.dll is present
    let tsf_dll = root.join("ekushe_tsf.dll");
    if tsf_dll.exists() {
        let _ = Command::new("regsvr32.exe")
            .args(["/s", &tsf_dll.to_string_lossy()])
            .status();
    }

    // Broadcast WM_FONTCHANGE so all open Windows applications immediately see Kalpurush & SolaimanLipi
    unsafe {
        let mut result: usize = 0;
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_FONTCHANGE,
            0,
            0,
            SMTO_ABORTIFHUNG,
            200,
            &mut result,
        );
    }

    let ico_path = icons_dir.join("ekushe.ico");

    // Create Start Menu Shortcut
    if let Some(appdata) = dirs::data_dir() {
        let start_menu = appdata
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Ekushe Next Bangla Keyboard.lnk");
        create_windows_shortcut(&start_menu, &target_exe, &root, &ico_path);
    }

    // Create Desktop Shortcut
    if create_desktop_shortcut {
        if let Some(desktop) = dirs::desktop_dir() {
            let desktop_lnk = desktop.join("Ekushe Next Bangla Keyboard.lnk");
            create_windows_shortcut(&desktop_lnk, &target_exe, &root, &ico_path);
        }
    }

    // Configure Auto-Start with Windows in HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    unsafe {
        if enable_startup {
            let cmd = format!("\"{}\" --minimized", target_exe.to_string_lossy());
            set_hkcu_reg_sz(
                r"Software\Microsoft\Windows\CurrentVersion\Run",
                "EkusheBanglaKeyboard",
                &cmd,
            )
            .map_err(|code| {
                format!("could not enable start with Windows (registry error {code})")
            })?;
        }

        // Register in Windows "Installed Apps / Add or Remove Programs"
        let uninstall_key =
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\EkusheBanglaKeyboard";
        set_hkcu_reg_sz(
            uninstall_key,
            "DisplayName",
            "Ekushe Next Bangla Keyboard & Spell Checker (একুশে নেক্সট বাংলা কীবোর্ড)",
        )
        .map_err(|code| format!("could not register in Installed Apps (registry error {code})"))?;
        let _ = set_hkcu_reg_sz(uninstall_key, "DisplayVersion", "0.1.0");
        let _ = set_hkcu_reg_sz(uninstall_key, "Publisher", "Ekushe Next Contributors");
        let _ = set_hkcu_reg_sz(uninstall_key, "DisplayIcon", &ico_path.to_string_lossy());
        let _ = set_hkcu_reg_sz(uninstall_key, "InstallLocation", &root.to_string_lossy());
        set_hkcu_reg_sz(
            uninstall_key,
            "UninstallString",
            &format!("\"{}\" --uninstall", uninstall_exe.to_string_lossy()),
        )
        .map_err(|code| format!("could not register the uninstaller (registry error {code})"))?;
    }

    if launch_after && target_exe.exists() {
        let _ = Command::new(&target_exe).current_dir(&root).spawn();
    }

    Ok(target_exe)
}

/// Cleanly uninstalls Ekushe from the user's PC.
pub fn perform_uninstall() -> Result<(), String> {
    let root = install_root_dir();
    stop_running_ekushe();

    let tsf_dll = root.join("ekushe_tsf.dll");
    if tsf_dll.exists() {
        let _ = Command::new("regsvr32.exe")
            .args(["/u", "/s", &tsf_dll.to_string_lossy()])
            .status();
    }

    unsafe {
        let _ = delete_hkcu_reg_value(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            "EkusheBanglaKeyboard",
        );
        let _ = delete_hkcu_reg_tree(
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\EkusheBanglaKeyboard",
        );
    }

    if let Some(desktop) = dirs::desktop_dir() {
        let _ = fs::remove_file(desktop.join("Ekushe Next Bangla Keyboard.lnk"));
        let _ = fs::remove_file(desktop.join("Ekushe Bangla Keyboard.lnk"));
    }
    if let Some(appdata) = dirs::data_dir() {
        let programs = appdata
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        let _ = fs::remove_file(programs.join("Ekushe Next Bangla Keyboard.lnk"));
        let _ = fs::remove_file(programs.join("Ekushe Bangla Keyboard.lnk"));
    }

    remove_install_folder(&root)
}

/// Stop other running copies of Ekushe (the tray app), but never this process.
fn stop_running_ekushe() {
    use std::os::windows::process::CommandExt;
    let me = std::process::id().to_string();
    let _ = Command::new("taskkill.exe")
        .args(["/F", "/IM", "ekushe.exe", "/FI", &format!("PID ne {me}")])
        .creation_flags(0x0800_0000)
        .output();
    std::thread::sleep(std::time::Duration::from_millis(250));
}

/// Copy a program file and confirm the copy is complete.
fn copy_checked(from: &Path, to: &Path) -> Result<(), String> {
    let mut last_err = String::new();
    // the old copy may still be closing after taskkill: retry briefly
    for _ in 0..8 {
        match fs::copy(from, to) {
            Ok(n) => {
                let expected = fs::metadata(from).map(|m| m.len()).unwrap_or(n);
                if n == expected && fs::metadata(to).map(|m| m.len()).unwrap_or(0) == expected {
                    return Ok(());
                }
                last_err = format!("incomplete copy ({n} of {expected} bytes)");
            }
            Err(e) => last_err = e.to_string(),
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    Err(format!("could not copy {}: {last_err}", to.display()))
}

/// Delete the installed program folder. Files in use by this running uninstaller are removed by a
/// short-lived helper after it exits. User settings (config.toml) are kept.
fn remove_install_folder(root: &Path) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    // safety: only ever delete Ekushe's own program folder
    if root.file_name().and_then(|n| n.to_str()) != Some("Ekushe") {
        return Err(format!(
            "refusing to delete unexpected folder {}",
            root.display()
        ));
    }
    let me = env::current_exe().ok();
    let mut in_use = false;
    if let Ok(entries) = fs::read_dir(root) {
        for e in entries.flatten() {
            let p = e.path();
            if me.as_deref() == Some(p.as_path()) {
                in_use = true;
                continue;
            }
            let res = if p.is_dir() {
                fs::remove_dir_all(&p)
            } else {
                fs::remove_file(&p)
            };
            if res.is_err() {
                in_use = true;
            }
        }
    }
    if in_use || fs::remove_dir(root).is_err() {
        use std::os::windows::process::CommandExt;
        // wait for this process to exit, then remove what is left
        let cmd = format!(
            "ping 127.0.0.1 -n 4 >nul & rmdir /S /Q \"{}\"",
            root.display()
        );
        Command::new("cmd.exe")
            .args(["/C", &cmd])
            .creation_flags(0x0800_0000 | 0x0000_0008) // CREATE_NO_WINDOW | DETACHED_PROCESS
            .spawn()
            .map_err(|e| format!("could not schedule removal of {}: {e}", root.display()))?;
    }
    Ok(())
}

/// Runs the native Windows GUI Setup Wizard (`Ekushe-Setup.exe`) in Light Mode.
pub unsafe fn run_gui_installer_window() {
    let _ = SetProcessDPIAware();
    ensure_self_configured();

    let hinstance = GetModuleHandleW(ptr::null());
    let ico_file = install_root_dir()
        .join("assets")
        .join("icons")
        .join("ekushe.ico");
    APP_ICON_HANDLE = LoadImageW(
        ptr::null_mut(),
        wide_null(&ico_file.to_string_lossy()).as_ptr(),
        IMAGE_ICON,
        64,
        64,
        LR_LOADFROMFILE,
    ) as HICON;
    if APP_ICON_HANDLE.is_null() {
        APP_ICON_HANDLE = LoadIconW(hinstance, ptr::without_provenance::<u16>(1));
    }

    let class_name = wide_null("EkusheModernLightSetupWizardClass");
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(installer_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: APP_ICON_HANDLE,
        hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
        hbrBackground: CreateSolidBrush(0x00F5_F7F3),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    let win_w = 856;
    let win_h = 576;
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    let x = ((screen_w - win_w) / 2).max(20);
    let y = ((screen_h - win_h) / 2).max(20);

    let hwnd = CreateWindowExW(
        WS_EX_LAYERED,
        class_name.as_ptr(),
        wide_null("একুশে নেক্সট বাংলা কীবোর্ড — Ekushe Next One-Click Installer").as_ptr(),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        x,
        y,
        win_w,
        win_h,
        ptr::null_mut(),
        ptr::null_mut(),
        hinstance,
        ptr::null(),
    );

    // Apply subtle Frosted Glass window translucency (96% = 245/255)
    let _ = SetLayeredWindowAttributes(hwnd, 0, 245, LWA_ALPHA);

    // Keep Light Mode title bar (0), Windows 11 rounded corners (33 = DWMWCP_ROUND), and Mica backdrop (38)
    let dark_mode: i32 = 0;
    let _ = DwmSetWindowAttribute(
        hwnd,
        20,
        &dark_mode as *const _ as *const _,
        size_of::<i32>() as u32,
    );
    let round_pref: i32 = 2;
    let _ = DwmSetWindowAttribute(
        hwnd,
        33,
        &round_pref as *const _ as *const _,
        size_of::<i32>() as u32,
    );
    let backdrop_type: i32 = 2;
    let _ = DwmSetWindowAttribute(
        hwnd,
        38,
        &backdrop_type as *const _ as *const _,
        size_of::<i32>() as u32,
    );

    ShowWindow(hwnd, SW_SHOW);

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn installer_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_SETCURSOR => {
            let cursor = if HOVER_ZONE > 0 {
                LoadCursorW(ptr::null_mut(), IDC_HAND)
            } else {
                LoadCursorW(ptr::null_mut(), IDC_ARROW)
            };
            SetCursor(cursor);
            1
        }
        WM_MOUSEMOVE => {
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            ScreenToClient(hwnd, &mut pt);
            let new_zone = hit_test_zone(pt.x, pt.y);
            if new_zone != HOVER_ZONE {
                HOVER_ZONE = new_zone;
                InvalidateRect(hwnd, ptr::null(), 0);
            }
            0
        }
        WM_LBUTTONDOWN => {
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            ScreenToClient(hwnd, &mut pt);
            let zone = hit_test_zone(pt.x, pt.y);

            if zone == 6 {
                IS_DARK_THEME = !IS_DARK_THEME;
                let dark_mode: i32 = if IS_DARK_THEME { 1 } else { 0 };
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    20,
                    &dark_mode as *const _ as *const _,
                    size_of::<i32>() as u32,
                );
                InvalidateRect(hwnd, ptr::null(), 0);
                return 0;
            }

            if WIZARD_STEP == 0 {
                match zone {
                    1 => {
                        OPT_CREATE_SHORTCUTS = !OPT_CREATE_SHORTCUTS;
                        InvalidateRect(hwnd, ptr::null(), 0);
                    }
                    2 => {
                        OPT_AUTO_STARTUP = !OPT_AUTO_STARTUP;
                        InvalidateRect(hwnd, ptr::null(), 0);
                    }
                    3 => {
                        match perform_full_install(OPT_CREATE_SHORTCUTS, OPT_AUTO_STARTUP, false) {
                            Ok(target_exe) => {
                                // Start Ekushe immediately in the System Tray so the 'বাং' icon is live!
                                let root = install_root_dir();
                                let _ = Command::new(&target_exe)
                                    .arg("--minimized")
                                    .current_dir(&root)
                                    .spawn();
                                WIZARD_STEP = 1;
                                HOVER_ZONE = 0;
                                InvalidateRect(hwnd, ptr::null(), 0);
                            }
                            Err(e) => {
                                let msg = format!("ইনস্টলেশনে সমস্যা হয়েছে: {e}");
                                MessageBoxW(
                                    hwnd,
                                    wide_null(&msg).as_ptr(),
                                    wide_null("Ekushe Next Installer").as_ptr(),
                                    MB_OK,
                                );
                            }
                        }
                    }
                    4 => {
                        if let Err(e) = perform_uninstall() {
                            let msg = format!("আনইনস্টলে সমস্যা হয়েছে: {e}");
                            MessageBoxW(
                                hwnd,
                                wide_null(&msg).as_ptr(),
                                wide_null("Ekushe Next Installer").as_ptr(),
                                MB_OK,
                            );
                        }
                        WIZARD_STEP = 2;
                        HOVER_ZONE = 0;
                        InvalidateRect(hwnd, ptr::null(), 0);
                    }
                    5 => {
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            } else if WIZARD_STEP == 1 {
                match zone {
                    3 => {
                        // Open Ekushe Studio & Close Installer
                        let root = install_root_dir();
                        let target_exe = root.join("ekushe.exe");
                        if target_exe.exists() {
                            let _ = Command::new(&target_exe)
                                .arg("--open-studio")
                                .current_dir(&root)
                                .spawn();
                        } else {
                            crate::open_studio_window(45210);
                        }
                        DestroyWindow(hwnd);
                    }
                    5 => {
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            } else {
                // WIZARD_STEP == 2 (Uninstalled Confirmation)
                if zone == 3 || zone == 5 {
                    DestroyWindow(hwnd);
                }
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc_screen = BeginPaint(hwnd, &mut ps);

            let mut rc_client = RECT {
                left: 0,
                top: 0,
                right: 840,
                bottom: 536,
            };
            GetClientRect(hwnd, &mut rc_client);
            let width = rc_client.right - rc_client.left;
            let height = rc_client.bottom - rc_client.top;

            // Double-buffered offscreen DC for flicker-free, crisp ClearType rendering
            let hdc = CreateCompatibleDC(hdc_screen);
            let bmp = CreateCompatibleBitmap(hdc_screen, width, height);
            let old_bmp = SelectObject(hdc, bmp as _);

            paint_installer_canvas(hdc, &rc_client);

            BitBlt(hdc_screen, 0, 0, width, height, hdc, 0, 0, SRCCOPY);
            SelectObject(hdc, old_bmp);
            DeleteObject(bmp as _);
            DeleteDC(hdc);

            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_installer_canvas(hdc: HDC, rc_client: &RECT) {
    // Palette (COLORREF = 0x00BBGGRR) — Default: Light Mode!
    let (
        bg_canvas,
        bg_card,
        bg_row_hover,
        bg_pill,
        border_emerald,
        border_subtle,
        emerald_primary,
        emerald_hover,
        text_heading,
        text_brand,
        text_muted,
        text_white,
        crimson_accent,
        border_crimson,
    ) = if !IS_DARK_THEME {
        (
            0x00F5_F7F3, // Soft Alabaster-Mint #F3F7F5
            0x00FF_FFFF, // Pure White Card #FFFFFF
            0x00EF_F5EB, // Soft Emerald Tint #EBF5EF
            0x00EB_F5E6, // Light Emerald Badge #E6F5EB
            0x0052_740E, // Bangladeshi Emerald #0E7452
            0x00D8_E2CF, // Clean Soft Border #CFE2D8
            0x004E_6E0D, // Primary Emerald #0D6E4E
            0x003D_5809, // Primary Hover #09583D
            0x001D_260D, // Deep Forest Charcoal #0D261D
            0x0040_5D0B, // Rich Emerald Brand #0B5D40
            0x005A_6848, // Slate-Emerald Muted #48685A
            0x00FF_FFFF, // Pure White #FFFFFF
            0x002E_1ED9, // Shaheed Crimson #D91E2E
            0x00B8_B0F2, // Soft Crimson Outline #F2B0B8
        )
    } else {
        (
            0x0013_180E, // Deep Obsidian-Emerald #0E1813
            0x001D_2616, // Elevated Card Surface #16261D
            0x0026_331E, // Interactive Row Surface #1E3326
            0x0024_361B, // Dark Pill Badge #1B3624
            0x005E_8A1B, // Emerald Border #1B8A5E
            0x0033_4426, // Subtle Card Border #264433
            0x0053_7D0E, // Primary Emerald #0E7D53
            0x0065_9612, // Primary Hover #129665
            0x00F4_FAF6, // Crisp Off-White #F6FAF4
            0x009B_DB4E, // Soft Mint Accent #4EDB9B
            0x00A6_B89C, // Muted Sage #9CB8A6
            0x00FF_FFFF, // Pure White #FFFFFF
            0x006B_5CE8, // Soft Coral-Crimson #E85C6B
            0x003A_327A, // Dark Crimson Outline #7A323A
        )
    };

    let br_canvas = CreateSolidBrush(bg_canvas);
    FillRect(hdc, rc_client, br_canvas);
    DeleteObject(br_canvas as _);

    SetBkMode(hdc, TRANSPARENT as i32);

    // Balanced, proportional typography optimized for ClearType
    let font_title = CreateFontW(
        -22,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        1,
        0,
        0,
        5,
        0,
        wide_null("Hind Siliguri SemiBold").as_ptr(),
    );
    let font_sub = CreateFontW(
        -15,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        1,
        0,
        0,
        5,
        0,
        wide_null("Hind Siliguri").as_ptr(),
    );
    let font_card_title = CreateFontW(
        -17,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        1,
        0,
        0,
        5,
        0,
        wide_null("Hind Siliguri SemiBold").as_ptr(),
    );
    let font_card_desc = CreateFontW(
        -14,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        1,
        0,
        0,
        5,
        0,
        wide_null("Kalpurush").as_ptr(),
    );
    let font_btn = CreateFontW(
        -17,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        1,
        0,
        0,
        5,
        0,
        wide_null("Hind Siliguri SemiBold").as_ptr(),
    );

    let old_font = SelectObject(hdc, font_title as _);

    // ── 1. Top Banner Card (32, 18 .. 808, 108) ──────────────────────────────
    let top_border = if WIZARD_STEP == 1 {
        border_emerald
    } else {
        border_subtle
    };
    let top_border_w = if WIZARD_STEP == 1 { 2 } else { 1 };
    draw_rounded_box(hdc, 32, 18, 808, 108, 16, bg_card, top_border, top_border_w);

    if !APP_ICON_HANDLE.is_null() {
        DrawIconEx(
            hdc,
            50,
            31,
            APP_ICON_HANDLE,
            62,
            62,
            0,
            ptr::null_mut(),
            DI_NORMAL,
        );
    }

    // Interactive Theme Switcher & Version Pill in Top-Right of Header
    let pill_bg = if HOVER_ZONE == 6 {
        bg_row_hover
    } else {
        bg_pill
    };
    let pill_border = if HOVER_ZONE == 6 {
        border_emerald
    } else {
        border_subtle
    };
    draw_rounded_box(
        hdc,
        RECT_BTN_THEME.left,
        RECT_BTN_THEME.top,
        RECT_BTN_THEME.right,
        RECT_BTN_THEME.bottom,
        12,
        pill_bg,
        pill_border,
        1,
    );
    SelectObject(hdc, font_card_desc as _);
    SetTextColor(hdc, text_brand);
    let theme_label = if !IS_DARK_THEME {
        "☀️ Light · v0.1.0"
    } else {
        "🌙 Dark · v0.1.0"
    };
    draw_text_centered(hdc, &RECT_BTN_THEME, theme_label);

    if WIZARD_STEP == 0 {
        // ══════════════════════════════════════════════════════════════════════
        // STEP 0: INITIAL ONE-CLICK INSTALLER SCREEN
        // ══════════════════════════════════════════════════════════════════════
        SelectObject(hdc, font_title as _);
        SetTextColor(hdc, text_brand);
        draw_text_left(
            hdc,
            128,
            28,
            645,
            64,
            "একুশে নেক্সট (Ekushe Next) — বাংলা কীবোর্ড ও বানান পরীক্ষক",
        );

        SelectObject(hdc, font_sub as _);
        SetTextColor(hdc, text_muted);
        draw_text_left(
            hdc,
            128,
            64,
            785,
            94,
            "কোনো কমান্ড ছাড়াই এক ক্লিকে স্বয়ংক্রিয় ইনস্টলেশন, ফন্ট ও শর্টকাট কনফিগারেশন",
        );

        // 2x2 Feature Bento Grid
        let cards: [(RECT, &str, &str, &str); 4] = [
            (
                RECT {
                    left: 32,
                    top: 122,
                    right: 412,
                    bottom: 206,
                },
                "বাং",
                "অভ্র ফোনেটিক ও ৪টি ফিক্সড লেআউট",
                "Avro Phonetic, প্রভাত, জাতীয়, বর্ণনা ও মুনীর অপটিমা",
            ),
            (
                RECT {
                    left: 428,
                    top: 122,
                    right: 808,
                    bottom: 206,
                },
                "ক",
                "বাংলাদেশী ইউনিকোড ফন্ট সংযুক্ত",
                "কালপুরুষ (ডিফল্ট), সোলাইমানলিপি, নিকশ ও সিয়াম রূপালী",
            ),
            (
                RECT {
                    left: 32,
                    top: 218,
                    right: 412,
                    bottom: 302,
                },
                "স্ব",
                "সহজ যুক্তবর্ণ, কার/ফলা ও বানান পরীক্ষক",
                "স্ব, শ্ব, ধ্ব, ক্ষ, জ্ঞ সহায়িকা, বানান শুদ্ধি ও বিজয় কনভার্টার",
            ),
            (
                RECT {
                    left: 428,
                    top: 218,
                    right: 808,
                    bottom: 302,
                },
                "⇄",
                "ডাবল-ট্যাপ Shift ও সিস্টেম ট্রে",
                "দুইবার Shift চেপে বাংলা (বাং) এবং ইংরেজি (EN) পরিবর্তন",
            ),
        ];

        for (rc, badge, title, desc) in &cards {
            draw_rounded_box(
                hdc,
                rc.left,
                rc.top,
                rc.right,
                rc.bottom,
                14,
                bg_card,
                border_subtle,
                1,
            );
            let pill_rc = RECT {
                left: rc.left + 16,
                top: rc.top + 18,
                right: rc.left + 62,
                bottom: rc.top + 64,
            };
            draw_rounded_box(
                hdc,
                pill_rc.left,
                pill_rc.top,
                pill_rc.right,
                pill_rc.bottom,
                10,
                bg_pill,
                border_subtle,
                1,
            );
            SelectObject(hdc, font_card_title as _);
            SetTextColor(hdc, text_brand);
            draw_text_centered(hdc, &pill_rc, badge);

            SetTextColor(hdc, text_heading);
            draw_text_left(
                hdc,
                rc.left + 76,
                rc.top + 16,
                rc.right - 14,
                rc.top + 46,
                title,
            );

            SelectObject(hdc, font_card_desc as _);
            SetTextColor(hdc, text_muted);
            draw_text_left(
                hdc,
                rc.left + 76,
                rc.top + 46,
                rc.right - 14,
                rc.top + 74,
                desc,
            );
        }

        // Toggle Rows
        draw_checkbox_row(
            hdc,
            &RECT_CHK_SHORTCUT,
            OPT_CREATE_SHORTCUTS,
            HOVER_ZONE == 1,
            "ডেস্কটপ ও স্টার্ট মেনুতে শর্টকাট তৈরি করুন",
            "Desktop & Start Menu",
            bg_card,
            bg_row_hover,
            border_emerald,
            border_subtle,
            emerald_primary,
            text_heading,
            text_muted,
            font_card_title as _,
            font_card_desc as _,
        );

        draw_checkbox_row(
            hdc,
            &RECT_CHK_STARTUP,
            OPT_AUTO_STARTUP,
            HOVER_ZONE == 2,
            "উইন্ডোজ চালু হলে সিস্টেম ট্রে-তে সক্রিয় রাখুন",
            "Auto-Start in System Tray",
            bg_card,
            bg_row_hover,
            border_emerald,
            border_subtle,
            emerald_primary,
            text_heading,
            text_muted,
            font_card_title as _,
            font_card_desc as _,
        );

        // Action Buttons Bar
        SelectObject(hdc, font_btn as _);

        let inst_bg = if HOVER_ZONE == 3 {
            emerald_hover
        } else {
            emerald_primary
        };
        draw_rounded_box(
            hdc,
            RECT_BTN_INSTALL.left,
            RECT_BTN_INSTALL.top,
            RECT_BTN_INSTALL.right,
            RECT_BTN_INSTALL.bottom,
            12,
            inst_bg,
            emerald_hover,
            1,
        );
        SetTextColor(hdc, text_white);
        draw_text_centered(
            hdc,
            &RECT_BTN_INSTALL,
            "✔ এক ক্লিকে ইনস্টল ও চালু করুন (Install & Launch)",
        );

        let uninst_bg = if HOVER_ZONE == 4 {
            bg_row_hover
        } else {
            bg_card
        };
        draw_rounded_box(
            hdc,
            RECT_BTN_UNINSTALL.left,
            RECT_BTN_UNINSTALL.top,
            RECT_BTN_UNINSTALL.right,
            RECT_BTN_UNINSTALL.bottom,
            12,
            uninst_bg,
            border_crimson,
            1,
        );
        SetTextColor(hdc, crimson_accent);
        draw_text_centered(hdc, &RECT_BTN_UNINSTALL, "আনইনস্টল (Uninstall)");

        let close_bg = if HOVER_ZONE == 5 {
            bg_row_hover
        } else {
            bg_card
        };
        draw_rounded_box(
            hdc,
            RECT_BTN_CLOSE.left,
            RECT_BTN_CLOSE.top,
            RECT_BTN_CLOSE.right,
            RECT_BTN_CLOSE.bottom,
            12,
            close_bg,
            border_subtle,
            1,
        );
        SetTextColor(hdc, text_heading);
        draw_text_centered(hdc, &RECT_BTN_CLOSE, "বন্ধ (Close)");
    } else if WIZARD_STEP == 1 {
        // ══════════════════════════════════════════════════════════════════════
        // STEP 1: MODERN INSTALLATION SUCCESS CONFIRMATION SCREEN
        // ══════════════════════════════════════════════════════════════════════
        SelectObject(hdc, font_title as _);
        SetTextColor(hdc, text_brand);
        draw_text_left(
            hdc,
            128,
            28,
            645,
            64,
            "✓ অভিনন্দন! একুশে বাংলা কীবোর্ড সফলভাবে চালু হয়েছে",
        );

        SelectObject(hdc, font_sub as _);
        SetTextColor(hdc, text_muted);
        draw_text_left(
            hdc,
            128,
            64,
            785,
            94,
            "সিস্টেম ট্রে, বাংলাদেশী ফন্ট (কালপুরুষ) এবং শর্টকাট স্বয়ংক্রিয়ভাবে কনফিগার সম্পন্ন হয়েছে",
        );

        // 2x2 Verified Status Bento Cards with Emerald Checkmark Badges
        let status_cards: [(RECT, &str, &str); 4] = [
            (
                RECT {
                    left: 32,
                    top: 122,
                    right: 412,
                    bottom: 206,
                },
                "সিস্টেম ট্রে-তে 'বাং' আইকন সক্রিয়",
                "টাস্কবারে 'বাং' আইকনে Right-Click করে যেকোনো সময় Studio খুলুন",
            ),
            (
                RECT {
                    left: 428,
                    top: 122,
                    right: 808,
                    bottom: 206,
                },
                "ডাবল-ট্যাপ Shift শর্টকাট চালু",
                "দুইবার Shift চাপলে বাংলা (বাং) ও ইংরেজি (EN) পরিবর্তন হবে",
            ),
            (
                RECT {
                    left: 32,
                    top: 218,
                    right: 412,
                    bottom: 302,
                },
                "বাংলাদেশী ফন্ট ইনস্টল সম্পন্ন",
                "কালপুরুষ (ডিফল্ট), সোলাইমানলিপি, নিকশ ও সিয়াম রূপালী প্রস্তুত",
            ),
            (
                RECT {
                    left: 428,
                    top: 218,
                    right: 808,
                    bottom: 302,
                },
                "ডেস্কটপ ও স্টার্ট মেনু শর্টকাট তৈরি",
                "ডেস্কটপ ও স্টার্ট মেনুতে 'Ekushe Next Bangla Keyboard' যুক্ত হয়েছে",
            ),
        ];

        for (rc, title, desc) in &status_cards {
            draw_rounded_box(
                hdc,
                rc.left,
                rc.top,
                rc.right,
                rc.bottom,
                14,
                bg_card,
                border_emerald,
                1,
            );
            let pill_rc = RECT {
                left: rc.left + 16,
                top: rc.top + 18,
                right: rc.left + 62,
                bottom: rc.top + 64,
            };
            draw_rounded_box(
                hdc,
                pill_rc.left,
                pill_rc.top,
                pill_rc.right,
                pill_rc.bottom,
                10,
                emerald_primary,
                emerald_hover,
                1,
            );
            SelectObject(hdc, font_card_title as _);
            SetTextColor(hdc, text_white);
            draw_text_centered(hdc, &pill_rc, "✓");

            SetTextColor(hdc, text_heading);
            draw_text_left(
                hdc,
                rc.left + 76,
                rc.top + 16,
                rc.right - 14,
                rc.top + 46,
                title,
            );

            SelectObject(hdc, font_card_desc as _);
            SetTextColor(hdc, text_muted);
            draw_text_left(
                hdc,
                rc.left + 76,
                rc.top + 46,
                rc.right - 14,
                rc.top + 74,
                desc,
            );
        }

        // Quick-Start Highlight Banner Card (32, 318 .. 808, 430)
        draw_rounded_box(hdc, 32, 318, 808, 430, 14, bg_row_hover, border_emerald, 1);
        let tip_pill = RECT {
            left: 50,
            top: 342,
            right: 98,
            bottom: 390,
        };
        draw_rounded_box(
            hdc,
            tip_pill.left,
            tip_pill.top,
            tip_pill.right,
            tip_pill.bottom,
            12,
            bg_card,
            border_emerald,
            1,
        );
        SelectObject(hdc, font_card_title as _);
        SetTextColor(hdc, text_brand);
        draw_text_centered(hdc, &tip_pill, "বাং");

        SetTextColor(hdc, text_brand);
        draw_text_left(
            hdc,
            114,
            328,
            792,
            360,
            "সিস্টেম ট্রে (System Tray) থেকে একুশে স্টুডিও খোলার উপায়:",
        );

        SelectObject(hdc, font_card_desc as _);
        SetTextColor(hdc, text_heading);
        draw_text_left(
            hdc,
            114,
            362,
            792,
            390,
            "১. টাস্কবারের ডানপাশে 'বাং' আইকনে Right-Click করে \"✨ একুশে স্টুডিও খুলুন\" নির্বাচন করুন।",
        );
        SetTextColor(hdc, text_muted);
        draw_text_left(
            hdc,
            114,
            390,
            792,
            418,
            "২. যেকোনো অ্যাপে দুইবার Shift চেপে অথবা 'বাং' আইকনে Left-Click করে বাংলা ⇄ ইংরেজি পরিবর্তন করুন।",
        );

        // Confirmation Screen Action Buttons
        SelectObject(hdc, font_btn as _);
        let studio_bg = if HOVER_ZONE == 3 {
            emerald_hover
        } else {
            emerald_primary
        };
        draw_rounded_box(
            hdc,
            RECT_BTN_SUCCESS_STUDIO.left,
            RECT_BTN_SUCCESS_STUDIO.top,
            RECT_BTN_SUCCESS_STUDIO.right,
            RECT_BTN_SUCCESS_STUDIO.bottom,
            12,
            studio_bg,
            emerald_hover,
            1,
        );
        SetTextColor(hdc, text_white);
        draw_text_centered(
            hdc,
            &RECT_BTN_SUCCESS_STUDIO,
            "✨ একুশে নেক্সট স্টুডিও খুলুন (Open Ekushe Next Studio)",
        );

        let done_bg = if HOVER_ZONE == 5 {
            bg_row_hover
        } else {
            bg_card
        };
        draw_rounded_box(
            hdc,
            RECT_BTN_SUCCESS_DONE.left,
            RECT_BTN_SUCCESS_DONE.top,
            RECT_BTN_SUCCESS_DONE.right,
            RECT_BTN_SUCCESS_DONE.bottom,
            12,
            done_bg,
            border_emerald,
            1,
        );
        SetTextColor(hdc, text_brand);
        draw_text_centered(
            hdc,
            &RECT_BTN_SUCCESS_DONE,
            "✔ সম্পন্ন ও বন্ধ করুন (Finish & Close)",
        );
    } else {
        // ══════════════════════════════════════════════════════════════════════
        // STEP 2: UNINSTALLED CONFIRMATION SCREEN
        // ══════════════════════════════════════════════════════════════════════
        SelectObject(hdc, font_title as _);
        SetTextColor(hdc, crimson_accent);
        draw_text_left(hdc, 128, 28, 645, 64, "একুশে বাংলা কীবোর্ড আনইনস্টল সম্পন্ন হয়েছে");

        SelectObject(hdc, font_sub as _);
        SetTextColor(hdc, text_muted);
        draw_text_left(
            hdc,
            128,
            64,
            785,
            94,
            "সিস্টেম ট্রে সার্ভিস, শর্টকাট এবং অটো-স্টার্টআপ কনফিগারেশন মুছে ফেলা হয়েছে",
        );

        draw_rounded_box(hdc, 32, 140, 808, 410, 16, bg_card, border_subtle, 1);
        SelectObject(hdc, font_card_title as _);
        SetTextColor(hdc, text_heading);
        let rc_msg = RECT {
            left: 60,
            top: 230,
            right: 780,
            bottom: 280,
        };
        draw_text_centered(
            hdc,
            &rc_msg,
            "একুশে নেক্সট (Ekushe Next) আপনার কম্পিউটার থেকে সফলভাবে আনইনস্টল করা হয়েছে।",
        );
        SelectObject(hdc, font_card_desc as _);
        SetTextColor(hdc, text_muted);
        let rc_sub = RECT {
            left: 60,
            top: 280,
            right: 780,
            bottom: 320,
        };
        draw_text_centered(
            hdc,
            &rc_sub,
            "আবার ইনস্টল করতে চাইলে যেকোনো সময় Ekushe-Setup.exe চালু করুন।",
        );

        SelectObject(hdc, font_btn as _);
        let done_bg = if HOVER_ZONE == 5 {
            bg_row_hover
        } else {
            bg_card
        };
        draw_rounded_box(
            hdc,
            RECT_BTN_SUCCESS_DONE.left,
            RECT_BTN_SUCCESS_DONE.top,
            RECT_BTN_SUCCESS_DONE.right,
            RECT_BTN_SUCCESS_DONE.bottom,
            12,
            done_bg,
            border_subtle,
            1,
        );
        SetTextColor(hdc, text_heading);
        draw_text_centered(hdc, &RECT_BTN_SUCCESS_DONE, "বন্ধ করুন (Close)");
    }

    SelectObject(hdc, old_font);
    DeleteObject(font_title as _);
    DeleteObject(font_sub as _);
    DeleteObject(font_card_title as _);
    DeleteObject(font_card_desc as _);
    DeleteObject(font_btn as _);
}

#[allow(clippy::too_many_arguments)]
unsafe fn draw_checkbox_row(
    hdc: HDC,
    rc: &RECT,
    checked: bool,
    hovered: bool,
    bn_label: &str,
    en_tag: &str,
    bg_normal: COLORREF,
    bg_hover: COLORREF,
    border_active: COLORREF,
    border_idle: COLORREF,
    box_fill: COLORREF,
    text_col: COLORREF,
    muted_col: COLORREF,
    font_main: *mut core::ffi::c_void,
    font_tag: *mut core::ffi::c_void,
) {
    let bg = if hovered { bg_hover } else { bg_normal };
    let border = if hovered || checked {
        border_active
    } else {
        border_idle
    };
    draw_rounded_box(hdc, rc.left, rc.top, rc.right, rc.bottom, 12, bg, border, 1);

    // Modern Checkbox square (22x22)
    let box_left = rc.left + 16;
    let box_top = rc.top + 13;
    let box_right = box_left + 22;
    let box_bottom = box_top + 22;
    let inner_fill = if checked { box_fill } else { bg_normal };
    draw_rounded_box(
        hdc,
        box_left,
        box_top,
        box_right,
        box_bottom,
        6,
        inner_fill,
        border_active,
        1,
    );

    SelectObject(hdc, font_main);
    if checked {
        let chk_rc = RECT {
            left: box_left,
            top: box_top - 1,
            right: box_right,
            bottom: box_bottom,
        };
        SetTextColor(hdc, 0x00FF_FFFF);
        draw_text_centered(hdc, &chk_rc, "✓");
    }

    SetTextColor(hdc, text_col);
    draw_text_left(
        hdc,
        box_right + 14,
        rc.top,
        rc.right - 210,
        rc.bottom,
        bn_label,
    );

    SelectObject(hdc, font_tag);
    SetTextColor(hdc, muted_col);
    draw_text_right(
        hdc,
        rc.right - 220,
        rc.top,
        rc.right - 18,
        rc.bottom,
        en_tag,
    );
}

#[allow(clippy::too_many_arguments)]
unsafe fn draw_rounded_box(
    hdc: HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
    fill_color: COLORREF,
    border_color: COLORREF,
    border_width: i32,
) {
    let brush: HBRUSH = CreateSolidBrush(fill_color);
    let pen = CreatePen(PS_SOLID, border_width, border_color);
    let old_pen = SelectObject(hdc, pen as _);
    let old_brush = SelectObject(hdc, brush as _);
    RoundRect(hdc, left, top, right, bottom, radius, radius);
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    DeleteObject(pen as _);
    DeleteObject(brush as _);
}

unsafe fn draw_text_left(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32, text: &str) {
    let mut rc = RECT {
        left,
        top,
        right,
        bottom,
    };
    let mut w = wide_null(text);
    DrawTextW(
        hdc,
        w.as_mut_ptr(),
        (w.len() - 1) as i32,
        &mut rc,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
}

unsafe fn draw_text_right(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32, text: &str) {
    let mut rc = RECT {
        left,
        top,
        right,
        bottom,
    };
    let mut w = wide_null(text);
    DrawTextW(
        hdc,
        w.as_mut_ptr(),
        (w.len() - 1) as i32,
        &mut rc,
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
}

unsafe fn draw_text_centered(hdc: HDC, rc: &RECT, text: &str) {
    let mut r = *rc;
    let mut w = wide_null(text);
    DrawTextW(
        hdc,
        w.as_mut_ptr(),
        (w.len() - 1) as i32,
        &mut r,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
}

fn hit_test_zone(x: i32, y: i32) -> i32 {
    if pt_in_rect(&RECT_BTN_THEME, x, y) {
        return 6;
    }
    unsafe {
        if WIZARD_STEP == 0 {
            if pt_in_rect(&RECT_CHK_SHORTCUT, x, y) {
                1
            } else if pt_in_rect(&RECT_CHK_STARTUP, x, y) {
                2
            } else if pt_in_rect(&RECT_BTN_INSTALL, x, y) {
                3
            } else if pt_in_rect(&RECT_BTN_UNINSTALL, x, y) {
                4
            } else if pt_in_rect(&RECT_BTN_CLOSE, x, y) {
                5
            } else {
                0
            }
        } else if WIZARD_STEP == 1 {
            if pt_in_rect(&RECT_BTN_SUCCESS_STUDIO, x, y) {
                3
            } else if pt_in_rect(&RECT_BTN_SUCCESS_DONE, x, y) {
                5
            } else {
                0
            }
        } else if pt_in_rect(&RECT_BTN_SUCCESS_DONE, x, y) {
            5
        } else {
            0
        }
    }
}

fn pt_in_rect(r: &RECT, x: i32, y: i32) -> bool {
    x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
}

fn write_if_missing(path: &Path, bytes: &[u8]) {
    if !path.exists() {
        let _ = fs::write(path, bytes);
    }
}

fn create_windows_shortcut(lnk_path: &Path, target_exe: &Path, work_dir: &Path, icon_path: &Path) {
    let ps_cmd = format!(
        "$wsh = New-Object -ComObject WScript.Shell; $sc = $wsh.CreateShortcut('{}'); $sc.TargetPath = '{}'; $sc.WorkingDirectory = '{}'; $sc.IconLocation = '{}'; $sc.Description = 'Ekushe Bangla Keyboard & Spell Checker'; $sc.Save()",
        lnk_path.to_string_lossy().replace('\'', "''"),
        target_exe.to_string_lossy().replace('\'', "''"),
        work_dir.to_string_lossy().replace('\'', "''"),
        icon_path.to_string_lossy().replace('\'', "''"),
    );
    use std::os::windows::process::CommandExt;
    let _ = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &ps_cmd,
        ])
        .creation_flags(0x0800_0000)
        .status();
}

unsafe fn set_hkcu_reg_sz(subkey: &str, value_name: &str, data: &str) -> Result<(), u32> {
    let mut hkey: HKEY = ptr::null_mut();
    let subkey_w = wide_null(subkey);
    let status = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        subkey_w.as_ptr(),
        0,
        ptr::null(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        ptr::null(),
        &mut hkey,
        ptr::null_mut(),
    );
    if status != 0 {
        return Err(status);
    }
    let name_w = wide_null(value_name);
    let data_w = wide_null(data);
    let bytes_len = (data_w.len() * 2) as u32;
    let status = RegSetValueExW(
        hkey,
        name_w.as_ptr(),
        0,
        REG_SZ,
        data_w.as_ptr() as *const u8,
        bytes_len,
    );
    RegCloseKey(hkey);
    if status != 0 {
        return Err(status);
    }
    Ok(())
}

unsafe fn delete_hkcu_reg_value(subkey: &str, value_name: &str) -> Result<(), u32> {
    let mut hkey: HKEY = ptr::null_mut();
    let subkey_w = wide_null(subkey);
    let status = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        subkey_w.as_ptr(),
        0,
        KEY_WRITE,
        &mut hkey,
    );
    if status != 0 {
        return Err(status);
    }
    let name_w = wide_null(value_name);
    let status = RegDeleteValueW(hkey, name_w.as_ptr());
    RegCloseKey(hkey);
    if status != 0 {
        return Err(status);
    }
    Ok(())
}

unsafe fn delete_hkcu_reg_tree(subkey: &str) -> Result<(), u32> {
    let subkey_w = wide_null(subkey);
    let status = RegDeleteTreeW(HKEY_CURRENT_USER, subkey_w.as_ptr());
    if status != 0 {
        return Err(status);
    }
    Ok(())
}
