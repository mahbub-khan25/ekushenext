//! Convert selected text in any app (Ctrl+Alt+C by default): old Bijoy text becomes Unicode, Banglish
//! becomes Bangla.
//!
//! The selection is read the way a user would: copy (Ctrl+C), convert, paste (Ctrl+V). Whatever was
//! on the clipboard before is put back afterwards, so the user's clipboard is not lost.
//!
//! Runs on its own thread, never in the keyboard hook: it waits for the user to let go of the
//! hotkey (an app must not see Ctrl+Alt still held when Ctrl+C arrives) and for the app to answer.

use std::mem::size_of;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use ekushe_core::convert::{bijoy_to_unicode, detect_selection, map_words, SelectionKind};
use windows_sys::Win32::Foundation::{GlobalFree, HANDLE, HWND};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, HWND_MESSAGE};

const CF_UNICODETEXT: u32 = 13;
/// Formats that are not memory blocks (GDI objects, metafiles, owner-drawn): Windows makes the
/// bitmap ones again from CF_DIB, the rest can't be copied safely and are left out.
const NOT_MEMORY: &[u32] = &[2, 3, 9, 14, 0x80, 0x82, 0x83, 0x8E];
/// Text formats Windows makes from CF_UNICODETEXT by itself
const DERIVED_TEXT: &[u32] = &[1, 7, 16];
/// More than this is not kept (and not converted)
const MAX_CLIPBOARD_BYTES: usize = 32 << 20;
const MAX_TEXT_CHARS: usize = 20_000;

static BUSY: AtomicBool = AtomicBool::new(false);

/// What happened, for the badge
#[derive(Clone, Copy)]
pub enum Outcome {
    Converted(SelectionKind),
    NothingSelected,
    NothingToConvert,
    TooLong,
    Failed,
}

/// A hotkey such as `Ctrl+Alt+C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub vk: u16,
}

impl Hotkey {
    /// Parse `Ctrl+Alt+C`, `Ctrl+Shift+F9` and the like. Needs at least one of Ctrl, Alt or Win.
    pub fn parse(s: &str) -> Option<Hotkey> {
        let mut hk = Hotkey {
            ctrl: false,
            alt: false,
            shift: false,
            win: false,
            vk: 0,
        };
        for part in s.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()) {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => hk.ctrl = true,
                "alt" => hk.alt = true,
                "shift" => hk.shift = true,
                "win" | "super" => hk.win = true,
                k => {
                    if hk.vk != 0 {
                        return None;
                    }
                    let b = k.as_bytes();
                    hk.vk = if b.len() == 1 && b[0].is_ascii_alphanumeric() {
                        b[0].to_ascii_uppercase() as u16
                    } else {
                        let n = k.strip_prefix('f').and_then(|n| n.parse::<u16>().ok())?;
                        if !(1..=24).contains(&n) {
                            return None;
                        }
                        0x70 + n - 1
                    }
                }
            }
        }
        (hk.vk != 0 && (hk.ctrl || hk.alt || hk.win)).then_some(hk)
    }

    pub fn matches(&self, vk: u16, ctrl: bool, alt: bool, shift: bool, win: bool) -> bool {
        self.vk == vk
            && self.ctrl == ctrl
            && self.alt == alt
            && self.shift == shift
            && self.win == win
    }
}

/// Start converting the selection in the focused app. `banglish_word` turns one Roman word into
/// Bangla (as Space would); `done` reports the outcome. Ignored while a conversion is running.
pub fn start(
    banglish_word: impl Fn(&str) -> String + Send + 'static,
    done: impl FnOnce(Outcome) + Send + 'static,
) {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("ekushe-convert-selection".into())
        .spawn(move || {
            let outcome = unsafe { run(&banglish_word) };
            BUSY.store(false, Ordering::SeqCst);
            done(outcome);
        });
}

fn key_down(vk: u16) -> bool {
    unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 }
}

