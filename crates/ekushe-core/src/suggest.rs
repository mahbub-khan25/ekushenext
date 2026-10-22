//! Smart Suggestion, Phonetic Variant Expansion, Autocorrect, Emoji, Snippet,
//! Next-Word Prediction, and User Learning Engine.
//!
//! How it helps both beginners and advanced users:
//! - Forgiving phonetic variants: even if a beginner types lowercase `s` instead of `sh`/`Sh`,
//!   or `n` instead of `N`, or `i` instead of `I`, or omits inherent `o` (`bngla` / `bangla` / `kmn`),
//!   the suggestion engine matches the right dictionary words (`বাংলা`, `কেমন`, `মানুষ`, `স্বাধীনতা`).
//! - English-word awareness: typing `facebook`, `download`, `computer`, `mobile`, `school` suggests
//!   both the Bangla transliteration (`ফেসবুক`, `ডাউনলোড`, `কম্পিউটার`) AND the English word.
//! - Emoji suggestions: typing `kanna` / `কান্না` suggests `😢`, `bangladesh` / `বাংলাদেশ` suggests `🇧🇩`,
//!   `bhalobasha` / `ভালোবাসা` suggests `❤️`, `hashi` / `হাসি` suggests `😊`.
//! - Text expansion snippets: `/sal` -> `আসসালামু আলাইকুম`, `/bdt` -> `৳`, `/ekushe` -> `অমর একুশে`.
//! - User learning: remembers which candidate the user picked for a given roman input.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::conjuncts::{contextual_sign_hints, SignHint};
use crate::dict::{Dictionary, RankParams, WordClass};
use crate::phonetic::PhoneticEngine;
use crate::spell::{bangla_weighted_distance, SpellChecker};

/// Type of a candidate shown in the popup window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateKind {
    /// Exact literal phonetic output
    Phonetic,
    /// Autocorrect or high-confidence dictionary match
    Dictionary,
    /// User-learned preference
    UserLearned,
    /// Spell-checker correction for a close confusable
    SpellCorrection,
    /// Emoji associated with the Bangla word
    Emoji,
    /// Text expansion snippet
    Snippet,
    /// Preserved English word
    English,
}

/// A single candidate item returned to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub text: String,
    pub kind: CandidateKind,
    /// Optional small annotation (e.g., `"ক্ষ = ক+্+ষ"` or `"Autocorrect"` or `"Spell fix"`)
    pub annotation: Option<String>,
}

/// Complete suggestion & learning engine.
#[derive(Debug, Clone)]
pub struct SuggestionEngine {
    /// Bangla word -> frequency score
    dictionary: HashMap<String, u32>,
    /// Normalized Roman skeleton -> list of Bangla words
    roman_index: HashMap<String, Vec<String>>,
    /// Roman token -> Autocorrect Bangla replacement (including English-loanwords in Bangla)
    autocorrect: HashMap<String, String>,
    /// Bangla word or Roman token -> Emoji list
    emojis: HashMap<String, Vec<&'static str>>,
    /// User / builtin snippets (e.g. `/sal` -> `আসসালামু আলাইকুম`)
    snippets: HashMap<String, String>,
    /// Next-word bigrams: previous Bangla word -> top following words
    bigrams: HashMap<String, Vec<String>>,
    /// User-learned mappings: roman token -> (bangla_word, selection_count)
    user_choices: HashMap<String, HashMap<String, u32>>,
    /// Spell checker instance sharing the dictionary
    spell: SpellChecker,
    /// Ekushe's full dictionary (`ekushe.dict`); when present it drives ranking
    dict: Option<&'static Dictionary>,
    /// Ranking weights for the dictionary
    rank_params: RankParams,
    /// Increases whenever the user's learned choices change (hosts save when it moves)
    learned_version: u64,
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SuggestionEngine {
    pub fn new() -> Self {
        let mut dictionary = HashMap::new();
        let mut roman_index: HashMap<String, Vec<String>> = HashMap::new();

        for &(roman, bangla, freq) in BUILTIN_LEXICON {
            dictionary.insert(bangla.to_string(), freq);
            let skel = roman_skeleton(roman);
            roman_index
                .entry(skel)
                .or_default()
                .push(bangla.to_string());
            roman_index
                .entry(roman.to_lowercase())
                .or_default()
                .push(bangla.to_string());
        }

        // Also index all words from the Connected Letters & Signs Guide
        for entry in crate::conjuncts::SIGN_AND_CONJUNCT_GUIDE {
            for w in entry.words {
                dictionary.entry(w.bangla.to_string()).or_insert(8_200);
                for r_part in w.roman.split('/') {
                    let r_clean = r_part.trim();
                    if !r_clean.is_empty() {
                        let skel = roman_skeleton(r_clean);
                        roman_index
                            .entry(skel)
                            .or_default()
                            .push(w.bangla.to_string());
                        roman_index
                            .entry(r_clean.to_lowercase())
                            .or_default()
                            .push(w.bangla.to_string());
                    }
                }
            }
        }

        let mut autocorrect = HashMap::new();
        for &(r, b) in BUILTIN_AUTOCORRECT {
            autocorrect.insert(r.to_string(), b.to_string());
        }

        let mut emojis: HashMap<String, Vec<&'static str>> = HashMap::new();
        for &(key, em_slice) in BUILTIN_EMOJIS {
            emojis.insert(key.to_string(), em_slice.to_vec());
        }

        let mut snippets = HashMap::new();
        for &(trig, exp) in BUILTIN_SNIPPETS {
            snippets.insert(trig.to_string(), exp.to_string());
        }

        let mut bigrams = HashMap::new();
        for &(prev, nexts) in BUILTIN_BIGRAMS {
            bigrams.insert(
                prev.to_string(),
                nexts.iter().map(|s| (*s).to_string()).collect(),
            );
        }

        let mut spell = SpellChecker::new(dictionary.clone());
        let dict = Dictionary::global();
        spell.set_dictionary(dict);

        Self {
            dictionary,
            roman_index,
            autocorrect,
            emojis,
            snippets,
            bigrams,
            user_choices: HashMap::new(),
            spell,
            dict,
            rank_params: RankParams::default(),
            learned_version: 0,
        }
    }

    /// Attach (or detach) the full dictionary. `new()` attaches [`Dictionary::global`] automatically.
    pub fn set_dictionary(&mut self, dict: Option<&'static Dictionary>) {
        self.dict = dict;
        self.spell.set_dictionary(dict);
    }

    pub fn dictionary(&self) -> Option<&'static Dictionary> {
        self.dict
    }

