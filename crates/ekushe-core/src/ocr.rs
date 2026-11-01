//! Clean-up of Bangla text read by OCR (a picture or scan of a page turned into text).
//!
//! OCR engines make their own kind of mistakes in Bangla, different from typing mistakes: letters
//! that look alike (ভ/ত, এ/ঐ, ন/ল), a চন্দ্রবিন্দু or রেফ that disappears, ূ read as ু, a dari read as
//! `|` or `৷`, digits read as letters. This module fixes them with Ekushe's dictionary:
//!
//! 1. glyph look-alikes that are never right in Bangla text are replaced outright;
//! 2. every word is checked against the dictionary; words that aren't Bangla words, and rare words
//!    that are likely misreads, get candidates made by undoing the mistakes OCR is known to make
//!    (`data/ocr/rules.tsv`, learned by comparing OCR output with the pages it read);
//! 3. candidates are scored by how common the word is, how likely that misread is, and whether it
//!    fits the words around it. A known word is only replaced when the evidence is clearly better.
//!
//! Every change is reported, so an app can show the corrected words and let the person undo one.

use crate::dict::{Dictionary, WordClass};
use std::collections::HashMap;
use std::sync::OnceLock;

/// How the clean-up weighs its evidence (tuned on rendered and degraded pages: clean, scanner and
/// phone-photo conditions).
#[derive(Debug, Clone, Copy)]
pub struct OcrParams {
    /// How much more likely a correction must be than a known word as read
    pub margin: f32,
    /// Weight of how likely the misread is
    pub channel: f32,
    /// Weight of the neighbouring words
    pub context: f32,
    /// What a word the dictionary doesn't know (a name, a rare word) counts as, in ln(frequency per
    /// million): the lower, the more readily such a word is corrected
    pub unknown: f32,
}

impl Default for OcrParams {
    fn default() -> Self {
        Self {
            margin: 1.5,
            channel: 1.5,
            context: 0.6,
            unknown: -9.0,
        }
    }
}
const LN10: f32 = std::f32::consts::LN_10;

/// One word the clean-up changed.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct OcrChange {
    /// Byte offset of the new word in the corrected text
    pub at: usize,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct OcrResult {
    pub text: String,
    pub changes: Vec<OcrChange>,
}

struct Rules {
    /// observed text -> (intended text, log probability of that misread)
    by_observed: HashMap<String, Vec<(String, f32)>>,
    /// letters OCR tends to drop (inserted back as candidates)
    dropped: Vec<(String, f32)>,
}

fn rules() -> &'static Rules {
    static R: OnceLock<Rules> = OnceLock::new();
    R.get_or_init(|| {
        let mut by_observed: HashMap<String, Vec<(String, f32)>> = HashMap::new();
        let mut dropped = Vec::new();
        // learned from rendered pages, then priors for old print and archive scans (a dropped া,
        // ন/স, য/ষ, র/ব, the old ৎ): the misreads real pages show more often than rendered ones
        let learned = include_str!("../../../data/ocr/rules.tsv");
        let extra = include_str!("../../../data/ocr/rules_extra.tsv");
        for line in learned.lines().chain(extra.lines()) {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 4 {
                continue;
            }
            let Ok(lp) = f[3].parse::<f32>() else {
                continue;
            };
            if f[1].is_empty() {
                if lp > -6.0 && !f[0].is_empty() {
                    match dropped
                        .iter_mut()
                        .find(|(i, _): &&mut (String, f32)| i == f[0])
                    {
                        Some(d) => d.1 = d.1.max(lp),
                        None => dropped.push((f[0].to_string(), lp)),
                    }
                }
            } else {
                let list = by_observed.entry(f[1].to_string()).or_default();
                match list.iter_mut().find(|(i, _)| i == f[0]) {
                    Some(r) => r.1 = r.1.max(lp),
                    None => list.push((f[0].to_string(), lp)),
                }
            }
        }
        Rules {
            by_observed,
            dropped,
        }
    })
}

fn is_bangla(c: char) -> bool {
    ('\u{0980}'..='\u{09FF}').contains(&c)
}

fn is_bangla_digit(c: char) -> bool {
    ('০'..='৯').contains(&c)
}