unsafe fn send_keys(keys: &[(u16, bool)]) -> bool {
    let inputs: Vec<INPUT> = keys
        .iter()
        .map(|&(vk, up)| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: crate::EKUSHE_EXTRA_INFO,
                },
            },
        })
        .collect();
    SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        size_of::<INPUT>() as i32,
    ) as usize
        == inputs.len()
}

/// Ctrl+<key>, as if typed (with no other modifier held)
unsafe fn send_ctrl(vk: u16) -> bool {
    send_keys(&[
        (VK_CONTROL, false),
        (vk, false),
        (vk, true),
        (VK_CONTROL, true),
    ])
}

/// The clipboard, opened for this thread's window; closed when dropped.
struct Clipboard;

impl Clipboard {
    unsafe fn open(owner: HWND) -> Option<Clipboard> {
        // Another program may have it open for a moment
        for _ in 0..25 {
            if OpenClipboard(owner) != 0 {
                return Some(Clipboard);
            }
            sleep(Duration::from_millis(20));
        }
        None
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}

/// Everything on the clipboard that can be copied: (format, bytes).
unsafe fn snapshot() -> Vec<(u32, Vec<u8>)> {
    let mut saved = Vec::new();
    let mut total = 0usize;
    let has_unicode = {
        let mut f = 0;
        let mut found = false;
        loop {
            f = EnumClipboardFormats(f);
            if f == 0 {
                break;
            }
            found |= f == CF_UNICODETEXT;
        }
        found
    };
    let mut fmt = 0;
    loop {
        fmt = EnumClipboardFormats(fmt);
        if fmt == 0 {
            break;
        }
        if NOT_MEMORY.contains(&fmt)
            || (0x300..=0x3FF).contains(&fmt)
            || (has_unicode && DERIVED_TEXT.contains(&fmt))
        {
            continue;
        }
        let h = GetClipboardData(fmt);
        if h.is_null() {
            continue;
        }
        let size = GlobalSize(h);
        if size == 0 || total + size > MAX_CLIPBOARD_BYTES {
            continue;
        }
        let p = GlobalLock(h) as *const u8;
        if p.is_null() {
            continue;
        }
        let bytes = std::slice::from_raw_parts(p, size).to_vec();
        GlobalUnlock(h);
        total += size;
        saved.push((fmt, bytes));
    }
    saved
}

unsafe fn put(fmt: u32, bytes: &[u8]) -> bool {
    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1));
    if h.is_null() {
        return false;
    }
    let p = GlobalLock(h) as *mut u8;
    if p.is_null() {
        GlobalFree(h);
        return false;
    }
    ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    GlobalUnlock(h);
    if SetClipboardData(fmt, h as HANDLE).is_null() {
        GlobalFree(h);
        return false;
    }
    true
}

unsafe fn read_text() -> Option<String> {
    let h = GetClipboardData(CF_UNICODETEXT);
    if h.is_null() {
        return None;
    }
    let p = GlobalLock(h) as *const u16;
    if p.is_null() {
        return None;
    }
    let max = GlobalSize(h) / 2;
    let len = (0..max).take_while(|&i| *p.add(i) != 0).count();
    let text = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
    GlobalUnlock(h);
    Some(text)
}

unsafe fn write_text(text: &str) -> bool {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    let bytes = std::slice::from_raw_parts(units.as_ptr() as *const u8, units.len() * 2);
    EmptyClipboard() != 0 && put(CF_UNICODETEXT, bytes)
}

/// Put the saved clipboard back, unless something else has written to it in the meantime.
unsafe fn restore(owner: HWND, saved: &[(u32, Vec<u8>)], ours: u32) {
    if GetClipboardSequenceNumber() != ours {
        return;
    }
    if let Some(_cb) = Clipboard::open(owner) {
        EmptyClipboard();
        for (fmt, bytes) in saved {
            put(*fmt, bytes);
        }
    }
}

