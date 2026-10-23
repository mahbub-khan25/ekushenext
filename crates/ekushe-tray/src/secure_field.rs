//! Password boxes in browsers and Store apps.
//!
//! The keyboard hook can recognise a classic Windows password box from its window style, but not a
//! password field inside Chrome, Edge, Firefox or a Store app: there the whole page is one window.
//! Windows' accessibility interface (UI Automation) knows, though: the focused element reports
//! `IsPassword`. This module watches focus changes on its own thread and keeps the answer in an atomic
//! the hook reads without waiting.
//!
//! - Focus events only trigger a check while Bangla typing is on, so browsers are not asked
//!   anything while you type English.
//! - From a focus change until its check is done (normally a few milliseconds, never more than
//!   [`PENDING_LIMIT_MS`]) the field counts as a password box, so the first keys typed into a
//!   freshly focused password field are never shown or learned.

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
    DispatchMessageW, GetAncestor, GetForegroundWindow, GetMessageW, PeekMessageW,
    PostThreadMessageW, TranslateMessage, EVENT_OBJECT_FOCUS, GA_ROOT, MSG, PM_REMOVE,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP,
};

/// Longest time a field counts as a password box only because its check is still running
pub const PENDING_LIMIT_MS: u64 = 400;
/// UI Automation gives up on an unresponsive application after this long
const UIA_TIMEOUT_MS: u32 = 350;
const WM_CHECK: u32 = WM_APP + 41;

/// Root window whose focused element is a password box (0: none)
static SECRET_ROOT: AtomicIsize = AtomicIsize::new(0);
/// Root window whose focus check is running, and since when (ms)
static PENDING_ROOT: AtomicIsize = AtomicIsize::new(0);
static PENDING_SINCE: AtomicU64 = AtomicU64::new(0);
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

/// The hook reports whether Bangla typing is on. Turning it on checks the focused field at once.
pub fn set_active(on: bool) {
    let was = ACTIVE.swap(on, Ordering::SeqCst);
    if on && !was {
        let root = unsafe { GetAncestor(GetForegroundWindow(), GA_ROOT) };
        request_check(root.0 as isize);
    }
    if !on {
        SECRET_ROOT.store(0, Ordering::SeqCst);
        PENDING_ROOT.store(0, Ordering::SeqCst);
    }
}

/// Whether the field focused in the window `root` (a top-level window) is a password box, or is
/// still being checked.
pub fn is_secret(root: isize) -> bool {
    if root == 0 {
        return false;
    }
    if SECRET_ROOT.load(Ordering::SeqCst) == root {
        return true;
    }
    PENDING_ROOT.load(Ordering::SeqCst) == root
        && now_ms().saturating_sub(PENDING_SINCE.load(Ordering::SeqCst)) < PENDING_LIMIT_MS
}

fn request_check(root: isize) {
    let tid = THREAD.load(Ordering::SeqCst);
    if tid == 0 || root == 0 {
        return;
    }
    PENDING_SINCE.store(now_ms(), Ordering::SeqCst);
    PENDING_ROOT.store(root, Ordering::SeqCst);
    unsafe {
        let _ = PostThreadMessageW(tid, WM_CHECK, WPARAM(0), LPARAM(root));
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
    if !ACTIVE.load(Ordering::SeqCst) || hwnd.0.is_null() {
        return;
    }
    let root = GetAncestor(hwnd, GA_ROOT);
    request_check(if root.0.is_null() {
        hwnd.0 as isize
    } else {
        root.0 as isize
    });
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
            let secret = uia
                .as_ref()
                .and_then(|u| u.GetFocusedElement().ok())
                .and_then(|e| e.CurrentIsPassword().ok())
                .is_some_and(|b| b.as_bool());
            if secret {
                SECRET_ROOT.store(root, Ordering::SeqCst);
            } else if SECRET_ROOT.load(Ordering::SeqCst) == root {
                SECRET_ROOT.store(0, Ordering::SeqCst);
            }
            if PENDING_ROOT.load(Ordering::SeqCst) == root {
                PENDING_ROOT.store(0, Ordering::SeqCst);
            }
            continue;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
