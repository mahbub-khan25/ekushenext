//! Bangla Spell Checker with Confusable-Pair Weighted Edit Distance & Suffix Morphology.
//!
//! Understands real-world Bangla spelling mistakes:
//! - ই-কার (`ি`) <-> ঈ-কার (`ী`) and ই <-> ঈ
//! - উ-কার (`ু`) <-> ঊ-কার (`ূ`) and উ <-> ঊ
//! - শ <-> ষ <-> স (Sibilants)
//! - ন <-> ণ (Dental vs Retroflex nasal)
//! - জ <-> য (Ja vs Antahstha Ja)
//! - র <-> ড় <-> ঢ় (Ra vs Daa-shunya Raa)
//! - ঙ <-> ং (Uma vs Anusvara)
//! - ত <-> ৎ (Ta vs Khanda Ta)
//! - ট <-> ত, ঠ <-> থ, ড <-> দ, ঢ <-> ধ (Retroflex vs Dental stops)
//! - Common Bangla inflectional suffixes (`-টি`, `-টা`, `-গুলো`, `-ের`, `-কে`, `-রা`, `-দের`, `-তে`, `-য়`).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::dict::{Dictionary, WordClass, FLAG_CONFIDENT};

/// A detected spelling issue in a document or sentence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellIssue {
    /// The misspelled Bangla word
    pub word: String,
    /// Byte offset in the input text
    pub byte_offset: usize,
    /// Ranked list of suggested corrections
    pub suggestions: Vec<String>,
    /// `misspelling` (not a Bangla word) or `non_standard` (a real but non-standard spelling, e.g.
    /// সরকারী → সরকারি by the Bangla Academy rules)
    #[serde(default)]
    pub kind: String,
}

/// Bangla-aware spell checker backed by a frequency dictionary and user dictionary.
#[derive(Debug, Clone)]
pub struct SpellChecker {
    /// Known valid words mapped to frequency weight (higher = more common)
    words: HashMap<String, u32>,
    /// User-added valid words
    user_words: HashSet<String>,
    /// Ekushe's full dictionary, when available
    dict: Option<&'static Dictionary>,
}

/// Bring a word to Ekushe's canonical form (Unicode NFC for Bangla): precomposed ড় ঢ় য় become
/// letter + nukta, and split vowel signs (ে + া) are joined.
pub fn normalize_bangla(word: &str) -> String {
    word.replace('\u{09DC}', "\u{09A1}\u{09BC}")
        .replace('\u{09DD}', "\u{09A2}\u{09BC}")
        .replace('\u{09DF}', "\u{09AF}\u{09BC}")
        .replace("\u{09C7}\u{09BE}", "\u{09CB}")
        .replace("\u{09C7}\u{09D7}", "\u{09CC}")
}

