//! Ekushe User Configuration, Hotkey Options, Typing Modes, and Per-App Profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::layout::{LayoutAutomation, LayoutId};

/// Switch hotkey presets for toggling between Normal Default Keyboard (EN) and Ekushe (বাং).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SwitchHotkey {
    /// Default: tap Shift twice quickly (~350ms) — zero conflict with IDEs/apps, super easy for beginners
    #[default]
    DoubleTapShift,
    /// Classic Linux / macOS IME shortcut
    CtrlSpace,
    /// Classic Avro Keyboard shortcut
    F12,
    /// Three-key chord (`Ctrl + Alt + B`)
    CtrlAltB,
    /// User-defined custom chord (e.g., `"Alt+Space"`, `"Ctrl+Shift+B"`)
    Custom(String),
}

impl SwitchHotkey {
    pub fn display_name(&self) -> String {
        match self {
            Self::DoubleTapShift => "Double-Tap Shift (Recommended)".to_string(),
            Self::CtrlSpace => "Ctrl + Space".to_string(),
            Self::F12 => "F12 (Classic Avro)".to_string(),
            Self::CtrlAltB => "Ctrl + Alt + B".to_string(),
            Self::Custom(s) => format!("Custom ({s})"),
        }
    }
}

/// Typing behavior mode (inspired by Lekho & Avro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TypingMode {
    /// The literal phonetic spelling is #1 when it is a common real word; otherwise the dictionary's choice
    PhoneticFirst,
    /// Default: Ekushe's dictionary ranking picks the best word automatically on Space
    #[default]
    Smart,
    /// Pure character-by-character phonetic with no popup window
    PhoneticOnly,
}

/// Input language state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InputLanguage {
    /// Normal default English keyboard passthrough
    English,
    /// Ekushe Bangla active
    #[default]
    Bangla,
}

/// Design and layout programs people use with SutonnyMJ fonts (Windows program names, and the
/// Mac apps' bundle IDs in small letters)
pub const DEFAULT_BIJOY_APPS: &[&str] = &[
    "photoshop.exe",
    "illustrator.exe",
    "indesign.exe",
    "coreldrw.exe",
    "qxp.exe",
    "quarkxpress.exe",
    "pm70.exe",
    "pm65.exe",
    "afterfx.exe",
    "com.adobe.photoshop",
    "com.adobe.illustrator",
    "com.adobe.indesign",
    "com.adobe.aftereffects",
    "com.quark.quarkxpress",
    "com.corel.coreldraw",
];

/// The Bengali font Ekushe uses unless the user picks another: Shishir Bangla, bundled with Ekushe
/// (`assets/fonts/ShishirBangla-*.ttf`, four weights) and installed with it on Windows, Linux and macOS.
pub const DEFAULT_BENGALI_FONT: &str = "Shishir Bangla";

/// The Bengali font that was the default before Shishir Bangla (on Windows, Linux and macOS alike)
pub const OLD_DEFAULT_BENGALI_FONT: &str = "Kalpurush";

/// The settings format this version writes. 1: the old default font (Kalpurush) was moved to Shishir
/// Bangla once; a file without `settings_version` is from before that.
pub const SETTINGS_VERSION: u32 = 1;

