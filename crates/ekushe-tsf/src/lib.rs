//! `ekushe-tsf`: Ekushe as a Windows input method (Text Services Framework text input processor),
//! built as `ekushe_tsf.dll`.
//!
//! **Preview.** This is the input-method alternative to the tray app's keyboard hook. Instead of
//! injecting key presses, it writes into the application through TSF: the word being typed is a TSF
//! *composition* (the application knows it is unfinished text), and committing ends the composition.
//! That makes it work in places the hook cannot reach well (UWP/Store apps, elevated windows, password
//! fields are skipped by Windows itself) and removes the backspace-and-retype flicker.
//!
//! What it does today:
//! - the full Ekushe engine (Smart / phonetic modes, the built-in dictionary, learned words shared with
//!   the tray app's `learned.tsv`, per-app language memory)
//! - Space commits the highlighted word, Enter commits and passes the Enter on, Tab cycles candidates
//!   in place, Escape keeps the text as typed, Backspace edits the word and, right after a commit,
//!   brings back the Roman text; Ctrl+Backspace flips the last word between Bangla and Roman
//! - the language hotkey configured in Settings (Double-Tap Shift, Ctrl+Space, F12, Ctrl+Alt+B)
//!
//! Not yet: a candidate list window (only the top choice is shown inline; Tab cycles), a language-bar
//! button, a display attribute (some applications do not underline the unfinished word), 32-bit
//! applications (needs a 32-bit build of this DLL registered too).
//!
//! Register from an administrator prompt with `regsvr32 ekushe_tsf.dll` (TSF profiles live under
//! HKLM); remove with `regsvr32 /u ekushe_tsf.dll`. Do not run it at the same time as the tray app's
//! keyboard hook. Store apps only load input methods from folders they can read (e.g. Program Files).

#![cfg(windows)]
#![allow(non_snake_case)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::time::Instant;

use ekushe_core::config::{Config, InputLanguage, SwitchHotkey};
use ekushe_core::{Ekushe, SessionOutput};
use windows::core::{implement, Interface, Result, GUID, HRESULT};
use windows::Win32::Foundation::{
    BOOL, CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_INVALIDARG, E_UNEXPECTED,
    FALSE, HINSTANCE, HMODULE, LPARAM, S_FALSE, S_OK, TRUE, WPARAM,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IClassFactory, IClassFactory_Impl,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE,
    KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, HKL, VIRTUAL_KEY, VK_BACK,
    VK_CONTROL, VK_DELETE, VK_ESCAPE, VK_F12, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RETURN, VK_RSHIFT,
    VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, ITfCategoryMgr, ITfComposition,
    ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfContextComposition, ITfEditSession,
    ITfEditSession_Impl, ITfInputProcessorProfileMgr, ITfInsertAtSelection, ITfKeyEventSink,
    ITfKeyEventSink_Impl, ITfKeystrokeMgr, ITfRange, ITfTextInputProcessor,
    ITfTextInputProcessor_Impl, ITfThreadMgr, GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT, GUID_TFCAT_TIPCAP_UIELEMENTENABLED, GUID_TFCAT_TIP_KEYBOARD,
    INSERT_TEXT_AT_SELECTION_FLAGS, TF_AE_NONE, TF_ANCHOR_END, TF_ANCHOR_START,
    TF_DEFAULT_SELECTION, TF_ES_ASYNCDONTCARE, TF_ES_READWRITE, TF_IAS_QUERYONLY, TF_SELECTION,
    TF_SELECTIONSTYLE,
};
use windows::Win32::UI::TextServices::{
    ITfCompartmentMgr, ITfInputScope, InputScope, GUID_COMPARTMENT_EMPTYCONTEXT,
    GUID_COMPARTMENT_KEYBOARD_DISABLED, GUID_PROP_INPUTSCOPE, IS_ALPHANUMERIC_PIN,
    IS_ALPHANUMERIC_PIN_SET, IS_NUMERIC_PASSWORD, IS_NUMERIC_PIN, IS_PASSWORD, TF_ES_READ,
    TF_ES_SYNC,
};
use windows_core::IUnknownImpl;

/// Ekushe text service CLSID: `{E21B0945-2102-1952-B09C-0AD55559B326}`
pub const EKUSHE_CLSID_STR: &str = "{E21B0945-2102-1952-B09C-0AD55559B326}";
/// Ekushe language profile GUID: `{E21B0945-2102-1952-B09C-0AD55559B327}`
pub const EKUSHE_PROFILE_GUID_STR: &str = "{E21B0945-2102-1952-B09C-0AD55559B327}";

