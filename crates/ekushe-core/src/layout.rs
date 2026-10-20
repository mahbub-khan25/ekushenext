//! Fixed Keyboard Layout Engine (Probhat, National/Jatiya, Bornona, Munir Optima, + Custom JSON)
//! with Smart Sign & Connected-Letter Automation:
//! - Automatic pre-base kar (`ি`, `ে`, `ৈ`) reordering (can type before OR after the consonant!)
//! - Automatic compound kar joining (`ে` + `া` -> `ো`, `ে` + `ৗ` -> `ৌ`, `অ` + `া` -> `আ`)
//! - Automatic Chandrabindu (`ঁ`) canonical ordering (`ক` + `ঁ` + `া` -> `কাঁ`)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::conjuncts::{is_bangla_consonant, is_bangla_kar};

/// Built-in layout identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayoutId {
    #[default]
    AvroPhonetic,
    Probhat,
    NationalJatiya,
    Bornona,
    MunirOptima,
}

impl LayoutId {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AvroPhonetic => "Ekushe Phonetic (Avro-compatible)",
            Self::Probhat => "প্রভাত (Probhat)",
            Self::NationalJatiya => "জাতীয় (National / Jatiya)",
            Self::Bornona => "বর্ণনা (Bornona)",
            Self::MunirOptima => "মুনীর অপটিমা (Munir Optima)",
        }
    }
}

/// Typing automation flags for fixed layouts (makes signs and connected letters easy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutAutomation {
    /// Allow typing `ি`, `ে`, `ৈ` before the consonant and auto-reorder to valid Unicode
    pub auto_prebase_kar_reorder: bool,
    /// Join `অ` + `া` -> `আ`, `ে` + `া` -> `ো`, `ে` + `ৗ` -> `ৌ`
    pub auto_vowel_forming: bool,
    /// Ensure `ঁ` (Chandrabindu) is placed after any kar (`কঁা` -> `কাঁ`)
    pub auto_chandrabindu_fix: bool,
}

impl Default for LayoutAutomation {
    fn default() -> Self {
        Self {
            auto_prebase_kar_reorder: true,
            auto_vowel_forming: true,
            auto_chandrabindu_fix: true,
        }
    }
}

/// Key mapping definition for a fixed layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedLayout {
    pub id: String,
    pub name: String,
    /// Normal / Shift ASCII char -> Bangla string
    pub normal: HashMap<char, String>,
    /// AltGr (Ctrl+Alt) char -> Bangla string
    pub altgr: HashMap<char, String>,
}

impl FixedLayout {
    pub fn builtin(id: LayoutId) -> Option<Self> {
        match id {
            LayoutId::AvroPhonetic => None,
            LayoutId::Probhat => Some(Self::probhat()),
            LayoutId::NationalJatiya => Some(Self::national_jatiya()),
            LayoutId::Bornona => Some(Self::bornona()),
            LayoutId::MunirOptima => Some(Self::munir_optima()),
        }
    }

    /// Standard Probhat layout mapping.
    pub fn probhat() -> Self {
        let pairs: &[(char, &str)] = &[
            ('1', "১"),
            ('2', "২"),
            ('3', "৩"),
            ('4', "৪"),
            ('5', "৫"),
            ('6', "৬"),
            ('7', "৭"),
            ('8', "৮"),
            ('9', "৯"),
            ('0', "০"),
            ('!', "!"),
            ('@', "@"),
            ('#', "#"),
            ('$', "৳"),
            ('%', "%"),
            ('^', "ঁ"),
            ('&', "ঞ"),
            ('*', "ৎ"),
            ('`', "‍"),
            ('~', "‌"),
            ('q', "দ"),
            ('Q', "ধ"),
            ('w', "ূ"),
            ('W', "ঊ"),
            ('e', "ী"),
            ('E', "ঈ"),
            ('r', "র"),
            ('R', "ড়"),
            ('t', "ট"),
            ('T', "ঠ"),
            ('y', "এ"),
            ('Y', "ঐ"),
            ('u', "ু"),
            ('U', "উ"),
            ('i', "ি"),
            ('I', "ই"),
            ('o', "ও"),
            ('O', "ঔ"),
            ('p', "প"),
            ('P', "ফ"),
            ('[', "ে"),
            ('{', "ৈ"),
            (']', "ো"),
            ('}', "ৌ"),
            ('a', "া"),
            ('A', "অ"),
            ('s', "স"),
            ('S', "ষ"),
            ('d', "ড"),
            ('D', "ঢ"),
            ('f', "ত"),
            ('F', "থ"),
            ('g', "গ"),
            ('G', "ঘ"),
            ('h', "হ"),
            ('H', "ঃ"),
            ('j', "জ"),
            ('J', "ঝ"),
            ('k', "ক"),
            ('K', "খ"),
            ('l', "ল"),
            ('L', "ং"),
            ('z', "য়"),
            ('Z', "য"),
            ('x', "শ"),
            ('X', "ঢ়"),
            ('c', "চ"),
            ('C', "ছ"),
            ('v', "আ"),
            ('V', "ঋ"),
            ('b', "ব"),
            ('B', "ভ"),
            ('n', "ন"),
            ('N', "ণ"),
            ('m', "ম"),
            ('M', "ঙ"),
            ('/', "্"),
            ('?', "?"),
            ('.', "।"),
            ('<', "ৃ"),
        ];
        Self::from_slice("probhat", "প্রভাত (Probhat)", pairs, &[])
    }

