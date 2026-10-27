//! C ABI for `ekushe-core`.
//!
//! Two layers:
//! - `ekushe_*`: a handle-based session API (one engine per handle).
//! - `ek_*`: one shared engine for a whole input-method process (macOS InputMethodKit): settings
//!   and learned words load from disk, and the Studio server (feature `studio`) works on the same
//!   engine, so a change made in Studio applies to typing right away.
//!
//! Strings returned to the caller are NUL-terminated UTF-8 and must be freed with
//! `ekushe_string_free`. Every entry point catches panics.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use ekushe_core::{
    config::{Config, InputLanguage},
    convert::{bijoy_to_unicode, unicode_to_bijoy},
    Ekushe, SessionOutput,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

#[cfg(feature = "studio")]
pub mod studio;

pub struct EkusheHandle {
    inner: Ekushe,
}

fn to_c(s: String) -> *mut c_char {
    CString::new(s)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

fn json(out: &SessionOutput) -> *mut c_char {
    to_c(serde_json::to_string(out).unwrap_or_else(|_| "{}".to_string()))
}

#[no_mangle]
pub extern "C" fn ekushe_new() -> *mut EkusheHandle {
    catch_unwind(|| {
        Box::into_raw(Box::new(EkusheHandle {
            inner: Ekushe::default(),
        }))
    })
    .unwrap_or(ptr::null_mut())
}

/// # Safety
/// `handle` must be a valid pointer returned by `ekushe_new`.
#[no_mangle]
pub unsafe extern "C" fn ekushe_free(handle: *mut EkusheHandle) {
    if !handle.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(handle))));
    }
}

/// Feed a character and return a newly allocated JSON string (`SessionOutput`).
///
/// # Safety
/// `handle` must be a valid pointer returned by `ekushe_new`.
#[no_mangle]
pub unsafe extern "C" fn ekushe_push_char(
    handle: *mut EkusheHandle,
    codepoint: u32,
    altgr: bool,
) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }
    let Some(ch) = char::from_u32(codepoint) else {
        return ptr::null_mut();
    };
    let h = &mut *handle;
    catch_unwind(AssertUnwindSafe(|| json(&h.inner.push_char(ch, altgr))))
        .unwrap_or(ptr::null_mut())
}

/// Convert Bijoy ANSI text to Unicode Bangla.
///
/// # Safety
/// `input` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ekushe_bijoy_to_unicode(input: *const c_char) -> *mut c_char {
    match cstr(input) {
        Some(s) => to_c(bijoy_to_unicode(s)),
        None => ptr::null_mut(),
    }
}

/// Convert Unicode Bangla text to Bijoy ANSI.
///
/// # Safety
/// `input` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ekushe_unicode_to_bijoy(input: *const c_char) -> *mut c_char {
    match cstr(input) {
        Some(s) => to_c(unicode_to_bijoy(s)),
        None => ptr::null_mut(),
    }
}

/// Free a C string returned by any `ekushe_*` / `ek_*` function.
///
/// # Safety
/// `s` must be a pointer returned by this library (or null).
#[no_mangle]
pub unsafe extern "C" fn ekushe_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

unsafe fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        CStr::from_ptr(p).to_str().ok()
    }
}

// ───────────────────────── Shared engine (one per process) ─────────────────────────

pub(crate) struct Shared {
    pub ekushe: Ekushe,
    pub saved_learned_version: u64,
    /// When `learned.tsv` was last read or written by this process (another process, such as Studio
    /// restoring a backup, may change it; it is then read again)
    pub learned_mtime: Option<std::time::SystemTime>,
    /// The last key's result, for C callers that read it field by field (`ek_out_*`)
    pub last: SessionOutput,
}

/// Remember `out` for the `ek_out_*` accessors and return it as JSON.
fn record(sh: &mut Shared, out: SessionOutput) -> *mut c_char {
    let j = json(&out);
    sh.last = out;
    j
}