pub const EKUSHE_CLSID: GUID = GUID::from_u128(0xE21B0945_2102_1952_B09C_0AD55559B326);
pub const EKUSHE_PROFILE_GUID: GUID = GUID::from_u128(0xE21B0945_2102_1952_B09C_0AD55559B327);
/// Bangla (Bangladesh)
const LANGID_BN_BD: u16 = 0x0845;
const DESCRIPTION: &str = "Ekushe Next Bangla (একুশে নেক্সট বাংলা)";

/// Live COM objects + LockServer count; the DLL may unload only at zero.
static DLL_REFS: AtomicUsize = AtomicUsize::new(0);
static DLL_INSTANCE: AtomicIsize = AtomicIsize::new(0);

fn dll_add_ref() {
    DLL_REFS.fetch_add(1, Ordering::SeqCst);
}
fn dll_release() {
    DLL_REFS.fetch_sub(1, Ordering::SeqCst);
}

// ───────────────────────────── DLL entry points ─────────────────────────────

/// # Safety
/// Called by the Windows loader.
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        DLL_INSTANCE.store(hinst.0 as isize, Ordering::SeqCst);
    }
    TRUE
}

/// # Safety
/// Called by COM.
#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if DLL_REFS.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

/// # Safety
/// Called by COM with valid pointers.
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || rclsid.is_null() || riid.is_null() {
        return E_INVALIDARG;
    }
    *ppv = std::ptr::null_mut();
    if *rclsid != EKUSHE_CLSID {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = ClassFactory.into();
    factory.query(riid, ppv)
}

/// Registers the COM class and the TSF keyboard profile (Bangla, Bangladesh). Needs administrator
/// rights: TSF keeps input-method profiles under HKLM.
///
/// # Safety
/// Called by `regsvr32`.
#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    match register() {
        Ok(()) => S_OK,
        Err(e) => {
            let _ = unregister();
            e.code()
        }
    }
}

/// # Safety
/// Called by `regsvr32 /u`.
#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    match unregister() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    }
}