    /// Bangladesh National (Jatiya) layout mapping.
    pub fn national_jatiya() -> Self {
        let pairs: &[(char, &str)] = &[
            ('1', "১"),
            ('2', "২"),
            ('3', "৩"),
            ('4', "৪"),
            ('5', "৫"),
            ('6', "৬"),
            ('7', "৭"),
            ('8', "৮"),
            ('9', "৯"),
            ('0', "০"),
            ('$', "৳"),
            ('^', "ঁ"),
            ('&', "্"),
            ('q', "ঙ"),
            ('Q', "ং"),
            ('w', "য"),
            ('W', "য়"),
            ('e', "ড"),
            ('E', "ঢ"),
            ('r', "প"),
            ('R', "ফ"),
            ('t', "ট"),
            ('T', "ঠ"),
            ('y', "চ"),
            ('Y', "ছ"),
            ('u', "জ"),
            ('U', "ঝ"),
            ('i', "হ"),
            ('I', "ঞ"),
            ('o', "গ"),
            ('O', "ঘ"),
            ('p', "ড়"),
            ('P', "ঢ়"),
            ('a', "ৃ"),
            ('A', "র্"),
            ('s', "ু"),
            ('S', "ূ"),
            ('d', "ি"),
            ('D', "ী"),
            ('f', "া"),
            ('F', "অ"),
            ('g', "্"),
            ('G', "।"),
            ('h', "ব"),
            ('H', "ভ"),
            ('j', "ক"),
            ('J', "খ"),
            ('k', "ত"),
            ('K', "থ"),
            ('l', "দ"),
            ('L', "ধ"),
            ('z', "্র"),
            ('Z', "্য"),
            ('x', "ও"),
            ('X', "ৌ"),
            ('c', "ে"),
            ('C', "ৈ"),
            ('v', "র"),
            ('V', "ল"),
            ('b', "ন"),
            ('B', "ণ"),
            ('n', "স"),
            ('N', "ষ"),
            ('m', "ম"),
            ('M', "শ"),
            ('.', "।"),
        ];
        let altgr: &[(char, &str)] = &[
            ('f', "আ"),
            ('d', "ই"),
            ('D', "ঈ"),
            ('s', "উ"),
            ('S', "ঊ"),
            ('a', "ঋ"),
            ('c', "এ"),
            ('C', "ঐ"),
            ('x', "ও"),
            ('X', "ঔ"),
        ];
        Self::from_slice("national", "জাতীয় (National / Jatiya)", pairs, altgr)
    }