    pub fn set_rank_params(&mut self, p: RankParams) {
        self.rank_params = p;
    }

    pub fn spell_checker(&self) -> &SpellChecker {
        &self.spell
    }

    pub fn spell_checker_mut(&mut self) -> &mut SpellChecker {
        &mut self.spell
    }

    pub fn set_autocorrect(&mut self, roman: impl Into<String>, bangla: impl Into<String>) {
        self.autocorrect.insert(roman.into(), bangla.into());
    }

    pub fn set_snippet(&mut self, trigger: impl Into<String>, expansion: impl Into<String>) {
        self.snippets.insert(trigger.into(), expansion.into());
    }

    pub fn snippets(&self) -> &HashMap<String, String> {
        &self.snippets
    }

    pub fn autocorrect_entries(&self) -> &HashMap<String, String> {
        &self.autocorrect
    }

    /// Record that the user selected `chosen_bangla` for `roman_input`, boosting it in future rankings
    /// and adding it to the spell-checker user dictionary.
    pub fn record_user_choice(&mut self, roman_input: &str, chosen_bangla: &str) {
        if roman_input.is_empty() || chosen_bangla.is_empty() {
            return;
        }
        let entry = self
            .user_choices
            .entry(roman_input.to_lowercase())
            .or_default();
        *entry.entry(chosen_bangla.to_string()).or_insert(0) += 1;
        self.learned_version += 1;
        self.dictionary
            .entry(chosen_bangla.to_string())
            .or_insert(5_000);
        self.spell.add_user_word(chosen_bangla.to_string());
    }

    /// Version counter of the learned choices; changes whenever something new is learned.
    pub fn learned_version(&self) -> u64 {
        self.learned_version
    }

    /// The learned choices as TSV (`roman \t bangla \t count`, sorted), ready to write to disk.
    pub fn user_dict_tsv(&self) -> String {
        let mut rows: Vec<(&String, &String, u32)> = Vec::new();
        for (roman, map) in &self.user_choices {
            for (bangla, count) in map {
                rows.push((roman, bangla, *count));
            }
        }
        rows.sort();
        let mut lines = String::new();
        for (roman, bangla, count) in rows {
            lines.push_str(&format!("{roman}\t{bangla}\t{count}\n"));
        }
        lines
    }

    /// Save user learned dictionary to a TSV file (`roman \t bangla \t count`). Writes a temporary file
    /// first and renames it, so a crash never leaves a half-written dictionary.
    pub fn save_user_dict(&self, path: &Path) -> std::io::Result<()> {
        write_atomic(path, &self.user_dict_tsv())
    }

