//! `ekushe-core`: Clean-room Bangla Input Method Engine.
//!
//! Unites:
//! - [`phonetic`]: Avro-compatible phonetic transliteration with `+` explicit conjunct joiner & `` ` `` breaker
//! - [`conjuncts`]: Vowel signs (কার), marks (চিহ্ন), folas (ফলা), and connected letters (যুক্তবর্ণ) guide & live next-key hints
//! - [`layout`]: Fixed layouts (Probhat, National/Jatiya, Bornona, Munir Optima) with smart pre-base kar reordering
//! - [`suggest`]: Dictionary suggestions, forgiving phonetic skeleton matching, autocorrect, emojis, snippets, bigrams, and user learning
//! - [`spell`]: Bangla spell checker with confusable-pair weighted edit distance and suffix stripping
//! - [`convert`]: Bijoy (ANSI / SutonnyMJ) <-> Unicode converter and Banglish selection converter
//! - [`config`]: Hotkeys (Double-Tap Shift default, Ctrl+Space, F12, Ctrl+Alt+B), modes, and per-app profiles

pub mod backup;
pub mod config;

/// This build's release, e.g. `v0.1.0-beta.10` (from .github/release/VERSION).
pub fn release() -> &'static str {
    include_str!("../../../.github/release/VERSION").trim()
}

/// A version as `(major, minor, patch, pre-release number)`; a final release sorts after its betas.
fn version_key(v: &str) -> Option<(u64, u64, u64, u64)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let (main, pre) = match v.split_once('-') {
        Some((m, p)) => (m, Some(p)),
        None => (v, None),
    };
    let mut n = main.split('.').map(|x| x.parse::<u64>().ok());
    let (a, b, c) = (
        n.next()??,
        n.next().unwrap_or(Some(0))?,
        n.next().unwrap_or(Some(0))?,
    );
    let pre = match pre {
        None => u64::MAX,
        Some(p) => p
            .rsplit(|ch: char| !ch.is_ascii_digit())
            .next()
            .and_then(|d| d.parse().ok())
            .unwrap_or(0),
    };
    Some((a, b, c, pre))
}

/// True when release tag `latest` (e.g. `v0.1.0-beta.16`) is newer than version `current`.
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    matches!((version_key(latest), version_key(current)), (Some(l), Some(c)) if l > c)
}

/// The web form for reporting a problem, with this build's details filled in (nothing typed is
/// included). `os` names the computer, e.g. `Windows 11` or `macOS`.
pub fn report_url(os: &str, config: &config::Config) -> String {
    let enc = |s: &str| {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' => {
                    (b as char).to_string()
                }
                _ => format!("%{b:02X}"),
            })
            .collect::<String>()
    };
    format!(
        "https://ekushenext.com/report/?v={}&os={}&layout={}&mode={}&from=app",
        enc(release().trim_start_matches('v')),
        enc(os),
        enc(&format!("{:?}", config.active_layout)),
        enc(&format!("{:?}", config.typing_mode)),
    )
}
pub mod conjuncts;
pub mod convert;
pub mod dates;
pub mod dict;
pub mod layout;
pub mod marks;
pub mod phonetic;
pub mod roman;
pub mod spell;
pub mod suggest;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::{Config, InputLanguage, TypingMode};
use crate::conjuncts::SignHint;
use crate::layout::{FixedLayout, LayoutId};
use crate::phonetic::PhoneticEngine;
use crate::suggest::{Candidate, CandidateKind, SuggestionEngine};

/// Snapshot of the current IME state returned after every key event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionOutput {
    /// Active input language (`English` passthrough vs `Bangla` active)
    pub language: InputLanguage,
    /// Current Bangla preedit composition text (underlined in the editor)
    pub preedit: String,
    /// Raw Roman buffer typed so far (for preview tooltip)
    pub roman_buffer: String,
    /// Ranked candidate list (1..=9)
    pub candidates: Vec<Candidate>,
    /// Currently highlighted candidate index
    pub selected_index: usize,
    /// Contextual next-key hints for vowel signs (কার), ফলা, and connected letters (যুক্তবর্ণ)
    pub sign_hints: Vec<SignHint>,
    /// Next-word predictions (populated right after a word is committed)
    pub next_word_predictions: Vec<String>,
    /// If the key caused text to be committed to the application, holds the committed string
    pub committed: Option<String>,
    /// UTF-16 units the host must delete *before* inserting `committed` (used by Backspace undo,
    /// which replaces the just-committed word and its space with the original Roman text)
    #[serde(default)]
    pub delete_before: usize,
}

