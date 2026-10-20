//! Bijoy (ANSI / SutonnyMJ) <-> Unicode Converter & Banglish Selection Converter.
//!
//! Handles:
//! - Pre-base vowel signs (`ি`, `ে`, `ৈ`) which appear *before* the consonant cluster in Bijoy ANSI
//!   and *after* the consonant cluster in Unicode.
//! - Split vowel signs (`ে` + `া` <-> `ো`, `ে` + `ৗ` <-> `ৌ`).
//! - Reph (`র্`) which appears *after* the consonant cluster in Bijoy (`©`) and *before* it in Unicode.
//! - Common SutonnyMJ conjunct ligatures (`ÿ` <-> `ক্ষ`, `Á` <-> `জ্ঞ`, `³` <-> `ক্ত`, `›` + `˜` <-> `ন্দ`, etc.).

use crate::conjuncts::is_bangla_consonant;
use crate::phonetic::PhoneticEngine;

/// Convert Banglish (Roman phonetic text) directly into Unicode Bangla.
pub fn banglish_to_bangla(input: &str) -> String {
    PhoneticEngine::new().transliterate(input)
}

/// SutonnyMJ / Bijoy ANSI token -> Unicode mapping table (ordered longest-first).
const BIJOY_TO_UNI_TOKENS: &[(&str, &str)] = &[
    // Compound vowels
    ("অা", "আ"),
    ("আা", "আ"),
    // Conjuncts & special glyphs in SutonnyMJ
    ("ÿ", "ক্ষ"),
    ("Á", "জ্ঞ"),
    ("³", "ক্ত"),
    ("²", "ক্ক"),
    ("¶", "ক্ষ"),
    ("·", "ক্স"),
    ("¸", "গু"),
    ("¹", "গ্ধ"),
    ("º", "গ্ন"),
    ("»", "গ্ম"),
    ("¼", "ঙ্ক"),
    ("½", "ঙ্গ"),
    ("¾", "জ্জ"),
    ("¿", "্ত্র"),
    ("À", "জ্ঝ"),
    ("Â", "ঞ্চ"),
    ("Ã", "ঞ্ছ"),
    ("Ä", "ঞ্জ"),
    ("Å", "ঞ্ঝ"),
    ("Æ", "ট্ট"),
    ("Ç", "ড্ড"),
    ("È", "ণ্ট"),
    ("É", "ণ্ঠ"),
    ("Ê", "ণ্ড"),
    ("Ë", "ত্ত"),
    ("Ì", "ত্থ"),
    ("Í", "ত্ন"),
    ("Î", "ত্র"),
    ("Ï", "দ্দ"),
    ("Ð", "দ্ধ"),
    ("Ñ", "দ্ব"),
    ("Ò", "দ্ম"),
    ("Ó", "ধ্ব"),
    ("Ô", "ন্ট"),
    ("Õ", "ন্ঠ"),
    ("Ö", "ন্ড"),
    ("×", "ন্ত"),
    ("Ø", "ন্থ"),
    ("Ù", "ন্দ"),
    ("Ú", "ন্ধ"),
    ("Û", "ন্ন"),
    ("Ü", "প্ট"),
    ("Ý", "প্ত"),
    ("Þ", "প্ন"),
    ("ß", "প্প"),
    ("à", "ব্জ"),
    ("á", "ব্দ"),
    ("â", "ব্ধ"),
    ("ã", "ব্ব"),
    ("ä", "ভ্র"),
    ("å", "ম্প"),
    ("æ", "ম্ফ"),
    ("ç", "ম্ব"),
    ("è", "ম্ন"),
    ("é", "ম্ম"),
    ("ê", "ল্ক"),
    ("ë", "ল্গ"),
    ("ì", "ল্ট"),
    ("í", "ল্ড"),
    ("î", "ল্প"),
    ("ï", "শু"),
    ("ð", "শ্চ"),
    ("ñ", "শ্ন"),
    ("ò", "শ্ম"),
    ("ó", "ষ্ট"),
    ("ô", "ষ্ঠ"),
    ("õ", "ষ্ণ"),
    ("ö", "ষ্প"),
    ("÷", "ষ্ফ"),
    ("ø", "স্ক"),
    ("ù", "স্ট"),
    ("ú", "স্ত"),
    ("û", "স্থ"),
    ("ü", "স্ন"),
    ("ý", "স্প"),
    ("þ", "স্ফ"),
    ("Ÿ", "স্ব"),
    ("ü", "হু"),
    ("ý", "হৃ"),
    // Folas & marks in SutonnyMJ
    ("¨", "্য"),
    ("Ö", "্র"),
    ("«", "্র"),
    ("ª", "্র"),
    ("©", "র্"),
    ("¯", "্ব"),
    ("&", "্"),
    // Vowels & Kars
    ("A", "অ"),
    ("B", "ই"),
    ("C", "ঈ"),
    ("D", "উ"),
    ("E", "ঊ"),
    ("F", "ঋ"),
    ("G", "এ"),
    ("H", "ঐ"),
    ("I", "ও"),
    ("J", "ঔ"),
    ("v", "া"),
    ("w", "ি"),
    ("x", "ী"),
    ("y", "ু"),
    ("z", "ু"),
    ("~", "ূ"),
    ("„", "ৃ"),
    ("†", "ে"),
    ("‡", "ে"),
    ("ˆ", "ৈ"),
    ("Š", "ৌ"),
    // Consonants
    ("K", "ক"),
    ("L", "খ"),
    ("M", "গ"),
    ("N", "ঘ"),
    ("O", "ঙ"),
    ("P", "চ"),
    ("Q", "ছ"),
    ("R", "জ"),
    ("S", "ঝ"),
    ("T", "ঞ"),
    ("U", "ট"),
    ("V", "ঠ"),
    ("W", "ড"),
    ("X", "ঢ"),
    ("Y", "ণ"),
    ("Z", "ত"),
    ("_", "থ"),
    ("`", "দ"),
    ("a", "ধ"),
    ("b", "ন"),
    ("c", "প"),
    ("d", "ফ"),
    ("e", "ব"),
    ("f", "ভ"),
    ("g", "ম"),
    ("h", "য"),
    ("i", "র"),
    ("j", "ল"),
    ("k", "শ"),
    ("l", "ষ"),
    ("m", "স"),
    ("n", "হ"),
    ("o", "ড়"),
    ("p", "ঢ়"),
    ("q", "য়"),
    ("r", "ৎ"),
    ("s", "ং"),
    ("t", "ঃ"),
    ("u", "ঁ"),
    ("|", "।"),
    // Digits
    ("0", "০"),
    ("1", "১"),
    ("2", "২"),
    ("3", "৩"),
    ("4", "৪"),
    ("5", "৫"),
    ("6", "৬"),
    ("7", "৭"),
    ("8", "৮"),
    ("9", "৯"),
];

