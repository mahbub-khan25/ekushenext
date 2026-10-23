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

pub mod config;
pub mod conjuncts;
pub mod convert;
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
use crate::suggest::{Candidate, SuggestionEngine};

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
            .or_else(|| self.config.per_app_defaults.get(&key).copied());
        if let Some(lang) = next {
            if lang != self.language {
                self.reset_composition();
                self.language = lang;
            }
        }
        self.current_app = Some(key);
        self.language
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
            // (letters mixed with digits, or symbols such as @ # $ %)
            if !self.private_field && !looks_secret(stem) {
                self.suggestions.record_user_choice(stem, chosen_stem);
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
        marks::honor_marks(stem, &literal, &mut cands);
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
/// Ekushe's own marks (`,,` `^` `~` `` ` `` `+` `:` `.`) do not count.
pub fn looks_secret(roman: &str) -> bool {
    let letters = roman.chars().any(|c| c.is_ascii_alphabetic());
    let digits = roman.chars().any(|c| c.is_ascii_digit());
    let symbols = roman
        .chars()
        .any(|c| "@#$%&*=_|<>{}[]\\/'\"!?;".contains(c));
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