/// Interactive Ekushe typing session.
#[derive(Debug, Clone)]
pub struct Ekushe {
    config: Config,
    language: InputLanguage,
    phonetic: PhoneticEngine,
    suggestions: SuggestionEngine,
    fixed_layout: Option<FixedLayout>,
    /// Raw Roman characters in Phonetic mode
    roman_buf: String,
    /// Composed Bangla buffer in Fixed Layout mode
    fixed_buf: String,
    /// Pending pre-base kar (`ি`, `ে`, `ৈ`) in Fixed Layout mode
    pending_prebase_kar: Option<char>,
    /// Currently highlighted candidate index
    selected_idx: usize,
    /// Last committed `(roman_input, committed_bangla)` for one-key Backspace Undo Conversion
    last_commit_undo: Option<(String, String)>,
    /// Last Shift key release timestamp (ms) for Double-Tap Shift detection
    last_shift_tap_ms: Option<u64>,
    /// Whether any non-Shift key was pressed since the last Shift tap
    intervening_key_since_shift: bool,
    /// Last committed Bangla word, used as context for ranking the next one
    prev_word: Option<String>,
    /// Executable of the focused application (lower-case), for per-app language memory
    current_app: Option<String>,
    /// Language last used in each application this session
    app_languages: HashMap<String, InputLanguage>,
    /// The host says the focused field is private (a password, PIN or "incognito" field): nothing
    /// typed there is learned or kept as context
    private_field: bool,
    /// The last few Bangla words committed, newest last, so the user can say one was wrong
    /// ([`recent_words`](Self::recent_words)). Memory only: never saved, never from a private field.
    recent: std::collections::VecDeque<RecentWord>,
}

/// How many committed words [`Ekushe::recent_words`] remembers
pub const RECENT_WORDS: usize = 12;

/// A word Ekushe placed, kept briefly so the user can report it as wrong.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RecentWord {
    /// What was typed (without trailing punctuation)
    pub roman: String,
    /// What Ekushe placed
    pub word: String,
    /// The other suggestions that were offered, best first
    pub others: Vec<String>,
    /// The word before it, if it was used as context
    pub prev: Option<String>,
}

/// A word the user reported as wrong, with the word they meant.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WrongWord {
    pub roman: String,
    pub wrong: String,
    pub right: String,
    pub prev: Option<String>,
}

impl WrongWord {
    /// One line of the corrections file: `roman \t wrong \t right \t previous word`
    pub fn tsv_line(&self) -> String {
        let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
        format!(
            "{}\t{}\t{}\t{}",
            clean(&self.roman),
            clean(&self.wrong),
            clean(&self.right),
            clean(self.prev.as_deref().unwrap_or(""))
        )
    }
}

