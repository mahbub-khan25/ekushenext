//! Bangla Unicode normalisation and well-formedness checks.
//!
//! Canonical form used across Ekushe Next = Unicode NFC, which keeps the nukta letters
//! decomposed (য় = য + ়, ড় = ড + ়, ঢ় = ঢ + ়) because U+09DC/09DD/09DF are composition exclusions.

use unicode_normalization::UnicodeNormalization;

pub const HASANTA: char = '\u{09CD}';
pub const NUKTA: char = '\u{09BC}';
pub const CHANDRABINDU: char = '\u{0981}';
pub const ZWJ: char = '\u{200D}';
pub const ZWNJ: char = '\u{200C}';

#[inline]
pub fn is_independent_vowel(c: char) -> bool {
    matches!(c, '\u{0985}'..='\u{098C}' | '\u{098F}' | '\u{0990}' | '\u{0993}'..='\u{0994}' | '\u{09E0}' | '\u{09E1}')
}
#[inline]
pub fn is_consonant(c: char) -> bool {
    matches!(c, '\u{0995}'..='\u{09A8}' | '\u{09AA}'..='\u{09B0}' | '\u{09B2}' | '\u{09B6}'..='\u{09B9}' | '\u{09DC}' | '\u{09DD}' | '\u{09DF}')
}
#[inline]
pub fn is_vowel_sign(c: char) -> bool {
    matches!(c, '\u{09BE}'..='\u{09C4}' | '\u{09C7}' | '\u{09C8}' | '\u{09CB}' | '\u{09CC}' | '\u{09D7}' | '\u{09E2}' | '\u{09E3}')
}
#[inline]
pub fn is_modifier(c: char) -> bool {
    matches!(c, '\u{0981}'..='\u{0983}')
}
/// Characters that may appear inside a Bangla word token.
#[inline]
pub fn is_word_char(c: char) -> bool {
    is_independent_vowel(c)
        || is_consonant(c)
        || is_vowel_sign(c)
        || is_modifier(c)
        || c == HASANTA
        || c == NUKTA
        || c == '\u{09CE}'
        || c == ZWJ
        || c == ZWNJ
}

/// Normalise one raw token. Returns None if the result is not a well-formed Bangla word.
pub fn normalize_word(raw: &str) -> Option<String> {
    // 1. Legacy / mis-encoded sequences
    let mut s: String = raw.replace("\u{0985}\u{09BE}", "\u{0986}"); // অ + া -> আ
    // ত + ্ + ZWJ  -> ৎ (old khanda-ta encoding)
    s = s.replace("\u{09A4}\u{09CD}\u{200D}", "\u{09CE}");
    // Keep ZWJ only in র‍্য (ra + ZWJ + hasanta + ya); normalise র্‍য to that form.
    s = s.replace("\u{09B0}\u{09CD}\u{200D}\u{09AF}", "\u{09B0}\u{200D}\u{09CD}\u{09AF}");
    let keep_rya = "\u{09B0}\u{200D}\u{09CD}\u{09AF}";
    let marker = '\u{E000}';
    s = s.replace(keep_rya, &marker.to_string());
    s.retain(|c| c != ZWJ && c != ZWNJ);
    s = s.replace(marker, keep_rya);

    // 2. Chandrabindu must follow the vowel sign: চঁা -> চাঁ
    let mut chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i + 1 < chars.len() {
        if chars[i] == CHANDRABINDU && is_vowel_sign(chars[i + 1]) {
            chars.swap(i, i + 1);
        }
        i += 1;
    }
    // Remove doubled marks (ঁঁ, ংং, ্্, ়়)
    chars.dedup_by(|a, b| *a == *b && (is_modifier(*a) || *a == HASANTA || *a == NUKTA));

    // 3. Canonical NFC (composes ে+া -> ো, ে+ৗ -> ৌ, decomposes য়/ড়/ঢ়)
    let out: String = chars.into_iter().collect::<String>().nfc().collect();
    if is_well_formed(&out) {
        Some(out)
    } else {
        None
    }
}

/// Orthographic state machine for a Bangla word.
pub fn is_well_formed(w: &str) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Prev {
        Start,
        Vowel,     // independent vowel
        Cons,      // consonant (+ optional nukta)
        Sign,      // dependent vowel sign
        Hasanta,
        Mod,       // ং ঃ ঁ
        Khanda,
        Zwj,
    }
    let mut prev = Prev::Start;
    let mut prev_char = '\0';
    let mut n = 0usize;
    for c in w.chars() {
        n += 1;
        if n > 32 {
            return false;
        }
        prev = if is_independent_vowel(c) {
            if prev == Prev::Hasanta || prev == Prev::Zwj {
                return false;
            }
            Prev::Vowel
        } else if is_consonant(c) {
            Prev::Cons
        } else if c == NUKTA {
            if !(prev == Prev::Cons && matches!(prev_char, 'ড' | 'ঢ' | 'য')) {
                return false;
            }
            Prev::Cons
        } else if is_vowel_sign(c) {
            if prev != Prev::Cons {
                return false;
            }
            Prev::Sign
        } else if c == HASANTA {
            if !(prev == Prev::Cons || prev == Prev::Zwj) {
                return false;
            }
            Prev::Hasanta
        } else if c == ZWJ {
            if !(prev == Prev::Cons && prev_char == 'র') {
                return false;
            }
            Prev::Zwj
        } else if is_modifier(c) {
            if matches!(prev, Prev::Start | Prev::Hasanta | Prev::Zwj) {
                return false;
            }
            if prev == Prev::Mod && (c == prev_char || c == '\u{0983}') {
                return false;
            }
            Prev::Mod
        } else if c == '\u{09CE}' {
            if matches!(prev, Prev::Start | Prev::Hasanta | Prev::Zwj) {
                return false;
            }
            Prev::Khanda
        } else {
            return false;
        };
        prev_char = c;
    }
    n > 0 && prev != Prev::Hasanta && prev != Prev::Zwj
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalises_common_variants() {
        assert_eq!(normalize_word("চঁাদ").as_deref(), Some("চাঁদ"));
        assert_eq!(normalize_word("অাম").as_deref(), Some("আম"));
        assert_eq!(normalize_word("হঠাত্\u{200D}").as_deref(), Some("হঠাৎ"));
        assert_eq!(normalize_word("\u{09AC}\u{09C7}\u{09BE}\u{09A8}").as_deref(), Some("বোন"));
        assert_eq!(normalize_word("\u{09AE}\u{09C7}\u{09DF}\u{09C7}").as_deref(), Some("মেয়ে"));
        assert_eq!(normalize_word("র‍্যাব").as_deref(), Some("র\u{200D}্যাব"));
        assert_eq!(normalize_word("বাংলা\u{200C}").as_deref(), Some("বাংলা"));
    }
    #[test]
    fn rejects_malformed() {
        for bad in ["ািক", "ক্", "ংক", "কাা", "অ্ক", "ক়"] {
            assert!(normalize_word(bad).is_none(), "{bad}");
        }
    }
}