/// Join letter + nukta into one char so ড়/ঢ়/য় take part in confusable-pair costs.
fn compose_nukta(w: &[char]) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(w.len());
    for &c in w {
        if c == '\u{09BC}' {
            match out.last().copied() {
                Some('\u{09A1}') => {
                    out.pop();
                    out.push('\u{09DC}');
                    continue;
                }
                Some('\u{09A2}') => {
                    out.pop();
                    out.push('\u{09DD}');
                    continue;
                }
                Some('\u{09AF}') => {
                    out.pop();
                    out.push('\u{09DF}');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// Common Bangla inflectional suffixes that can be attached to valid dictionary roots.
const BANGLA_SUFFIXES: &[&str] = &[
    "গুলোকে",
    "গুলোতে",
    "গুলোর",
    "গুলো",
    "গুলি",
    "দেরকে",
    "দের",
    "টিকে",
    "টির",
    "টিতে",
    "টি",
    "টাকে",
    "টার",
    "টাতে",
    "টা",
    "খানা",
    "খানি",
    "গুলোয়",
    "কে",
    "রে",
    "রা",
    "তে",
    "য়",
    "য়ে",
    "ের",
    "র",
];

impl SpellChecker {
    pub fn new(words: HashMap<String, u32>) -> Self {
        Self {
            words,
            user_words: HashSet::new(),
            dict: None,
        }
    }

    /// Use Ekushe's full dictionary for checking and suggestions.
    pub fn set_dictionary(&mut self, dict: Option<&'static Dictionary>) {
        self.dict = dict;
    }

    fn dict_knows(&self, word: &str) -> bool {
        self.dict
            .and_then(|d| d.lookup(word))
            .map(|i| i.class != WordClass::Typo)
            .unwrap_or(false)
    }

    pub fn add_user_word(&mut self, word: impl Into<String>) {
        self.user_words.insert(word.into());
    }

    /// Returns true if `word` is valid in Bangla (either exact dictionary match, user word,
    /// non-Bangla token/number, or valid root + standard Bangla inflectional suffix).
    pub fn is_correct(&self, word: &str) -> bool {
        let norm = normalize_bangla(word);
        let trimmed = norm.trim_matches(|c: char| !is_bangla_letter_or_mark(c));
        if let Some(d) = self.dict {
            if trimmed.is_empty()
                || trimmed.chars().all(|c| ('০'..='৯').contains(&c))
                || self.user_words.contains(trimmed)
            {
                return true;
            }
            if let Some(info) = d.lookup(trimmed) {
                return info.class != WordClass::Typo;
            }
            for &suf in BANGLA_SUFFIXES {
                if let Some(stem) = trimmed.strip_suffix(suf) {
                    if stem.chars().count() >= 2
                        && (self.dict_knows(stem) || self.user_words.contains(stem))
                    {
                        return true;
                    }
                }
            }
            return false;
        }
        if trimmed.is_empty() {
            return true;
        }
        // Pure Bangla digits are valid
        if trimmed.chars().all(|c| ('০'..='৯').contains(&c)) {
            return true;
        }
        if self.words.contains_key(trimmed) || self.user_words.contains(trimmed) {
            return true;
        }
        // Check morphological suffix stripping
        for &suf in BANGLA_SUFFIXES {
            if let Some(stem) = trimmed.strip_suffix(suf) {
                if stem.chars().count() >= 2
                    && (self.words.contains_key(stem) || self.user_words.contains(stem))
                {
                    return true;
                }
            }
        }
        false
    }

    /// Return up to `max_results` ranked correction suggestions for a misspelled Bangla word.
    pub fn suggest(&self, word: &str, max_results: usize) -> Vec<String> {
        if let Some(d) = self.dict {
            let out = self.suggest_with_dict(d, word, max_results);
            if !out.is_empty() {
                return out;
            }
        }
        let trimmed = word.trim_matches(|c: char| !is_bangla_letter_or_mark(c));
        if trimmed.is_empty() {
            return Vec::new();
        }
        let target_chars: Vec<char> = trimmed.chars().collect();
        let target_len = target_chars.len();

        let mut scored: Vec<(f32, u32, &String)> = Vec::new();

        for (cand, &freq) in &self.words {
            let cand_chars: Vec<char> = cand.chars().collect();
            if cand_chars.len().abs_diff(target_len) > 2 {
                continue;
            }
            let dist = bangla_weighted_distance(&target_chars, &cand_chars);
            if dist <= 2.1 {
                scored.push((dist, freq, cand));
            }
        }

        // Also check if stripping a suffix yields a close root, and re-attach the suffix
        let mut suffixed_candidates: Vec<(f32, u32, String)> = Vec::new();
        for &suf in BANGLA_SUFFIXES {
            if let Some(stem) = trimmed.strip_suffix(suf) {
                let stem_chars: Vec<char> = stem.chars().collect();
                if stem_chars.len() >= 2 {
                    for (cand, &freq) in &self.words {
                        let cand_chars: Vec<char> = cand.chars().collect();
                        if cand_chars.len().abs_diff(stem_chars.len()) > 1 {
                            continue;
                        }
                        let dist = bangla_weighted_distance(&stem_chars, &cand_chars);
                        if dist <= 1.2 {
                            suffixed_candidates.push((dist + 0.15, freq, format!("{cand}{suf}")));
                        }
                    }
                }
                break;
            }
        }

        scored.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });

        let mut out: Vec<String> = Vec::with_capacity(max_results);
        for (_, _, w) in scored {
            if !out.contains(w) {
                out.push(w.clone());
                if out.len() >= max_results {
                    return out;
                }
            }
        }

        suffixed_candidates.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
        for (_, _, w) in suffixed_candidates {
            if !out.contains(&w) {
                out.push(w);
                if out.len() >= max_results {
                    break;
                }
            }
        }

        out
    }

    /// Dictionary-backed suggestions: the recorded standard form first (typo or variant), then words
    /// that sound alike (same Roman key) ranked by Bangla-aware edit distance and frequency.
    fn suggest_with_dict(&self, d: &Dictionary, word: &str, max_results: usize) -> Vec<String> {
        let norm = normalize_bangla(word);
        let w = norm.trim_matches(|c: char| !is_bangla_letter_or_mark(c));
        if w.is_empty() {
            return Vec::new();
        }
        let mut out: Vec<String> = Vec::new();
        if let Some(p) = d.lookup(w).and_then(|i| i.preferred) {
            out.push(d.word(p).to_string());
        }
        let wc: Vec<char> = w.chars().collect();
        let mut keys: Vec<String> = crate::roman::romanize(w, 4)
            .iter()
            .map(|(_, r)| crate::roman::skeleton(r))
            .collect();
        keys.sort();
        keys.dedup();
        let mut scored: Vec<(f32, String)> = Vec::new();
        let mut seen = HashSet::new();
        for k in keys {
            for id in d.bucket(&k).into_iter().take(300) {
                if !seen.insert(id) {
                    continue;
                }
                let info = d.info(id);
                if !matches!(info.class, WordClass::Standard) {
                    continue;
                }
                let cand = d.word(id);
                if cand == w {
                    continue;
                }
                let cc: Vec<char> = cand.chars().collect();
                if cc.len().abs_diff(wc.len()) > 2 {
                    continue;
                }
                let dist = bangla_weighted_distance(&wc, &cc);
                if dist <= 2.1 {
                    scored.push((dist - 0.12 * info.log_ppm, cand.to_string()));
                }
            }
        }
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, c) in scored {
            if !out.contains(&c) {
                out.push(c);
            }
            if out.len() >= max_results {
                break;
            }
        }
        out
    }

    /// A real word whose standard spelling differs (সরকারী → সরকারি), when the decision is confident.
    fn non_standard(&self, word: &str) -> Option<String> {
        let d = self.dict?;
        let norm = normalize_bangla(word);
        let info = d.lookup(&norm)?;
        if info.class == WordClass::Variant && info.flags & FLAG_CONFIDENT != 0 {
            return info.preferred.map(|p| d.word(p).to_string());
        }
        None
    }

    fn issue_for(&self, word: &str, byte_offset: usize) -> Option<SpellIssue> {
        if !self.is_correct(word) {
            return Some(SpellIssue {
                word: word.to_string(),
                byte_offset,
                suggestions: self.suggest(word, 5),
                kind: "misspelling".into(),
            });
        }
        self.non_standard(word).map(|p| SpellIssue {
            word: word.to_string(),
            byte_offset,
            suggestions: vec![p],
            kind: "non_standard".into(),
        })
    }

    /// Check an entire Bangla text and return all detected spelling issues with offsets and suggestions.
    pub fn check_text(&self, text: &str) -> Vec<SpellIssue> {
        let mut issues = Vec::new();
        let mut start_byte: Option<usize> = None;

        for (idx, ch) in text.char_indices() {
            if is_bangla_letter_or_mark(ch) {
                if start_byte.is_none() {
                    start_byte = Some(idx);
                }
            } else if let Some(s) = start_byte.take() {
                issues.extend(self.issue_for(&text[s..idx], s));
            }
        }

        if let Some(s) = start_byte {
            issues.extend(self.issue_for(&text[s..], s));
        }

        issues
    }
}

/// Weighted Damerau-Levenshtein distance where common Bangla phonetic/orthographic confusions cost 0.3
/// instead of 1.0.
pub fn bangla_weighted_distance(a: &[char], b: &[char]) -> f32 {
    let (a, b) = (compose_nukta(a), compose_nukta(b));
    let (a, b) = (a.as_slice(), b.as_slice());
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0.0f32; n + 1]; m + 1];

    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i as f32;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(n + 1) {
        *val = j as f32;
    }

    for i in 1..=m {
        for j in 1..=n {
            let sub_cost = if a[i - 1] == b[j - 1] {
                0.0
            } else if is_bangla_confusable_pair(a[i - 1], b[j - 1]) {
                0.3
            } else {
                1.0
            };

            let del_cost = if a[i - 1] == 'ঁ' || a[i - 1] == '্' {
                0.5
            } else {
                1.0
            };
            let ins_cost = if b[j - 1] == 'ঁ' || b[j - 1] == '্' {
                0.5
            } else {
                1.0
            };

            let mut best = (dp[i - 1][j] + del_cost)
                .min(dp[i][j - 1] + ins_cost)
                .min(dp[i - 1][j - 1] + sub_cost);

            // Adjacent transposition (e.g. kar order slip)
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(dp[i - 2][j - 2] + 0.4);
            }

            dp[i][j] = best;
        }
    }

    dp[m][n]
}