impl Default for Ekushe {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Ekushe {
    pub fn new(config: Config) -> Self {
        let phonetic =
            PhoneticEngine::with_overrides(config.phonetic_overrides.clone(), config.bangla_digits);
        let fixed_layout = FixedLayout::builtin(config.active_layout);
        let mut suggestions = SuggestionEngine::new();
        for (trig, exp) in &config.custom_snippets {
            suggestions.set_snippet(trig.clone(), exp.clone());
        }
        Self {
            config,
            language: InputLanguage::Bangla,
            phonetic,
            suggestions,
            fixed_layout,
            roman_buf: String::new(),
            fixed_buf: String::new(),
            pending_prebase_kar: None,
            selected_idx: 0,
            last_commit_undo: None,
            last_shift_tap_ms: None,
            intervening_key_since_shift: false,
            prev_word: None,
            current_app: None,
            app_languages: HashMap::new(),
            private_field: false,
            recent: std::collections::VecDeque::new(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.phonetic =
            PhoneticEngine::with_overrides(config.phonetic_overrides.clone(), config.bangla_digits);
        self.fixed_layout = FixedLayout::builtin(config.active_layout);
        for (trig, exp) in &config.custom_snippets {
            self.suggestions.set_snippet(trig.clone(), exp.clone());
        }
        self.config = config;
    }

    pub fn language(&self) -> InputLanguage {
        self.language
    }

    pub fn suggestions_engine(&self) -> &SuggestionEngine {
        &self.suggestions
    }

    pub fn suggestions_engine_mut(&mut self) -> &mut SuggestionEngine {
        &mut self.suggestions
    }

    /// Toggle between Normal Default Keyboard (`English`) and Ekushe (`Bangla`).
    pub fn toggle_language(&mut self) -> InputLanguage {
        self.reset_composition();
        self.language = match self.language {
            InputLanguage::Bangla => InputLanguage::English,
            InputLanguage::English => InputLanguage::Bangla,
        };
        self.language
    }

    /// Handle a standalone Shift key tap at `now_ms`.
    /// If the user tapped Shift twice within 350ms with no intervening keys, toggles Bangla <-> English!
    pub fn on_shift_tap(&mut self, now_ms: u64) -> Option<InputLanguage> {
        if !self.intervening_key_since_shift {
            if let Some(prev_ms) = self.last_shift_tap_ms {
                if now_ms.saturating_sub(prev_ms) <= 350 {
                    self.last_shift_tap_ms = None;
                    return Some(self.toggle_language());
                }
            }
        }
        self.last_shift_tap_ms = Some(now_ms);
        self.intervening_key_since_shift = false;
        None
    }

    /// Call when the focused application changes. Remembers the language used in the app being left,
    /// then switches to the language last used in the new app, or its configured default
    /// (`per_app_defaults`, e.g. terminals → English), or keeps the current language.
    pub fn on_app_focus(&mut self, executable_name: &str) -> InputLanguage {
        let key = executable_name.to_lowercase();
        if self.current_app.as_deref() == Some(key.as_str()) {
            return self.language;
        }
        if let Some(prev) = self.current_app.take() {
            self.app_languages.insert(prev, self.language);
        }
        let next = self
            .app_languages
            .get(&key)
            .copied()
            .or_else(|| self.default_language_for(&key));
        if let Some(lang) = next {
            if lang != self.language {
                self.reset_composition();
                self.language = lang;
            }
        }
        self.current_app = Some(key);
        self.language
    }

    /// The language an app starts in, before the user has switched there. Terminals start in English
    /// while "English in terminals" is on, unless the user set them to Bangla in `per_app_defaults`;
    /// with it off they are like any other app.
    fn default_language_for(&self, app: &str) -> Option<InputLanguage> {
        let configured = self.config.per_app_defaults.get(app).copied();
        if is_terminal_app(app) {
            if configured == Some(InputLanguage::Bangla) {
                return configured;
            }
            return self
                .config
                .auto_english_in_terminals
                .then_some(InputLanguage::English);
        }
        configured
    }

    /// Feed a printable character into the session.
    pub fn push_char(&mut self, ch: char, altgr: bool) -> SessionOutput {
        self.intervening_key_since_shift = true;
        self.last_commit_undo = None;

        // If in English passthrough mode, pass character straight through
        if self.language == InputLanguage::English {
            return SessionOutput {
                language: InputLanguage::English,
                committed: Some(ch.to_string()),
                ..Default::default()
            };
        }

        // Word boundary (whitespace) commits the active composition
        if ch.is_whitespace() {
            return self.commit_default(Some(ch));
        }

        match self.config.active_layout {
            LayoutId::AvroPhonetic => {
                self.roman_buf.push(ch);
                self.selected_idx = 0;
                self.build_phonetic_output(None)
            }
            _ => {
                if let Some(layout) = &self.fixed_layout {
                    layout.apply_key(
                        &mut self.fixed_buf,
                        &mut self.pending_prebase_kar,
                        ch,
                        altgr,
                        &self.config.layout_automation,
                    );
                }
                let sign_hints = if self.config.show_sign_and_conjunct_hints {
                    conjuncts::contextual_sign_hints("", &self.fixed_buf)
                } else {
                    Vec::new()
                };
                SessionOutput {
                    language: self.language,
                    preedit: self.fixed_buf.clone(),
                    roman_buffer: String::new(),
                    candidates: Vec::new(),
                    selected_index: 0,
                    sign_hints,
                    next_word_predictions: Vec::new(),
                    committed: None,
                    delete_before: 0,
                }
            }
        }
    }

    /// Handle Backspace:
    /// - If currently composing a word, removes the last character.
    /// - If pressed immediately after committing a word, **undoes the conversion** back to the Roman word!
    pub fn backspace(&mut self) -> SessionOutput {
        self.intervening_key_since_shift = true;

        if !self.roman_buf.is_empty() {
            self.roman_buf.pop();
            self.selected_idx = 0;
            return self.build_phonetic_output(None);
        }

        if !self.fixed_buf.is_empty() {
            self.fixed_buf.pop();
            return SessionOutput {
                language: self.language,
                preedit: self.fixed_buf.clone(),
                ..Default::default()
            };
        }

        // Undo conversion: Backspace right after a word was committed with a space replaces that word
        // and its space with the original Roman text (স্বাধীন␠ → swadhin)
        if let Some((roman, committed_bangla)) = self.last_commit_undo.take() {
            return SessionOutput {
                language: self.language,
                delete_before: committed_bangla.encode_utf16().count() + 1,
                committed: Some(roman),
                ..Default::default()
            };
        }

        SessionOutput {
            language: self.language,
            ..Default::default()
        }
    }

    /// Select candidate at `index` (0-based) and commit it, recording user learning.
    pub fn select_candidate(&mut self, index: usize, trailing: Option<char>) -> SessionOutput {
        self.intervening_key_since_shift = true;
        // Fixed layouts compose into their own buffer; committing must take that text, not the
        // (empty) phonetic buffer, or the word is lost (ক + Space used to commit only " ").
        if self.config.active_layout != LayoutId::AvroPhonetic {
            return self.commit_default(trailing);
        }
        let (cands, _) = self.phonetic_candidates();

        let chosen = cands
            .get(index)
            .map(|c| c.text.clone())
            .unwrap_or_else(|| self.phonetic.transliterate_word(&self.roman_buf));

        let roman = std::mem::take(&mut self.roman_buf);
        // Undo / flip know exactly what was typed only when the word ended with a plain space
        // (after Enter the application inserted the line break itself).
        self.last_commit_undo = None;
        if !roman.is_empty() {
            let (stem, tail) = split_trailing_punctuation(&roman);
            let tail_bn = self.phonetic.transliterate_word(tail);
            let chosen_stem = chosen.strip_suffix(tail_bn.as_str()).unwrap_or(&chosen);
            // Never learn from a private field, or anything that looks like a password or code
            // (letters mixed with digits, or symbols such as @ # $ %). Only a real choice is learned
            // (a word other than Ekushe's first suggestion); taking the first one teaches the spell
            // checker the word but leaves the ranking free to follow context next time.
            if !self.private_field && !looks_secret(stem) {
                if index > 0 {
                    self.suggestions.record_user_choice(stem, chosen_stem);
                } else {
                    self.suggestions.note_committed_word(chosen_stem);
                }
                if chosen_stem
                    .chars()
                    .any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
                {
                    let others: Vec<String> = cands
                        .iter()
                        .filter(|c| {
                            c.kind != CandidateKind::Emoji && c.kind != CandidateKind::English
                        })
                        .map(|c| {
                            c.text
                                .strip_suffix(tail_bn.as_str())
                                .unwrap_or(&c.text)
                                .to_string()
                        })
                        .filter(|t| t != chosen_stem && !t.is_empty())
                        .take(5)
                        .collect();
                    if self.recent.len() == RECENT_WORDS {
                        self.recent.pop_front();
                    }
                    self.recent.push_back(RecentWord {
                        roman: stem.to_string(),
                        word: chosen_stem.to_string(),
                        others,
                        prev: self.prev_word.clone(),
                    });
                }
            }
            if trailing == Some(' ') && !self.private_field {
                self.last_commit_undo = Some((roman, chosen.clone()));
            }
        }

        let predictions = if self.config.enable_next_word_prediction {
            self.suggestions.predict_next(&chosen)
        } else {
            Vec::new()
        };

        // Context for the next word: only across a plain space between Bangla words
        let is_bangla = chosen
            .chars()
            .any(|c| ('\u{0980}'..='\u{09FF}').contains(&c));
        let ends_clean = chosen
            .chars()
            .last()
            .map(|c| ('\u{0980}'..='\u{09FF}').contains(&c) && c != '।')
            .unwrap_or(false);
        self.prev_word = if trailing == Some(' ') && is_bangla && ends_clean {
            Some(chosen.clone())
        } else {
            None
        };

        let mut committed = chosen;
        if let Some(tr) = trailing {
            committed.push(tr);
        }

        SessionOutput {
            language: self.language,
            preedit: String::new(),
            roman_buffer: String::new(),
            candidates: Vec::new(),
            selected_index: 0,
            sign_hints: Vec::new(),
            next_word_predictions: predictions,
            committed: Some(committed),
            delete_before: 0,
        }
    }

    /// Commit the currently highlighted candidate (or fixed-layout buffer).
    pub fn commit_default(&mut self, trailing: Option<char>) -> SessionOutput {
        if self.config.active_layout != LayoutId::AvroPhonetic {
            if let Some(pk) = self.pending_prebase_kar.take() {
                self.fixed_buf.push(pk);
            }
            let mut committed = std::mem::take(&mut self.fixed_buf);
            if let Some(tr) = trailing {
                committed.push(tr);
            }
            return SessionOutput {
                language: self.language,
                committed: Some(committed),
                ..Default::default()
            };
        }

        let idx = self.selected_idx;
        self.select_candidate(idx, trailing)
    }

    /// The words Ekushe placed most recently, newest first (memory only; see [`RECENT_WORDS`]).
    pub fn recent_words(&self) -> Vec<RecentWord> {
        self.recent.iter().rev().cloned().collect()
    }

    /// Forget the recent words (for example when the user locks the computer or asks to).
    pub fn clear_recent_words(&mut self) {
        self.recent.clear();
    }

    /// The user says the recent word `wrong` (typed as `roman`) should have been `right`: Ekushe
    /// learns it at once (`right` comes first for `roman` from now on). Returns the correction for the
    /// host to keep in the corrections file; `None` if no such recent word exists.
    pub fn correct_recent_word(
        &mut self,
        roman: &str,
        wrong: &str,
        right: &str,
    ) -> Option<WrongWord> {
        let right = right.trim();
        if right.is_empty() || right == wrong {
            return None;
        }
        let pos = self
            .recent
            .iter()
            .rposition(|r| r.roman == roman && r.word == wrong)?;
        let r = self.recent.remove(pos)?;
        self.suggestions
            .correct_user_choice(&r.roman, &r.word, right);
        Some(WrongWord {
            roman: r.roman,
            wrong: r.word,
            right: right.to_string(),
            prev: r.prev,
        })
    }

    /// True while Backspace would undo the last conversion (a word was just committed with a space).
    pub fn has_undo(&self) -> bool {
        self.last_commit_undo.is_some()
    }

    /// True while a word is being typed (Roman or fixed-layout buffer not empty).
    pub fn is_composing(&self) -> bool {
        !self.roman_buf.is_empty() || !self.fixed_buf.is_empty()
    }

    /// Tell the engine the focused field is private (password, PIN, a field marked private) or not.
    /// In a private field nothing is learned and no word is kept for context or undo. Hosts should
    /// also not show a preview there, and for passwords type plain characters instead.
    pub fn set_private_field(&mut self, private: bool) {
        if private != self.private_field {
            self.private_field = private;
            self.reset_composition();
        }
    }

    /// Whether the focused field was marked private ([`Self::set_private_field`]).
    pub fn is_private_field(&self) -> bool {
        self.private_field
    }

    /// Forget the word being composed, the context word and the last-commit undo. Hosts call this when
    /// the caret moves or focus changes, so nothing can later be backspaced at the wrong position.
    pub fn reset_composition(&mut self) {
        self.prev_word = None;
        self.last_commit_undo = None;
        self.roman_buf.clear();
        self.fixed_buf.clear();
        self.pending_prebase_kar = None;
        self.selected_idx = 0;
    }

    /// Flip the last committed word between its Bangla output and its original English/Roman word!
    /// Returns `(utf16_units_to_backspace, replacement_string)`.
    pub fn flip_last_committed_word(&mut self) -> Option<(usize, String)> {
        let (alt_word, current_word) = self.last_commit_undo.take()?;
        // Account for the trailing space committed with the word
        let delete_utf16 = current_word.encode_utf16().count() + 1;
        let replacement = format!("{alt_word} ");
        // Swap so pressing again flips back!
        self.last_commit_undo = Some((current_word, alt_word));
        Some((delete_utf16, replacement))
    }

    /// Ranked candidates for the Roman buffer. Punctuation typed straight after a word (`gai,`
    /// `bangla.` `keno?`) is not part of the word: the word is looked up on its own and the
    /// punctuation is added back to every candidate, so `bangla.` gives `বাংলা।`.
    fn phonetic_candidates(&self) -> (Vec<Candidate>, Vec<SignHint>) {
        self.suggest_roman(&self.roman_buf)
    }

    /// Candidates for any Roman word (used by Studio's writing pad), with the same handling of
    /// punctuation typed after the word as live typing.
    /// Convert selected text (Ctrl+Alt+C): Bijoy becomes Unicode; Banglish becomes Bangla word by word,
    /// each word as Space would place it. `None` when there is nothing to convert.
    pub fn convert_selection(&self, text: &str) -> Option<(String, convert::SelectionKind)> {
        let kind = convert::detect_selection(text)?;
        let out = match kind {
            convert::SelectionKind::BijoyToUnicode => convert::bijoy_to_unicode(text),
            convert::SelectionKind::BanglishToBangla => {
                convert::map_words(text, |w| self.banglish_word(w))
            }
        };
        (out != text).then_some((out, kind))
    }

    /// Is this Roman word a Bangla word written in English letters ("ami", "tomake", "bhalobashi")?
    /// True when a reasonably common dictionary word is spelt exactly this way. The phone keyboard's
    /// English mode uses it to leave Banglish alone instead of "correcting" it into English.
    pub fn is_banglish(&self, roman: &str) -> bool {
        let w = roman.trim().to_ascii_lowercase();
        if w.len() < 2 || !w.bytes().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }
        // the dictionary this engine uses (the phone hands it over itself; desktops find it on disk)
        let Some(dict) = self
            .suggestions
            .dictionary()
            .or_else(crate::dict::Dictionary::global)
        else {
            return false;
        };
        let ids = dict.bucket(&crate::roman::skeleton(&w));
        ids.iter().take(80).any(|&id| {
            dict.info(id).log_ppm > -0.5
                && dict
                    .romanizations(id)
                    .iter()
                    .any(|(cost, r)| *cost <= 1.5 && r.eq_ignore_ascii_case(&w))
        })
    }

    /// One Banglish word as Space would place it (the first suggestion that isn't an emoji).
    pub fn banglish_word(&self, word: &str) -> String {
        self.suggest_roman(word)
            .0
            .into_iter()
            .find(|c| c.kind != CandidateKind::Emoji)
            .map(|c| c.text)
            .unwrap_or_else(|| word.to_string())
    }

    pub fn suggest_roman(&self, roman: &str) -> (Vec<Candidate>, Vec<SignHint>) {
        let smart = self.config.typing_mode == TypingMode::Smart;
        let (stem, tail) = split_trailing_punctuation(roman);
        let (mut cands, mut hints) = self.suggestions.suggest_with_context(
            stem,
            &self.phonetic,
            smart,
            self.config.enable_emoji_suggestions,
            self.config.max_candidates,
            self.prev_word.as_deref(),
        );
        // A deliberate mark (`,,` `^` `rri` `rr` `t``` `:` `Z` `+` `` ` ``) puts the word it asks for
        // first; and words in the list with a mark the user did not type get a tip on how to type them.
        let literal = self.phonetic.transliterate_word(stem);
        if !self.suggestions.has_autocorrect(stem) {
            marks::honor_marks(stem, &literal, &mut cands);
        }
        let first_for = |v: &str| self.suggest_roman(v).0.into_iter().next().map(|c| c.text);
        let tips = marks::mark_tips(stem, &cands, &self.phonetic, &first_for);
        if !tips.is_empty() {
            hints.splice(0..0, tips);
        }
        if !tail.is_empty() {
            let tail_bn = self.phonetic.transliterate_word(tail);
            for c in &mut cands {
                c.text.push_str(&tail_bn);
            }
        }
        (cands, hints)
    }

    fn build_phonetic_output(&self, committed: Option<String>) -> SessionOutput {
        if self.roman_buf.is_empty() {
            return SessionOutput {
                language: self.language,
                committed,
                ..Default::default()
            };
        }

        let preedit = self.phonetic.transliterate_word(&self.roman_buf);
        if self.config.typing_mode == TypingMode::PhoneticOnly {
            return SessionOutput {
                language: self.language,
                preedit,
                roman_buffer: self.roman_buf.clone(),
                committed,
                ..Default::default()
            };
        }

        let smart = self.config.typing_mode == TypingMode::Smart;
        let (candidates, mut sign_hints) = self.phonetic_candidates();
        if !self.config.show_sign_and_conjunct_hints {
            sign_hints.clear();
        } else if self.config.compact_popup {
            // Compact popup: no conjunct table, but keep the tips for words on offer (হেঁসে: he^se)
            sign_hints.retain(|h| candidates.iter().any(|c| c.text == h.result_bangla));
        }

        // Use Candidate #1 as the live preedit in Smart mode, OR whenever Candidate #1 is a
        // Snippet (`;salam` -> `আসসালামু আলাইকুম`) or Inline English escape (`/office` -> `office`).
        let first_is_special = candidates
            .first()
            .map(|c| {
                c.kind == crate::suggest::CandidateKind::Snippet
                    || c.kind == crate::suggest::CandidateKind::English
            })
            .unwrap_or(false);

        // With the full dictionary, candidate #1 is exactly what Space will commit, so show it inline.
        let has_dict = self.suggestions.dictionary().is_some();
        let active_preedit = if smart || first_is_special || has_dict {
            candidates
                .first()
                .map(|c| c.text.clone())
                .unwrap_or(preedit)
        } else {
            preedit
        };

        SessionOutput {
            language: self.language,
            preedit: active_preedit,
            roman_buffer: self.roman_buf.clone(),
            candidates,
            selected_index: self.selected_idx,
            sign_hints,
            next_word_predictions: Vec::new(),
            committed,
            delete_before: 0,
        }
    }
}

/// Roman text that looks like a password, code or address rather than a word: letters mixed with
/// digits (`abc123`, `2ta` too, which is fine to not learn), or symbols no Bangla word is typed with.
/// Ekushe's own marks (`,,` `^` `~` `` ` `` `+` `:` `.` `t/`) and quotes do not count.
/// Command-line apps, by Windows program name or macOS bundle ID (in small letters).
pub fn is_terminal_app(app: &str) -> bool {
    matches!(
        app,
        "windowsterminal.exe"
            | "wt.exe"
            | "openconsole.exe"
            | "conhost.exe"
            | "cmd.exe"
            | "powershell.exe"
            | "pwsh.exe"
            | "mintty.exe"
            | "putty.exe"
            | "kitty.exe"
            | "wezterm-gui.exe"
            | "alacritty.exe"
            | "conemu.exe"
            | "conemu64.exe"
            | "com.apple.terminal"
            | "com.googlecode.iterm2"
            | "dev.warp.warp-stable"
            | "net.kovidgoyal.kitty"
            | "org.alacritty"
            | "com.github.wez.wezterm"
            | "com.mitchellh.ghostty"
            | "co.zeit.hyper"
    )
}

pub fn looks_secret(roman: &str) -> bool {
    let letters = roman.chars().any(|c| c.is_ascii_alphabetic());
    let digits = roman.chars().any(|c| c.is_ascii_digit());
    let symbols = roman.chars().any(|c| "@#$%&*=_|<>{}[]\\'!?;".contains(c));
    (letters && digits) || symbols || roman.chars().count() > 40
}

/// Split `gai,` into (`gai`, `,`). Only for a plain word (letters and digits, starting with a
/// letter) followed by `. , ? ! ; ) "`; `,,` (hasanta), `...`, snippets (`;x`), English escapes
/// (`/x`) and emoji codes (`:x:`) are left whole.
fn split_trailing_punctuation(roman: &str) -> (&str, &str) {
    let stem = roman.trim_end_matches(['.', ',', '?', '!', ';', ')', '"']);
    let tail = &roman[stem.len()..];
    let plain_word = stem.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && stem.chars().all(|c| c.is_ascii_alphanumeric());
    if tail.is_empty() || !plain_word || tail.contains(",,") {
        return (roman, "");
    }
    (stem, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banglish_is_told_apart_from_english_typos() {
        // needs the dictionary (EKUSHE_DICT, or data/dictionary from the workspace)
        if crate::dict::Dictionary::global().is_none() {
            return;
        }
        let ek = Ekushe::new(Config::default());
        for w in [
            "ami",
            "tomake",
            "bhalobashi",
            "kemon",
            "acho",
            "valo",
            "kothay",
            "jabo",
            "korcho",
        ] {
            assert!(ek.is_banglish(w), "{w} is Banglish");
        }
        for w in [
            "teh",
            "recieve",
            "thier",
            "becuase",
            "definately",
            "goverment",
            "tommorow",
            "freind",
            "makhan",
        ] {
            assert!(!ek.is_banglish(w), "{w} is not Banglish");
        }
    }

    #[test]
    fn date_shortcuts_type_today_in_bangla() {
        let mut ek = Ekushe::new(Config::default());
        ek.suggestions_engine_mut()
            .set_fixed_time(Some(crate::dates::LocalTime {
                year: 2026,
                month: 10,
                day: 24,
                weekday: 6,
                hour: 19,
                minute: 50,
            }));
        assert_eq!(type_word(&mut ek, ";aj"), "২৪ অক্টোবর ২০২৬ ");
        assert_eq!(type_word(&mut ek, ";bongabdo"), "৮ কার্তিক ১৪৩৩ বঙ্গাব্দ ");
        assert_eq!(type_word(&mut ek, ";bar"), "শনিবার ");
        assert_eq!(type_word(&mut ek, ";somoy"), "সন্ধ্যা ৭:৫০ ");
        // Other snippets still work, and a user's own `;aj` wins
        assert_eq!(type_word(&mut ek, ";barik"), "আল্লাহুম্মা বারিক লাহু ");
        ek.suggestions_engine_mut().set_snippet(";aj", "আজকে");
        assert_eq!(type_word(&mut ek, ";aj"), "আজকে ");
    }

    #[test]
    fn newer_versions_are_recognised() {
        assert!(is_newer_version("v0.1.0-beta.16", "0.1.0-beta.15"));
        assert!(is_newer_version("v0.1.0-beta.10", "0.1.0-beta.9"));
        assert!(is_newer_version("v0.1.0", "0.1.0-beta.15"));
        assert!(is_newer_version("v1.0.0", "0.9.9"));
        assert!(!is_newer_version("v0.1.0-beta.15", "0.1.0-beta.15"));
        assert!(!is_newer_version("v0.1.0-beta.14", "0.1.0-beta.15"));
        assert!(!is_newer_version("v0.1.0-beta.20", "0.1.0"));
        assert!(!is_newer_version("nightly", "0.1.0"));
    }

    fn type_word(ek: &mut Ekushe, w: &str) -> String {
        for c in w.chars() {
            ek.push_char(c, false);
        }
        ek.push_char(' ', false).committed.unwrap_or_default()
    }

    #[test]
    fn a_wrong_word_can_be_corrected_and_is_learned() {
        let mut ek = Ekushe::new(Config::default());
        if ek.language() != InputLanguage::Bangla {
            ek.toggle_language();
        }
        let placed = type_word(&mut ek, "kemon").trim_end().to_string();
        type_word(&mut ek, "ase");
        let recent = ek.recent_words();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].roman, "ase");
        assert_eq!(recent[1].word, placed);
        assert_eq!(recent[0].prev.as_deref(), Some(placed.as_str()));
        // the user says the word for `ase` should have been আসে
        let wrong = recent[0].word.clone();
        let target = if wrong == "আসে" {
            "আছে"
        } else {
            "আসে"
        };
        let fix = ek
            .correct_recent_word("ase", &wrong, target)
            .expect("a recent word");
        assert_eq!(fix.right, target);
        assert!(fix.tsv_line().starts_with("ase\t"));
        assert_eq!(
            ek.recent_words().len(),
            1,
            "the corrected word leaves the recent list"
        );
        let (c, _) = ek.suggest_roman("ase");
        assert_eq!(
            c.first().map(|c| c.text.as_str()),
            Some(target),
            "learned at once"
        );
        // nothing to correct for a word that was not placed
        assert!(ek.correct_recent_word("xyz", "ক", "খ").is_none());
    }