    /// Bornona layout mapping.
    pub fn bornona() -> Self {
        let pairs: &[(char, &str)] = &[
            ('1', "১"),
            ('2', "২"),
            ('3', "৩"),
            ('4', "৪"),
            ('5', "৫"),
            ('6', "৬"),
            ('7', "৭"),
            ('8', "৮"),
            ('9', "৯"),
            ('0', "০"),
            ('$', "৳"),
            ('^', "ঁ"),
            ('q', "ঙ"),
            ('Q', "ঞ"),
            ('w', "য"),
            ('W', "য়"),
            ('e', "ে"),
            ('E', "ৈ"),
            ('r', "র"),
            ('R', "ড়"),
            ('t', "ট"),
            ('T', "ঠ"),
            ('y', "ত"),
            ('Y', "থ"),
            ('u', "ু"),
            ('U', "ূ"),
            ('i', "ি"),
            ('I', "ী"),
            ('o', "ো"),
            ('O', "ৌ"),
            ('p', "প"),
            ('P', "ফ"),
            ('a', "া"),
            ('A', "অ"),
            ('s', "স"),
            ('S', "শ"),
            ('d', "দ"),
            ('D', "ধ"),
            ('f', "ড"),
            ('F', "ঢ"),
            ('g', "গ"),
            ('G', "ঘ"),
            ('h', "হ"),
            ('H', "ঃ"),
            ('j', "জ"),
            ('J', "ঝ"),
            ('k', "ক"),
            ('K', "খ"),
            ('l', "ল"),
            ('L', "ৃ"),
            ('z', "্"),
            ('Z', "ৎ"),
            ('x', "ষ"),
            ('X', "ঢ়"),
            ('c', "চ"),
            ('C', "ছ"),
            ('v', "্র"),
            ('V', "্য"),
            ('b', "ব"),
            ('B', "ভ"),
            ('n', "ন"),
            ('N', "ণ"),
            ('m', "ম"),
            ('M', "ং"),
            ('.', "।"),
        ];
        Self::from_slice("bornona", "বর্ণনা (Bornona)", pairs, &[])
    }

    /// Munir Optima layout mapping.
    pub fn munir_optima() -> Self {
        let pairs: &[(char, &str)] = &[
            ('1', "১"),
            ('2', "২"),
            ('3', "৩"),
            ('4', "৪"),
            ('5', "৫"),
            ('6', "৬"),
            ('7', "৭"),
            ('8', "৮"),
            ('9', "৯"),
            ('0', "০"),
            ('q', "স"),
            ('Q', "ষ"),
            ('w', "ড়"),
            ('W', "ঢ়"),
            ('e', "দ"),
            ('E', "ধ"),
            ('r', "ত"),
            ('R', "থ"),
            ('t', "ট"),
            ('T', "ঠ"),
            ('y', "চ"),
            ('Y', "ছ"),
            ('u', "জ"),
            ('U', "ঝ"),
            ('i', "হ"),
            ('I', "ঞ"),
            ('o', "গ"),
            ('O', "ঘ"),
            ('p', "প"),
            ('P', "ফ"),
            ('a', "ো"),
            ('A', "ৌ"),
            ('s', "ে"),
            ('S', "ৈ"),
            ('d', "ি"),
            ('D', "ী"),
            ('f', "া"),
            ('F', "অ"),
            ('g', "্"),
            ('G', "।"),
            ('h', "ব"),
            ('H', "ভ"),
            ('j', "ক"),
            ('J', "খ"),
            ('k', "র"),
            ('K', "ল"),
            ('l', "ন"),
            ('L', "ণ"),
            ('z', "্র"),
            ('Z', "্য"),
            ('x', "ু"),
            ('X', "ূ"),
            ('c', "ৃ"),
            ('C', "ঁ"),
            ('v', "আ"),
            ('V', "ই"),
            ('b', "ম"),
            ('B', "শ"),
            ('n', "য"),
            ('N', "য়"),
            ('m', "ঙ"),
            ('M', "ং"),
            ('.', "।"),
        ];
        Self::from_slice("munir_optima", "মুনীর অপটিমা (Munir Optima)", pairs, &[])
    }