fn dll_path() -> String {
    let mut buf = [0u16; 1024];
    let module = HMODULE(DLL_INSTANCE.load(Ordering::SeqCst) as *mut c_void);
    let n = unsafe { GetModuleFileNameW(module, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

/// COM on the calling thread for the duration of a closure (regsvr32 may or may not have done it).
fn with_com<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let result = f();
    if hr.is_ok() {
        unsafe { CoUninitialize() };
    }
    result
}

const CATEGORIES: [GUID; 4] = [
    GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, // Store / UWP apps
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED, // no own UI that would need hiding in full-screen apps
];

/// Machine-wide registration makes every program load this DLL, so it must live where only
/// administrators can change it (Program Files or Windows); registering a copy in a user-writable
/// folder would let that user swap in other code. Returns the folder rules as an error otherwise.
fn in_protected_folder(path: &str) -> bool {
    let p = path.to_lowercase().replace('/', "\\");
    ["ProgramFiles", "ProgramW6432", "SystemRoot"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .map(|d| format!("{}\\", d.trim_end_matches('\\').to_lowercase()))
        .any(|d| p.starts_with(&d) && !p[d.len()..].contains("..\\"))
}

unsafe fn register() -> Result<()> {
    let path = dll_path();
    if !in_protected_folder(&path) {
        // 0x80070005: copy the DLL into C:\Program Files\Ekushe Next\ first (see the README)
        return Err(windows::Win32::Foundation::E_ACCESSDENIED.into());
    }
    let clsid_key = format!("Software\\Classes\\CLSID\\{EKUSHE_CLSID_STR}");
    set_reg_sz(
        HKEY_LOCAL_MACHINE,
        &clsid_key,
        None,
        "Ekushe Next Bangla Input Method",
    )?;
    let inproc = format!("{clsid_key}\\InprocServer32");
    set_reg_sz(HKEY_LOCAL_MACHINE, &inproc, None, &path)?;
    set_reg_sz(
        HKEY_LOCAL_MACHINE,
        &inproc,
        Some("ThreadingModel"),
        "Apartment",
    )?;

    with_com(|| {
        let profiles: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        let desc: Vec<u16> = DESCRIPTION.encode_utf16().collect();
        let icon: Vec<u16> = path.encode_utf16().collect();
        profiles.RegisterProfile(
            &EKUSHE_CLSID,
            LANGID_BN_BD,
            &EKUSHE_PROFILE_GUID,
            &desc,
            &icon,
            0,
            HKL::default(),
            0,
            TRUE,
            0,
        )?;
        let categories: ITfCategoryMgr =
            CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for cat in CATEGORIES {
            categories.RegisterCategory(&EKUSHE_CLSID, &cat, &EKUSHE_CLSID)?;
        }
        Ok(())
    })
}

unsafe fn unregister() -> Result<()> {
    let _ = with_com(|| {
        if let Ok(categories) =
            CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
        {
            for cat in CATEGORIES {
                let _ = categories.UnregisterCategory(&EKUSHE_CLSID, &cat, &EKUSHE_CLSID);
            }
        }
        let profiles: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        profiles.UnregisterProfile(&EKUSHE_CLSID, LANGID_BN_BD, &EKUSHE_PROFILE_GUID, 0)
    });
    let key = wide_null(&format!("Software\\Classes\\CLSID\\{EKUSHE_CLSID_STR}"));
    let _ = RegDeleteTreeW(HKEY_LOCAL_MACHINE, windows::core::PCWSTR(key.as_ptr()));
    Ok(())
}

unsafe fn set_reg_sz(root: HKEY, subkey: &str, name: Option<&str>, value: &str) -> Result<()> {
    let subkey_w = wide_null(subkey);
    let mut hkey = HKEY::default();
    RegCreateKeyExW(
        root,
        windows::core::PCWSTR(subkey_w.as_ptr()),
        0,
        None,
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        None,
        &mut hkey,
        None,
    )
    .ok()?;
    let name_w = name.map(wide_null);
    let name_ptr = name_w.as_ref().map_or(windows::core::PCWSTR::null(), |v| {
        windows::core::PCWSTR(v.as_ptr())
    });
    let value_w = wide_null(value);
    let bytes = std::slice::from_raw_parts(value_w.as_ptr() as *const u8, value_w.len() * 2);
    let status = RegSetValueExW(hkey, name_ptr, 0, REG_SZ, Some(bytes));
    let _ = RegCloseKey(hkey);
    status.ok()
}

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

// ───────────────────────────── Class factory ─────────────────────────────

#[implement(IClassFactory)]
struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        outer: Option<&windows::core::IUnknown>,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> Result<()> {
        if ppv.is_null() || riid.is_null() {
            return Err(E_INVALIDARG.into());
        }
        unsafe { *ppv = std::ptr::null_mut() };
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let tip: ITfTextInputProcessor = TextService::new().into();
        unsafe { tip.query(riid, ppv).ok() }
    }

    fn LockServer(&self, lock: BOOL) -> Result<()> {
        if lock.as_bool() {
            dll_add_ref()
        } else {
            dll_release()
        }
        Ok(())
    }
}

// ───────────────────────────── Text service ─────────────────────────────

/// What a key does, decided without changing any state (TSF asks "would you eat this key?" first).
#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyAction {
    /// Not ours: the application gets the key
    Pass,
    /// The application gets the key, but the word being typed is finished first (shortcuts, arrows)
    FinishThenPass,
    ToggleLanguage,
    Char(char, bool),
    Backspace,
    Space,
    Enter,
    Escape,
    Tab,
    FlipLastWord,
}

struct State {
    thread_mgr: Option<ITfThreadMgr>,
    client_id: u32,
    ekushe: Ekushe,
    /// Last engine output while composing (candidates, highlighted index)
    output: SessionOutput,
    /// Text of the last word committed with a trailing space (what Backspace-undo expects before the caret)
    last_committed: String,
    shift_clean_down: bool,
    started: Instant,
    saved_learned_version: u64,
}

#[implement(ITfTextInputProcessor, ITfKeyEventSink, ITfCompositionSink)]
struct TextService {
    state: RefCell<State>,
    /// The open composition; shared with edit sessions, which run while `state` may be borrowed
    composition: Rc<RefCell<Option<ITfComposition>>>,
}

fn learned_words_path() -> PathBuf {
    Config::default_dir().join("learned.tsv")
}

fn key_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
}

impl TextService {
    fn new() -> Self {
        dll_add_ref();
        let cfg = Config::load_from(&Config::default_config_path()).unwrap_or_default();
        let mut ekushe = Ekushe::new(cfg);
        let _ = ekushe
            .suggestions_engine_mut()
            .load_user_dict(&learned_words_path());
        let saved_learned_version = ekushe.suggestions_engine().learned_version();
        Self {
            state: RefCell::new(State {
                thread_mgr: None,
                client_id: 0,
                ekushe,
                output: SessionOutput::default(),
                last_committed: String::new(),
                shift_clean_down: false,
                started: Instant::now(),
                saved_learned_version,
            }),
            composition: Rc::new(RefCell::new(None)),
        }
    }
}

impl Drop for TextService {
    fn drop(&mut self) {
        dll_release();
    }
}

fn save_learned(st: &mut State) {
    let v = st.ekushe.suggestions_engine().learned_version();
    if v != st.saved_learned_version {
        st.saved_learned_version = v;
        let tsv = st.ekushe.suggestions_engine().user_dict_tsv();
        let _ = ekushe_core::suggest::write_atomic(&learned_words_path(), &tsv);
    }
}