    #[test]
    fn taking_the_first_suggestion_is_not_learned_and_private_fields_are_not_remembered() {
        let mut ek = Ekushe::new(Config::default());
        if ek.language() != InputLanguage::Bangla {
            ek.toggle_language();
        }
        type_word(&mut ek, "ami");
        assert!(
            ek.suggestions_engine().user_dict_tsv().trim().is_empty(),
            "nothing learned from a plain Space"
        );
        ek.set_private_field(true);
        type_word(&mut ek, "tumi");
        assert_eq!(ek.recent_words().len(), 1, "a private field keeps nothing");
        ek.set_private_field(false);
        for _ in 0..(RECENT_WORDS + 3) {
            type_word(&mut ek, "ami");
        }
        assert_eq!(ek.recent_words().len(), RECENT_WORDS);
        ek.clear_recent_words();
        assert!(ek.recent_words().is_empty());
    }

    #[test]
    fn new_snippets_expand() {
        let mut ek = Ekushe::new(Config::default());
        if ek.language() != InputLanguage::Bangla {
            ek.toggle_language();
        }
        for (trig, want) in [
            (";innalillah", "ইন্না লিল্লাহি ওয়া ইন্না ইলাইহি রাজিউন"),
            (";barik", "আল্লাহুম্মা বারিক লাহু"),
            (";brb", "এখুনি আসছি"),
        ] {
            let (c, _) = ek.suggest_roman(trig);
            assert_eq!(c.first().map(|c| c.text.as_str()), Some(want), "{trig}");
        }
    }