    /// Load user learned dictionary from a TSV file.
    pub fn load_user_dict(&mut self, path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let roman = parts[0].trim();
                let bangla = parts[1].trim();
                let count: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
                if !roman.is_empty() && !bangla.is_empty() {
                    self.user_choices
                        .entry(roman.to_lowercase())
                        .or_default()
                        .insert(bangla.to_string(), count);
                    self.dictionary.entry(bangla.to_string()).or_insert(5_000);
                    self.spell.add_user_word(bangla.to_string());
                }
            }
        }
        Ok(())
    }

    /// Compute ranked candidates + contextual sign/conjunct hints for `roman_input`.
    pub fn suggest(
        &self,
        roman_input: &str,
        phonetic_engine: &PhoneticEngine,
        smart_mode: bool,
        include_emojis: bool,
        max_candidates: usize,
    ) -> (Vec<Candidate>, Vec<SignHint>) {
        self.suggest_with_context(
            roman_input,
            phonetic_engine,
            smart_mode,
            include_emojis,
            max_candidates,
            None,
        )
    }

    /// Like [`suggest`](Self::suggest), with the previously committed word as context.
    pub fn suggest_with_context(
        &self,
        roman_input: &str,
        phonetic_engine: &PhoneticEngine,
        smart_mode: bool,
        include_emojis: bool,
        max_candidates: usize,
        prev_word: Option<&str>,
    ) -> (Vec<Candidate>, Vec<SignHint>) {
        if roman_input.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let literal_bangla = phonetic_engine.transliterate_word(roman_input);
        let hints = contextual_sign_hints(roman_input, &literal_bangla);

        let mut candidates: Vec<Candidate> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let push_unique = |list: &mut Vec<Candidate>,
                           seen: &mut std::collections::HashSet<String>,
                           cand: Candidate| {
            if !cand.text.is_empty() && seen.insert(cand.text.clone()) {
                list.push(cand);
            }
        };

        // 1. Snippet trigger (exact or prefix on `;` / `/`, e.g., `;salam`, `/sal`, `;tk`, `;bd`)
        if let Some(expanded) = self.snippets.get(roman_input) {
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: expanded.clone(),
                    kind: CandidateKind::Snippet,
                    annotation: Some("Snippet".to_string()),
                },
            );
        } else if (roman_input.starts_with(';') || roman_input.starts_with('/'))
            && roman_input.len() >= 2
        {
            let mut prefix_snips: Vec<(&String, &String)> = self
                .snippets
                .iter()
                .filter(|(k, _)| k.starts_with(roman_input))
                .collect();
            prefix_snips.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(b.0)));
            for (_, exp) in prefix_snips {
                push_unique(
                    &mut candidates,
                    &mut seen,
                    Candidate {
                        text: exp.clone(),
                        kind: CandidateKind::Snippet,
                        annotation: Some("Snippet".to_string()),
                    },
                );
            }
        }

        // 1b. Inline Mix-English Escape (`/word` or `` `word ``):
        // When typing in Bangla mode, starting a word with `/` or `` ` `` (e.g., `/office`, `` `email ``)
        // puts the clean English word first without switching the keyboard!
        if (roman_input.starts_with('/') || roman_input.starts_with('`')) && roman_input.len() >= 2
        {
            let eng_word = &roman_input[1..];
            if !eng_word.is_empty() && candidates.is_empty() {
                push_unique(
                    &mut candidates,
                    &mut seen,
                    Candidate {
                        text: eng_word.to_string(),
                        kind: CandidateKind::English,
                        annotation: Some("EN".to_string()),
                    },
                );
                let bn_word = phonetic_engine.transliterate_word(eng_word);
                push_unique(
                    &mut candidates,
                    &mut seen,
                    Candidate {
                        text: bn_word,
                        kind: CandidateKind::Phonetic,
                        annotation: Some("বাং".to_string()),
                    },
                );
                return (candidates, Vec::new());
            }
        }

        // 1c. Number Mode: In Ekushe Bangla mode, digits 0-9 produce Bengali ০-৯ by default,
        // while also offering the English digits (0-9) and Taka currency (৳) right in the candidate bar!
        if roman_input.chars().all(|c| c.is_ascii_digit()) {
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: literal_bangla.clone(),
                    kind: CandidateKind::Phonetic,
                    annotation: Some("বাং".to_string()),
                },
            );
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: roman_input.to_string(),
                    kind: CandidateKind::English,
                    annotation: Some("EN".to_string()),
                },
            );
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: format!("৳{literal_bangla}"),
                    kind: CandidateKind::Snippet,
                    annotation: Some("টাকা".to_string()),
                },
            );
            return (candidates, Vec::new());
        }

        // 2. User-learned choices for this exact roman input
        let roman_lc = roman_input.to_lowercase();
        if let Some(learned_map) = self.user_choices.get(&roman_lc) {
            let mut pairs: Vec<(&String, &u32)> = learned_map.iter().collect();
            pairs.sort_by(|a, b| b.1.cmp(a.1));
            for (w, _) in pairs {
                push_unique(
                    &mut candidates,
                    &mut seen,
                    Candidate {
                        text: w.clone(),
                        kind: CandidateKind::UserLearned,
                        annotation: Some("Learned".to_string()),
                    },
                );
            }
        }

        // Full dictionary path: Ekushe's own ranking replaces the small built-in heuristics below.
        if let Some(dict) = self.dict {
            self.dictionary_candidates(
                dict,
                roman_input,
                &roman_lc,
                &literal_bangla,
                smart_mode,
                include_emojis,
                max_candidates,
                prev_word,
                &mut candidates,
                &mut seen,
            );
            return (candidates, hints);
        }

        // 3. In Phonetic-First mode (default), literal phonetic comes right after user-learned
        if !smart_mode {
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: literal_bangla.clone(),
                    kind: CandidateKind::Phonetic,
                    annotation: None,
                },
            );
        }

        // 3b. Zero-Memorization `ri` -> `ৃ` (ঋ-কার) Assistant:
        // Whenever the user types `ri` after a consonant (e.g., `krishi`, `srishti`, `smriti`, `nripo`),
        // automatically offer the `rri` (ঋ-কার `ৃ`) variant right in the top candidates!
        if roman_lc.len() >= 2
            && roman_lc.contains("ri")
            && !roman_lc.contains("rri")
            && !roman_lc.starts_with("ri")
        {
            let rri_roman = roman_input.replacen("ri", "rri", 1);
            let rri_bangla = phonetic_engine.transliterate_word(&rri_roman);
            if rri_bangla != literal_bangla {
                push_unique(
                    &mut candidates,
                    &mut seen,
                    Candidate {
                        text: rri_bangla,
                        kind: CandidateKind::Dictionary,
                        annotation: Some("ঋ-কার (ৃ)".to_string()),
                    },
                );
            }
        }

        // 3c. `ch` = `ছ` (primary) and also offer `চ` (`c`) as a quick candidate:
        // In Ekushe, `c` = `চ` and `ch` (or `chh`) = `ছ` (e.g. `chati` -> `ছাতি`), matching `k/kh`, `g/gh`, `j/jh`, `t/th`, `d/dh`, `p/ph`, `b/bh`.
        // Also offer the `চ` (`c`) variant in the candidate bar if someone typed `ch` intending `চ` (e.g. `cha` -> `1. ছা`, `2. চা`)!
        if roman_lc.contains("ch") && !roman_lc.contains("chh") && !roman_lc.contains("cch") {
            let c_roman = roman_input.replacen("ch", "c", 1);
            let c_bangla = phonetic_engine.transliterate_word(&c_roman);
            if c_bangla != literal_bangla {
                push_unique(
                    &mut candidates,
                    &mut seen,
                    Candidate {
                        text: c_bangla,
                        kind: CandidateKind::Dictionary,
                        annotation: Some("চ (c)".to_string()),
                    },
                );
            }
        }

        // 4. Autocorrect / English-loanword table (e.g., `bangladesh` -> `বাংলাদেশ`, `facebook` -> `ফেসবুক`)
        if let Some(ac) = self
            .autocorrect
            .get(roman_input)
            .or_else(|| self.autocorrect.get(&roman_lc))
        {
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: ac.clone(),
                    kind: CandidateKind::Dictionary,
                    annotation: Some("Smart".to_string()),
                },
            );
        }

        // 5. Forgiving skeleton & prefix lookup in the dictionary
        let skel = roman_skeleton(roman_input);
        let mut dict_matches: Vec<(u32, String)> = Vec::new();

        if let Some(exact_list) = self.roman_index.get(&roman_lc) {
            for w in exact_list {
                let freq = self.dictionary.get(w).copied().unwrap_or(1_000) + 50_000;
                dict_matches.push((freq, w.clone()));
            }
        }
        if let Some(skel_list) = self.roman_index.get(&skel) {
            for w in skel_list {
                let freq = self.dictionary.get(w).copied().unwrap_or(1_000) + 20_000;
                dict_matches.push((freq, w.clone()));
            }
        }
        // Prefix matches on roman index for autocomplete (even on 1st letter so the candidate bar shows rich word pills!)
        if !roman_lc.is_empty() {
            let max_extra = if roman_lc.len() == 1 { 5 } else { 4 };
            for (k, words) in &self.roman_index {
                if k.starts_with(&roman_lc) && k.len() <= roman_lc.len() + max_extra {
                    for w in words {
                        let freq = self.dictionary.get(w).copied().unwrap_or(500);
                        dict_matches.push((freq, w.clone()));
                    }
                }
            }
        }

        dict_matches.sort_by_key(|b| std::cmp::Reverse(b.0));
        let dict_cap = max_candidates.saturating_sub(1).max(4);
        for (_, w) in dict_matches {
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: w,
                    kind: CandidateKind::Dictionary,
                    annotation: None,
                },
            );
            if candidates.len() >= dict_cap {
                break;
            }
        }

        // 6. In Smart mode, ensure the literal phonetic output is also available if not already #1
        if smart_mode {
            push_unique(
                &mut candidates,
                &mut seen,
                Candidate {
                    text: literal_bangla.clone(),
                    kind: CandidateKind::Phonetic,
                    annotation: None,
                },
            );
        }

        // 7. Spell-checker confusable suggestions if the literal phonetic isn't yet in the dictionary
        if !self.spell.is_correct(&literal_bangla) && candidates.len() < dict_cap {
            let lit_chars: Vec<char> = literal_bangla.chars().collect();
            for fix in self.spell.suggest(&literal_bangla, 4) {
                let fix_chars: Vec<char> = fix.chars().collect();
                let d = bangla_weighted_distance(&lit_chars, &fix_chars);
                if d <= 1.5 {
                    push_unique(
                        &mut candidates,
                        &mut seen,
                        Candidate {
                            text: fix,
                            kind: CandidateKind::SpellCorrection,
                            annotation: Some("বানান".to_string()),
                        },
                    );
                }
            }
        }

        // 8. Emoji suggestions
        if include_emojis {
            if let Some(ems) = self
                .emojis
                .get(&roman_lc)
                .or_else(|| self.emojis.get(&literal_bangla))
            {
                for &em in ems {
                    push_unique(
                        &mut candidates,
                        &mut seen,
                        Candidate {
                            text: em.to_string(),
                            kind: CandidateKind::Emoji,
                            annotation: None,
                        },
                    );
                }
            }
        }

        // 9. Preserve the English word itself right in the visible candidate bar (slot #5 max) so 1 click or Tab picks English!
        if roman_input.len() >= 2 && roman_input.chars().all(|c| c.is_ascii_alphabetic()) {
            let en_cand = Candidate {
                text: roman_input.to_string(),
                kind: CandidateKind::English,
                annotation: Some("EN".to_string()),
            };
            if seen.insert(roman_input.to_string()) {
                if candidates.len() >= 5 {
                    candidates.insert(4, en_cand);
                } else {
                    candidates.push(en_cand);
                }
            }
        }

        candidates.truncate(max_candidates);
        (candidates, hints)
    }

    /// Predict likely next Bangla words after committing `prev_word`.
    pub fn predict_next(&self, prev_word: &str) -> Vec<String> {
        if let Some(dict) = self.dict {
            let p = dict.predict_next(prev_word, 5);
            if !p.is_empty() {
                return p;
            }
        }
        self.bigrams.get(prev_word).cloned().unwrap_or_default()
    }

    /// Candidate list from the full dictionary.
    ///
    /// Order: user-defined autocorrect, then (Phonetic-First mode only) the literal phonetic output when it
    /// is itself a standard word, then Ekushe's ranking. The literal output is always kept in the list.
    #[allow(clippy::too_many_arguments)]
    fn dictionary_candidates(
        &self,
        dict: &'static Dictionary,
        roman_input: &str,
        roman_lc: &str,
        literal_bangla: &str,
        smart_mode: bool,
        include_emojis: bool,
        max_candidates: usize,
        prev_word: Option<&str>,
        candidates: &mut Vec<Candidate>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        let mut push = |list: &mut Vec<Candidate>, cand: Candidate| {
            if !cand.text.is_empty() && seen.insert(cand.text.clone()) {
                list.push(cand);
            }
        };
        if let Some(ac) = self
            .autocorrect
            .get(roman_input)
            .or_else(|| self.autocorrect.get(roman_lc))
        {
            push(
                candidates,
                Candidate {
                    text: ac.clone(),
                    kind: CandidateKind::Dictionary,
                    annotation: Some("Smart".to_string()),
                },
            );
        }
        let mut ranked = dict.rank(
            roman_input,
            prev_word,
            max_candidates.max(5),
            &self.rank_params,
        );
        // Completions (longer words) belong in the bar, never in slot #1: slot #1 is shown inline and
        // committed by Space, so it must be a whole-word match for what was typed.
        if let Some(pos) = ranked.iter().position(|r| !r.completion) {
            if pos > 0 {
                let first = ranked.remove(pos);
                ranked.insert(0, first);
            }
        }
        let phonetic = |t: &str| Candidate {
            text: t.to_string(),
            kind: CandidateKind::Phonetic,
            annotation: None,
        };
        // The first keystroke shows the plain letter (s → স) instead of jumping to a word. An explicit
        // hasanta (`,,`) is a deliberate spelling, so its exact output leads too (allah,, → আল্লাহ্).
        if roman_input.chars().count() == 1 || roman_input.contains(",,") {
            push(candidates, phonetic(literal_bangla));
        }
        // Phonetic-First: the literal phonetic output leads when it is a real, standard word, not much rarer
        // than Ekushe's top choice, and among its top 3 (tomake → তমাকে is literal but rare; তোমাকে wins).
        if !smart_mode {
            if let Some(li) = dict.lookup(literal_bangla) {
                let top_ppm = ranked
                    .first()
                    .map(|r| dict.info(r.id).log_ppm)
                    .unwrap_or(f32::MIN);
                let in_top3 = ranked.iter().take(3).any(|r| r.word == literal_bangla);
                if li.class == WordClass::Standard
                    && li.flags & crate::dict::FLAG_SUSPECT == 0
                    && li.log_ppm >= top_ppm - 1.0
                    && in_top3
                {
                    push(candidates, phonetic(literal_bangla));
                }
            }
        }
        let mut literal_slot_done = false;
        for (i, r) in ranked.into_iter().enumerate() {
            if i == 3 && !literal_slot_done {
                push(candidates, phonetic(literal_bangla));
                literal_slot_done = true;
            }
            let annotation = match r.class {
                WordClass::Colloquial => Some("কথ্য".to_string()),
                WordClass::Sadhu => Some("সাধু".to_string()),
                WordClass::Variant => Some("ভিন্ন বানান".to_string()),
                _ if r.completion => Some("…".to_string()),
                _ => None,
            };
            push(
                candidates,
                Candidate {
                    text: r.word,
                    kind: CandidateKind::Dictionary,
                    annotation,
                },
            );
        }
        if !literal_slot_done {
            push(candidates, phonetic(literal_bangla));
        }
        // Emoji go in the visible part of the bar (after the top 3 words), not after the full word list
        if include_emojis {
            let top = candidates
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default();
            let ems = self
                .emojis
                .get(roman_lc)
                .or_else(|| self.emojis.get(literal_bangla))
                .or_else(|| self.emojis.get(&top));
            if let Some(ems) = ems {
                let mut at = candidates.len().min(3);
                for &em in ems {
                    if seen.insert(em.to_string()) {
                        candidates.insert(
                            at,
                            Candidate {
                                text: em.to_string(),
                                kind: CandidateKind::Emoji,
                                annotation: None,
                            },
                        );
                        at += 1;
                    }
                }
            }
        }
        // Keep the English word itself reachable in slot 5 (one click / Tab picks English)
        if roman_input.len() >= 2
            && roman_input.chars().all(|c| c.is_ascii_alphabetic())
            && seen.insert(roman_input.to_string())
        {
            let en = Candidate {
                text: roman_input.to_string(),
                kind: CandidateKind::English,
                annotation: Some("EN".to_string()),
            };
            if candidates.len() >= 5 {
                candidates.insert(4, en);
            } else {
                candidates.push(en);
            }
        }
        candidates.truncate(max_candidates);
    }
}

