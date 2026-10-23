//! The Android keyboard's engine: ekushe-core behind a few JNI calls on
//! `com.ekushenext.keyboard.Engine`. One engine per process, guarded by a lock; every call
//! returns the session output as JSON (or nothing) and never panics across the JNI boundary.
//!
//! The dictionary arrives as the compressed bytes of `ekushe.dict.z` (an app asset); settings
//! (`config.toml`) and learned words (`learned.tsv`) live in the app's private files folder.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use ekushe_core::config::{Config, InputLanguage, TypingMode};
use ekushe_core::dict::Dictionary;
use ekushe_core::{Ekushe, SessionOutput};
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jint, jstring, JNI_TRUE};
use jni::JNIEnv;
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

fn out_json(env: &mut JNIEnv, out: &SessionOutput) -> jstring {
    let s = serde_json::to_string(out).unwrap_or_default();
    env.new_string(s)
        .map(|j| j.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn text(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s)
        .map(|j| j.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn trailing(t: jint) -> Option<char> {
    if t <= 0 {
        None
    } else {
        char::from_u32(t as u32)
    }
}

/// Load the dictionary (compressed bytes), settings and learned words. Returns the number of
/// dictionary words (0 if the dictionary could not be read; typing still works with the small
/// built-in word list).
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeInit(
    mut env: JNIEnv,
    _c: JClass,
    dict: JByteArray,
    dir: JString,
) -> jint {
    let bytes = env.convert_byte_array(&dict).unwrap_or_default();
    let dir: Option<String> = env.get_string(&dir).ok().map(Into::into);
    guard(0, move || {
        // Everything slow happens before taking the lock, so typing never waits for the dictionary
        let dir = dir.map(PathBuf::from);
        let cfg = dir
            .as_ref()
            .and_then(|d| Config::load_from(&d.join("config.toml")).ok())
            .unwrap_or_default();
        let mut ekushe = Ekushe::new(cfg);
        let mut n = 0;
        if !bytes.is_empty() {
            if let Ok(d) = Dictionary::from_compressed(&bytes) {
                n = d.len() as jint;
                let d: &'static Dictionary = Box::leak(Box::new(d));
                ekushe.suggestions_engine_mut().set_dictionary(Some(d));
            }
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

#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeKey(
    mut env: JNIEnv,
    _c: JClass,
    cp: jint,
) -> jstring {
    let out = guard(None, || {
        let ch = char::from_u32(cp as u32)?;
        let mut st = STATE.lock();
        st.keep_bangla();
        let out = st.ekushe.push_char(ch, false);
        if out.committed.is_some() {
            st.save_learned(false);
        }
        Some(out)
    });
    match out {
        Some(o) => out_json(&mut env, &o),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeBackspace(
    mut env: JNIEnv,
    _c: JClass,
) -> jstring {
    let out = guard(SessionOutput::default(), || STATE.lock().ekushe.backspace());
    out_json(&mut env, &out)
}

#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeSelect(
    mut env: JNIEnv,
    _c: JClass,
    index: jint,
    t: jint,
) -> jstring {
    let out = guard(SessionOutput::default(), || {
        let mut st = STATE.lock();
        let out = st
            .ekushe
            .select_candidate(index.max(0) as usize, trailing(t));
        st.save_learned(false);
        out
    });
    out_json(&mut env, &out)
}

#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeCommit(
    mut env: JNIEnv,
    _c: JClass,
    t: jint,
) -> jstring {
    let out = guard(SessionOutput::default(), || {
        STATE.lock().ekushe.commit_default(trailing(t))
    });
    out_json(&mut env, &out)
}

/// Drop the word being typed (the text already shown in the app stays).
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeReset(_env: JNIEnv, _c: JClass) {
    guard((), || STATE.lock().ekushe.reset_composition());
}

/// A password, PIN or "incognito" field: nothing is learned or kept as context.
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeSetPrivate(
    _env: JNIEnv,
    _c: JClass,
    on: jboolean,
) {
    guard((), || STATE.lock().ekushe.set_private_field(on == JNI_TRUE));
}

#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeHasUndo(
    _env: JNIEnv,
    _c: JClass,
) -> jboolean {
    guard(0, || STATE.lock().ekushe.has_undo() as jboolean)
}

/// Settings the keyboard shows, as JSON.
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeGetConfig(
    mut env: JNIEnv,
    _c: JClass,
) -> jstring {
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
    text(&mut env, &s)
}

/// Change settings from JSON (only the fields given) and save them.
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeSetConfig(
    mut env: JNIEnv,
    _c: JClass,
    json: JString,
) -> jboolean {
    let raw: String = env.get_string(&json).map(Into::into).unwrap_or_default();
    guard(0, move || {
        let Ok(p) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return 0;
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
        // Snippets removed from the config must leave the engine too: rebuild it around the same
        // dictionary and learned words
        if p["remove_snippet"].is_string() {
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
        saved as jboolean
    })
}

/// Write learned words now (the keyboard calls this when it closes).
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeSave(_env: JNIEnv, _c: JClass) {
    guard((), || STATE.lock().save_learned(true));
}

/// Forget every learned word.
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeForgetLearned(
    _env: JNIEnv,
    _c: JClass,
) {
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

/// The app's version line, e.g. `v0.1.0-beta.11`.
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeRelease(
    mut env: JNIEnv,
    _c: JClass,
) -> jstring {
    text(&mut env, ekushe_core::release())
}

/// The problem-report form, with the version, phone and settings filled in.
#[no_mangle]
pub extern "system" fn Java_com_ekushenext_keyboard_Engine_nativeReportUrl(
    mut env: JNIEnv,
    _c: JClass,
    os: JString,
) -> jstring {
    let os: String = env.get_string(&os).map(Into::into).unwrap_or_default();
    let url = guard(String::new(), || {
        ekushe_core::report_url(&os, STATE.lock().ekushe.config())
    });
    text(&mut env, &url)
}