    #[test]
    fn report_url_carries_the_build_details_only() {
        let url = report_url("Windows 11 (build 26100)", &Config::default());
        assert!(
            url.starts_with("https://ekushenext.com/report/?v=0.1.0"),
            "{url}"
        );
        assert!(
            url.contains(
                "&os=Windows%2011%20%28build%2026100%29&layout=AvroPhonetic&mode=Smart&from=app"
            ),
            "{url}"
        );
        assert!(release().starts_with('v'));
    }

    #[test]
    fn terminals_follow_the_english_in_terminals_setting() {
        let mut ek = Ekushe::new(Config::default());
        if ek.language() != InputLanguage::Bangla {
            ek.toggle_language();
        }
        ek.on_app_focus("winword.exe");
        assert_eq!(
            ek.on_app_focus("WindowsTerminal.exe"),
            InputLanguage::English
        );
        assert_eq!(
            ek.on_app_focus("com.apple.Terminal"),
            InputLanguage::English
        );
        // Switching to Bangla in a terminal is remembered there
        ek.toggle_language();
        ek.on_app_focus("winword.exe");
        assert_eq!(ek.on_app_focus("com.apple.Terminal"), InputLanguage::Bangla);

        // Off: a terminal is like any other app (the language stays as it was)
        let mut ek = Ekushe::new(Config {
            auto_english_in_terminals: false,
            ..Config::default()
        });
        if ek.language() != InputLanguage::Bangla {
            ek.toggle_language();
        }
        ek.on_app_focus("winword.exe");
        assert_eq!(ek.on_app_focus("cmd.exe"), InputLanguage::Bangla);
    }

