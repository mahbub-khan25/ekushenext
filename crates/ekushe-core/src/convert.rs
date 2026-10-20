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

/// Internal placeholder for reph (`র্`) while converting; Bijoy writes it as `©` after the consonant.
const REPH_MARK: char = '\u{E000}';

#[inline]
fn is_vowel_sign(c: char) -> bool {
    matches!(c, 'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ' | 'ৗ')
}

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
    // `Ö` is র-ফলা (cÖ = প্র), listed with the folas below; it is not ন্ড
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
    ("©", "\u{E000}"), // reph: written after its consonant in Bijoy, moved before it below
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
            // Consume the consonant cluster `C (়)? (্ C (়)?)*`; a nukta belongs to its consonant (য়, ড়, ঢ়)
            reordered.push(chars[i]);
            i += 1;
            if chars.get(i) == Some(&'\u{09BC}') {
                reordered.push('\u{09BC}');
                i += 1;
            }
            while i + 1 < chars.len() && chars[i] == '্' && is_bangla_consonant(chars[i + 1]) {
                reordered.push('্');
                reordered.push(chars[i + 1]);
                i += 2;
                if chars.get(i) == Some(&'\u{09BC}') {
                    reordered.push('\u{09BC}');
                    i += 1;
                }
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

        // Reph mark (`©`) after a consonant cluster (and its vowel signs): `র্` goes before the cluster
        if ch == REPH_MARK {
            let mut k = reordered.len();
            while k > 0 && is_vowel_sign(reordered[k - 1]) {
                k -= 1;
            }
            let start = find_cluster_start(&reordered[..k]);
            reordered.insert(start, '্');
            reordered.insert(start, 'র');
            i += 1;
            continue;
        }

        reordered.push(ch);
        i += 1;
    }

    // Join split vowel signs that a reph mark kept apart (পর্নো: `ে` … `©` … `া` → `ো`)
    let joined: String = reordered.into_iter().collect();
    joined
        .replace("\u{09C7}\u{09BE}", "\u{09CB}")
        .replace("\u{09C7}\u{09D7}", "\u{09CC}")
        .replace("\u{09C7}\u{09CC}", "\u{09CC}")
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
            // Reph `র্` + consonant cluster: Bijoy writes the cluster first, then `©`
            // (only at the start of a cluster: in ব্র্য, ত্র্য the র is a ফলা, not a reph)
            'র' if (i == 0 || chars[i - 1] != '্')
                && chars.get(i + 1) == Some(&'্')
                && chars.get(i + 2).copied().is_some_and(is_bangla_consonant) =>
            {
                i += 2;
                stage.push(chars[i]);
                while i + 2 < chars.len() && chars[i + 1] == '্' && is_bangla_consonant(chars[i + 2]) {
                    stage.push('্');
                    stage.push(chars[i + 2]);
                    i += 2;
                }
                stage.push(REPH_MARK);
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
        ("\u{E000}", "©"),
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
    // a trailing reph mark belongs to the cluster before it
    let buf = if buf.last() == Some(&REPH_MARK) { &buf[..buf.len() - 1] } else { buf };
    // consonant (with optional nukta) ending at position `end` (exclusive): returns its start
    let cons_start = |end: usize| -> Option<usize> {
        if end >= 2 && buf[end - 1] == '\u{09BC}' && is_bangla_consonant(buf[end - 2]) {
            Some(end - 2)
        } else if end >= 1 && is_bangla_consonant(buf[end - 1]) {
            Some(end - 1)
        } else {
            None
        }
    };
    let Some(mut idx) = cons_start(buf.len()) else {
        return buf.len();
    };
    while idx >= 1 && buf[idx - 1] == '্' {
        match cons_start(idx - 1) {
            Some(s) => idx = s,
            None => break,
        }
    }
    idx
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
            ("পর্নো", "c†b©v"),
            ("ব্র্যাক", "eÖ¨vK"),
            ("পুনর্যৌবন", "cyb†h©Šeb"),
        ] {
            assert_eq!(unicode_to_bijoy(uni), bijoy, "encode {uni}");
            assert_eq!(normalize_bangla_unicode(&bijoy_to_unicode(bijoy)), normalize_bangla_unicode(uni), "decode {bijoy}");
        }
        // য় with a pre-base vowel sign keeps its dot on the letter
        for w in ["হয়েছে", "নিয়ে", "দিয়ে", "মেয়ে", "বড়ি"] {
            assert_eq!(normalize_bangla_unicode(&bijoy_to_unicode(&unicode_to_bijoy(w))), normalize_bangla_unicode(w), "{w}");
        }
    }

    #[test]
    fn test_bijoy_unicode_roundtrip() {
        let sample = "আমি বাংলায় গান গাই। বাংলাদেশ আমার স্বদেশ।";
        let bijoy = unicode_to_bijoy(sample);
        assert_eq!(bijoy, "Avwg evsjvq Mvb MvB| evsjv†`k Avgvi Ÿ†`k|");
        let back = bijoy_to_unicode(&bijoy);
        assert_eq!(normalize_bangla_unicode(&back), normalize_bangla_unicode(sample));
    }
}