/// Full Ekushe configuration persisted in `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Which upgrades have been made to this file ([`SETTINGS_VERSION`]); missing means 0
    #[serde(default)]
    pub settings_version: u32,
    /// Hotkey to toggle between Normal Default Keyboard and Ekushe
    pub switch_hotkey: SwitchHotkey,
    /// Hotkey to convert selected Banglish/Bijoy text in any app
    pub convert_selection_hotkey: String,
    /// Active layout (Phonetic, Probhat, National/Jatiya, Bornona, Munir Optima, or Custom: a layout file)
    pub active_layout: LayoutId,
    /// The imported layout file used when `active_layout` is `Custom` (its id)
    pub custom_layout: Option<String>,
    /// Active typing mode
    pub typing_mode: TypingMode,
    /// Whether digits 0-9 produce ০-৯ when typing in Bangla (Ekushe mode = ০-৯, English mode = 0-9)
    pub bangla_digits: bool,
    /// Show live next-key hints for vowel signs (কার) and connected letters (যুক্তবর্ণ)
    pub show_sign_and_conjunct_hints: bool,
    /// Compact 2-tier floating popup mode (hides the 3rd যুক্তবর্ণ row when true)
    pub compact_popup: bool,
    /// Automatically pass through English in Terminal / Command Prompt apps (cmd, PowerShell, Windows Terminal)
    pub auto_english_in_terminals: bool,
    /// Show Roman preview tooltip above the Bangla composition
    pub show_roman_preview: bool,
    /// Include emoji candidates in suggestions
    pub enable_emoji_suggestions: bool,
    /// Enable next-word predictions after committing a word
    pub enable_next_word_prediction: bool,
    /// Maximum number of candidates shown (1..=9)
    pub max_candidates: usize,
    /// Bengali font family for the suggestion window and Studio's writing: [`DEFAULT_BENGALI_FONT`]
    /// unless the user picks another bundled one (`Kalpurush`, `SolaimanLipi`, `Siyam Rupali`, `Hind Siliguri`)
    pub candidate_font_family: String,
    /// Candidate window font size in pt
    pub candidate_font_size: u16,
    /// Fixed-layout typing automation (pre-base kar reordering, auto vowel forming, chandrabindu fix)
    pub layout_automation: LayoutAutomation,
    /// Fixed layouts: when a word isn't spelled the standard way, offer the standard spelling next
    /// to it (Space still types the word as written)
    pub fixed_spelling_hint: bool,
    /// Fixed layouts: offer common words that start with what is typed
    pub fixed_word_suggestions: bool,
    /// Type Bijoy (SutonnyMJ) text instead of Unicode in the apps of `bijoy_output_apps` (design
    /// and layout programs used with SutonnyMJ fonts)
    pub bijoy_output: bool,
    /// Programs that get Bijoy text when `bijoy_output` is on (Windows program names, small letters)
    pub bijoy_output_apps: Vec<String>,
    /// The "how do you type Bangla?" welcome in Studio was answered (or put away)
    pub welcome_done: bool,
    /// Paste the clipboard as the app in front needs it: Unicode, or Bijoy in a Bijoy-output app
    pub smart_paste_hotkey: String,
    /// Per-application default language override (e.g. `"code.exe" -> English`, `"winword.exe" -> Bangla`)
    pub per_app_defaults: HashMap<String, InputLanguage>,
    /// User phonetic overrides (`roman -> bangla`)
    pub phonetic_overrides: HashMap<String, String>,
    /// User custom text expansion snippets (`;trigger -> expansion`)
    pub custom_snippets: HashMap<String, String>,
    /// Backspace right after a word is committed brings back the Roman text that was typed
    pub backspace_undo: bool,
    /// Windows: show the word being typed inside the app as it changes (on by default). Off: the
    /// word waits in Ekushe's suggestion bar and is typed into the app once, when it is placed
    /// (Space, Enter, punctuation), for an app that doesn't take quick retyping well.
    pub type_while_composing: bool,
    /// The language Ekushe starts in when the keyboard starts (when you log in): English by
    /// default, so the computer starts like any other; double-tap Shift for Bangla
    pub start_language: InputLanguage,
    /// Once a day, ask GitHub whether a newer Ekushe Next has been released (Windows; Linux updates
    /// come through the package manager). Off unless the user turns it on: Ekushe stays offline.
    pub check_for_updates: bool,
    /// Voice typing: the shortcut that starts (and stops) dictation in any app
    pub voice_hotkey: String,
    /// Voice typing: say "দাঁড়ি", "কমা", "নতুন লাইন" to type punctuation
    pub voice_spoken_punctuation: bool,
    /// Voice typing: numbers said in words are typed as digits
    pub voice_numbers_as_digits: bool,
    /// Voice typing: "fast" (greedy) or "best" (beam search, slower)
    pub voice_quality: String,
    /// Voice typing: a pause this long (ms) after speaking ends the dictation
    pub voice_stop_after_ms: u32,
    /// Voice typing: which speech model to use, "large" (most accurate) or "small" (faster, for
    /// slower computers). If only the other one is downloaded, that one is used.
    pub voice_model: String,
    /// Voice typing: end the dictation by itself after a pause (`voice_stop_after_ms`). Off: it
    /// keeps listening, typing at each pause, until the shortcut or Esc is pressed.
    pub voice_auto_stop: bool,
    /// Voice typing: a soft sound when the microphone opens and closes
    pub voice_sounds: bool,
    /// Voice typing: use the graphics card when there is one (off: the processor only)
    pub voice_use_gpu: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut per_app = HashMap::new();
        per_app.insert("windowsterminal.exe".to_string(), InputLanguage::English);
        per_app.insert("cmd.exe".to_string(), InputLanguage::English);
        per_app.insert("powershell.exe".to_string(), InputLanguage::English);
        per_app.insert("pwsh.exe".to_string(), InputLanguage::English);

        Self {
            settings_version: SETTINGS_VERSION,
            switch_hotkey: SwitchHotkey::DoubleTapShift,
            convert_selection_hotkey: "Ctrl+Alt+C".to_string(),
            active_layout: LayoutId::AvroPhonetic,
            custom_layout: None,
            typing_mode: TypingMode::Smart,
            bangla_digits: true,
            show_sign_and_conjunct_hints: true,
            compact_popup: false,
            auto_english_in_terminals: true,
            show_roman_preview: true,
            enable_emoji_suggestions: true,
            enable_next_word_prediction: true,
            max_candidates: 9,
            candidate_font_family: DEFAULT_BENGALI_FONT.to_string(),
            candidate_font_size: 16,
            layout_automation: LayoutAutomation::default(),
            fixed_spelling_hint: true,
            fixed_word_suggestions: false,
            bijoy_output: false,
            bijoy_output_apps: DEFAULT_BIJOY_APPS.iter().map(|s| s.to_string()).collect(),
            welcome_done: false,
            smart_paste_hotkey: "Ctrl+Alt+V".into(),
            per_app_defaults: per_app,
            phonetic_overrides: HashMap::new(),
            custom_snippets: HashMap::new(),
            backspace_undo: true,
            type_while_composing: true,
            check_for_updates: false,
            start_language: InputLanguage::English,
            voice_hotkey: "Ctrl+Alt+Space".into(),
            voice_spoken_punctuation: true,
            voice_numbers_as_digits: true,
            voice_quality: "fast".into(),
            voice_stop_after_ms: 1400,
            voice_model: "live".into(),
            voice_auto_stop: false,
            voice_sounds: true,
            voice_use_gpu: true,
        }
    }
}