/// Write `contents` to `path` via a temporary file + rename (no half-written file after a crash).
pub fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

/// Normalize a Roman string into a consonant/vowel skeleton so beginners who write
/// `bngla`, `bangla`, `kmn`, `manus`, `shikkha`, `sikha`, `swadhin`, `sbadhin`
/// still hit the target Bangla words.
fn roman_skeleton(roman: &str) -> String {
    let lc = roman.to_lowercase();
    let mut s = lc
        .replace("sh", "s")
        .replace("chh", "c")
        .replace("ch", "c")
        .replace("kkh", "ksh")
        .replace("kh", "k")
        .replace("gh", "g")
        .replace("th", "t")
        .replace("dh", "d")
        .replace("ph", "p")
        .replace("bh", "b")
        .replace("ee", "i")
        .replace("oo", "u")
        .replace('v', "b")
        .replace('f', "p")
        .replace('z', "j")
        .replace('w', "b");
    // Strip internal `o` vowels (since Bangla inherent vowel `o` is often omitted or added freely)
    if s.len() > 2 {
        let first = s.chars().next().unwrap();
        let rest: String = s[first.len_utf8()..]
            .chars()
            .filter(|&c| c != 'o')
            .collect();
        s = format!("{first}{rest}");
    }
    s
}

/// Built-in high-frequency Bangla lexicon (`(roman, bangla, frequency)`).
static BUILTIN_LEXICON: &[(&str, &str, u32)] = &[
    ("ami", "আমি", 100_000),
    ("amra", "আমরা", 95_000),
    ("amar", "আমার", 94_000),
    ("amader", "আমাদের", 90_000),
    ("apni", "আপনি", 92_000),
    ("apnar", "আপনার", 89_000),
    ("tumi", "তুমি", 91_000),
    ("tomar", "তোমার", 88_000),
    ("tomake", "তোমাকে", 85_000),
    ("se", "সে", 80_000),
    ("tara", "তারা", 78_000),
    ("bangla", "বাংলা", 100_000),
    ("banglay", "বাংলায়", 92_000),
    ("bangladesh", "বাংলাদেশ", 100_000),
    ("bangladeshi", "বাংলাদেশী", 85_000),
    ("bangali", "বাঙালি", 88_000),
    ("bhasha", "ভাষা", 90_000),
    ("ekushe", "একুশে", 95_000),
    ("ekush", "একুশ", 82_000),
    ("manush", "মানুষ", 96_000),
    ("jibon", "জীবন", 93_000),
    ("पानी", "পানি", 85_000),
    ("pani", "পানি", 88_000),
    ("jol", "জল", 80_000),
    ("bhalobasha", "ভালোবাসা", 94_000),
    ("bhalobashi", "ভালোবাসি", 93_000),
    ("bhalo", "ভালো", 95_000),
    ("kemon", "কেমন", 92_000),
    ("achen", "আছেন", 90_000),
    ("acho", "আছো", 87_000),
    ("achi", "আছি", 88_000),
    ("dhonnobad", "ধন্যবাদ", 95_000),
    ("swagatam", "স্বাগতম", 90_000),
    ("swadhin", "স্বাধীন", 92_000),
    ("swadhinota", "স্বাধীনতা", 94_000),
    ("swadesh", "স্বদেশ", 89_000),
    ("swapno", "স্বপ্ন", 91_000),
    ("shikkha", "শিক্ষা", 93_000),
    ("shikkhok", "শিক্ষক", 88_000),
    ("ggan", "জ্ঞান", 90_000),
    ("biggan", "বিজ্ঞান", 91_000),
    ("bishwo", "বিশ্ব", 92_000),
    ("bishwobidyaloy", "বিশ্ববিদ্যালয়", 89_000),
    ("dhaka", "ঢাকা", 95_000),
    ("chottogram", "চট্টগ্রাম", 86_000),
    ("khulna", "খুলনা", 82_000),
    ("rajshahi", "রাজশাহী", 82_000),
    ("sylhet", "সিলেট", 84_000),
    ("barishal", "বরিশাল", 80_000),
    ("rangpur", "রংপুর", 80_000),
    ("mymensingh", "ময়মনসিংহ", 80_000),
    ("kolkata", "কলকাতা", 85_000),
    ("gan", "গান", 88_000),
    ("gai", "গাই", 84_000),
    ("kotha", "কথা", 91_000),
    ("kaj", "কাজ", 90_000),
    ("shomoy", "সময়", 92_000),
    ("din", "দিন", 89_000),
    ("rat", "রাত", 86_000),
    ("shokal", "সকাল", 85_000),
    ("bikel", "বিকেল", 82_000),
    ("bondhu", "বন্ধু", 91_000),
    ("poribar", "পরিবার", 88_000),
    ("ma", "মা", 95_000),
    ("baba", "বাবা", 94_000),
    ("bhai", "ভাই", 90_000),
    ("bon", "বোন", 88_000),
    ("desh", "দেশ", 93_000),
    ("matribhasha", "মাতৃভাষা", 92_000),
    ("shohid", "শহীদ", 90_000),
    ("february", "ফেব্রুয়ারি", 88_000),
    ("antulon", "আন্দোলন", 84_000),
    ("andolon", "আন্দোলন", 89_000),
    ("shuvechha", "শুভেচ্ছা", 88_000),
    ("abhinandan", "অভিনন্দন", 86_000),
    ("prothom", "প্রথম", 90_000),
    ("dwitIyo", "দ্বিতীয়", 85_000),
    ("shomoshya", "সমস্যা", 88_000),
    ("shomadhan", "সমাধান", 87_000),
    ("proyojon", "প্রয়োজন", 89_000),
    ("gurutto", "গুরুত্ব", 86_000),
    ("gurutwopurno", "গুরুত্বপূর্ণ", 88_000),
    ("onurodh", "অনুরোধ", 85_000),
    ("sahajjo", "সাহায্য", 87_000),
    ("sundor", "সুন্দর", 90_000),
    ("anondo", "আনন্দ", 88_000),
    ("kanna", "কান্না", 82_000),
    ("hashi", "হাসি", 85_000),
    ("hridoy", "হৃদয়", 86_000),
    ("akash", "আকাশ", 87_000),
    ("brishTi", "বৃষ্টি", 86_000),
    ("shurjo", "সূর্য", 85_000),
    ("chad", "চাঁদ", 85_000),
    ("taka", "টাকা", 90_000),
    ("chhati", "ছাতি", 90_000),
    ("chati", "ছাতি", 90_000),
    ("chhata", "ছাতা", 90_000),
    ("chata", "ছাতা", 90_000),
    ("chhele", "ছেলে", 93_000),
    ("chele", "ছেলে", 93_000),
    ("chhobi", "ছবি", 92_000),
    ("chobi", "ছবি", 92_000),
    ("chhoto", "ছোট", 92_000),
    ("choto", "ছোট", 92_000),
    ("chhuti", "ছুটি", 90_000),
    ("chuti", "ছুটি", 90_000),
    ("chhatro", "ছাত্র", 91_000),
    ("chatro", "ছাত্র", 91_000),
    ("chhaya", "ছায়া", 88_000),
    ("chaya", "ছায়া", 88_000),
    ("chhora", "ছড়া", 87_000),
    ("chora", "ছড়া", 87_000),
    ("chhagol", "ছাগল", 86_000),
    ("chagol", "ছাগল", 86_000),
    ("chhuri", "ছুরি", 85_000),
    ("chhad", "ছাদ", 86_000),
];

