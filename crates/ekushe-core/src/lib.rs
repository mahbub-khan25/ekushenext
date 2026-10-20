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
pub mod dict;
pub mod convert;
pub mod layout;
pub mod phonetic;
pub mod roman;
pub mod spell;
pub mod suggest;

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
}

impl Default for Ekushe {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Ekushe {
    pub fn new(config: Config) -> Self {
        let phonetic = PhoneticEngine::with_overrides(
            config.phonetic_overrides.clone(),
            config.bangla_digits,
        );
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
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.phonetic = PhoneticEngine::with_overrides(
            config.phonetic_overrides.clone(),
            config.bangla_digits,
        );
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

    /// Apply per-application automatic language switching when window focus changes.
    pub fn on_app_focus(&mut self, executable_name: &str) -> InputLanguage {
        let key = executable_name.to_lowercase();
        if let Some(&lang) = self.config.per_app_defaults.get(&key) {
            self.language = lang;
        }
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

        // Undo conversion: if user presses Backspace right after a word commit, restore the Roman text
        if let Some((roman, _committed_bangla)) = self.last_commit_undo.take() {
            return SessionOutput {
                language: self.language,
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
        let smart = self.config.typing_mode == TypingMode::Smart;
        let (cands, _) = self.suggestions.suggest_with_context(
            &self.roman_buf,
            &self.phonetic,
            smart,
            self.config.enable_emoji_suggestions,
            self.config.max_candidates,
            self.prev_word.as_deref(),
        );

        let chosen = cands
            .get(index)
            .map(|c| c.text.clone())
            .unwrap_or_else(|| self.phonetic.transliterate_word(&self.roman_buf));

        let roman = std::mem::take(&mut self.roman_buf);
        if !roman.is_empty() {
            self.suggestions.record_user_choice(&roman, &chosen);
            self.last_commit_undo = Some((roman, chosen.clone()));
        }

        let predictions = if self.config.enable_next_word_prediction {
            self.suggestions.predict_next(&chosen)
        } else {
            Vec::new()
        };

        // Context for the next word: only across a plain space between Bangla words
        let is_bangla = chosen.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c));
        let ends_clean = chosen.chars().last().map(|c| ('\u{0980}'..='\u{09FF}').contains(&c) && c != '।').unwrap_or(false);
        self.prev_word = if trailing == Some(' ') && is_bangla && ends_clean { Some(chosen.clone()) } else { None };

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
        let (candidates, mut sign_hints) = self.suggestions.suggest_with_context(
            &self.roman_buf,
            &self.phonetic,
            smart,
            self.config.enable_emoji_suggestions,
            self.config.max_candidates,
            self.prev_word.as_deref(),
        );
        if !self.config.show_sign_and_conjunct_hints || self.config.compact_popup {
            sign_hints.clear();
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_layouts_commit_their_text() {
        for layout in [LayoutId::Probhat, LayoutId::NationalJatiya, LayoutId::Bornona, LayoutId::MunirOptima] {
            for commit_via_select in [true, false] {
                let mut ek = Ekushe::new(Config { active_layout: layout, ..Config::default() });
                let pre = ek.push_char('k', false).preedit;
                assert!(!pre.is_empty(), "{layout:?} composes nothing for k");
                let out = if commit_via_select { ek.select_candidate(0, Some(' ')) } else { ek.commit_default(Some(' ')) };
                assert_eq!(out.committed, Some(format!("{pre} ")), "{layout:?} lost its text on commit");
            }
        }
    }

    #[test]
    fn test_reset_forgets_undo_and_context() {
        let mut ek = Ekushe::new(Config { typing_mode: TypingMode::PhoneticFirst, ..Config::default() });
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
        let mut ek = Ekushe::new(Config { typing_mode: TypingMode::PhoneticFirst, ..Config::default() });

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
