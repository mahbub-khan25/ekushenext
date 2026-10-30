//! Bijoy (ANSI / SutonnyMJ) <-> Unicode Converter & Banglish Selection Converter.
//!
//! Handles:
//! - Pre-base vowel signs (`ি`, `ে`, `ৈ`) which appear *before* the consonant cluster in Bijoy ANSI
//!   and *after* the consonant cluster in Unicode.
//! - Split vowel signs (`ে` + `া` <-> `ো`, `ে` + `ৗ` <-> `ৌ`).
//! - Reph (`র্`) which appears *after* the consonant cluster in Bijoy (`©`) and *before* it in Unicode.
//! - SutonnyMJ conjunct glyphs (`ÿ` <-> `ক্ষ`, `Á` <-> `জ্ঞ`, `¯^` <-> `স্ব`, `š¿` <-> `ন্ত্র`, ...).
//!
//! The Bijoy conversion itself lives in [`crate::bijoy`].

use crate::phonetic::PhoneticEngine;

/// Convert Banglish (Roman phonetic text) directly into Unicode Bangla.
pub fn banglish_to_bangla(input: &str) -> String {
    PhoneticEngine::new().transliterate(input)
}

/// What [`convert_selection`] did to a piece of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionKind {
    /// Old Bijoy (SutonnyMJ) text became Unicode Bangla
    BijoyToUnicode,
    /// Bangla written in Roman letters became Bangla
    BanglishToBangla,
}

/// Work out whether `text` is Bijoy (SutonnyMJ) text or Banglish. `None` when there is nothing to
/// convert: the text is already Bangla, or has no letters.
///
/// Bijoy text is plain ASCII plus a few Windows-1252 glyphs (`†` `‡` `©` `ÿ` …). Without those
/// glyphs it still looks unlike English or Banglish: `v` (া) is everywhere and vowels are rare
/// (`Avgvi evsjv` is আমার বাংলা).
pub fn detect_selection(text: &str) -> Option<SelectionKind> {
    if text.trim().is_empty() || text.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c)) {
        return None;
    }
    let letters: Vec<char> = text.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    if letters.is_empty() {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let bijoy_glyph = chars.iter().enumerate().any(|(i, &c)| match c {
        // Windows-1252 glyphs SutonnyMJ uses for Bangla signs (curly quotes and dashes are left out:
        // ordinary English text has them too)
        '†' | '‡' | 'ˆ' | '‰' | 'Š' | '‹' | 'Œ' | 'š' | '›' | 'œ' | 'Ÿ' | '™' => {
            true
        }
        // Reph: © straight after a letter
        '©' => i > 0 && chars[i - 1].is_ascii_alphabetic(),
        '®' | '°' | '×' | '÷' => false,
        c => ('\u{00A1}'..='\u{00FF}').contains(&c),
    });
    let n = letters.len() as f32;
    let v = letters.iter().filter(|&&c| c == 'v').count() as f32 / n;
    let vowels = letters.iter().filter(|&&c| "aeiou".contains(c)).count() as f32 / n;
    Some(if bijoy_glyph || (v > 0.12 && vowels < 0.35) {
        SelectionKind::BijoyToUnicode
    } else {
        SelectionKind::BanglishToBangla
    })
}

/// Replace each run of non-space characters in `text` by `f(run)`, keeping spaces and line breaks.
pub fn map_words(text: &str, mut f: impl FnMut(&str) -> String) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    let mut start = None;
    for (i, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if let Some(s) = start.take() {
                out.push_str(&f(&text[s..i]));
            }
            out.push(ch);
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        out.push_str(&f(&text[s..]));
    }
    out
}

/// Convert Bijoy (ANSI / SutonnyMJ) text to standard Unicode Bangla (see [`crate::bijoy`]).
pub fn bijoy_to_unicode(input: &str) -> String {
    crate::bijoy::to_unicode(input)
}

/// Canonicalize decomposed Bangla nukta sequences (`ড`+`়` -> `\u{09DC}`, `ঢ`+`়` -> `\u{09DD}`, `য`+`়` -> `\u{09DF}`)
/// so that NFC and NFD Bangla strings behave identically everywhere.
pub fn normalize_bangla_unicode(input: &str) -> String {
    input
        .replace("ড\u{09BC}", "\u{09DC}")
        .replace("ঢ\u{09BC}", "\u{09DD}")
        .replace("য\u{09BC}", "\u{09DF}")
}

/// Convert standard Unicode Bangla text into Bijoy (ANSI / SutonnyMJ) encoding (see [`crate::bijoy`]).
pub fn unicode_to_bijoy(input: &str) -> String {
    crate::bijoy::from_unicode(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reph_ra_fola_and_nukta() {
        for (uni, bijoy) in [
            ("কর্ম", "Kg©"),
            ("নির্বাচন", "wbe©vPb"),
            ("মূর্তি", "g~wZ©"),
            ("প্রথম", "cÖ_g"),
            ("কার্য", "Kvh©"),
            ("পর্নো", "c‡b©v"),
            ("ব্র্যাক", "eÖ¨vK"),
            ("পুনর্যৌবন", "cyb‡h©Šeb"),
        ] {
            assert_eq!(unicode_to_bijoy(uni), bijoy, "encode {uni}");
            assert_eq!(
                normalize_bangla_unicode(&bijoy_to_unicode(bijoy)),
                normalize_bangla_unicode(uni),
                "decode {bijoy}"
            );
        }
        // য় with a pre-base vowel sign keeps its dot on the letter
        for w in ["হয়েছে", "নিয়ে", "দিয়ে", "মেয়ে", "বড়ি"]
        {
            assert_eq!(
                normalize_bangla_unicode(&bijoy_to_unicode(&unicode_to_bijoy(w))),
                normalize_bangla_unicode(w),
                "{w}"
            );
        }
    }

    #[test]
    fn test_bijoy_unicode_roundtrip() {
        let sample = "আমি বাংলায় গান গাই। বাংলাদেশ আমার স্বদেশ।";
        let bijoy = unicode_to_bijoy(sample);
        assert_eq!(bijoy, "Avwg evsjvq Mvb MvB| evsjv‡`k Avgvi ¯^‡`k|");
        let back = bijoy_to_unicode(&bijoy);
        assert_eq!(
            normalize_bangla_unicode(&back),
            normalize_bangla_unicode(sample)
        );
    }
    #[test]
    fn selection_kind_is_recognised() {
        use SelectionKind::*;
        assert_eq!(detect_selection("Avgvi †mvbvi evsjv"), Some(BijoyToUnicode));
        assert_eq!(
            detect_selection("Avwg evsjvq Mvb MvB"),
            Some(BijoyToUnicode)
        );
        assert_eq!(detect_selection("Kg© I wbe©vPb"), Some(BijoyToUnicode));
        assert_eq!(
            detect_selection("amar sonar bangla"),
            Some(BanglishToBangla)
        );
        assert_eq!(detect_selection("valobashi tomay"), Some(BanglishToBangla));
        assert_eq!(
            detect_selection("kemon acho? (c) 2026"),
            Some(BanglishToBangla)
        );
        assert_eq!(detect_selection("আমার সোনার বাংলা"), None);
        assert_eq!(detect_selection("  123 "), None);
        assert_eq!(detect_selection(""), None);
    }
}