/// Built-in autocorrect & English-word phonetic mappings (OpenBangla / Avro parity).
static BUILTIN_AUTOCORRECT: &[(&str, &str)] = &[
    ("bangladesh", "বাংলাদেশ"),
    ("bngla", "বাংলা"),
    ("kmn", "কেমন"),
    ("acn", "আছেন"),
    ("dhonnobad", "ধন্যবাদ"),
    ("assalamualaikum", "আসসালামু আলাইকুম"),
    ("walaikumussalam", "ওয়ালাইকুমুস সালাম"),
    ("inshaallah", "ইনশাআল্লাহ"),
    ("alhamdulillah", "আলহামদুলিল্লাহ"),
    ("mashallah", "মাশাআল্লাহ"),
    ("subhanallah", "সুবহানাল্লাহ"),
    ("nomoshkar", "নমস্কার"),
    ("swadhinota", "স্বাধীনতা"),
    ("swagatam", "স্বাগতম"),
    ("facebook", "ফেসবুক"),
    ("youtube", "ইউটিউব"),
    ("google", "গুগল"),
    ("whatsapp", "হোয়াটসঅ্যাপ"),
    ("messenger", "মেসেঞ্জার"),
    ("internet", "ইন্টারনেট"),
    ("computer", "কম্পিউটার"),
    ("mobile", "মোবাইল"),
    ("phone", "ফোন"),
    ("download", "ডাউনলোড"),
    ("software", "সফটওয়্যার"),
    ("keyboard", "কীবোর্ড"),
    ("university", "ইউনিভার্সিটি"),
    ("school", "স্কুল"),
    ("college", "কলেজ"),
    ("hospital", "হাসপাতাল"),
    ("doctor", "ডাক্তার"),
    ("police", "পুলিশ"),
    ("government", "সরকার"),
    ("january", "জানুয়ারি"),
    ("february", "ফেব্রুয়ারি"),
    ("march", "মার্চ"),
    ("april", "এপ্রিল"),
    ("august", "আগস্ট"),
    ("september", "সেপ্টেম্বর"),
    ("october", "অক্টোবর"),
    ("november", "নভেম্বর"),
    ("december", "ডিসেম্বর"),
];