/// Character a key types with the current keyboard layout and modifiers (this runs on the
/// application's own thread, so the thread keyboard state is the real one).
fn key_char(vk: u32, lparam: LPARAM) -> Option<char> {
    if (0x60..=0x69).contains(&vk) && !key_down(VK_SHIFT) {
        return char::from_u32('0' as u32 + (vk - 0x60));
    }
    let mut state = [0u8; 256];
    unsafe {
        GetKeyboardState(&mut state).ok()?;
        // Plain Ctrl (without Alt) would give control characters; only AltGr combinations type
        let ctrl = state[VK_CONTROL.0 as usize] & 0x80 != 0;
        let alt = state[VK_MENU.0 as usize] & 0x80 != 0;
        if ctrl != alt {
            return None;
        }
        let scan = ((lparam.0 >> 16) & 0xFF) as u32;
        let mut buf = [0u16; 4];
        let layout = GetKeyboardLayout(0);
        // Flag 0x4: leave the keyboard state alone so dead keys still work in the application
        let n = ToUnicodeEx(vk, scan, &state, &mut buf, 0x4, layout);
        if n == 1 {
            char::from_u32(buf[0] as u32).filter(|c| !c.is_control())
        } else {
            None
        }
    }
}

fn classify(st: &State, vk: u32, lparam: LPARAM) -> KeyAction {
    let ctrl = key_down(VK_CONTROL);
    let alt = key_down(VK_MENU);
    let win = key_down(VK_LWIN) || key_down(VK_RWIN);
    let vk16 = VIRTUAL_KEY(vk as u16);

    let toggle = match &st.ekushe.config().switch_hotkey {
        SwitchHotkey::CtrlSpace => ctrl && !alt && vk16 == VK_SPACE,
        SwitchHotkey::F12 => vk16 == VK_F12,
        SwitchHotkey::CtrlAltB => ctrl && alt && vk == b'B' as u32,
        _ => false,
    };
    if toggle {
        return KeyAction::ToggleLanguage;
    }
    if st.ekushe.language() == InputLanguage::English {
        return KeyAction::Pass;
    }
    let composing = st.ekushe.is_composing();
    if ctrl && !alt && vk16 == VK_BACK && !composing && st.ekushe.has_undo() {
        return KeyAction::FlipLastWord;
    }
    // Shortcuts belong to the application (AltGr = Ctrl+Alt still types)
    if ctrl != alt || win || vk16 == VK_LWIN || vk16 == VK_RWIN {
        return if composing {
            KeyAction::FinishThenPass
        } else {
            KeyAction::Pass
        };
    }
    if composing {
        match vk16 {
            VK_BACK => return KeyAction::Backspace,
            VK_SPACE => return KeyAction::Space,
            VK_RETURN => return KeyAction::Enter,
            VK_ESCAPE => return KeyAction::Escape,
            VK_TAB if st.output.candidates.len() > 1 => return KeyAction::Tab,
            _ => {}
        }
        if (0x21..=0x28).contains(&vk) || vk16 == VK_DELETE || vk16 == VK_TAB {
            return KeyAction::FinishThenPass;
        }
    } else if vk16 == VK_BACK && st.ekushe.has_undo() && st.ekushe.config().backspace_undo {
        return KeyAction::Backspace;
    }
    match key_char(vk, lparam) {
        Some(ch) => KeyAction::Char(ch, ctrl && alt),
        None if composing => KeyAction::FinishThenPass,
        None => KeyAction::Pass,
    }
}

/// Edit to make in the document.
enum Edit {
    /// Show `text` as the unfinished word (starts a composition if needed; empty text removes it)
    Compose(String),
    /// Write `text` and finish the word
    Commit(String),
    /// Replace the `expect`ed text just before the caret with `text` (Backspace-undo, Ctrl+Backspace).
    /// If the text there is different (the caret moved), behave like a normal Backspace.
    ReplaceBefore { expect: String, text: String },
}

impl TextService_Impl {
    fn request_edit(&self, context: &ITfContext, edit: Edit) -> Result<()> {
        let client_id = self.state.borrow().client_id;
        let session: ITfEditSession = EditSession {
            context: context.clone(),
            sink: self.to_interface::<ITfCompositionSink>(),
            composition: self.composition.clone(),
            edit,
        }
        .into();
        // ASYNCDONTCARE runs at once when the document is free (the usual case), later otherwise
        unsafe {
            let _ = context.RequestEditSession(
                client_id,
                &session,
                TF_ES_ASYNCDONTCARE | TF_ES_READWRITE,
            )?;
        }
        Ok(())
    }