pub(crate) static SHARED: Lazy<Mutex<Shared>> = Lazy::new(|| {
    // Settings, snippets and learned words are the user's own: owner-only (older versions were not)
    ekushe_core::suggest::tighten_permissions(&Config::default_dir());
    let cfg = Config::load_from(&Config::default_config_path()).unwrap_or_default();
    let mut ekushe = Ekushe::new(cfg);
    let _ = ekushe
        .suggestions_engine_mut()
        .load_user_dict(&learned_words_path());
    let saved_learned_version = ekushe.suggestions_engine().learned_version();
    if ekushe.language() != InputLanguage::Bangla {
        ekushe.toggle_language();
    }
    Mutex::new(Shared {
        ekushe,
        saved_learned_version,
        learned_mtime: learned_mtime(),
        last: SessionOutput::default(),
    })
});

pub(crate) fn learned_words_path() -> std::path::PathBuf {
    Config::default_dir().join("learned.tsv")
}

fn learned_mtime() -> Option<std::time::SystemTime> {
    std::fs::metadata(learned_words_path())
        .and_then(|m| m.modified())
        .ok()
}

pub(crate) fn save_learned(sh: &mut Shared) {
    let v = sh.ekushe.suggestions_engine().learned_version();
    if v != sh.saved_learned_version {
        // Words another process added since this one last looked are kept, not overwritten
        merge_learned_if_changed(sh);
        sh.saved_learned_version = sh.ekushe.suggestions_engine().learned_version();
        let tsv = sh.ekushe.suggestions_engine().user_dict_tsv();
        let _ = ekushe_core::suggest::write_atomic(&learned_words_path(), &tsv);
        sh.learned_mtime = learned_mtime();
    }
}

/// Read `learned.tsv` again if another process changed it, adding its words to this engine's.
fn merge_learned_if_changed(sh: &mut Shared) {
    let now = learned_mtime();
    if now.is_some() && now != sh.learned_mtime {
        let _ = sh
            .ekushe
            .suggestions_engine_mut()
            .load_user_dict(&learned_words_path());
        sh.learned_mtime = now;
    }
}

/// Re-read settings and learned words from disk (after a backup was restored).
pub fn reload_from_disk() {
    with_shared((), |sh| {
        if let Ok(cfg) = Config::load_from(&Config::default_config_path()) {
            sh.ekushe.set_config(cfg);
        }
        let unsaved = sh.ekushe.suggestions_engine().learned_version() != sh.saved_learned_version;
        sh.learned_mtime = None;
        merge_learned_if_changed(sh);
        if !unsaved {
            sh.saved_learned_version = sh.ekushe.suggestions_engine().learned_version();
        }
    });
}

fn with_shared<T>(fallback: T, f: impl FnOnce(&mut Shared) -> T) -> T {
    catch_unwind(AssertUnwindSafe(|| f(&mut SHARED.lock()))).unwrap_or(fallback)
}

/// Load settings, the dictionary and learned words now (otherwise on first use).
#[no_mangle]
pub extern "C" fn ek_init() {
    #[cfg(all(target_os = "linux", feature = "studio"))]
    studio::keyboard::remove_old_cosmic_env();
    with_shared((), |_| ());
}

/// The keyboard is starting (at login): use the language the user chose to start in (English by
/// default). Studio, which shares this engine code, keeps Bangla for its writing pad.
#[no_mangle]
pub extern "C" fn ek_use_start_language() {
    with_shared((), |sh| {
        if sh.ekushe.language() != sh.ekushe.config().start_language {
            sh.ekushe.toggle_language();
        }
    });
}

/// Type a character. Returns `SessionOutput` JSON.
#[no_mangle]
pub extern "C" fn ek_key(codepoint: u32) -> *mut c_char {
    let Some(ch) = char::from_u32(codepoint) else {
        return ptr::null_mut();
    };
    with_shared(ptr::null_mut(), |sh| {
        let out = sh.ekushe.push_char(ch, false);
        if out.committed.is_some()
            && sh.ekushe.suggestions_engine().learned_version() >= sh.saved_learned_version + 10
        {
            save_learned(sh);
        }
        record(sh, out)
    })
}

/// Backspace (edits the word being typed, or undoes the last conversion). Returns JSON.
#[no_mangle]
pub extern "C" fn ek_backspace() -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| {
        let out = sh.ekushe.backspace();
        record(sh, out)
    })
}