    #[test]
    fn selected_text_is_converted() {
        let ek = Ekushe::new(Config::default());
        let (out, kind) = ek.convert_selection("amar bangla").unwrap();
        assert_eq!(kind, convert::SelectionKind::BanglishToBangla);
        assert_eq!(out, "আমার বাংলা");
        let (out, kind) = ek.convert_selection("Avgvi †mvbvi evsjv").unwrap();
        assert_eq!(kind, convert::SelectionKind::BijoyToUnicode);
        assert_eq!(
            convert::normalize_bangla_unicode(&out),
            convert::normalize_bangla_unicode("আমার সোনার বাংলা")
        );
        assert!(ek.convert_selection("আমার সোনার বাংলা").is_none());
        // Spacing and line breaks stay as they were
        assert_eq!(
            ek.convert_selection("ami  tumi\nami").unwrap().0,
            "আমি  তুমি\nআমি"
        );
    }

    #[test]
    fn test_fixed_layouts_commit_their_text() {
        for layout in [
            LayoutId::Probhat,
            LayoutId::NationalJatiya,
            LayoutId::Bornona,
            LayoutId::MunirOptima,
        ] {
            for commit_via_select in [true, false] {
                let mut ek = Ekushe::new(Config {
                    active_layout: layout,
                    ..Config::default()
                });
                let pre = ek.push_char('k', false).preedit;
                assert!(!pre.is_empty(), "{layout:?} composes nothing for k");
                let out = if commit_via_select {
                    ek.select_candidate(0, Some(' '))
                } else {
                    ek.commit_default(Some(' '))
                };
                assert_eq!(
                    out.committed,
                    Some(format!("{pre} ")),
                    "{layout:?} lost its text on commit"
                );
            }
        }
    }