/// Glyph look-alikes OCR produces that are never what a Bangla page shows.
pub fn fix_glyphs(text: &str) -> String {
    let s: String = text
        .chars()
        .filter(|c| *c != '\u{200C}' && *c != '\u{200D}')
        .map(|c| match c {
            '\u{09F7}' => '।', // ৷ (a currency numerator) for the dari
            'ৰ' => 'র',        // the Assamese ra, never in Bangla
            _ => c,
        })
        .collect();
    let chars: Vec<char> = s.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // `|` standing after Bangla (and before a space or the end) is a dari
        if c == '|' {
            let prev = out.iter().rev().find(|c| !c.is_whitespace()).copied();
            let next = chars.get(i + 1).copied();
            if prev.is_some_and(is_bangla) && next.is_none_or(char::is_whitespace) {
                while out.last() == Some(&' ') {
                    out.pop();
                }
                out.push('।');
                i += 1;
                continue;
            }
        }
        // no space before a dari or a comma after a Bangla word
        if c == ' ' {
            let mut j = i;
            while chars.get(j) == Some(&' ') {
                j += 1;
            }
            let next = chars.get(j).copied();
            if out.last().copied().is_some_and(is_bangla)
                && matches!(next, Some('।' | '?' | '!' | ',' | ';' | ':' | ')'))
            {
                i = j;
                continue;
            }
        }
        // a doubled hasanta
        if c == '্' && out.last() == Some(&'্') {
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    let s: String = out.into_iter().collect();
    // অ্যা is often read as আ্যা
    let s = s.replace("আ্য", "অ্য");
    fix_digits(&s)
}

/// Inside a number, letters that look like digits are digits (এ/৫, ও/৩, Latin 8/o).
fn fix_digits(s: &str) -> String {
    let look = |c: char| match c {
        'এ' => Some('৫'),
        'ও' => Some('৩'),
        '8' => Some('৪'),
        'o' | 'O' => Some('০'),
        _ => None,
    };
    let numeric = |c: char| is_bangla_digit(c) || look(c).is_some();
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if numeric(chars[i]) && (i == 0 || !is_bangla_letter(chars[i - 1])) {
            // a run of digit-like characters, with . , : / - between groups
            let mut j = i;
            while j < chars.len()
                && (numeric(chars[j])
                    || (matches!(chars[j], '.' | ',' | ':' | '/' | '-')
                        && chars.get(j + 1).is_some_and(|c| numeric(*c))
                        && j > i))
            {
                j += 1;
            }
            let run = &chars[i..j];
            let after_letter = chars.get(j).is_some_and(|c| is_bangla_letter(*c));
            let digits = run.iter().filter(|c| is_bangla_digit(**c)).count();
            let lookalikes = run.iter().filter(|c| look(**c).is_some()).count();
            if !after_letter && digits >= 1 && digits >= lookalikes {
                out.extend(run.iter().map(|c| look(*c).unwrap_or(*c)));
            } else {
                out.extend(run.iter());
            }
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn is_bangla_letter(c: char) -> bool {
    is_bangla(c) && !is_bangla_digit(c)
}

struct Scorer<'a> {
    dict: &'a Dictionary,
    rules: &'a Rules,
    cache: HashMap<String, Vec<(String, f32)>>,
}

impl<'a> Scorer<'a> {
    /// ln(frequency per million) of a real (non-typo) word
    fn freq(&self, w: &str) -> Option<f32> {
        let id = self.dict.id_of(w)?;
        let info = self.dict.info(id);
        (info.class != WordClass::Typo).then_some(info.log_ppm * LN10)
    }

    /// How well `b` follows `a`: the dictionary's word-pair counts, or a back-off to `b` alone
    fn pair(&self, a: Option<&str>, b: Option<&str>) -> f32 {
        let (Some(a), Some(b)) = (a, b) else {
            return 0.0;
        };
        let (Some(ia), Some(ib)) = (self.dict.id_of(a), self.dict.id_of(b)) else {
            return (0.4f32 * 1e-6 * 0.5).ln();
        };
        let fol = self.dict.followers(ia);
        let total: f32 = fol.iter().map(|(_, c)| *c as f32).sum::<f32>().max(1.0);
        if let Some((_, c)) = fol.iter().find(|(id, _)| *id == ib) {
            return (0.5 * *c as f32 / total).ln();
        }
        let ppm = 10f32.powf(self.dict.info(ib).log_ppm);
        (0.4 * (ppm + 0.5) * 1e-6).ln()
    }

    fn variants(&self, w: &str) -> HashMap<String, f32> {
        let chars: Vec<char> = w.chars().collect();
        let mut out: HashMap<String, f32> = HashMap::new();
        let mut add = |v: String, lp: f32| {
            if v != w {
                let e = out.entry(v).or_insert(f32::MIN);
                if *e < lp {
                    *e = lp;
                }
            }
        };
        for i in 0..chars.len() {
            for n in 1..=3 {
                if i + n > chars.len() {
                    break;
                }
                let observed: String = chars[i..i + n].iter().collect();
                if let Some(list) = self.rules.by_observed.get(&observed) {
                    for (intended, lp) in list {
                        let v: String = chars[..i]
                            .iter()
                            .chain(intended.chars().collect::<Vec<_>>().iter())
                            .chain(chars[i + n..].iter())
                            .collect();
                        add(v, *lp);
                    }
                }
            }
        }
        for i in 0..=chars.len() {
            for (intended, lp) in &self.rules.dropped {
                let v: String = chars[..i]
                    .iter()
                    .copied()
                    .chain(intended.chars())
                    .chain(chars[i..].iter().copied())
                    .collect();
                add(v, *lp);
            }
        }
        out
    }

    /// A known word with a common ending (পরস্মৈপদীও, ব্রহ্মসূত্রকে): as good as known
    fn known_with_ending(&self, w: &str) -> bool {
        const ENDINGS: &[&str] = &[
            "গুলোকে",
            "গুলিকে",
            "গুলোতে",
            "গুলিতে",
            "গুলোর",
            "গুলির",
            "গুলো",
            "গুলি",
            "দেরকে",
            "দের",
            "কেও",
            "তেও",
            "েরও",
            "ের",
            "কে",
            "তে",
            "রা",
            "টি",
            "টা",
            "ও",
            "এর",
        ];
        // the stem must be a common word (at least one in a million), so a misread doesn't pass
        ENDINGS.iter().any(|e| {
            w.strip_suffix(e).is_some_and(|stem| {
                stem.chars().count() >= 2 && self.freq(stem).is_some_and(|f| f >= 0.0)
            })
        })
    }

    /// Dictionary words this OCR word may have been, with the log probability of the misread
    fn candidates(&mut self, w: &str) -> Vec<(String, f32)> {
        if let Some(c) = self.cache.get(w) {
            return c.clone();
        }
        let known = self.freq(w).is_some();
        let mut found: HashMap<String, f32> = HashMap::new();
        let one = self.variants(w);
        for (v, lp) in &one {
            if self.freq(v).is_some() {
                let e = found.entry(v.clone()).or_insert(f32::MIN);
                *e = e.max(*lp);
            }
        }
        if !known {
            let mut best: Vec<(&String, &f32)> = one.iter().collect();
            best.sort_by(|a, b| b.1.total_cmp(a.1));
            for (v, lp) in best.into_iter().take(120) {
                for (v2, lp2) in self.variants(v) {
                    if self.freq(&v2).is_some() {
                        let e = found.entry(v2).or_insert(f32::MIN);
                        *e = e.max(lp + lp2);
                    }
                }
            }
        }
        let list: Vec<(String, f32)> = found.into_iter().collect();
        self.cache.insert(w.to_string(), list.clone());
        list
    }
}

/// Clean up OCR output: glyph fixes, then dictionary correction of each Bangla word.
pub fn correct(text: &str) -> OcrResult {
    correct_with(text, OcrParams::default())
}

/// [`correct`] with chosen weights (for tuning).
pub fn correct_with(text: &str, p: OcrParams) -> OcrResult {
    let text = fix_glyphs(text);
    let Some(dict) = Dictionary::global() else {
        return OcrResult {
            text,
            changes: Vec::new(),
        };
    };
    let mut sc = Scorer {
        dict,
        rules: rules(),
        cache: HashMap::new(),
    };
    // Split into Bangla words and the text between them
    let mut pieces: Vec<(bool, String)> = Vec::new();
    for c in text.chars() {
        let b = is_bangla(c);
        match pieces.last_mut() {
            Some((pb, s)) if *pb == b => s.push(c),
            _ => pieces.push((b, c.to_string())),
        }
    }
    let words: Vec<usize> = (0..pieces.len()).filter(|i| pieces[*i].0).collect();
    let mut fixed: Vec<String> = Vec::with_capacity(words.len());
    for (k, &pi) in words.iter().enumerate() {
        let w = pieces[pi].1.clone();
        let prev = fixed.last().map(String::as_str);
        let next = words.get(k + 1).map(|&j| pieces[j].1.as_str());
        // part of a word written with an apostrophe (মা'তুফ, র'র): only a piece, not a word
        let glued = |i: Option<usize>, at_end: bool| {
            i.and_then(|i| pieces.get(i)).is_some_and(|(b, t)| {
                !*b && t.chars().count() == 1 && {
                    let c = t.chars().next().unwrap();
                    matches!(c, '\'' | '’' | '‘' | '`' | '"' | '“' | '”')
                        && pieces
                            .get(if at_end { pi + 2 } else { pi.wrapping_sub(2) })
                            .is_some_and(|(b, _)| *b)
                }
            })
        };
        // numbers with a letter (১৪ই, ৫ম), very short words and word pieces are left as read
        if w.chars().any(is_bangla_digit)
            || w.chars().count() < 2
            || glued(pi.checked_sub(1), false)
            || glued(Some(pi + 1), true)
        {
            fixed.push(w);
            continue;
        }
        let cands = sc.candidates(&w);
        if cands.is_empty() {
            fixed.push(w);
            continue;
        }
        let score = |sc: &Scorer, x: &str, chan: f32| -> f32 {
            sc.freq(x).unwrap_or(p.unknown)
                + p.channel * chan
                + p.context * (sc.pair(prev, Some(x)) + sc.pair(Some(x), next))
        };
        let keep = match sc.freq(&w) {
            Some(_) => score(&sc, &w, 0.0),
            None if sc.known_with_ending(&w) => {
                fixed.push(w);
                continue;
            }
            None => score(&sc, &w, 0.0),
        };
        let (best, bs) = cands
            .iter()
            .map(|(c, lp)| (c, score(&sc, c, *lp)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        if bs > keep + p.margin {
            fixed.push(best.clone());
        } else {
            fixed.push(w);
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut changes = Vec::new();
    let mut wi = 0;
    for (i, (b, s)) in pieces.iter().enumerate() {
        if *b {
            let new = &fixed[wi];
            if new != s {
                changes.push(OcrChange {
                    at: out.len(),
                    from: s.clone(),
                    to: new.clone(),
                });
            }
            out.push_str(new);
            wi += 1;
        } else {
            out.push_str(s);
        }
        let _ = i;
    }
    OcrResult { text: out, changes }
}

fn letters(w: &str) -> usize {
    w.chars().filter(|c| is_bangla_letter(*c)).count()
}

fn known_word(sc: &Scorer, w: &str) -> bool {
    sc.freq(w).is_some() || sc.known_with_ending(w)
}

/// One word as OCR read it, with its confidence (0–100).
pub type OcrWord = (String, f32);

const SHORT_WORDS: &[&str] = &[
    "এ",
    "ও",
    "ই",
    "উ",
    "আ",
    "অ",
    "কি",
    "না",
    "যে",
    "সে",
    "তা",
    "বা",
    "নয়",
    "হয়",
];

/// Is this word real text, or something OCR made of a picture, a rule, a stain or another script?
fn word_is_text(w: &str, conf: f32) -> bool {
    let core = w.trim_matches(|c: char| "()[]{}'\"‘’“”,.;:!?-—–…।॥".contains(c));
    if core.is_empty() {
        return conf >= 50.0; // punctuation
    }
    let ascii_digit = core.chars().any(|c| c.is_ascii_digit());
    let bn_digit = core.chars().any(is_bangla_digit);
    let bl = letters(core);
    let junk = core.chars().any(|c| "£€$%&*\\|<>©¢~^=+@#»«".contains(c));
    if ascii_digit && (bn_digit || bl > 0) {
        return false; // ১১7) চ1 — never printed
    }
    if bl == 0 {
        let number = core
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit() || is_bangla_digit(c))
            && core
                .chars()
                .all(|c| c.is_ascii_digit() || is_bangla_digit(c) || ".,:/-%".contains(c));
        if number {
            return conf >= if bn_digit { 30.0 } else { 50.0 };
        }
        return conf >= 80.0 && !junk;
    }
    if junk && conf < 50.0 {
        return false;
    }
    if bl == 1 && !SHORT_WORDS.contains(&core) {
        return conf >= 60.0;
    }
    true
}

/// The text of a page from OCR's lines (`None` between paragraphs), leaving out what OCR made of
/// pictures, decorations, stains and other scripts. Returns the text and what was left out.
pub fn clean_lines(lines: &[Option<Vec<OcrWord>>]) -> (String, Vec<String>) {
    let mut out: Vec<String> = Vec::new();
    let mut dropped = Vec::new();
    for line in lines {
        let Some(words) = line else {
            if out.last().is_some_and(|l| !l.is_empty()) {
                out.push(String::new());
            }
            continue;
        };
        if words.is_empty() {
            continue;
        }
        let conf = words.iter().map(|(_, c)| *c).sum::<f32>() / words.len() as f32;
        let kept: Vec<&str> = words
            .iter()
            .filter(|(w, c)| word_is_text(w, *c))
            .map(|(w, _)| w.as_str())
            .collect();
        let real = kept.iter().filter(|w| letters(w) >= 2).count();
        let number = kept.iter().any(|w| {
            let t = w.trim_matches(|c: char| "()[].,".contains(c));
            !t.is_empty()
                && t.chars()
                    .all(|c| c.is_ascii_digit() || is_bangla_digit(c) || ".,:/-".contains(c))
        });
        let all: Vec<&str> = words.iter().map(|(w, _)| w.as_str()).collect();
        if real == 0 && !(number && conf >= 70.0) {
            dropped.push(all.join(" "));
            continue;
        }
        if kept.len() < words.len() {
            let gone: Vec<&str> = words
                .iter()
                .filter(|(w, c)| !word_is_text(w, *c))
                .map(|(w, _)| w.as_str())
                .collect();
            dropped.push(gone.join(" "));
        }
        out.push(kept.join(" "));
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    (out.join("\n"), dropped)
}

/// Share of the Bangla words (two letters or more) that are dictionary words, and how many there are.
pub fn known_share(text: &str) -> (f32, usize) {
    let Some(dict) = Dictionary::global() else {
        return (0.0, 0);
    };
    let sc = Scorer {
        dict,
        rules: rules(),
        cache: HashMap::new(),
    };
    let mut n = 0;
    let mut k = 0;
    for w in text.split(|c: char| !is_bangla(c) || is_bangla_digit(c)) {
        if letters(w) < 2 {
            continue;
        }
        n += 1;
        if known_word(&sc, w) {
            k += 1;
        }
    }
    (if n > 0 { k as f32 / n as f32 } else { 0.0 }, n)
}

/// Where a PDF page's own text came from, when it can be used instead of reading the picture.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TextLayer {
    pub text: String,
    /// "unicode" or "bijoy" (typed with a Bijoy/SutonnyMJ font, converted to Unicode)
    pub kind: &'static str,
}

/// PDF readers drop the soft hyphen, which Bijoy fonts use for ল-ফলা (আল্লাহ comes out as আলাহ):
/// put it back where that turns an unknown word into a dictionary word.
fn restore_la_phala(text: &str) -> String {
    let Some(dict) = Dictionary::global() else {
        return text.to_string();
    };
    let sc = Scorer {
        dict,
        rules: rules(),
        cache: HashMap::new(),
    };
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        // unknown, or a rare spelling of a common ল্ল word (আলাহ 0.2 per million, আল্লাহ 100+)
        let have = if known_word(&sc, word) {
            sc.freq(word).unwrap_or(0.0)
        } else {
            f32::MIN
        };
        if word.contains('ল') && have < 2.0 {
            let chars: Vec<char> = word.chars().collect();
            for i in 0..chars.len() {
                if chars[i] == 'ল' && chars.get(i + 1) != Some(&'্') {
                    let v: String = chars[..=i]
                        .iter()
                        .chain(['্', 'ল'].iter())
                        .chain(chars[i + 1..].iter())
                        .collect();
                    let better = match sc.freq(&v) {
                        Some(f) => f > have + 4.0,
                        None => have == f32::MIN && known_word(&sc, &v),
                    };
                    if better {
                        out.push_str(&v);
                        word.clear();
                        return;
                    }
                }
            }
        }
        out.push_str(word);
        word.clear();
    };
    for c in text.chars() {
        if is_bangla(c) {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// A PDF page that carries its text (typed, not scanned) needs no OCR: use that text when it is
/// real Bangla, or Bijoy (ANSI) text that converts to real Bangla. Broken text layers (fonts with
/// no Unicode mapping, which is common for Bangla PDFs) fail the dictionary test and are read as
/// pictures instead.
pub fn text_layer(raw: &str) -> Option<TextLayer> {
    let raw = raw.replace(['\u{200C}', '\u{200D}'], "");
    let tidy = |s: &str| -> String {
        s.lines()
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    };
    let uni = tidy(&raw);
    let (us, un) = known_share(&uni);
    // Bijoy text is mostly Latin letters and symbols; only try the converter when it looks like it
    let latin = raw.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let bangla = raw.chars().filter(|c| is_bangla(*c)).count();
    if latin > bangla * 2 && latin >= 20 {
        let bij = restore_la_phala(&tidy(&crate::convert::bijoy_to_unicode(&raw)));
        let (bs, bn) = known_share(&bij);
        if bn >= 5 && bs >= 0.7 && (bs > us || un < 5) {
            return Some(TextLayer {
                text: bij,
                kind: "bijoy",
            });
        }
    }
    if un >= 5 && us >= 0.7 {
        return Some(TextLayer {
            text: uni,
            kind: "unicode",
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_look_alikes_are_fixed() {
        assert_eq!(fix_glyphs("সে বাড়ি গেল |"), "সে বাড়ি গেল।");
        assert_eq!(fix_glyphs("সে বাড়ি গেল৷ তারপর"), "সে বাড়ি গেল। তারপর");
        assert_eq!(fix_glyphs("বক্সার । তিনি"), "বক্সার। তিনি");
        assert_eq!(fix_glyphs("আ্যাসিড"), "অ্যাসিড");
        assert_eq!(fix_glyphs("উচ্চতা ৬.এ মিটার"), "উচ্চতা ৬.৫ মিটার");
        assert_eq!(fix_glyphs("সাল ১৭১এ"), "সাল ১৭১৫");
        // a real word stays: এক, ও
        assert_eq!(fix_glyphs("এক ও দুই"), "এক ও দুই");
    }

    #[test]
    fn misread_words_are_corrected_with_the_dictionary() {
        if Dictionary::global().is_none() {
            return;
        }
        let r = correct("১৪ই জুন আমাদের পুরনো এতিহ্য");
        assert_eq!(r.text, "১৪ই জুন আমাদের পুরনো ঐতিহ্য", "{:?}", r.changes);
        assert_eq!(r.changes.len(), 1);
        assert_eq!(r.changes[0].from, "এতিহ্য");
        assert_eq!(&r.text[r.changes[0].at..], "ঐতিহ্য");
        // a word written with an apostrophe is not taken apart
        let r = correct("মা'তুফ আলাইহী এবং মা'তুফ মিলে");
        assert!(r.changes.iter().all(|c| c.from != "তুফ"), "{:?}", r.changes);
    }

    #[test]
    fn pictures_and_other_scripts_are_left_out() {
        let w = |s: &str, c: f32| (s.to_string(), c);
        let lines = vec![
            Some(vec![w("চ", 40.0)]),
            Some(vec![w("\\", 80.0)]),
            Some(vec![
                w("১১7)", 70.0),
                w("হরফে", 90.0),
                w("জার", 92.0),
                w("£%££", 57.0),
            ]),
            None,
            Some(vec![w("আমি", 91.0), w("ভাত", 88.0), w("খাই।", 90.0)]),
            Some(vec![w("৭৬", 90.0)]),
        ];
        let (text, dropped) = clean_lines(&lines);
        assert_eq!(text, "হরফে জার\n\nআমি ভাত খাই।\n৭৬");
        assert_eq!(dropped.len(), 3, "{dropped:?}");
    }

    #[test]
    fn pdf_text_layers() {
        if Dictionary::global().is_none() {
            return;
        }
        // Bijoy (SutonnyMJ) text, as a PDF typed in Bijoy carries it
        let t =
            text_layer("Avwg evsjvq Mvb MvB| Avgvi †mvbvi evsjv, Avwg †Zvgvq fvjevwm|").unwrap();
        assert_eq!(t.kind, "bijoy");
        assert!(t.text.starts_with("আমি বাংলায় গান গাই।"), "{}", t.text);
        // the PDF reader lost the soft hyphen of আল্লাহ
        let t = text_layer("Avjvn Avcbvi Dci ing Z el©Y Kiæb| mKj cÖksmv Avjvni Rb¨|").unwrap();
        assert!(t.text.starts_with("আল্লাহ আপনার উপর"), "{}", t.text);
        let u = text_layer("আমি বাংলায় গান গাই। আমার সোনার বাংলা, আমি তোমায় ভালোবাসি।").unwrap();
        assert_eq!(u.kind, "unicode");
        // a broken text layer (glyph order lost) is not used
        assert!(text_layer("িাংো জহোবিরছ পইিিাে িথা বছে বকন্তুক কই পইিে নাই ি িান").is_none());
    }
}