    /// Finish the open word as shown (shortcuts, arrows, focus loss).
    fn finish_composition(&self, context: &ITfContext) -> Result<()> {
        let text = {
            let mut st = self.state.borrow_mut();
            let text = st.output.preedit.clone();
            st.ekushe.reset_composition();
            st.output = SessionOutput::default();
            text
        };
        if self.composition.borrow().is_some() {
            self.request_edit(context, Edit::Commit(text))?;
        }
        Ok(())
    }

    fn after_commit(st: &mut State, committed: &str) {
        st.last_committed = if committed.ends_with(' ') {
            committed.to_string()
        } else {
            String::new()
        };
        st.output = SessionOutput::default();
        // Persist learned words now and then, not on every word
        if st.ekushe.suggestions_engine().learned_version() >= st.saved_learned_version + 10 {
            save_learned(st);
        }
    }

    /// Bangla typing is about to start a word in a private field (checked once per word, not while
    /// a word is open).
    fn starting_word_in_private_field(&self, ctx: &ITfContext) -> bool {
        {
            let st = self.state.borrow();
            if st.ekushe.language() == InputLanguage::English || st.ekushe.is_composing() {
                return false;
            }
        }
        self.field_is_private(ctx)
    }

    /// Whether the focused field is private: the application turned the keyboard off for it (TSF's
    /// keyboard-disabled or empty-context compartments, which browsers set for password fields), or
    /// gave it a password or PIN input scope. Keys there are typed as plain characters: no preview,
    /// nothing learned.
    fn field_is_private(&self, context: &ITfContext) -> bool {
        let (thread_mgr, client_id) = {
            let st = self.state.borrow();
            (st.thread_mgr.clone(), st.client_id)
        };
        unsafe {
            let flag = |cm: &ITfCompartmentMgr, g: &GUID| -> bool {
                cm.GetCompartment(g)
                    .and_then(|c| c.GetValue())
                    .ok()
                    .and_then(|v| i32::try_from(&v).ok())
                    .is_some_and(|n| n != 0)
            };
            if let Some(cm) = thread_mgr
                .as_ref()
                .and_then(|t| t.cast::<ITfCompartmentMgr>().ok())
            {
                if flag(&cm, &GUID_COMPARTMENT_KEYBOARD_DISABLED) {
                    return true;
                }
            }
            if let Ok(cm) = context.cast::<ITfCompartmentMgr>() {
                if flag(&cm, &GUID_COMPARTMENT_KEYBOARD_DISABLED)
                    || flag(&cm, &GUID_COMPARTMENT_EMPTYCONTEXT)
                {
                    return true;
                }
            }
            // The input scope is a text property: read it in a short read-only edit session
            let secret = Rc::new(Cell::new(false));
            let session: ITfEditSession = ScopeSession {
                context: context.clone(),
                secret: secret.clone(),
            }
            .into();
            let _ = context.RequestEditSession(client_id, &session, TF_ES_SYNC | TF_ES_READ);
            secret.get()
        }
    }

    /// Run `action`. Returns whether the key was eaten.
    fn execute(&self, context: &ITfContext, action: KeyAction) -> Result<bool> {
        match action {
            KeyAction::Pass => Ok(false),
            KeyAction::FinishThenPass => {
                self.finish_composition(context)?;
                Ok(false)
            }
            KeyAction::ToggleLanguage => {
                self.finish_composition(context)?;
                self.state.borrow_mut().ekushe.toggle_language();
                Ok(true)
            }
            KeyAction::Char(ch, altgr) => {
                let edit = {
                    let mut st = self.state.borrow_mut();
                    let out = st.ekushe.push_char(ch, altgr);
                    if let Some(committed) = out.committed.clone() {
                        Self::after_commit(&mut st, &committed);
                        Edit::Commit(committed)
                    } else {
                        let text = out.preedit.clone();
                        st.output = out;
                        Edit::Compose(text)
                    }
                };
                self.request_edit(context, edit)?;
                Ok(true)
            }
            KeyAction::Backspace => {
                let edit = {
                    let mut st = self.state.borrow_mut();
                    if st.ekushe.is_composing() {
                        let out = st.ekushe.backspace();
                        let text = out.preedit.clone();
                        st.output = out;
                        Edit::Compose(text)
                    } else {
                        let out = st.ekushe.backspace();
                        let expect = std::mem::take(&mut st.last_committed);
                        match out.committed {
                            Some(roman) if out.delete_before > 0 => Edit::ReplaceBefore {
                                expect,
                                text: roman,
                            },
                            _ => return Ok(false),
                        }
                    }
                };
                self.request_edit(context, edit)?;
                Ok(true)
            }
            KeyAction::Space | KeyAction::Enter => {
                let committed = {
                    let mut st = self.state.borrow_mut();
                    let sel = st.output.selected_index;
                    let trailing = (action == KeyAction::Space).then_some(' ');
                    let out = st.ekushe.select_candidate(sel, trailing);
                    let committed = out.committed.unwrap_or_default();
                    Self::after_commit(&mut st, &committed);
                    committed
                };
                self.request_edit(context, Edit::Commit(committed))?;
                // Enter: the word is written, then the application gets its Enter
                Ok(action == KeyAction::Space)
            }
            KeyAction::Escape => {
                self.finish_composition(context)?;
                Ok(true)
            }
            KeyAction::Tab => {
                let text = {
                    let mut st = self.state.borrow_mut();
                    let n = st.output.candidates.len();
                    let next = (st.output.selected_index + 1) % n.max(1);
                    st.output.selected_index = next;
                    let text = st.output.candidates[next].text.clone();
                    st.output.preedit = text.clone();
                    text
                };
                self.request_edit(context, Edit::Compose(text))?;
                Ok(true)
            }
            KeyAction::FlipLastWord => {
                let edit = {
                    let mut st = self.state.borrow_mut();
                    let expect = st.last_committed.clone();
                    match st.ekushe.flip_last_committed_word() {
                        Some((_, replacement)) => {
                            st.last_committed = replacement.clone();
                            Edit::ReplaceBefore {
                                expect,
                                text: replacement,
                            }
                        }
                        None => return Ok(false),
                    }
                };
                self.request_edit(context, edit)?;
                Ok(true)
            }
        }
    }