unsafe fn run(banglish_word: &dyn Fn(&str) -> String) -> Outcome {
    // Let go of the hotkey first: the app must see a plain Ctrl+C. An unused key in between keeps a
    // released Alt from opening the app's menu.
    send_keys(&[(0xE8, false), (0xE8, true)]);
    let until = Instant::now() + Duration::from_millis(1500);
    let held = [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN];
    while held.iter().any(|&k| key_down(k)) && Instant::now() < until {
        sleep(Duration::from_millis(15));
    }
    let still: Vec<(u16, bool)> = held
        .iter()
        .filter(|&&k| key_down(k))
        .map(|&k| (k, true))
        .collect();
    if !still.is_empty() {
        send_keys(&still);
    }

    // A window of this thread owns the clipboard while we use it
    let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
    let owner = CreateWindowExW(
        0,
        class.as_ptr(),
        ptr::null(),
        0,
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null(),
    );
    if owner.is_null() {
        return Outcome::Failed;
    }
    let outcome = convert_with(owner, banglish_word);
    DestroyWindow(owner);
    outcome
}

unsafe fn convert_with(owner: HWND, banglish_word: &dyn Fn(&str) -> String) -> Outcome {
    let saved = match Clipboard::open(owner) {
        Some(_cb) => snapshot(),
        None => return Outcome::Failed,
    };

    // Copy the selection
    let before = GetClipboardSequenceNumber();
    if !send_ctrl(b'C' as u16) {
        return Outcome::Failed;
    }
    let until = Instant::now() + Duration::from_millis(700);
    while GetClipboardSequenceNumber() == before && Instant::now() < until {
        sleep(Duration::from_millis(15));
    }
    if GetClipboardSequenceNumber() == before {
        // Nothing was copied: no selection (the clipboard is untouched)
        return Outcome::NothingSelected;
    }
    // The app may still be writing more formats; give it a moment
    sleep(Duration::from_millis(40));
    let copied = Clipboard::open(owner).and_then(|_cb| read_text());
    let after_copy = GetClipboardSequenceNumber();
    let Some(text) = copied.filter(|t| !t.is_empty()) else {
        restore(owner, &saved, after_copy);
        return Outcome::NothingSelected;
    };
    if text.chars().count() > MAX_TEXT_CHARS {
        restore(owner, &saved, after_copy);
        return Outcome::TooLong;
    }

    let converted = detect_selection(&text).and_then(|kind| {
        let out = match kind {
            SelectionKind::BijoyToUnicode => bijoy_to_unicode(&text),
            SelectionKind::BanglishToBangla => map_words(&text, banglish_word),
        };
        (out != text).then_some((out, kind))
    });
    let Some((out, kind)) = converted else {
        restore(owner, &saved, after_copy);
        return Outcome::NothingToConvert;
    };

    // Paste the result over the selection
    let written = Clipboard::open(owner)
        .map(|_cb| write_text(&out))
        .unwrap_or(false);
    if !written {
        restore(owner, &saved, GetClipboardSequenceNumber());
        return Outcome::Failed;
    }
    let ours = GetClipboardSequenceNumber();
    let pasted = send_ctrl(b'V' as u16);
    // Apps read the clipboard a little after Ctrl+V arrives
    sleep(Duration::from_millis(450));
    restore(owner, &saved, ours);
    if pasted {
        Outcome::Converted(kind)
    } else {
        Outcome::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkeys_parse() {
        let hk = Hotkey::parse("Ctrl+Alt+C").unwrap();
        assert!(hk.ctrl && hk.alt && !hk.shift && !hk.win && hk.vk == b'C' as u16);
        assert_eq!(Hotkey::parse("ctrl + shift + f9").unwrap().vk, 0x78);
        assert!(Hotkey::parse("C").is_none());
        assert!(Hotkey::parse("Shift+C").is_none());
        assert!(Hotkey::parse("Ctrl+Alt").is_none());
        assert!(Hotkey::parse("Ctrl+Alt+C+D").is_none());
    }
}