/// Built-in Emoji map (Bangla & Roman triggers).
static BUILTIN_EMOJIS: &[(&str, &[&str])] = &[
    ("bangladesh", &["🇧🇩"]),
    ("বাংলাদেশ", &["🇧🇩"]),
    ("bhalobasha", &["❤️", "💖"]),
    ("ভালোবাসা", &["❤️", "💖"]),
    ("bhalobashi", &["❤️"]),
    ("ভালোবাসি", &["❤️"]),
    ("kanna", &["😢", "😭"]),
    ("কান্না", &["😢", "😭"]),
    ("hashi", &["😊", "😄"]),
    ("হাসি", &["😊", "😄"]),
    ("dhonnobad", &["🙏", "💐"]),
    ("ধন্যবাদ", &["🙏", "💐"]),
    ("shuvechha", &["🎉", "💐"]),
    ("শুভেচ্ছা", &["🎉", "💐"]),
    ("anondo", &["🥳", "✨"]),
    ("আনন্দ", &["🥳", "✨"]),
    ("chad", &["🌙"]),
    ("চাঁদ", &["🌙"]),
    ("brishTi", &["🌧️", "☔"]),
    ("বৃষ্টি", &["🌧️", "☔"]),
    ("hridoy", &["❤️"]),
    ("হৃদয়", &["❤️"]),
    ("agun", &["🔥"]),
    ("আগুন", &["🔥"]),
];