    /// Shift taps for the Double-Tap Shift hotkey (key-up of a Shift pressed on its own).
    fn shift_released(&self, context: Option<&ITfContext>) -> Result<()> {
        let tapped = {
            let mut st = self.state.borrow_mut();
            if !std::mem::take(&mut st.shift_clean_down)
                || st.ekushe.config().switch_hotkey != SwitchHotkey::DoubleTapShift
            {
                return Ok(());
            }
            let now = st.started.elapsed().as_millis() as u64;
            st.ekushe.on_shift_tap(now).is_some()
        };
        if tapped {
            if let Some(ctx) = context {
                // The language already changed; just close any open word
                let text = {
                    let mut st = self.state.borrow_mut();
                    let t = st.output.preedit.clone();
                    st.output = SessionOutput::default();
                    t
                };
                if self.composition.borrow().is_some() {
                    self.request_edit(ctx, Edit::Commit(text))?;
                }
            }
        }
        Ok(())
    }

    fn is_shift(vk: u32) -> bool {
        let v = VIRTUAL_KEY(vk as u16);
        v == VK_SHIFT || v == VK_LSHIFT || v == VK_RSHIFT
    }
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, thread_mgr: Option<&ITfThreadMgr>, client_id: u32) -> Result<()> {
        let thread_mgr = thread_mgr.ok_or(windows::core::Error::from(E_INVALIDARG))?;
        {
            let mut st = self.state.borrow_mut();
            st.thread_mgr = Some(thread_mgr.clone());
            st.client_id = client_id;
            // Per-app language memory: this input method lives inside the application's process
            if let Ok(exe) = std::env::current_exe() {
                if let Some(name) = exe.file_name() {
                    let name = name.to_string_lossy().to_lowercase();
                    st.ekushe.on_app_focus(&name);
                }
            }
        }
        let keystrokes: ITfKeystrokeMgr = thread_mgr.cast()?;
        let sink = self.to_interface::<ITfKeyEventSink>();
        unsafe { keystrokes.AdviseKeyEventSink(client_id, &sink, TRUE) }
    }

    fn Deactivate(&self) -> Result<()> {
        let (thread_mgr, client_id) = {
            let mut st = self.state.borrow_mut();
            save_learned(&mut st);
            st.ekushe.reset_composition();
            st.output = SessionOutput::default();
            (st.thread_mgr.take(), st.client_id)
        };
        // TSF ends open compositions itself when the service is deactivated
        self.composition.borrow_mut().take();
        if let Some(tm) = thread_mgr {
            if let Ok(keystrokes) = tm.cast::<ITfKeystrokeMgr>() {
                unsafe {
                    let _ = keystrokes.UnadviseKeyEventSink(client_id);
                }
            }
        }
        Ok(())
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        context: Option<&ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        if Self::is_shift(vk) {
            let mut st = self.state.borrow_mut();
            st.shift_clean_down = !key_down(VK_CONTROL) && !key_down(VK_MENU);
            return Ok(FALSE);
        }
        if let Some(ctx) = context {
            if self.starting_word_in_private_field(ctx) {
                return Ok(FALSE);
            }
        }
        let action = {
            let mut st = self.state.borrow_mut();
            st.shift_clean_down = false;
            classify(&st, vk, lparam)
        };
        match action {
            KeyAction::Pass => Ok(FALSE),
            // OnKeyDown is not called for keys we do not eat, so finish the word here
            KeyAction::FinishThenPass => {
                if let Some(ctx) = context {
                    self.finish_composition(ctx)?;
                }
                Ok(FALSE)
            }
            _ => Ok(TRUE),
        }
    }

    fn OnKeyDown(
        &self,
        context: Option<&ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        if Self::is_shift(vk) {
            return Ok(FALSE);
        }
        let Some(ctx) = context else {
            return Ok(FALSE);
        };
        if self.starting_word_in_private_field(ctx) {
            return Ok(FALSE);
        }
        let action = {
            let mut st = self.state.borrow_mut();
            st.shift_clean_down = false;
            classify(&st, vk, lparam)
        };
        Ok(self.execute(ctx, action)?.into())
    }

    fn OnTestKeyUp(
        &self,
        context: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        if Self::is_shift(wparam.0 as u32) {
            self.shift_released(context)?;
        }
        Ok(FALSE)
    }

    fn OnKeyUp(
        &self,
        context: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        if Self::is_shift(wparam.0 as u32) {
            self.shift_released(context)?;
        }
        Ok(FALSE)
    }

    fn OnPreservedKey(&self, _context: Option<&ITfContext>, _guid: *const GUID) -> Result<BOOL> {
        Ok(FALSE)
    }
}