/// Convert Bijoy (ANSI / SutonnyMJ) encoded string to standard Unicode Bangla.
pub fn bijoy_to_unicode(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Step 1: Token-level glyph translation
    let mut raw = String::with_capacity(input.len() * 3);
    let mut idx = 0;
    while idx < input.len() {
        let rem = &input[idx..];
        let mut matched = false;
        for &(b_tok, u_tok) in BIJOY_TO_UNI_TOKENS {
            if rem.starts_with(b_tok) {
                raw.push_str(u_tok);
                idx += b_tok.len();
                matched = true;
                break;
            }
        }
        if !matched {
            let ch = rem.chars().next().unwrap();
            raw.push(ch);
            idx += ch.len_utf8();
        }
    }

    // Step 2: Reorder pre-base kars (`ি`, `ে`, `ৈ`) from before consonant cluster to after consonant cluster,
    // reposition post-cluster Reph (`র্`) to before the cluster, and join `অ`+`া` -> `আ` and `ে`+`া` -> `ো`.
    let chars: Vec<char> = raw.chars().collect();
    let mut reordered: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        // `অ` + `া` -> `আ`
        if ch == 'অ' && chars.get(i + 1) == Some(&'া') {
            reordered.push('আ');
            i += 2;
            continue;
        }

        // Pre-base kar (`ি`, `ে`, `ৈ`) before a consonant (or consonant cluster `C (্ C)*`)
        if matches!(ch, 'ি' | 'ে' | 'ৈ') && chars.get(i + 1).copied().is_some_and(is_bangla_consonant)
        {
            let kar = ch;
            i += 1;
            // Consume the consonant cluster `C (্ C)*`
            reordered.push(chars[i]);
            i += 1;
            while i + 1 < chars.len() && chars[i] == '্' && is_bangla_consonant(chars[i + 1]) {
                reordered.push('্');
                reordered.push(chars[i + 1]);
                i += 2;
            }
            // If Reph (`র` + `্`) immediately follows in Bijoy, place `র্` BEFORE the cluster
            if i + 1 < chars.len() && chars[i] == 'র' && chars[i + 1] == '্' {
                // Find start of cluster we just pushed
                let cluster_start = find_cluster_start(&reordered);
                reordered.insert(cluster_start, '্');
                reordered.insert(cluster_start, 'র');
                i += 2;
            }
            // If `ে` is followed by `া` -> `ো`, or `ৗ` -> `ৌ`
            if kar == 'ে' && chars.get(i) == Some(&'া') {
                reordered.push('ো');
                i += 1;
            } else if kar == 'ে' && matches!(chars.get(i), Some(&'ৗ') | Some(&'ৌ')) {
                reordered.push('ৌ');
                i += 1;
            } else {
                reordered.push(kar);
            }
            continue;
        }

        reordered.push(ch);
        i += 1;
    }

    reordered.into_iter().collect()
}