/// Commit candidate `index`; `trailing` is a character to add after it (0 = none). Returns JSON.
#[no_mangle]
pub extern "C" fn ek_select(index: u32, trailing: u32) -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| {
        let t = if trailing == 0 {
            None
        } else {
            char::from_u32(trailing)
        };
        let out = sh.ekushe.select_candidate(index as usize, t);
        if sh.ekushe.suggestions_engine().learned_version() >= sh.saved_learned_version + 10 {
            save_learned(sh);
        }
        record(sh, out)
    })
}

// Field-by-field access to the last result (for C front ends without a JSON parser). Strings are
// newly allocated; free them with `ekushe_string_free`.

/// The word being typed (may be empty).
#[no_mangle]
pub extern "C" fn ek_out_preedit() -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| to_c(sh.last.preedit.clone()))
}

/// Text to write into the application, or null when nothing was committed.
#[no_mangle]
pub extern "C" fn ek_out_committed() -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| match &sh.last.committed {
        Some(c) => to_c(c.clone()),
        None => ptr::null_mut(),
    })
}

#[no_mangle]
pub extern "C" fn ek_out_candidate_count() -> u32 {
    with_shared(0, |sh| sh.last.candidates.len() as u32)
}

/// Candidate `i` (null past the end).
#[no_mangle]
pub extern "C" fn ek_out_candidate(i: u32) -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| {
        match sh.last.candidates.get(i as usize) {
            Some(c) => to_c(c.text.clone()),
            None => ptr::null_mut(),
        }
    })
}

#[no_mangle]
pub extern "C" fn ek_out_selected() -> u32 {
    with_shared(0, |sh| sh.last.selected_index as u32)
}

/// Forget the word being typed, the context word and the undo (caret moved, focus changed).
#[no_mangle]
pub extern "C" fn ek_reset() {
    with_shared((), |sh| {
        sh.ekushe.reset_composition();
        sh.last = SessionOutput::default();
    });
}

/// The focused field is private (a password, PIN or "private" field) or not. In a private field
/// nothing is learned and no context is kept; front ends type passwords as plain characters.
#[no_mangle]
pub extern "C" fn ek_set_private(private: bool) {
    with_shared((), |sh| {
        sh.ekushe.set_private_field(private);
        if private {
            sh.last = SessionOutput::default();
        }
    });
}

#[no_mangle]
pub extern "C" fn ek_is_composing() -> bool {
    with_shared(false, |sh| sh.ekushe.is_composing())
}

/// Backspace right after a word was committed with a space (and Backspace-undo is on): reopen the
/// word as the word being typed, with its suggestions. Returns the output as JSON (its
/// `delete_before` is the word and its space, in UTF-16 units, to delete before showing it) and
/// makes it the current output for the `ek_out_*` functions; null when there is nothing to reopen.
#[no_mangle]
pub extern "C" fn ek_reopen_last_word() -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| {
        if !sh.ekushe.config().backspace_undo {
            return ptr::null_mut();
        }
        match sh.ekushe.reopen_last_word() {
            Some(out) => record(sh, out),
            None => ptr::null_mut(),
        }
    })
}

/// Forget the Backspace-undo (the caret moved, or a key went to the application directly).
#[no_mangle]
pub extern "C" fn ek_forget_undo() {
    with_shared((), |sh| sh.ekushe.forget_undo());
}

/// True right after a word was committed with a space and Backspace-undo is turned on.
#[no_mangle]
pub extern "C" fn ek_can_undo() -> bool {
    with_shared(false, |sh| {
        sh.ekushe.has_undo() && sh.ekushe.config().backspace_undo
    })
}

/// 1 = Bangla, 0 = English.
#[no_mangle]
pub extern "C" fn ek_language() -> u8 {
    with_shared(1, |sh| {
        (sh.ekushe.language() == InputLanguage::Bangla) as u8
    })
}

#[no_mangle]
pub extern "C" fn ek_toggle_language() -> u8 {
    with_shared(1, |sh| {
        (sh.ekushe.toggle_language() == InputLanguage::Bangla) as u8
    })
}

/// A Shift key was pressed and released on its own at `now_ms`. Returns 1/0 (Bangla/English) when
/// this completed a double tap that switched language, or 255 when nothing changed.
#[no_mangle]
pub extern "C" fn ek_shift_tap(now_ms: u64) -> u8 {
    with_shared(255, |sh| {
        if sh.ekushe.config().switch_hotkey != ekushe_core::config::SwitchHotkey::DoubleTapShift {
            return 255;
        }
        match sh.ekushe.on_shift_tap(now_ms) {
            Some(l) => (l == InputLanguage::Bangla) as u8,
            None => 255,
        }
    })
}