impl ITfCompositionSink_Impl for TextService_Impl {
    /// The application (or TSF) ended our composition: a click elsewhere, focus change, etc. The text
    /// stays as it is; the engine forgets the half-typed word.
    fn OnCompositionTerminated(
        &self,
        _ec: u32,
        _composition: Option<&ITfComposition>,
    ) -> Result<()> {
        if let Ok(mut c) = self.composition.try_borrow_mut() {
            c.take();
        }
        if let Ok(mut st) = self.state.try_borrow_mut() {
            st.ekushe.reset_composition();
            st.output = SessionOutput::default();
            st.last_committed.clear();
        }
        Ok(())
    }
}

// ───────────────────────────── Edit session ─────────────────────────────

#[implement(ITfEditSession)]
struct EditSession {
    context: ITfContext,
    sink: ITfCompositionSink,
    composition: Rc<RefCell<Option<ITfComposition>>>,
    edit: Edit,
}

impl EditSession {
    /// Range of the open composition, starting one at the caret if there is none.
    unsafe fn composition_range(&self, ec: u32) -> Result<ITfRange> {
        if let Some(c) = self.composition.borrow().as_ref() {
            return c.GetRange();
        }
        let insert: ITfInsertAtSelection = self.context.cast()?;
        let range = insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])?;
        let ctx_comp: ITfContextComposition = self.context.cast()?;
        let comp = ctx_comp.StartComposition(ec, &range, &self.sink)?;
        let r = comp.GetRange()?;
        *self.composition.borrow_mut() = Some(comp);
        Ok(r)
    }

    unsafe fn caret_to_end(&self, ec: u32, range: &ITfRange) -> Result<()> {
        let caret = range.Clone()?;
        caret.Collapse(ec, TF_ANCHOR_END)?;
        let mut sel = [TF_SELECTION {
            range: ManuallyDrop::new(Some(caret)),
            style: TF_SELECTIONSTYLE {
                ase: TF_AE_NONE,
                fInterimChar: FALSE,
            },
        }];
        let r = self.context.SetSelection(ec, &sel);
        ManuallyDrop::drop(&mut sel[0].range);
        r
    }

    unsafe fn end_composition(&self, ec: u32) -> Result<()> {
        if let Some(c) = self.composition.borrow_mut().take() {
            c.EndComposition(ec)?;
        }
        Ok(())
    }

    /// The current selection (caret) as a range.
    unsafe fn selection(&self, ec: u32) -> Result<ITfRange> {
        let mut sel = [TF_SELECTION::default()];
        let mut fetched = 0u32;
        self.context
            .GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
        let range = ManuallyDrop::take(&mut sel[0].range);
        match (fetched, range) {
            (1, Some(r)) => Ok(r),
            _ => Err(E_FAIL.into()),
        }
    }

    unsafe fn run(&self, ec: u32) -> Result<()> {
        match &self.edit {
            Edit::Compose(text) => {
                if text.is_empty() {
                    // Everything was backspaced away: remove the composition and its text
                    if let Some(c) = self.composition.borrow().as_ref() {
                        c.GetRange()?.SetText(ec, 0, &[])?;
                    }
                    return self.end_composition(ec);
                }
                let range = self.composition_range(ec)?;
                range.SetText(ec, 0, &wide(text))?;
                self.caret_to_end(ec, &range)
            }
            Edit::Commit(text) => {
                let range = if self.composition.borrow().is_some() {
                    let r = self.composition_range(ec)?;
                    r.SetText(ec, 0, &wide(text))?;
                    r
                } else if text.is_empty() {
                    return Ok(());
                } else {
                    let insert: ITfInsertAtSelection = self.context.cast()?;
                    insert.InsertTextAtSelection(
                        ec,
                        INSERT_TEXT_AT_SELECTION_FLAGS(0),
                        &wide(text),
                    )?
                };
                self.caret_to_end(ec, &range)?;
                self.end_composition(ec)
            }
            Edit::ReplaceBefore { expect, text } => {
                self.end_composition(ec)?;
                let range = self.selection(ec)?;
                if !range.IsEmpty(ec)?.as_bool() {
                    // A selection: Backspace deletes it, as usual
                    range.SetText(ec, 0, &[])?;
                    return Ok(());
                }
                range.Collapse(ec, TF_ANCHOR_START)?;
                let want = expect.encode_utf16().count() as i32;
                let mut moved = 0i32;
                if want > 0 {
                    range.ShiftStart(ec, -want, &mut moved, std::ptr::null())?;
                }
                let mut buf = vec![0u16; want.max(0) as usize + 1];
                let mut got = 0u32;
                if want > 0 && -moved == want {
                    range.GetText(ec, 0, &mut buf, &mut got)?;
                }
                if want > 0 && String::from_utf16_lossy(&buf[..got as usize]) == *expect {
                    range.SetText(ec, 0, &wide(text))?;
                    self.caret_to_end(ec, &range)
                } else {
                    // The caret moved since the commit: an ordinary Backspace
                    let one = self.selection(ec)?;
                    one.Collapse(ec, TF_ANCHOR_START)?;
                    let mut m = 0i32;
                    one.ShiftStart(ec, -1, &mut m, std::ptr::null())?;
                    if m == -1 {
                        one.SetText(ec, 0, &[])?;
                    }
                    Ok(())
                }
            }
        }
    }
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let r = unsafe { self.run(ec) };
        if r.is_err() {
            // Never leave a dangling composition behind
            if let Ok(mut c) = self.composition.try_borrow_mut() {
                c.take();
            }
        }
        r.map_err(|e| {
            if e.code().is_ok() {
                E_UNEXPECTED.into()
            } else {
                e
            }
        })
    }
}