/// Returns true if `x` and `y` are commonly confused Bangla letters or vowel signs.
#[inline]
pub fn is_bangla_confusable_pair(x: char, y: char) -> bool {
    let (a, b) = if x <= y { (x, y) } else { (y, x) };
    matches!(
        (a, b),
        ('ি', 'ী')
            | ('ু', 'ূ')
            | ('ই', 'ঈ')
            | ('উ', 'ঊ')
            | ('শ', 'ষ')
            | ('শ', 'স')
            | ('ষ', 'স')
            | ('ণ', 'ন')
            | ('জ', 'য')
            | ('জ', '\u{09DF}')
            | ('য', '\u{09DF}')
            | ('র', '\u{09DC}')
            | ('র', '\u{09DD}')
            | ('\u{09DC}', '\u{09DD}')
            | ('ং', 'ঙ')
            | ('ত', 'ৎ')
            | ('ট', 'ত')
            | ('ঠ', 'থ')
            | ('ড', 'দ')
            | ('ঢ', 'ধ')
            | ('ক', 'খ')
            | ('গ', 'ঘ')
            | ('চ', 'ছ')
            | ('জ', 'ঝ')
            | ('প', 'ফ')
            | ('ব', 'ভ')
            | ('ে', 'ৈ')
            | ('ো', 'ৌ')
    )
}