/// Canonicalize decomposed Bangla nukta sequences (`ড`+`়` -> `\u{09DC}`, `ঢ`+`়` -> `\u{09DD}`, `য`+`়` -> `\u{09DF}`)
/// so that NFC and NFD Bangla strings behave identically everywhere.
pub fn normalize_bangla_unicode(input: &str) -> String {
    input
        .replace("ড\u{09BC}", "\u{09DC}")
        .replace("ঢ\u{09BC}", "\u{09DD}")
        .replace("য\u{09BC}", "\u{09DF}")
}

/// Convert standard Unicode Bangla string into Bijoy (ANSI / SutonnyMJ) encoding.
pub fn unicode_to_bijoy(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let normalized = normalize_bangla_unicode(input);

    // Step 1: Decompose compound kars (`ো` -> `ে` + `া`, `ৌ` -> `ে` + `ৌ`, `আ` -> `অ` + `া`)
    // and move pre-base kars (`ি`, `ে`, `ৈ`) BEFORE the preceding consonant cluster.
    let chars: Vec<char> = normalized.chars().collect();
    let mut stage: Vec<char> = Vec::with_capacity(chars.len() + 8);

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            'আ' => {
                stage.push('অ');
                stage.push('া');
            }
            'ো' => {
                let start = find_cluster_start(&stage);
                stage.insert(start, 'ে');
                stage.push('া');
            }
            'ৌ' => {
                let start = find_cluster_start(&stage);
                stage.insert(start, 'ে');
                stage.push('Š');
            }
            'ি' | 'ে' | 'ৈ' => {
                let start = find_cluster_start(&stage);
                stage.insert(start, ch);
            }
            _ => stage.push(ch),
        }
        i += 1;
    }

    // Step 2: Map Unicode sequences to Bijoy tokens
    let stage_str: String = stage.into_iter().collect();
    const UNI_TO_BIJOY: &[(&str, &str)] = &[
        ("ক্ষ", "ÿ"),
        ("জ্ঞ", "Á"),
        ("ক্ত", "³"),
        ("ক্ক", "²"),
        ("ঙ্গ", "½"),
        ("ঙ্ক", "¼"),
        ("ঞ্চ", "Â"),
        ("ঞ্জ", "Ä"),
        ("ত্ত", "Ë"),
        ("ত্র", "Î"),
        ("দ্দ", "Ï"),
        ("দ্ধ", "Ð"),
        ("দ্ব", "Ñ"),
        ("ন্ত", "×"),
        ("ন্দ", "Ù"),
        ("ন্ধ", "Ú"),
        ("ন্ন", "Û"),
        ("ম্প", "å"),
        ("ম্ব", "ç"),
        ("শ্চ", "ð"),
        ("ষ্ট", "ó"),
        ("ষ্ঠ", "ô"),
        ("ষ্ণ", "õ"),
        ("স্ত", "ú"),
        ("স্থ", "û"),
        ("স্ব", "Ÿ"),
        ("্য", "¨"),
        ("্র", "Ö"),
        ("্", "&"),
        ("অ", "A"),
        ("ই", "B"),
        ("ঈ", "C"),
        ("উ", "D"),
        ("ঊ", "E"),
        ("ঋ", "F"),
        ("এ", "G"),
        ("ঐ", "H"),
        ("ও", "I"),
        ("ঔ", "J"),
        ("া", "v"),
        ("ি", "w"),
        ("ী", "x"),
        ("ু", "y"),
        ("ূ", "~"),
        ("ৃ", "„"),
        ("ে", "†"),
        ("ৈ", "ˆ"),
        ("ক", "K"),
        ("খ", "L"),
        ("গ", "M"),
        ("ঘ", "N"),
        ("ঙ", "O"),
        ("চ", "P"),
        ("ছ", "Q"),
        ("জ", "R"),
        ("ঝ", "S"),
        ("ঞ", "T"),
        ("ট", "U"),
        ("ঠ", "V"),
        ("ড", "W"),
        ("ঢ", "X"),
        ("ণ", "Y"),
        ("ত", "Z"),
        ("থ", "_"),
        ("দ", "`"),
        ("ধ", "a"),
        ("ন", "b"),
        ("প", "c"),
        ("ফ", "d"),
        ("ব", "e"),
        ("ভ", "f"),
        ("ম", "g"),
        ("য", "h"),
        ("র", "i"),
        ("ল", "j"),
        ("শ", "k"),
        ("ষ", "l"),
        ("স", "m"),
        ("হ", "n"),
        ("ড়", "o"),
        ("ঢ়", "p"),
        ("য়", "q"),
        ("\u{09DC}", "o"),
        ("\u{09DD}", "p"),
        ("\u{09DF}", "q"),
        ("ৎ", "r"),
        ("ং", "s"),
        ("ঃ", "t"),
        ("ঁ", "u"),
        ("।", "|"),
        ("০", "0"),
        ("১", "1"),
        ("২", "2"),
        ("৩", "3"),
        ("৪", "4"),
        ("৫", "5"),
        ("৬", "6"),
        ("৭", "7"),
        ("৮", "8"),
        ("৯", "9"),
    ];

    let mut out = String::with_capacity(stage_str.len());
    let mut pos = 0;
    while pos < stage_str.len() {
        let rem = &stage_str[pos..];
        let mut matched = false;
        for &(u_tok, b_tok) in UNI_TO_BIJOY {
            if rem.starts_with(u_tok) {
                out.push_str(b_tok);
                pos += u_tok.len();
                matched = true;
                break;
            }
        }
        if !matched {
            let ch = rem.chars().next().unwrap();
            out.push(ch);
            pos += ch.len_utf8();
        }
    }
    out
}

fn find_cluster_start(buf: &[char]) -> usize {
    if buf.is_empty() {
        return 0;
    }
    let mut idx = buf.len() - 1;
    if !is_bangla_consonant(buf[idx]) {
        return buf.len();
    }
    while idx >= 2 && buf[idx - 1] == '্' && is_bangla_consonant(buf[idx - 2]) {
        idx -= 2;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bijoy_unicode_roundtrip() {
        let sample = "আমি বাংলায় গান গাই। বাংলাদেশ আমার স্বদেশ।";
        let bijoy = unicode_to_bijoy(sample);
        assert_eq!(bijoy, "Avwg evsjvq Mvb MvB| evsjv†`k Avgvi Ÿ†`k|");
        let back = bijoy_to_unicode(&bijoy);
        assert_eq!(normalize_bangla_unicode(&back), normalize_bangla_unicode(sample));
    }
}