impl Config {
    /// Where voice typing keeps its model and what it learned: `<config dir>/voice`
    pub fn voice_dir() -> PathBuf {
        Self::default_dir().join("voice")
    }

    /// Default OS config directory (`%APPDATA%\Ekushe` on Windows, `~/Library/Application Support/Ekushe` on macOS, `$XDG_CONFIG_HOME/ekushe` on Linux).
    pub fn default_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Ekushe")
    }

    pub fn default_config_path() -> PathBuf {
        Self::default_dir().join("config.toml")
    }

    /// Read a settings file, bringing one written by an older version up to date in memory (see
    /// [`Config::upgrade`]).
    pub fn load_from(path: &Path) -> std::io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let mut cfg: Self = toml::from_str(&raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        cfg.upgrade();
        Ok(cfg)
    }

    /// [`Config::load_from`], and write the file back when it was upgraded, so each upgrade happens
    /// once: a font chosen afterwards is never changed again.
    pub fn load_and_upgrade(path: &Path) -> std::io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let mut cfg: Self = toml::from_str(&raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if cfg.upgrade() {
            cfg.save_to(path)?;
        }
        Ok(cfg)
    }

    /// Upgrade settings from an older version; returns whether anything changed. Version 1: the old
    /// default font, Kalpurush, becomes Shishir Bangla (the setup wrote the default into every
    /// settings file, so Kalpurush there means "the default"); any other saved font is kept.
    pub fn upgrade(&mut self) -> bool {
        if self.settings_version >= SETTINGS_VERSION {
            return false;
        }
        if self.candidate_font_family == OLD_DEFAULT_BENGALI_FONT {
            self.candidate_font_family = DEFAULT_BENGALI_FONT.to_string();
        }
        self.settings_version = SETTINGS_VERSION;
        true
    }

    /// Save (owner-only on macOS and Linux, replaced in one step so a crash never leaves half a file).
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        let serialized = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::suggest::write_atomic(path, &serialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_bengali_font_is_shishir_bangla() {
        assert_eq!(DEFAULT_BENGALI_FONT, "Shishir Bangla");
        assert_eq!(Config::default().candidate_font_family, "Shishir Bangla");
        // A settings file that never chose a font gets the default
        let cfg: Config = toml::from_str("bangla_digits = true\n").unwrap();
        assert_eq!(cfg.candidate_font_family, DEFAULT_BENGALI_FONT);
    }

    #[test]
    fn a_font_the_user_chose_is_kept() {
        let cfg: Config = toml::from_str("candidate_font_family = \"Siyam Rupali\"\n").unwrap();
        assert_eq!(cfg.candidate_font_family, "Siyam Rupali");
        let saved = toml::to_string_pretty(&cfg).unwrap();
        let again: Config = toml::from_str(&saved).unwrap();
        assert_eq!(again.candidate_font_family, "Siyam Rupali");
    }

    /// A settings file in its own temporary folder
    fn settings_file(name: &str, contents: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ekushe-config-test-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn the_old_default_font_moves_to_shishir_bangla_once() {
        // Saved by a version from before Shishir Bangla, with the old default Kalpurush
        let path = settings_file("old", "candidate_font_family = \"Kalpurush\"\n");
        let cfg = Config::load_and_upgrade(&path).unwrap();
        assert_eq!(cfg.candidate_font_family, DEFAULT_BENGALI_FONT);
        assert_eq!(cfg.settings_version, SETTINGS_VERSION);
        // ... and the move is written down, so it is not done again
        let on_disk = fs::read_to_string(&path).unwrap();
        assert!(on_disk.contains("settings_version = 1"));
        assert!(on_disk.contains("candidate_font_family = \"Shishir Bangla\""));
        // In memory, too, an old file is moved (backups, the other apps that read the file)
        let cfg = Config::load_from(&settings_file(
            "old2",
            "candidate_font_family = \"Kalpurush\"\n",
        ))
        .unwrap();
        assert_eq!(cfg.candidate_font_family, DEFAULT_BENGALI_FONT);
    }

    #[test]
    fn another_saved_font_is_kept_when_upgrading() {
        for font in ["SolaimanLipi", "Siyam Rupali", "Hind Siliguri"] {
            let path = settings_file("custom", &format!("candidate_font_family = \"{font}\"\n"));
            let cfg = Config::load_and_upgrade(&path).unwrap();
            assert_eq!(cfg.candidate_font_family, font);
            assert_eq!(cfg.settings_version, SETTINGS_VERSION);
        }
    }

    #[test]
    fn kalpurush_chosen_after_the_upgrade_is_kept() {
        let path = settings_file(
            "chosen",
            "settings_version = 1\ncandidate_font_family = \"Kalpurush\"\n",
        );
        let before = fs::read_to_string(&path).unwrap();
        let cfg = Config::load_and_upgrade(&path).unwrap();
        assert_eq!(cfg.candidate_font_family, "Kalpurush");
        assert_eq!(fs::read_to_string(&path).unwrap(), before, "file rewritten");
        // A new install starts already upgraded
        assert_eq!(Config::default().settings_version, SETTINGS_VERSION);
    }
}
