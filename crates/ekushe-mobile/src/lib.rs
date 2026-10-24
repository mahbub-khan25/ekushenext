//! The iOS keyboard's engine: ekushe-core behind a few C calls (`ekm_*`, declared in
//! `ios/Shared/ekushe_mobile.h`). The same calls as the Android keyboard's JNI bridge: one engine per
//! process, guarded by a lock; every call returns the session output as JSON (a string the caller
//! frees with `ekm_free`) and never panics across the boundary.
//!
//! The dictionary is the uncompressed `ekushe.dict`, mapped from the app bundle rather than read
//! into memory: iOS stops a keyboard that uses too much memory, and mapped file pages don't count.
//! Settings (`config.toml`) and learned words (`learned.tsv`) live in the keyboard's own folder.

use std::borrow::Cow;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use ekushe_core::config::{Config, InputLanguage, TypingMode};
use ekushe_core::dict::Dictionary;
use ekushe_core::{Ekushe, SessionOutput};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

struct State {
    ekushe: Ekushe,
    dir: Option<PathBuf>,
    saved_learned: u64,
}

static STATE: Lazy<Mutex<State>> = Lazy::new(|| {
    Mutex::new(State {
        ekushe: Ekushe::new(Config::default()),
        dir: None,
        saved_learned: 0,
    })
});

impl State {
    fn config_path(&self) -> Option<PathBuf> {
        self.dir.as_ref().map(|d| d.join("config.toml"))
    }
    fn learned_path(&self) -> Option<PathBuf> {
        self.dir.as_ref().map(|d| d.join("learned.tsv"))
    }
    fn save_learned(&mut self, force: bool) {
        let v = self.ekushe.suggestions_engine().learned_version();
        if v == self.saved_learned || (!force && v < self.saved_learned + 10) {
            return;
        }
        if let Some(p) = self.learned_path() {
            let tsv = self.ekushe.suggestions_engine().user_dict_tsv();
            if ekushe_core::suggest::write_atomic(&p, &tsv).is_ok() {
                self.saved_learned = v;
            }
        }
    }
    fn keep_bangla(&mut self) {
        if self.ekushe.language() != InputLanguage::Bangla {
            self.ekushe.toggle_language();
        }
    }
}

fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(fallback)
}