#[inline]
pub fn is_bangla_letter_or_mark(c: char) -> bool {
    ('\u{0980}'..='\u{09FF}').contains(&c) || c == '\u{200C}' || c == '\u{200D}'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confusable_corrections_rank_first() {
        let mut dict = HashMap::new();
        dict.insert("বাংলাদেশ".to_string(), 10_000);
        dict.insert("মানুষ".to_string(), 8_000);
        dict.insert("জীবন".to_string(), 7_500);
        dict.insert("পানি".to_string(), 6_000);

        let checker = SpellChecker::new(dict);

        // `বাংলাদেস` (স instead of শ) -> `বাংলাদেশ`
        assert!(!checker.is_correct("বাংলাদেস"));
        let sugg = checker.suggest("বাংলাদেস", 3);
        assert_eq!(sugg.first().map(String::as_str), Some("বাংলাদেশ"));

        // `মানুস` (স instead of ষ) -> `মানুষ`
        assert_eq!(
            checker.suggest("মানুস", 3).first().map(String::as_str),
            Some("মানুষ")
        );

        // `জিবন` (ি instead of ী) -> `জীবন`
        assert_eq!(
            checker.suggest("জিবন", 3).first().map(String::as_str),
            Some("জীবন")
        );

        // Inflected word `বাংলাদেশের` (`বাংলাদেশ` + `-ের`) is recognized as correct
        assert!(checker.is_correct("বাংলাদেশের"));
    }
}
