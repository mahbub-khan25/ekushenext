//! Password boxes in browsers and Store apps.
//!
//! The keyboard hook can recognise a classic Windows password box from its window style, but not a
//! password field inside Chrome, Edge, Firefox, an Electron app or a Store app: there the whole page
//! is one window. Windows' accessibility interface (UI Automation) knows, though: the focused element
//! reports `IsPassword`. This module watches focus changes on its own thread and keeps the answer in
//! an atomic that the keyboard hook reads without ever waiting.
//!
//! - Checks run only while Bangla typing is on, and only for windows that host web or Store content
//!   (classic windows are covered by the window-style check), so other apps are never asked.
//! - Typing never waits for a check: until the answer arrives (normally a few milliseconds after the
//!   focus change) the field is treated as an ordinary one.
//! - While a window counts as a password box, it is re-checked now and then, so a login page that
//!   removes its password field without moving focus does not keep Bangla off.

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use windows::core::Interface;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomation2, SetWinEventHook, HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetAncestor, GetClassNameW, GetForegroundWindow, GetMessageW, PeekMessageW,
    PostThreadMessageW, TranslateMessage, EVENT_OBJECT_FOCUS, GA_ROOT, MSG, PM_REMOVE,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP,
};

/// UI Automation gives up on an unresponsive application after this long
const UIA_TIMEOUT_MS: u32 = 350;
/// A window that counts as a password box is checked again after this long
const RECHECK_MS: u64 = 1500;
const WM_CHECK: u32 = WM_APP + 41;

/// Root window whose focused element is a password box (0: none), and when that was last checked
static SECRET_ROOT: AtomicIsize = AtomicIsize::new(0);
static SECRET_CHECKED: AtomicU64 = AtomicU64::new(0);
/// Bangla typing is on (checks run only then)
static ACTIVE: AtomicBool = AtomicBool::new(false);
static THREAD: AtomicU32 = AtomicU32::new(0);
static STARTED: AtomicBool = AtomicBool::new(false);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Start the watcher thread (once).
pub fn start() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("ekushe-secure-field".into())
        .spawn(|| unsafe { run() });
}

/// The hook reports whether Bangla typing is on. Turning it on checks the focused field (in the
/// background; this key is not held up).
pub fn set_active(on: bool) {
    let was = ACTIVE.swap(on, Ordering::SeqCst);
    if on && !was {
        let fg = unsafe { GetForegroundWindow() };
        request_check(fg);
    }
    if !on {
        SECRET_ROOT.store(0, Ordering::SeqCst);
    }
}

/// Whether the field focused in the top-level window `root` is a password box, as last reported.
pub fn is_secret(root: isize) -> bool {
    if root == 0 || SECRET_ROOT.load(Ordering::SeqCst) != root {
        return false;
    }
    if now_ms().saturating_sub(SECRET_CHECKED.load(Ordering::SeqCst)) > RECHECK_MS {
        request_check(HWND(root as *mut core::ffi::c_void));
    }
    true
}

/// Windows whose fields only UI Automation can tell apart: browsers, Electron and WebView2 apps,
/// Store apps, and old embedded IE pages.
fn hosts_web_or_store_content(h: HWND) -> bool {
    let class = |w: HWND| -> String {
        let mut buf = [0u16; 96];
        let n = unsafe { GetClassNameW(w, &mut buf) };
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    };
    let root = unsafe { GetAncestor(h, GA_ROOT) };
    [class(h), class(root)].iter().any(|c| {
        c.starts_with("Chrome_")
            || c.starts_with("Mozilla")
            || c == "Windows.UI.Core.CoreWindow"
            || c == "ApplicationFrameWindow"
            || c == "Internet Explorer_Server"
    })
}

fn request_check(h: HWND) {
    let tid = THREAD.load(Ordering::SeqCst);
    if tid == 0 || h.0.is_null() {
        return;
    }
    let root = unsafe { GetAncestor(h, GA_ROOT) };
    let root = if root.0.is_null() { h } else { root };
    if !hosts_web_or_store_content(h) {
        // Not a window this check is for: whatever was reported for it no longer applies
        if SECRET_ROOT.load(Ordering::SeqCst) == root.0 as isize {
            SECRET_ROOT.store(0, Ordering::SeqCst);
        }
        return;
    }
    unsafe {
        let _ = PostThreadMessageW(tid, WM_CHECK, WPARAM(0), LPARAM(root.0 as isize));
    }
}

unsafe extern "system" fn on_focus(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if ACTIVE.load(Ordering::SeqCst) && !hwnd.0.is_null() {
        request_check(hwnd);
    }
}

unsafe fn run() {
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    let uia: Option<IUIAutomation> =
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok();
    if let Some(u2) = uia.as_ref().and_then(|u| u.cast::<IUIAutomation2>().ok()) {
        let _ = u2.SetConnectionTimeout(UIA_TIMEOUT_MS);
        let _ = u2.SetTransactionTimeout(UIA_TIMEOUT_MS);
    }
    THREAD.store(GetCurrentThreadId(), Ordering::SeqCst);
    let _hook = SetWinEventHook(
        EVENT_OBJECT_FOCUS,
        EVENT_OBJECT_FOCUS,
        None,
        Some(on_focus),
        0,
        0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    );
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
        if msg.message == WM_CHECK {
            // Only the latest focus change matters
            let mut root = msg.lParam.0;
            let mut more = MSG::default();
            while PeekMessageW(&mut more, HWND::default(), WM_CHECK, WM_CHECK, PM_REMOVE).as_bool()
            {
                root = more.lParam.0;
            }
            if !ACTIVE.load(Ordering::SeqCst) {
                continue;
            }
            let secret = uia
                .as_ref()
                .and_then(|u| u.GetFocusedElement().ok())
                .and_then(|e| e.CurrentIsPassword().ok())
                .is_some_and(|b| b.as_bool());
            // The answer is for the window that has the focus now
            let fg = GetAncestor(GetForegroundWindow(), GA_ROOT).0 as isize;
            let target = if fg != 0 { fg } else { root };
            if secret {
                SECRET_CHECKED.store(now_ms(), Ordering::SeqCst);
                SECRET_ROOT.store(target, Ordering::SeqCst);
            } else if SECRET_ROOT.load(Ordering::SeqCst) == target
                || SECRET_ROOT.load(Ordering::SeqCst) == root
            {
                SECRET_ROOT.store(0, Ordering::SeqCst);
            }
            continue;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