fn c_string(s: &str) -> *mut c_char {
    CString::new(s.replace('\0', ""))
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn out_json(out: &SessionOutput) -> *mut c_char {
    c_string(&serde_json::to_string(out).unwrap_or_default())
}

/// # Safety
/// `p` is null or a NUL-terminated string.
unsafe fn arg(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

fn trailing(t: u32) -> Option<char> {
    if t == 0 {
        None
    } else {
        char::from_u32(t)
    }
}

/// Map a dictionary file (raw `ekushe.dict`) for the life of the process.
fn map_dictionary(path: &str) -> Option<&'static Dictionary> {
    let file = std::fs::File::open(path).ok()?;
    // SAFETY: the file is part of the signed, read-only app bundle; nothing writes to it.
    let map = unsafe { memmap2::Mmap::map(&file) }.ok()?;
    let map: &'static memmap2::Mmap = Box::leak(Box::new(map));
    let d = Dictionary::from_bytes(Cow::Borrowed(&map[..])).ok()?;
    Some(Box::leak(Box::new(d)))
}

/// Load the dictionary, settings and learned words. Returns the number of dictionary words (0 if
/// the dictionary could not be read; typing still works with the small built-in word list).
///
/// # Safety
/// Both arguments are null or NUL-terminated strings.
#[no_mangle]
pub unsafe extern "C" fn ekm_init(dict_path: *const c_char, dir: *const c_char) -> i32 {
    let dict_path = arg(dict_path);
    let dir = arg(dir);
    guard(0, move || {
        let dir = (!dir.is_empty()).then(|| PathBuf::from(&dir));
        if let Some(d) = &dir {
            let _ = std::fs::create_dir_all(d);
        }
        let cfg = dir
            .as_ref()
            .and_then(|d| Config::load_from(&d.join("config.toml")).ok())
            .unwrap_or_default();
        let mut ekushe = Ekushe::new(cfg);
        let mut n = 0;
        if let Some(d) = map_dictionary(&dict_path) {
            n = d.len() as i32;
            ekushe.suggestions_engine_mut().set_dictionary(Some(d));
        }
        if let Some(d) = &dir {
            let _ = ekushe
                .suggestions_engine_mut()
                .load_user_dict(&d.join("learned.tsv"));
        }
        let mut st = STATE.lock();
        st.dir = dir;
        st.saved_learned = ekushe.suggestions_engine().learned_version();
        st.ekushe = ekushe;
        st.keep_bangla();
        n
    })
}

/// Free a string returned by any `ekm_*` call.
///
/// # Safety
/// `s` is null or a string returned by this library, freed once.
#[no_mangle]
pub unsafe extern "C" fn ekm_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[no_mangle]
pub extern "C" fn ekm_key(cp: u32) -> *mut c_char {
    let out = guard(None, || {
        let ch = char::from_u32(cp)?;
        let mut st = STATE.lock();
        st.keep_bangla();
        let out = st.ekushe.push_char(ch, false);
        if out.committed.is_some() {
            st.save_learned(false);
        }
        Some(out)
    });
    out.as_ref().map(out_json).unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn ekm_backspace() -> *mut c_char {
    out_json(&guard(SessionOutput::default(), || {
        STATE.lock().ekushe.backspace()
    }))
}

#[no_mangle]
pub extern "C" fn ekm_select(index: u32, t: u32) -> *mut c_char {
    out_json(&guard(SessionOutput::default(), || {
        let mut st = STATE.lock();
        let out = st.ekushe.select_candidate(index as usize, trailing(t));
        st.save_learned(false);
        out
    }))
}

#[no_mangle]
pub extern "C" fn ekm_commit(t: u32) -> *mut c_char {
    out_json(&guard(SessionOutput::default(), || {
        STATE.lock().ekushe.commit_default(trailing(t))
    }))
}

/// Drop the word being typed (the text already shown in the app stays).
#[no_mangle]
pub extern "C" fn ekm_reset() {
    guard((), || STATE.lock().ekushe.reset_composition());
}

/// A password or other private field: nothing is learned or kept as context.
#[no_mangle]
pub extern "C" fn ekm_set_private(on: bool) {
    guard((), || STATE.lock().ekushe.set_private_field(on));
}

#[no_mangle]
pub extern "C" fn ekm_has_undo() -> bool {
    guard(false, || STATE.lock().ekushe.has_undo())
}

/// A Bangla word written in English letters? (English typing leaves those alone.)
///
/// # Safety
/// `word` is null or a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn ekm_is_banglish(word: *const c_char) -> bool {
    let w = arg(word);
    guard(false, || STATE.lock().ekushe.is_banglish(&w))
}

/// Settings the keyboard shows, as JSON.
#[no_mangle]
pub extern "C" fn ekm_get_config() -> *mut c_char {
    let s = guard(String::new(), || {
        let st = STATE.lock();
        let c = st.ekushe.config();
        let mut snippets: Vec<_> = c.custom_snippets.iter().collect();
        snippets.sort();
        serde_json::json!({
            "typing_mode": format!("{:?}", c.typing_mode),
            "bangla_digits": c.bangla_digits,
            "emoji": c.enable_emoji_suggestions,
            "next_word": c.enable_next_word_prediction,
            "hints": c.show_sign_and_conjunct_hints,
            "backspace_undo": c.backspace_undo,
            "snippets": snippets,
        })
        .to_string()
    });
    c_string(&s)
}

/// Change settings from JSON (only the fields given) and save them.
///
/// # Safety
/// `json` is null or a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn ekm_set_config(json: *const c_char) -> bool {
    let raw = arg(json);
    guard(false, move || {
        let Ok(p) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return false;
        };
        let mut st = STATE.lock();
        let mut c = st.ekushe.config().clone();
        if let Some(m) = p["typing_mode"].as_str() {
            c.typing_mode = match m {
                "PhoneticFirst" => TypingMode::PhoneticFirst,
                "PhoneticOnly" => TypingMode::PhoneticOnly,
                _ => TypingMode::Smart,
            };
        }
        let flag = |k: &str| p[k].as_bool();
        if let Some(b) = flag("bangla_digits") {
            c.bangla_digits = b;
        }
        if let Some(b) = flag("emoji") {
            c.enable_emoji_suggestions = b;
        }
        if let Some(b) = flag("next_word") {
            c.enable_next_word_prediction = b;
        }
        if let Some(b) = flag("hints") {
            c.show_sign_and_conjunct_hints = b;
        }
        if let Some(b) = flag("backspace_undo") {
            c.backspace_undo = b;
        }
        if let Some(add) = p["add_snippet"].as_array() {
            if let (Some(k), Some(v)) = (
                add.first().and_then(|x| x.as_str()),
                add.get(1).and_then(|x| x.as_str()),
            ) {
                let (k, v) = (k.trim(), v.trim());
                if !k.is_empty() && !v.is_empty() {
                    c.custom_snippets.insert(k.to_string(), v.to_string());
                }
            }
        }
        if let Some(k) = p["remove_snippet"].as_str() {
            c.custom_snippets.remove(k);
        }
        let saved = st
            .config_path()
            .map(|path| c.save_to(&path).is_ok())
            .unwrap_or(false);
        if p["remove_snippet"].is_string() {
            // a removed snippet must leave the engine too: rebuild it around the same dictionary
            let dict = st.ekushe.suggestions_engine().dictionary();
            st.save_learned(true);
            let mut e = Ekushe::new(c);
            e.suggestions_engine_mut().set_dictionary(dict);
            if let Some(path) = st.learned_path() {
                let _ = e.suggestions_engine_mut().load_user_dict(&path);
            }
            st.saved_learned = e.suggestions_engine().learned_version();
            st.ekushe = e;
            st.keep_bangla();
        } else {
            st.ekushe.set_config(c);
        }
        saved
    })
}

