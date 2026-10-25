//! "Check for updates": asks GitHub which Ekushe Next release is newest (one HTTPS request to
//! api.github.com, nothing about the user is sent) and offers the download page when it is newer than
//! this one. Done only when the user asks from the tray menu, or once a day in the background if the
//! user turned that on in Studio (off by default: Ekushe makes no network request on its own).

use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Duration;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use windows_sys::Win32::Networking::WinHttp::*;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_INFO, NIIF_INFO, NIM_MODIFY, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_YESNO,
};

/// This version, as released (`v0.1.0-beta.15`).
pub fn current_version() -> &'static str {
    include_str!("../../../.github/release/VERSION").trim()
}

/// Where people download Ekushe Next.
pub const DOWNLOAD_PAGE: &str = "https://ekushenext.com/#download";
const RELEASES_PATH: &str = "/repos/mahbub-khan25/ekushenext/releases?per_page=10";

/// A newer release found by the last check (its tag), shown in the tray menu.
pub static AVAILABLE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
/// The tray window, for the "new version" notice
static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// GET `https://api.github.com{path}`; the body if the answer is 200.
fn https_get(path: &str) -> Option<String> {
    struct H(*mut core::ffi::c_void);
    impl Drop for H {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: the handle came from WinHttp and is closed once
                unsafe { WinHttpCloseHandle(self.0) };
            }
        }
    }
    let agent = wide(&format!("EkusheNext/{}", current_version()));
    // SAFETY: every pointer passed is a live NUL-terminated wide string or a buffer of the given size.
    unsafe {
        let session = H(WinHttpOpen(
            agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            std::ptr::null(),
            std::ptr::null(),
            0,
        ));
        if session.0.is_null() {
            return None;
        }
        WinHttpSetTimeouts(session.0, 10_000, 10_000, 10_000, 15_000);
        let host = wide("api.github.com");
        let conn = H(WinHttpConnect(
            session.0,
            host.as_ptr(),
            INTERNET_DEFAULT_HTTPS_PORT,
            0,
        ));
        if conn.0.is_null() {
            return None;
        }
        let (verb, obj) = (wide("GET"), wide(path));
        let req = H(WinHttpOpenRequest(
            conn.0,
            verb.as_ptr(),
            obj.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        ));
        if req.0.is_null() {
            return None;
        }
        let headers =
            wide("Accept: application/vnd.github+json\r\nX-GitHub-Api-Version: 2022-11-28\r\n");
        if WinHttpSendRequest(req.0, headers.as_ptr(), u32::MAX, std::ptr::null(), 0, 0, 0) == 0
            || WinHttpReceiveResponse(req.0, std::ptr::null_mut()) == 0
        {
            return None;
        }
        let mut status: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        WinHttpQueryHeaders(
            req.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            std::ptr::null(),
            &mut status as *mut u32 as *mut core::ffi::c_void,
            &mut len,
            std::ptr::null_mut(),
        );
        if status != 200 {
            return None;
        }
        let mut body = Vec::new();
        let mut buf = vec![0u8; 16 * 1024];
        loop {
            let mut got = 0u32;
            if WinHttpReadData(
                req.0,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
                &mut got,
            ) == 0
            {
                return None;
            }
            if got == 0 {
                break;
            }
            body.extend_from_slice(&buf[..got as usize]);
            if body.len() > 4 * 1024 * 1024 {
                return None;
            }
        }
        String::from_utf8(body).ok()
    }
}

/// The newest published release tag (beta releases count), or `None` if GitHub cannot be reached.
pub fn latest_release() -> Option<String> {
    let body = https_get(RELEASES_PATH)?;
    let list: serde_json::Value = serde_json::from_str(&body).ok()?;
    list.as_array()?
        .iter()
        .filter(|r| r.get("draft").and_then(|d| d.as_bool()) != Some(true))
        .filter_map(|r| r.get("tag_name").and_then(|t| t.as_str()))
        .max_by(|a, b| {
            if ekushe_core::is_newer_version(a, b) {
                std::cmp::Ordering::Greater
            } else if ekushe_core::is_newer_version(b, a) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .map(String::from)
}

/// Check now; remember a newer release for the tray menu. `Err` when GitHub cannot be reached.
fn check() -> Result<Option<String>, ()> {
    let latest = latest_release().ok_or(())?;
    let newer = ekushe_core::is_newer_version(&latest, current_version()).then_some(latest);
    *AVAILABLE.lock() = newer.clone();
    Ok(newer)
}

fn message(text: &str, flags: u32) -> i32 {
    let (t, c) = (wide(text), wide("একুশে নেক্সট (Ekushe Next)"));
    // SAFETY: both strings are live and NUL-terminated
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            t.as_ptr(),
            c.as_ptr(),
            flags | MB_SETFOREGROUND,
        )
    }
}

/// The user chose "Check for updates" in the tray menu (runs in the background, answers in a box).
pub fn check_now_and_tell() {
    std::thread::spawn(|| match check() {
        Ok(Some(tag)) => {
            let text = format!(
                "নতুন সংস্করণ {tag} এসেছে (আপনার আছে {}).\n\nডাউনলোড পেজ খুলব?\nA new version is available. Open the download page?",
                current_version()
            );
            if message(&text, MB_YESNO | MB_ICONINFORMATION) == IDYES {
                crate::open_in_browser(DOWNLOAD_PAGE);
            }
        }
        Ok(None) => {
            message(
                &format!(
                    "আপনি সর্বশেষ সংস্করণ ব্যবহার করছেন ({}).\nYou have the latest version.",
                    current_version()
                ),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        Err(()) => {
            message(
                "এখন হালনাগাদ দেখা গেল না। ইন্টারনেট সংযোগ দেখে আবার চেষ্টা করুন।\nCould not check for updates. Check your internet connection and try again.",
                MB_OK | MB_ICONERROR,
            );
        }
    });
}

/// Show the "new version" notice from the tray icon (clicking it opens the download page).
fn notify(tag: &str) {
    let hwnd = TRAY_HWND.load(Ordering::Relaxed);
    if hwnd == 0 {
        return;
    }
    // SAFETY: a zeroed NOTIFYICONDATAW is valid; the strings are copied into its fixed buffers.
    unsafe {
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd as _;
        nid.uID = 1;
        nid.uFlags = NIF_INFO;
        nid.dwInfoFlags = NIIF_INFO;
        let title = wide("একুশে নেক্সটের নতুন সংস্করণ এসেছে");
        let body = wide(&format!(
            "{tag} — ডাউনলোড করতে এখানে ক্লিক করুন। (New version available)"
        ));
        for (i, &c) in title.iter().take(63).enumerate() {
            nid.szInfoTitle[i] = c;
        }
        for (i, &c) in body.iter().take(255).enumerate() {
            nid.szInfo[i] = c;
        }
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

/// Start the daily background check (the first one a couple of minutes after Ekushe starts).
/// `enabled` is read before every check, so turning it off in Studio stops it.
pub fn start_background(hwnd: isize, enabled: fn() -> bool) {
    TRAY_HWND.store(hwnd, Ordering::Relaxed);
    std::thread::spawn(move || {
        let mut told: Option<String> = None;
        std::thread::sleep(Duration::from_secs(120));
        loop {
            if enabled() {
                if let Ok(Some(tag)) = check() {
                    if told.as_deref() != Some(tag.as_str()) {
                        notify(&tag);
                        told = Some(tag);
                    }
                }
            }
            std::thread::sleep(Duration::from_secs(24 * 60 * 60));
        }
    });
}