    #[test]
    fn test_backspace_undo_and_flip_lengths() {
        let mut ek = Ekushe::new(Config {
            typing_mode: TypingMode::PhoneticFirst,
            ..Config::default()
        });
        for c in "swadhin".chars() {
            ek.push_char(c, false);
        }
        let committed = ek.push_char(' ', false).committed.unwrap();
        let undo = ek.backspace();
        assert_eq!(undo.committed.as_deref(), Some("swadhin"));
        assert_eq!(undo.delete_before, committed.encode_utf16().count());

        // After Enter there is no undo or flip (the app inserted the line break itself)
        let mut ek = Ekushe::new(Config {
            typing_mode: TypingMode::PhoneticFirst,
            ..Config::default()
        });
        for c in "ami".chars() {
            ek.push_char(c, false);
        }
        ek.select_candidate(0, None);
        assert!(ek.flip_last_committed_word().is_none());
        assert_eq!(ek.backspace().delete_before, 0);
    }

    #[test]
    fn test_per_app_language_memory() {
        let mut ek = Ekushe::default();
        assert_eq!(ek.on_app_focus("WINWORD.EXE"), InputLanguage::Bangla);
        assert_eq!(ek.on_app_focus("cmd.exe"), InputLanguage::English); // configured default
        ek.toggle_language(); // user wants Bangla in this terminal
        assert_eq!(ek.on_app_focus("winword.exe"), InputLanguage::Bangla); // Word keeps Bangla
        ek.toggle_language(); // English in Word
        assert_eq!(ek.on_app_focus("cmd.exe"), InputLanguage::Bangla); // the terminal remembers Bangla
        assert_eq!(ek.on_app_focus("winword.exe"), InputLanguage::English); // Word remembers English
    }