/// Reads the input scope at the caret: password and PIN scopes make the field private.
#[implement(ITfEditSession)]
struct ScopeSession {
    context: ITfContext,
    secret: Rc<Cell<bool>>,
}

impl ITfEditSession_Impl for ScopeSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        unsafe {
            let prop = self.context.GetProperty(&GUID_PROP_INPUTSCOPE)?;
            let mut sel = [TF_SELECTION::default()];
            let mut fetched = 0u32;
            self.context
                .GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
            let range = std::mem::ManuallyDrop::take(&mut sel[0].range);
            let (true, Some(range)) = (fetched > 0, range) else {
                return Ok(());
            };
            let value = prop.GetValue(ec, &range)?;
            let Ok(unk) = windows_core::IUnknown::try_from(&value) else {
                return Ok(());
            };
            let scope: ITfInputScope = unk.cast()?;
            let mut list: *mut InputScope = std::ptr::null_mut();
            let mut count = 0u32;
            scope.GetInputScopes(&mut list, &mut count)?;
            if !list.is_null() {
                let scopes = std::slice::from_raw_parts(list, count as usize);
                let secret = [
                    IS_PASSWORD,
                    IS_NUMERIC_PASSWORD,
                    IS_NUMERIC_PIN,
                    IS_ALPHANUMERIC_PIN,
                    IS_ALPHANUMERIC_PIN_SET,
                ];
                self.secret.set(scopes.iter().any(|s| secret.contains(s)));
                windows::Win32::System::Com::CoTaskMemFree(Some(list as *const _));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_needs_a_protected_folder() {
        assert!(!in_protected_folder(
            r"C:\Users\someone\Downloads\ekushe_tsf.dll"
        ));
        assert!(!in_protected_folder(
            r"D:\a\ekushenext\target\release\ekushe_tsf.dll"
        ));
        if let Ok(pf) = std::env::var("ProgramFiles") {
            assert!(in_protected_folder(&format!(
                r"{pf}\Ekushe Next\ekushe_tsf.dll"
            )));
            assert!(!in_protected_folder(&format!(
                r"{pf}\..\Users\x\ekushe_tsf.dll"
            )));
        }
    }

    #[test]
    fn guids_match_their_strings() {
        assert_eq!(format!("{{{:?}}}", EKUSHE_CLSID), EKUSHE_CLSID_STR);
        assert_eq!(
            format!("{{{:?}}}", EKUSHE_PROFILE_GUID),
            EKUSHE_PROFILE_GUID_STR
        );
    }
}