    fn from_slice(id: &str, name: &str, normal: &[(char, &str)], altgr: &[(char, &str)]) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            normal: normal.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
            altgr: altgr.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
        }
    }

    /// Apply a keystroke in this fixed layout to an existing composition buffer,
    /// performing smart sign/kar reordering and compound vowel joining.
    pub fn apply_key(
        &self,
        buffer: &mut String,
        pending_prebase_kar: &mut Option<char>,
        ch: char,
        altgr: bool,
        automation: &LayoutAutomation,
    ) {
        let mapped = if altgr {
            self.altgr.get(&ch).or_else(|| self.normal.get(&ch))
        } else {
            self.normal.get(&ch)
        };

        let Some(piece) = mapped else {
            if let Some(pk) = pending_prebase_kar.take() {
                buffer.push(pk);
            }
            buffer.push(ch);
            return;
        };

        for out_ch in piece.chars() {
            self.push_smart_char(buffer, pending_prebase_kar, out_ch, automation);
        }
    }

    fn push_smart_char(
        &self,
        buffer: &mut String,
        pending_prebase_kar: &mut Option<char>,
        out_ch: char,
        automation: &LayoutAutomation,
    ) {
        // 1. Pre-base kar (`ি`, `ে`, `ৈ`) typed BEFORE a consonant: hold in pending_prebase_kar
        if automation.auto_prebase_kar_reorder && matches!(out_ch, 'ি' | 'ে' | 'ৈ') {
            let last = buffer.chars().last();
            let after_consonant = last.is_some_and(is_bangla_consonant);
            if !after_consonant {
                *pending_prebase_kar = Some(out_ch);
                return;
            }
        }

        // 2. If a consonant arrives while a pre-base kar is pending, emit consonant first,
        //    unless the next character is `্` (hasanta) to form a conjunct!
        if is_bangla_consonant(out_ch) {
            buffer.push(out_ch);
            if let Some(pk) = pending_prebase_kar.take() {
                buffer.push(pk);
            }
            return;
        }

        // 3. If `্` (hasanta) is typed right after a consonant that just absorbed a pending pre-base kar,
        //    move the pre-base kar back to pending so the entire conjunct gets the kar at the end!
        if out_ch == '্' && automation.auto_prebase_kar_reorder {
            if let Some(last) = buffer.chars().last() {
                if matches!(last, 'ি' | 'ে' | 'ৈ') {
                    buffer.pop();
                    *pending_prebase_kar = Some(last);
                }
            }
            buffer.push('্');
            return;
        }

        // Flush any remaining pending kar before non-consonants
        if let Some(pk) = pending_prebase_kar.take() {
            buffer.push(pk);
        }

        // 4. Auto vowel forming (`অ` + `া` -> `আ`, `ে` + `া` -> `ো`, `ে` + `ৗ` -> `ৌ`)
        if automation.auto_vowel_forming {
            if let Some(last) = buffer.chars().last() {
                match (last, out_ch) {
                    ('অ', 'া') => {
                        buffer.pop();
                        buffer.push('আ');
                        return;
                    }
                    ('ে', 'া') => {
                        buffer.pop();
                        buffer.push('ো');
                        return;
                    }
                    ('ে', 'ৗ') | ('ে', 'ৌ') => {
                        buffer.pop();
                        buffer.push('ৌ');
                        return;
                    }
                    _ => {}
                }
            }
        }

        // 5. Auto Chandrabindu (`ঁ`) canonical ordering:
        //    If a kar is typed right after `ঁ` (`কঁ` + `া`), swap to `কাঁ`
        if automation.auto_chandrabindu_fix && is_bangla_kar(out_ch) && buffer.ends_with('ঁ') {
            buffer.pop();
            buffer.push(out_ch);
            buffer.push('ঁ');
            return;
        }

        buffer.push(out_ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prebase_kar_reorder_and_compound_kar() {
        let layout = FixedLayout::national_jatiya();
        let auto = LayoutAutomation::default();
        let mut buf = String::new();
        let mut pending = None;

        // Type `ি` ('d' in Jatiya) BEFORE `ক` ('j' in Jatiya) -> should reorder to "কি"
        layout.apply_key(&mut buf, &mut pending, 'd', false, &auto);
        layout.apply_key(&mut buf, &mut pending, 'j', false, &auto);
        assert_eq!(buf, "কি");

        // Type `ে` ('c') + `ক` ('j') + `া` ('f') -> "কো"
        buf.clear();
        layout.apply_key(&mut buf, &mut pending, 'c', false, &auto);
        layout.apply_key(&mut buf, &mut pending, 'j', false, &auto);
        layout.apply_key(&mut buf, &mut pending, 'f', false, &auto);
        assert_eq!(buf, "কো");
    }
}