    #[test]
    fn test_reset_forgets_undo_and_context() {
        let mut ek = Ekushe::new(Config {
            typing_mode: TypingMode::PhoneticFirst,
            ..Config::default()
        });
        for c in "ami ".chars() {
            ek.push_char(c, false);
        }
        ek.reset_composition(); // e.g. the user clicked elsewhere
        assert!(ek.flip_last_committed_word().is_none());
        assert_eq!(ek.backspace().committed, None);
    }

    #[test]
    fn test_double_tap_shift_switch() {
        let mut ek = Ekushe::default();
        assert_eq!(ek.language(), InputLanguage::Bangla);

        // First Shift tap at t=1000ms -> does not toggle yet
        assert_eq!(ek.on_shift_tap(1000), None);
        // Second Shift tap at t=1220ms (220ms later) -> toggles to English!
        assert_eq!(ek.on_shift_tap(1220), Some(InputLanguage::English));
        assert_eq!(ek.language(), InputLanguage::English);

        // Tap again twice -> back to Bangla
        assert_eq!(ek.on_shift_tap(2000), None);
        assert_eq!(ek.on_shift_tap(2250), Some(InputLanguage::Bangla));
    }

    #[test]
    fn test_typing_sign_hints_and_undo_conversion() {
        // Phonetic-First keeps the literal spelling inline, which this test inspects letter by letter
        let mut ek = Ekushe::new(Config {
            typing_mode: TypingMode::PhoneticFirst,
            ..Config::default()
        });

        // Type 's' -> preedit "স", and sign_hints includes "স্ব (sw)"
        let out_s = ek.push_char('s', false);
        assert_eq!(out_s.preedit, "স");
        assert!(out_s.sign_hints.iter().any(|h| h.result_bangla == "স্ব"));

        // Type 'w', 'a', 'd', 'h', 'i', 'n' -> "স্বাধীন"
        for ch in ['w', 'a', 'd', 'h', 'i', 'n'] {
            ek.push_char(ch, false);
        }
        let committed = ek.push_char(' ', false);
        assert_eq!(committed.committed.as_deref(), Some("স্বাধীন "));
        // Next-word predictions or undo works:
        let undone = ek.backspace();
        assert_eq!(undone.committed.as_deref(), Some("swadhin"));
    }
}