/// Built-in text expansion snippets (`;trigger` and `/trigger`).
static BUILTIN_SNIPPETS: &[(&str, &str)] = &[
    (";salam", "আসসালামু আলাইকুম"),
    ("/sal", "আসসালামু আলাইকুম"),
    (";wsalam", "ওয়ালাইকুমুস সালাম"),
    ("/wsal", "ওয়ালাইকুমুস সালাম"),
    (";namaskar", "নমস্কার"),
    ("/nam", "নমস্কার"),
    (";dhonnobad", "অসংখ্য ধন্যবাদ"),
    ("/thx", "অসংখ্য ধন্যবাদ"),
    (";tk", "৳"),
    (";bdt", "৳"),
    ("/bdt", "৳"),
    (";bd", "গণপ্রজাতন্ত্রী বাংলাদেশ"),
    ("/bd", "গণপ্রজাতন্ত্রী বাংলাদেশ"),
    (";ekushe", "অমর একুশে ফেব্রুয়ারি"),
    ("/ekushe", "অমর একুশে ফেব্রুয়ারি"),
    (";shuvo", "শুভকামনা রইল"),
    ("/shuvo", "শুভ নববর্ষ"),
    (";inshaallah", "ইনশাআল্লাহ"),
    (";alhamdulillah", "আলহামদুলিল্লাহ"),
];