/// Ctrl+Backspace: flip the last word between Bangla and Roman. Returns
/// `{"delete": utf16_units, "text": replacement}` or null when there is nothing to flip.
#[no_mangle]
pub extern "C" fn ek_flip_last() -> *mut c_char {
    with_shared(ptr::null_mut(), |sh| {
        match sh.ekushe.flip_last_committed_word() {
            Some((n, text)) => to_c(serde_json::json!({ "delete": n, "text": text }).to_string()),
            None => ptr::null_mut(),
        }
    })
}

/// The user switched to another application (`app_id`: bundle identifier or exe name).
///
/// # Safety
/// `app_id` must be a valid NUL-terminated UTF-8 string or null.
#[no_mangle]
pub unsafe extern "C" fn ek_app_focus(app_id: *const c_char) -> u8 {
    let Some(id) = cstr(app_id) else {
        return ek_language();
    };
    let id = id.to_lowercase();
    with_shared(1, |sh| {
        (sh.ekushe.on_app_focus(&id) == InputLanguage::Bangla) as u8
    })
}

/// Re-read settings from disk (e.g. after another program changed them).
#[no_mangle]
pub extern "C" fn ek_reload_config() {
    reload_from_disk();
}

/// Write learned words to disk if anything new was learned (Rust callers).
pub fn save_learned_now() {
    with_shared((), save_learned);
}

/// Write learned words to disk if anything new was learned.
#[no_mangle]
pub extern "C" fn ek_save_learned() {
    with_shared((), save_learned);
}

/// Start Studio on 127.0.0.1 (once; later calls return the same port). Returns the port, or 0.
#[no_mangle]
pub extern "C" fn ek_studio_start() -> u16 {
    #[cfg(feature = "studio")]
    {
        catch_unwind(studio::start).unwrap_or(0)
    }
    #[cfg(not(feature = "studio"))]
    {
        0
    }
}

/// The web form for reporting a problem, with this build's details (`os`: e.g. `macOS 14.5`).
///
/// # Safety
/// `os` must be a valid NUL-terminated UTF-8 string or null.
#[no_mangle]
pub unsafe extern "C" fn ek_report_url(os: *const c_char) -> *mut c_char {
    let os = cstr(os).unwrap_or("").to_string();
    let url = with_shared(String::new(), |sh| {
        ekushe_core::report_url(&os, sh.ekushe.config())
    });
    if url.is_empty() {
        ptr::null_mut()
    } else {
        to_c(url)
    }
}

/// An address that opens Studio for this user with a fresh single-use launch code, e.g.
/// `http://127.0.0.1:45210/?code=…&tab=settings`. `tab` may be null. Returns null when Studio cannot
/// run. Open it straight away: the code works once and expires.
///
/// # Safety
/// `tab` must be a valid NUL-terminated UTF-8 string or null.
#[no_mangle]
pub unsafe extern "C" fn ek_studio_url(tab: *const c_char) -> *mut c_char {
    #[cfg(feature = "studio")]
    {
        let tab = cstr(tab);
        match catch_unwind(|| studio::launch_url(tab)).ok().flatten() {
            Some(u) => to_c(u),
            None => ptr::null_mut(),
        }
    }
    #[cfg(not(feature = "studio"))]
    {
        let _ = tab;
        ptr::null_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_engine_types_and_commits() {
        // Keep the test away from the real settings folder
        let _ = &*SHARED;
        let out = ek_key('a' as u32);
        assert!(!out.is_null());
        let s = unsafe { CStr::from_ptr(out) }.to_str().unwrap().to_string();
        unsafe { ekushe_string_free(out) };
        assert!(s.contains("preedit"));
        assert!(ek_is_composing());
        let c = ek_select(0, ' ' as u32);
        let s = unsafe { CStr::from_ptr(c) }.to_str().unwrap().to_string();
        unsafe { ekushe_string_free(c) };
        assert!(s.contains("committed"));
        assert!(!ek_is_composing());
    }
}