/// Write learned words now (the keyboard calls this when it closes).
#[no_mangle]
pub extern "C" fn ekm_save() {
    guard((), || STATE.lock().save_learned(true));
}

/// Forget every learned word.
#[no_mangle]
pub extern "C" fn ekm_forget_learned() {
    guard((), || {
        let mut st = STATE.lock();
        if let Some(p) = st.learned_path() {
            let _ = std::fs::remove_file(&p);
        }
        let dict = st.ekushe.suggestions_engine().dictionary();
        let mut e = Ekushe::new(st.ekushe.config().clone());
        e.suggestions_engine_mut().set_dictionary(dict);
        st.saved_learned = e.suggestions_engine().learned_version();
        st.ekushe = e;
        st.keep_bangla();
    });
}

/// The app's version line, e.g. `v0.1.0-beta.13`.
#[no_mangle]
pub extern "C" fn ekm_release() -> *mut c_char {
    c_string(ekushe_core::release())
}

/// The problem-report form, with the version, device and settings filled in.
///
/// # Safety
/// `os` is null or a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn ekm_report_url(os: *const c_char) -> *mut c_char {
    let os = arg(os);
    let url = guard(String::new(), || {
        ekushe_core::report_url(&os, STATE.lock().ekushe.config())
    });
    c_string(&url)
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn take(p: *mut c_char) -> String {
        let s = CStr::from_ptr(p).to_string_lossy().into_owned();
        ekm_free(p);
        s
    }

    #[test]
    fn types_bangla_through_the_c_calls() {
        let dict = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/ekushe.dict");
        // an uncompressed copy, made the way ios/build-rust.sh makes it
        if !std::path::Path::new(dict).exists() {
            let z = std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../data/dictionary/ekushe.dict.z"
            ))
            .unwrap();
            let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&z).unwrap();
            std::fs::write(dict, raw).unwrap();
        }
        let dict_c = CString::new(dict).unwrap();
        let dir = std::env::temp_dir().join("ekushe-mobile-test");
        let dir_c = CString::new(dir.to_string_lossy().as_ref()).unwrap();
        unsafe {
            let n = ekm_init(dict_c.as_ptr(), dir_c.as_ptr());
            assert!(n > 100_000, "dictionary words: {n}");
            let mut last = String::new();
            for ch in "amar".chars() {
                last = take(ekm_key(ch as u32));
            }
            assert!(last.contains("আমার"), "{last}");
            let placed = take(ekm_select(0, ' ' as u32));
            assert!(placed.contains("আমার "), "{placed}");
            let w = CString::new("tomake").unwrap();
            assert!(ekm_is_banglish(w.as_ptr()));
            let t = CString::new("teh").unwrap();
            assert!(!ekm_is_banglish(t.as_ptr()));
            assert!(take(ekm_get_config()).contains("typing_mode"));
        }
    }
}