/// Built-in bigram next-word predictions.
static BUILTIN_BIGRAMS: &[(&str, &[&str])] = &[
    ("আমি", &["বাংলায়", "ভালো", "তোমাকে", "মনে", "যাব"]),
    ("কেমন", &["আছেন", "আছো", "লাগছে"]),
    ("ভালো", &["আছি", "লাগে", "মানুষ", "থাকবেন"]),
    ("আমার", &["সোনার", "বাংলা", "নাম", "মনে"]),
    ("বাংলাদেশ", &["আমার", "সরকার", "থেকে"]),
    ("আসসালামু", &["আলাইকুম"]),
    ("শুভ", &["সকাল", "রাত্রি", "জন্মদিন", "নববর্ষ"]),
    ("অনেক", &["ধন্যবাদ", "ভালো", "সুন্দর"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestions_and_emoji_and_learning() {
        let mut engine = SuggestionEngine::new();
        let phon = PhoneticEngine::new();

        // Forgiving `bngla` suggests `বাংলা`
        let (cands, _) = engine.suggest("bngla", &phon, false, true, 9);
        assert!(cands.iter().any(|c| c.text == "বাংলা"));

        // `kanna` includes 😢 emoji
        let (cands_em, _) = engine.suggest("kanna", &phon, false, true, 9);
        assert!(cands_em.iter().any(|c| c.text == "😢"));

        // `facebook` includes `ফেসবুক` and `facebook`
        let (cands_fb, _) = engine.suggest("facebook", &phon, false, true, 9);
        assert!(cands_fb.iter().any(|c| c.text == "ফেসবুক"));
        assert!(cands_fb.iter().any(|c| c.text == "facebook"));

        // Learned choices survive a save / load round trip
        engine.record_user_choice("valo", "ভালো");
        let dir = std::env::temp_dir().join(format!("ekushe-test-{}", std::process::id()));
        let path = dir.join("learned.tsv");
        let v = engine.learned_version();
        engine.save_user_dict(&path).unwrap();
        let mut fresh = SuggestionEngine::new();
        fresh.load_user_dict(&path).unwrap();
        assert_eq!(fresh.user_dict_tsv(), engine.user_dict_tsv());
        assert!(v > 0);
        let _ = std::fs::remove_dir_all(&dir);

        // User learning boosts chosen word to the top
        engine.record_user_choice("kmn", "কেমনই");
        let (cands_learned, _) = engine.suggest("kmn", &phon, false, true, 9);
        assert_eq!(cands_learned[0].text, "কেমনই");
    }
}
