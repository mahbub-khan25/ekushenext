//! Fixed Keyboard Layout Engine (UniJoy, Probhat, National/Jatiya, Bornona, Munir Optima, and the
//! user's own layout files) with Smart Sign & Connected-Letter Automation. People type the way their
//! hands learned and the text is always stored as correct Unicode:
//! - Pre-base kar (`ি`, `ে`, `ৈ`) typed before OR after the consonant
//! - Compound kar joining (`ে` + `া` -> `ো`, `ে` + `ৗ` -> `ৌ`, `অ` + `া` -> `আ`)
//! - Chandrabindu (`ঁ`) canonical ordering (`ক` + `ঁ` + `া` -> `কাঁ`)
//! - A link key (UniJoy's `g`): link + consonant joins a conjunct, link + vowel sign gives the full
//!   vowel (`g` `f` -> `আ`), link twice shows a visible hasanta
//! - A vowel sign with no consonant before it becomes the full vowel (`া` -> `আ`)
//! - Optionally, reph typed after the letter it sits on (`ম` + `র্` -> `র্ম`)
//! - Backspace takes back the last key, never half a letter

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
    UniJoy,
    /// A layout file the user imported (`Config::custom_layout` names which one)
    Custom,
}

impl LayoutId {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AvroPhonetic => "Ekushe Phonetic (Avro-compatible)",
            Self::Probhat => "প্রভাত (Probhat)",
            Self::NationalJatiya => "জাতীয় (National / Jatiya)",
            Self::Bornona => "বর্ণনা (Bornona)",
            Self::MunirOptima => "মুনীর অপটিমা (Munir Optima)",
            Self::UniJoy => "ইউনিজয় (UniJoy)",
            Self::Custom => "নিজের লেআউট (Custom layout)",
        }
    }
}

/// Typing automation flags for fixed layouts (makes signs and connected letters easy).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutAutomation {
    /// Allow typing `ি`, `ে`, `ৈ` before the consonant and auto-reorder to valid Unicode
    pub auto_prebase_kar_reorder: bool,
    /// Join `অ` + `া` -> `আ`, `ে` + `া` -> `ো`, `ে` + `ৗ` -> `ৌ`
    pub auto_vowel_forming: bool,
    /// Ensure `ঁ` (Chandrabindu) is placed after any kar (`কঁা` -> `কাঁ`)
    pub auto_chandrabindu_fix: bool,
    /// A vowel sign with no consonant before it becomes the full vowel (`া` -> `আ`, `ু` -> `উ`)
    pub kar_alone_becomes_vowel: bool,
}

impl Default for LayoutAutomation {
    fn default() -> Self {
        Self {
            auto_prebase_kar_reorder: true,
            auto_vowel_forming: true,
            auto_chandrabindu_fix: true,
            kar_alone_becomes_vowel: true,
        }
    }
}

/// Key mapping definition for a fixed layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedLayout {
    pub id: String,
    pub name: String,
    /// One to three letters for tight places (the Android key, the Linux panel)
    pub short: String,
    /// Normal / Shift ASCII char -> Bangla string
    pub normal: HashMap<char, String>,
    /// AltGr (Ctrl+Alt) char -> Bangla string
    pub altgr: HashMap<char, String>,
    /// The link (hasanta) key: link + consonant joins a conjunct, link + vowel sign gives the full
    /// vowel, link twice shows a visible hasanta. `None`: the layout's `্` key is a plain hasanta.
    pub link_key: Option<char>,
    /// After the link key: what a key gives instead (normally the full vowel for a vowel sign;
    /// keys not listed use [`independent_vowel`])
    pub independent: HashMap<char, String>,
    /// Reph (`র্`) is typed after the letter it sits on, the way the letters look (`ম` `র্` -> `র্ম`)
    pub reph_after: bool,
}

/// The full vowel for a vowel sign (`া` -> `আ`), or the double danda for a danda.
pub fn independent_vowel(c: char) -> Option<char> {
    Some(match c {
        'া' => 'আ',
        'ি' => 'ই',
        'ী' => 'ঈ',
        'ু' => 'উ',
        'ূ' => 'ঊ',
        'ৃ' => 'ঋ',
        'ে' => 'এ',
        'ৈ' => 'ঐ',
        'ো' => 'ও',
        'ৌ' => 'ঔ',
        '।' => '॥',
        _ => return None,
    })
}

/// The pre-base vowel signs, written before the consonant they follow in Unicode
fn is_prebase_kar(c: char) -> bool {
    matches!(c, 'ি' | 'ে' | 'ৈ')
}

/// Signs that sit on a consonant and stay with it (vowel signs, nukta, chandrabindu, anusvara, visarga)
fn is_attached_sign(c: char) -> bool {
    is_bangla_kar(c) || matches!(c, '়' | 'ঁ' | 'ং' | 'ঃ' | '\u{09D7}')
}

/// A word being typed in a fixed layout, with every key's state kept so that Backspace takes back
/// exactly one key.
#[derive(Debug, Clone, Default)]
pub struct FixedState {
    /// The text so far, in correct Unicode order
    pub buf: String,
    /// A pre-base kar typed before its consonant, waiting for it
    pub pending_kar: Option<char>,
    /// The link key was just pressed
    pub link: bool,
    /// A snippet is being typed (`;aj`): keys are plain letters until the word ends
    pub raw: bool,
    history: Vec<(String, Option<char>, bool, bool)>,
}

impl FixedState {
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty() && self.pending_kar.is_none() && !self.link
    }

    /// What to show while typing: the text, the link as a hasanta on the last letter, and a waiting
    /// pre-base kar on its own (after a zero-width non-joiner, so it doesn't land on the last letter).
    pub fn preedit(&self) -> String {
        let mut s = self.buf.clone();
        if self.link {
            s.push('্');
        }
        if let Some(k) = self.pending_kar {
            if !s.is_empty() {
                s.push('\u{200C}');
            }
            s.push(k);
        }
        if self.raw {
            s
        } else {
            crate::suggest::fix_initial_ra_ya(s)
        }
    }

    /// The finished text: a waiting kar is kept as typed, and a link left at the end is a hasanta.
    pub fn take(&mut self) -> String {
        let mut s = std::mem::take(&mut self.buf);
        if std::mem::take(&mut self.link) {
            s.push('্');
        }
        if let Some(k) = self.pending_kar.take() {
            s.push(k);
        }
        self.history.clear();
        // র + য-ফলা starting a word is the English "rya" sound (র‍্যাব), with a ZWJ
        if std::mem::take(&mut self.raw) {
            s
        } else {
            crate::suggest::fix_initial_ra_ya(s)
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Take back the last key. False when there was nothing to take back.
    pub fn backspace(&mut self) -> bool {
        if let Some((buf, pending, link, raw)) = self.history.pop() {
            self.buf = buf;
            self.pending_kar = pending;
            self.link = link;
            self.raw = raw;
            return true;
        }
        // Text that didn't come from keys (none today): remove one character
        if self.link {
            self.link = false;
        } else if self.pending_kar.take().is_none() && self.buf.pop().is_none() {
            return false;
        }
        true
    }

    fn remember(&mut self) {
        if self.history.len() == 256 {
            self.history.remove(0);
        }
        self.history
            .push((self.buf.clone(), self.pending_kar, self.link, self.raw));
    }
}

impl FixedLayout {
    pub fn builtin(id: LayoutId) -> Option<Self> {
        match id {
            LayoutId::AvroPhonetic => None,
            LayoutId::Probhat => Some(Self::probhat()),
            LayoutId::NationalJatiya => Some(Self::national_jatiya()),
            LayoutId::Bornona => Some(Self::bornona()),
            LayoutId::MunirOptima => Some(Self::munir_optima()),
            LayoutId::UniJoy => Some(Self::unijoy()),
            LayoutId::Custom => None,
        }
    }

    /// The layout a configuration types with: a built-in one, or the user's imported layout file
    /// (`None` for phonetic typing, and for a custom layout whose file is missing or invalid).
    pub fn for_config(config: &crate::config::Config) -> Option<Self> {
        if config.active_layout == LayoutId::Custom {
            return config
                .custom_layout
                .as_deref()
                .and_then(|id| load_custom_layout(id).ok());
        }
        Self::builtin(config.active_layout)
    }

    /// UniJoy, the Unicode layout from the Ekushey project (key map as published in the m17n
    /// database, LGPL-2.1+). `g` is the link key.
    pub fn unijoy() -> Self {
        let pairs: &[(char, &str)] = &[
            ('`', "\u{200C}"),
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
            ('q', "ঙ"),
            ('w', "য"),
            ('e', "ড"),
            ('r', "প"),
            ('t', "ট"),
            ('y', "চ"),
            ('u', "জ"),
            ('i', "হ"),
            ('o', "গ"),
            ('p', "ড়"),
            ('\\', "ঃ"),
            ('a', "ৃ"),
            ('s', "ু"),
            ('d', "ি"),
            ('f', "া"),
            ('h', "ব"),
            ('j', "ক"),
            ('k', "ত"),
            ('l', "দ"),
            ('z', "্র"),
            ('x', "ো"),
            ('c', "ে"),
            ('v', "র"),
            ('b', "ন"),
            ('n', "স"),
            ('m', "ম"),
            ('~', "\u{200D}"),
            ('$', "৳"),
            ('^', "÷"),
            ('&', "ঁ"),
            ('*', "×"),
            ('Q', "ং"),
            ('W', "য়"),
            ('E', "ঢ"),
            ('R', "ফ"),
            ('T', "ঠ"),
            ('Y', "ছ"),
            ('U', "ঝ"),
            ('I', "ঞ"),
            ('O', "ঘ"),
            ('P', "ঢ়"),
            ('|', "ৎ"),
            ('A', "র্"),
            ('S', "ূ"),
            ('D', "ী"),
            ('F', "অ"),
            ('G', "।"),
            ('H', "ভ"),
            ('J', "খ"),
            ('K', "থ"),
            ('L', "ধ"),
            ('Z', "্য"),
            ('X', "ৌ"),
            ('C', "ৈ"),
            ('V', "ল"),
            ('B', "ণ"),
            ('N', "ষ"),
            ('M', "শ"),
        ];
        let altgr: &[(char, &str)] = &[
            ('=', "≠"),
            ('e', "ঈ"),
            ('u', "ঊ"),
            ('i', "ঐ"),
            ('o', "ঔ"),
            ('a', "ঋ"),
            ('s', "উ"),
            ('d', "ই"),
            ('f', "আ"),
            ('g', "্"),
            ('h', "ৰ"),
            ('x', "ও"),
            ('c', "এ"),
            ('v', "ৱ"),
            ('.', "়"),
            ('!', "৴"),
            ('@', "৵"),
            ('#', "৶"),
            ('$', "৷"),
            ('%', "৲"),
            ('&', "৺"),
            (')', "৸"),
            ('_', "৹"),
            ('Q', "ঌ"),
            ('W', "ৡ"),
            ('I', "ঽ"),
            ('Z', "ৢ"),
            ('X', "ৗ"),
            ('C', "ৠ"),
            ('V', "ৣ"),
            ('B', "ৄ"),
            ('<', "≤"),
            ('>', "≥"),
        ];
        let mut layout = Self::from_slice("unijoy", "ইউনিজয় (UniJoy)", pairs, altgr);
        layout.short = "ইউ".into();
        layout.link_key = Some('g');
        layout
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
            short: name.chars().take(1).collect(),
            normal: normal.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
            altgr: altgr.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
            link_key: None,
            independent: HashMap::new(),
            reph_after: false,
        }
    }

    /// Apply a keystroke in this fixed layout to the word being typed, performing smart sign/kar
    /// reordering, compound vowel joining, the link key and reph placement.
    pub fn apply_key(
        &self,
        state: &mut FixedState,
        ch: char,
        altgr: bool,
        automation: &LayoutAutomation,
    ) {
        state.remember();

        // Snippets and date shortcuts (`;aj`): a `;` starting a word (when the layout has no
        // letter on it) makes the rest of the word plain letters
        if state.raw {
            state.buf.push(ch);
            return;
        }
        if ch == ';' && !altgr && state.is_empty() && !self.normal.contains_key(&';') {
            state.raw = true;
            state.buf.push(';');
            return;
        }

        if !altgr && Some(ch) == self.link_key {
            if state.link {
                // Link twice: a visible hasanta (ক্‌)
                state.link = false;
                state.buf.push('্');
                state.buf.push('\u{200C}');
                return;
            }
            // A pre-base kar that just landed on the last consonant belongs after the whole conjunct
            if automation.auto_prebase_kar_reorder && state.pending_kar.is_none() {
                if let Some(last) = state.buf.chars().last() {
                    if is_prebase_kar(last) {
                        state.buf.pop();
                        state.pending_kar = Some(last);
                    }
                }
            }
            state.link = true;
            return;
        }

        let mapped = if altgr {
            self.altgr.get(&ch).or_else(|| self.normal.get(&ch))
        } else {
            self.normal.get(&ch)
        };

        let Some(piece) = mapped else {
            // A key the layout doesn't use (punctuation, say): finish what's waiting, then the key
            if std::mem::take(&mut state.link) {
                state.buf.push('্');
            }
            if let Some(pk) = state.pending_kar.take() {
                state.buf.push(pk);
            }
            state.buf.push(ch);
            return;
        };

        let mut piece = piece.clone();
        if std::mem::take(&mut state.link) {
            let mut chars = piece.chars();
            let single = chars.next().filter(|_| chars.next().is_none());
            let independent = self
                .independent
                .get(&ch)
                .cloned()
                .or_else(|| single.and_then(independent_vowel).map(|c| c.to_string()));
            if let Some(vowel) = independent {
                // Link + vowel sign: the full vowel (g f -> আ)
                if let Some(pk) = state.pending_kar.take() {
                    state.buf.push(pk);
                }
                state.buf.push_str(&vowel);
                return;
            }
            if piece.chars().next().is_some_and(is_bangla_consonant) {
                piece.insert(0, '্');
            } else {
                state.buf.push('্');
            }
        }

        if self.reph_after && piece == "র্" && self.place_reph_after(state) {
            return;
        }

        for out_ch in piece.chars() {
            // ড়, ঢ়, য় in their standard Unicode form (letter + nukta), the form the dictionary,
            // phonetic typing and search use
            match out_ch {
                '\u{09DC}' => {
                    push_smart_char(state, 'ড', automation);
                    push_smart_char(state, '়', automation);
                }
                '\u{09DD}' => {
                    push_smart_char(state, 'ঢ', automation);
                    push_smart_char(state, '়', automation);
                }
                '\u{09DF}' => {
                    push_smart_char(state, 'য', automation);
                    push_smart_char(state, '়', automation);
                }
                c => push_smart_char(state, c, automation),
            }
        }
    }

    /// Reph typed after the letters it sits on: put `র্` in front of the last consonant cluster
    /// (`ধম` + `র্` -> `ধর্ম`, `কর্মী` typed as `কমী` + `র্`). False when there is no cluster to
    /// move it in front of, so it is typed as it comes.
    fn place_reph_after(&self, state: &mut FixedState) -> bool {
        if state.pending_kar.is_some() {
            return false;
        }
        let chars: Vec<char> = state.buf.chars().collect();
        let mut i = chars.len();
        while i > 0 && is_attached_sign(chars[i - 1]) {
            i -= 1;
        }
        if i == 0 || !is_bangla_consonant(chars[i - 1]) {
            return false;
        }
        i -= 1;
        // Back over a conjunct: consonant + hasanta + consonant ...
        while i >= 2 && chars[i - 1] == '্' && is_bangla_consonant(chars[i - 2]) {
            // A reph already there ends the cluster
            if chars[i - 2] == 'র'
                && (i < 3 || !is_bangla_consonant(chars[i - 3]) || chars[i - 3] == '্')
            {
                break;
            }
            i -= 2;
        }
        let mut out: String = chars[..i].iter().collect();
        out.push_str("র্");
        out.extend(&chars[i..]);
        state.buf = out;
        true
    }
}

/// One output character, with the automations applied.
fn push_smart_char(state: &mut FixedState, out_ch: char, automation: &LayoutAutomation) {
    let buffer = &mut state.buf;
    let pending = &mut state.pending_kar;

    // 1. Pre-base kar (`ি`, `ে`, `ৈ`) typed BEFORE a consonant: hold it until the consonant comes
    if automation.auto_prebase_kar_reorder && is_prebase_kar(out_ch) {
        let after_consonant = buffer
            .chars()
            .last()
            .is_some_and(|c| is_bangla_consonant(c) || c == '়');
        if !after_consonant {
            *pending = Some(out_ch);
            return;
        }
    }

    // 2. A consonant arriving while a pre-base kar waits: consonant first, then the kar
    if is_bangla_consonant(out_ch) {
        buffer.push(out_ch);
        if let Some(pk) = pending.take() {
            buffer.push(pk);
        }
        return;
    }

    // 3. Hasanta right after a consonant that just took a waiting kar: the kar goes after the whole
    //    conjunct
    if out_ch == '্' && automation.auto_prebase_kar_reorder {
        if let Some(last) = buffer.chars().last() {
            if is_prebase_kar(last) {
                buffer.pop();
                *pending = Some(last);
            }
        }
        buffer.push('্');
        return;
    }

    // Nukta stays on its consonant, before a waiting kar
    if out_ch == '়' {
        if let Some(last) = buffer.chars().last() {
            if is_prebase_kar(last) {
                buffer.pop();
                buffer.push('়');
                buffer.push(last);
                return;
            }
        }
    }

    // Flush any remaining pending kar before non-consonants
    if let Some(pk) = pending.take() {
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

    // 5. Chandrabindu canonical order: a kar typed right after `ঁ` (`কঁ` + `া`) goes before it
    if automation.auto_chandrabindu_fix && is_bangla_kar(out_ch) && buffer.ends_with('ঁ') {
        let before = buffer.chars().rev().nth(1);
        if before.is_some_and(|c| is_bangla_consonant(c) || c == '়') {
            buffer.pop();
            buffer.push(out_ch);
            buffer.push('ঁ');
            return;
        }
    }

    // 6. A vowel sign with no consonant before it is the full vowel (`া` -> `আ`)
    if automation.kar_alone_becomes_vowel && is_bangla_kar(out_ch) {
        let attached = buffer
            .chars()
            .last()
            .is_some_and(|c| is_bangla_consonant(c) || c == '়' || c == '\u{200D}');
        if !attached {
            if let Some(v) = independent_vowel(out_ch) {
                buffer.push(v);
                return;
            }
        }
    }

    buffer.push(out_ch);
}

// ---- Layout files: a person's own layout, imported in Studio ----

/// The largest layout file accepted (a full layout is a few kilobytes)
pub const MAX_LAYOUT_FILE_BYTES: usize = 64 * 1024;

/// A layout file as written on disk (`.ekushe-layout`, JSON). Only text for keys: nothing in it runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFile {
    /// Always `"ekushe-layout"`
    pub format: String,
    /// Format version (1)
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub short: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    /// The link (hasanta) key, if the layout has one
    #[serde(default)]
    pub link_key: Option<String>,
    /// Reph typed after the letter it sits on
    #[serde(default)]
    pub reph_after: bool,
    /// Key -> text, for the keys with and without Shift (`"j": "ক"`, `"J": "খ"`)
    pub keys: std::collections::BTreeMap<String, String>,
    /// AltGr (right Alt, or Ctrl+Alt) + key -> text
    #[serde(default)]
    pub altgr: std::collections::BTreeMap<String, String>,
    /// Link key + key -> text, where it isn't simply the full vowel of a vowel sign
    #[serde(default)]
    pub after_link: std::collections::BTreeMap<String, String>,
}

/// What Studio lists for an imported layout.
#[derive(Debug, Clone, Serialize)]
pub struct CustomLayoutInfo {
    pub id: String,
    pub name: String,
    pub short: String,
    pub author: String,
    pub description: String,
}

fn allowed_output_char(c: char) -> bool {
    matches!(c,
        '\u{0980}'..='\u{09FF}'          // Bangla
        | '\u{0964}' | '\u{0965}'        // danda, double danda
        | '\u{200C}' | '\u{200D}'        // zero-width non-joiner / joiner
        | ' '..='~'                      // plain ASCII letters, digits and punctuation
        | '×' | '÷' | '≠' | '≤' | '≥' | '‘' | '’' | '“' | '”' | '–' | '—' | '…' | '•'
    )
}

fn check_map(
    what: &str,
    map: &std::collections::BTreeMap<String, String>,
) -> Result<HashMap<char, String>, String> {
    if map.len() > 200 {
        return Err(format!("{what}: too many keys ({})", map.len()));
    }
    let mut out = HashMap::new();
    for (k, v) in map {
        let mut kc = k.chars();
        let key = match (kc.next(), kc.next()) {
            (Some(c), None) if c.is_ascii_graphic() => c,
            _ => {
                return Err(format!(
                    "{what}: \"{k}\" is not a single key (a letter, digit or symbol)"
                ))
            }
        };
        let n = v.chars().count();
        if n == 0 || n > 8 {
            return Err(format!(
                "{what}: the text for \"{k}\" must be 1 to 8 characters"
            ));
        }
        if let Some(bad) = v.chars().find(|c| !allowed_output_char(*c)) {
            return Err(format!(
                "{what}: the text for \"{k}\" has a character a keyboard layout can't type (U+{:04X})",
                bad as u32
            ));
        }
        out.insert(key, v.clone());
    }
    Ok(out)
}

fn clean_label(s: &str, max: usize) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect::<String>()
        .trim()
        .to_string()
}

impl FixedLayout {
    /// Read and check a layout file. The error says, in plain words, what is wrong with it.
    pub fn from_file_text(text: &str) -> Result<(Self, LayoutFile), String> {
        if text.len() > MAX_LAYOUT_FILE_BYTES {
            return Err("The file is too large to be a keyboard layout.".into());
        }
        let file: LayoutFile = serde_json::from_str(text.trim_start_matches('\u{FEFF}'))
            .map_err(|e| format!("This isn't an Ekushe layout file ({e})."))?;
        if file.format != "ekushe-layout" {
            return Err(
                "This isn't an Ekushe layout file (\"format\" should be \"ekushe-layout\").".into(),
            );
        }
        if file.version != 1 {
            return Err(format!(
                "This layout file is for a newer version of Ekushe (format version {}). Please update Ekushe.",
                file.version
            ));
        }
        let name = clean_label(&file.name, 40);
        if name.is_empty() {
            return Err("The layout has no name.".into());
        }
        let normal = check_map("keys", &file.keys)?;
        let altgr = check_map("altgr", &file.altgr)?;
        let independent = check_map("after_link", &file.after_link)?;
        let bangla_keys = normal
            .values()
            .filter(|v| v.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c)))
            .count();
        if bangla_keys < 10 {
            return Err(
                "The layout types fewer than 10 Bangla letters: it looks incomplete.".into(),
            );
        }
        let link_key = match file.link_key.as_deref() {
            None | Some("") => None,
            Some(k) => {
                let mut c = k.chars();
                match (c.next(), c.next()) {
                    (Some(ch), None) if ch.is_ascii_graphic() => Some(ch),
                    _ => return Err(format!("link_key: \"{k}\" is not a single key")),
                }
            }
        };
        let mut short = clean_label(&file.short, 3);
        if short.is_empty() {
            short = name.chars().take(1).collect();
        }
        let id = layout_id_for(&name);
        let layout = FixedLayout {
            id,
            name,
            short,
            normal,
            altgr,
            link_key,
            independent,
            reph_after: file.reph_after,
        };
        Ok((layout, file))
    }

    /// This layout as a layout file (a starting point for someone's own layout).
    pub fn to_file_text(&self, author: &str, description: &str) -> String {
        let map = |m: &HashMap<char, String>| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let file = LayoutFile {
            format: "ekushe-layout".into(),
            version: 1,
            name: self.name.clone(),
            short: self.short.clone(),
            author: author.into(),
            description: description.into(),
            link_key: self.link_key.map(|c| c.to_string()),
            reph_after: self.reph_after,
            keys: map(&self.normal),
            altgr: map(&self.altgr),
            after_link: map(&self.independent),
        };
        serde_json::to_string_pretty(&file).unwrap_or_default() + "\n"
    }
}

/// A stable file-friendly id for a layout name (`"My Layout"` -> `my-layout`; names without Latin
/// letters get a short number instead).
pub fn layout_id_for(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.len() >= 2 {
        return slug.chars().take(40).collect();
    }
    // FNV-1a: the same name always gets the same id
    let mut h: u32 = 0x811c_9dc5;
    for b in name.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("layout-{h:08x}")
}

static CUSTOM_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Keep imported layouts in `dir` instead (Android: the app's private files folder). Call before
/// the engine is made.
pub fn set_custom_layouts_dir(dir: std::path::PathBuf) {
    let _ = CUSTOM_DIR.set(dir);
}

/// Where imported layouts are kept (`<settings folder>/layouts`).
pub fn custom_layouts_dir() -> std::path::PathBuf {
    CUSTOM_DIR
        .get()
        .cloned()
        .unwrap_or_else(|| crate::config::Config::default_dir().join("layouts"))
}

/// The built-in layouts, in the order settings and menus show them.
pub const BUILT_IN_LAYOUTS: &[LayoutId] = &[
    LayoutId::AvroPhonetic,
    LayoutId::UniJoy,
    LayoutId::NationalJatiya,
    LayoutId::Probhat,
    LayoutId::Bornona,
    LayoutId::MunirOptima,
];

/// The layout setting as one value: a built-in layout's name (`UniJoy`) or `Custom:<id>`.
pub fn setting_value(cfg: &crate::config::Config) -> String {
    match (cfg.active_layout, cfg.custom_layout.as_deref()) {
        (LayoutId::Custom, Some(id)) => format!("Custom:{id}"),
        (l, _) => format!("{l:?}"),
    }
}

/// Apply a layout setting value; unknown values (or a layout file that is gone) mean phonetic typing.
pub fn apply_setting(cfg: &mut crate::config::Config, value: &str) {
    if let Some(id) = value.strip_prefix("Custom:") {
        if load_custom_layout(id).is_ok() {
            cfg.active_layout = LayoutId::Custom;
            cfg.custom_layout = Some(id.to_string());
            return;
        }
    }
    cfg.active_layout = BUILT_IN_LAYOUTS
        .iter()
        .copied()
        .find(|l| format!("{l:?}") == value)
        .unwrap_or(LayoutId::AvroPhonetic);
}

/// Every layout to choose from: (setting value, name, imported?)
pub fn all_layouts() -> Vec<(String, String, bool)> {
    let mut v: Vec<(String, String, bool)> = BUILT_IN_LAYOUTS
        .iter()
        .map(|l| (format!("{l:?}"), l.display_name().to_string(), false))
        .collect();
    for c in list_custom_layouts() {
        v.push((format!("Custom:{}", c.id), c.name, true));
    }
    v
}

/// Import a layout file's text: check it and keep it under its id. Returns what Studio shows.
pub fn import_layout(text: &str) -> Result<CustomLayoutInfo, String> {
    import_layout_into(&custom_layouts_dir(), text)
}

pub fn import_layout_into(dir: &std::path::Path, text: &str) -> Result<CustomLayoutInfo, String> {
    let (layout, file) = FixedLayout::from_file_text(text)?;
    std::fs::create_dir_all(dir).map_err(|e| format!("Couldn't save the layout: {e}"))?;
    let path = dir.join(format!("{}.json", layout.id));
    std::fs::write(&path, text).map_err(|e| format!("Couldn't save the layout: {e}"))?;
    Ok(CustomLayoutInfo {
        id: layout.id,
        name: layout.name,
        short: layout.short,
        author: clean_label(&file.author, 60),
        description: clean_label(&file.description, 200),
    })
}

/// The imported layouts, by name.
pub fn list_custom_layouts() -> Vec<CustomLayoutInfo> {
    list_custom_layouts_in(&custom_layouts_dir())
}

pub fn list_custom_layouts_in(dir: &std::path::Path) -> Vec<CustomLayoutInfo> {
    let mut out: Vec<CustomLayoutInfo> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let (layout, file) = FixedLayout::from_file_text(&text).ok()?;
            // Only files named after their own id (the ones Ekushe saved)
            (e.path().file_stem()?.to_str()? == layout.id).then(|| CustomLayoutInfo {
                id: layout.id,
                name: layout.name,
                short: layout.short,
                author: clean_label(&file.author, 60),
                description: clean_label(&file.description, 200),
            })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// An imported layout by id.
pub fn load_custom_layout(id: &str) -> Result<FixedLayout, String> {
    load_custom_layout_from(&custom_layouts_dir(), id)
}

pub fn load_custom_layout_from(dir: &std::path::Path, id: &str) -> Result<FixedLayout, String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Unknown layout.".into());
    }
    let text = std::fs::read_to_string(dir.join(format!("{id}.json")))
        .map_err(|_| "This layout is no longer installed.".to_string())?;
    FixedLayout::from_file_text(&text).map(|(l, _)| l)
}

/// Remove an imported layout.
pub fn remove_custom_layout(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Unknown layout.".into());
    }
    std::fs::remove_file(custom_layouts_dir().join(format!("{id}.json")))
        .map_err(|e| format!("Couldn't remove the layout: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Type plain keys in a layout and return (preedit, committed text)
    fn typ(layout: &FixedLayout, keys: &str) -> FixedState {
        let auto = LayoutAutomation::default();
        let mut st = FixedState::default();
        for k in keys.chars() {
            layout.apply_key(&mut st, k, false, &auto);
        }
        st
    }

    fn word(layout: &FixedLayout, keys: &str) -> String {
        typ(layout, keys).take()
    }

    #[test]
    fn test_prebase_kar_reorder_and_compound_kar() {
        let layout = FixedLayout::national_jatiya();
        // `ি` ('d') before `ক` ('j') -> "কি"
        assert_eq!(word(&layout, "dj"), "কি");
        // `ে` ('c') + `ক` ('j') + `া` ('f') -> "কো"
        assert_eq!(word(&layout, "cjf"), "কো");
    }

    #[test]
    fn ra_ya_at_the_start_of_a_word_gets_its_zwj_in_fixed_layouts() {
        let l = FixedLayout::unijoy();
        let auto = LayoutAutomation::default();
        let typed = |keys: &str| {
            let mut st = FixedState::default();
            for c in keys.chars() {
                l.apply_key(&mut st, c, false, &auto);
            }
            (st.preedit(), st.take())
        };
        assert_eq!(typed("vZfh"), ("র\u{200D}্যাব".into(), "র\u{200D}্যাব".into()));
        // Inside a word it stays a reph + য (কার্য)
        assert_eq!(typed("jfvZ").1, "কার্য");
    }

    #[test]
    fn unijoy_types_words_the_way_hands_know() {
        let u = FixedLayout::unijoy();
        assert_eq!(word(&u, "gfmfv"), "আমার"); // link + া = আ
        assert_eq!(word(&u, "dj"), "কি"); // ি before the consonant
        assert_eq!(word(&u, "jd"), "কি"); // ... or after
        assert_eq!(word(&u, "cjfb"), "কোন"); // ে + ক + া = কো
        assert_eq!(word(&u, "jxb"), "কোন"); // the ো key
        assert_eq!(word(&u, "jgk"), "ক্ত"); // link joins a conjunct
        assert_eq!(word(&u, "djgk"), "ক্তি"); // ি typed first goes after the whole conjunct
        assert_eq!(word(&u, "cjgk"), "ক্তে");
        assert_eq!(word(&u, "jgkd"), "ক্তি"); // ... and typed after it
        assert_eq!(word(&u, "ngkzD"), "স্ত্রী"); // র-ফলা and ী
        assert_eq!(word(&u, "hZhifv"), "ব্যবহার"); // য-ফলা
        assert_eq!(word(&u, "LAm"), "ধর্ম"); // reph in Unicode order
        assert_eq!(word(&u, "j&f"), "কাঁ"); // chandrabindu before the kar is put after it
        assert_eq!(word(&u, "Fv|"), "অরৎ"); // khanda ta
        assert_eq!(word(&u, "j."), "ক."); // a key the layout doesn't use passes through
        assert_eq!(word(&u, "G"), "।");
    }

    #[test]
    fn rra_rha_yya_are_stored_in_standard_unicode() {
        let u = FixedLayout::unijoy();
        assert_eq!(word(&u, "p"), "ড\u{09BC}");
        assert_eq!(word(&u, "dp"), "ড\u{09BC}ি"); // ি typed first lands after the nukta
        assert_eq!(word(&u, "hfpD"), "বাড\u{09BC}ী");
        assert_eq!(word(&u, "Wf"), "য\u{09BC}া");
        assert!(!word(&u, "pPW").contains(['\u{09DC}', '\u{09DD}', '\u{09DF}']));
    }

    #[test]
    fn a_semicolon_starts_a_snippet_in_plain_letters() {
        let u = FixedLayout::unijoy();
        let st = typ(&u, ";aj");
        assert!(st.raw);
        assert_eq!(st.buf, ";aj");
        let mut st = typ(&u, ";a");
        st.backspace();
        st.backspace();
        assert!(!st.raw && st.is_empty());
        // Mid-word, `;` is just punctuation
        assert_eq!(word(&u, "j;"), "ক;");
    }

    #[test]
    fn a_vowel_sign_without_a_consonant_is_the_full_vowel() {
        let u = FixedLayout::unijoy();
        assert_eq!(word(&u, "sbd"), "উনি");
        assert_eq!(word(&u, "f"), "আ");
        assert_eq!(word(&u, "gc"), "এ"); // the link way works too
        assert_eq!(word(&u, "gG"), "॥");
        // But a vowel sign after a consonant stays a sign
        assert_eq!(word(&u, "js"), "কু");
        // And the automation can be turned off
        let auto = LayoutAutomation {
            kar_alone_becomes_vowel: false,
            ..Default::default()
        };
        let mut st = FixedState::default();
        u.apply_key(&mut st, 'f', false, &auto);
        assert_eq!(st.take(), "া");
    }

    #[test]
    fn link_twice_is_a_visible_hasanta() {
        let u = FixedLayout::unijoy();
        assert_eq!(word(&u, "jgg"), "ক্\u{200C}");
        // A link left at the end of a word is a hasanta
        assert_eq!(word(&u, "jg"), "ক্");
    }

    #[test]
    fn backspace_takes_back_exactly_one_key() {
        let u = FixedLayout::unijoy();
        let mut st = typ(&u, "dj");
        assert_eq!(st.preedit(), "কি");
        assert!(st.backspace());
        // The ি typed first is still waiting, and shown
        assert_eq!(st.preedit(), "ি");
        assert!(st.backspace());
        assert!(st.is_empty());
        assert!(!st.backspace());

        let mut st = typ(&u, "ngkz");
        assert_eq!(st.preedit(), "স্ত্র");
        st.backspace(); // the র-ফলা key, whole
        assert_eq!(st.preedit(), "স্ত");
        st.backspace(); // the ত (with its link)
        assert_eq!(st.preedit(), "স্");
        st.backspace(); // the link
        assert_eq!(st.preedit(), "স");

        let mut st = typ(&u, "cjf");
        assert_eq!(st.preedit(), "কো");
        st.backspace();
        assert_eq!(st.preedit(), "কে");
    }

    #[test]
    fn a_waiting_kar_is_shown_apart_from_the_last_letter() {
        let u = FixedLayout::unijoy();
        let st = typ(&u, "jfc");
        assert_eq!(st.buf, "কা");
        assert_eq!(st.preedit(), "কা\u{200C}ে");
        assert_eq!(typ(&u, "jfcm").take(), "কামে");
        // A link shows as a hasanta on the last letter
        assert_eq!(typ(&u, "jg").preedit(), "ক্");
    }

    #[test]
    fn reph_can_be_typed_after_the_letter() {
        let mut u = FixedLayout::unijoy();
        u.reph_after = true;
        assert_eq!(word(&u, "LmA"), "ধর্ম");
        assert_eq!(word(&u, "jmDA"), "কর্মী");
        assert_eq!(word(&u, "dmA"), "র্মি"); // ি typed first, then ম, then reph
        assert_eq!(word(&u, "ngkA"), "র্স্ত"); // in front of the whole conjunct
        assert_eq!(word(&u, "Am"), "র্ম"); // nothing to sit on yet: typed as it comes
        assert_eq!(word(&u, "LmAf"), "ধর্মা");
    }

    #[test]
    fn layout_files_round_trip_and_are_checked() {
        let u = FixedLayout::unijoy();
        let text = u.to_file_text("Ekushe", "UniJoy as a layout file");
        let (back, file) = FixedLayout::from_file_text(&text).unwrap();
        assert_eq!(back.normal, u.normal);
        assert_eq!(back.altgr, u.altgr);
        assert_eq!(back.link_key, Some('g'));
        assert_eq!(back.short, "ইউ");
        assert_eq!(file.author, "Ekushe");
        assert_eq!(word(&back, "gfmfv"), "আমার");

        let bad = |t: &str| FixedLayout::from_file_text(t).unwrap_err();
        assert!(bad("not json").contains("isn't an Ekushe layout"));
        assert!(bad(r#"{"format":"other","version":1,"name":"x","keys":{}}"#).contains("format"));
        assert!(
            bad(r#"{"format":"ekushe-layout","version":2,"name":"x","keys":{}}"#)
                .contains("newer version")
        );
        assert!(
            bad(r#"{"format":"ekushe-layout","version":1,"name":"x","keys":{"j":"ক"}}"#)
                .contains("fewer than 10")
        );
        let control = text.replacen("\"ক\"", "\"ক\\u0007\"", 1);
        assert!(bad(&control).contains("can't type"));
        let two_keys = text.replacen("\"j\":", "\"jj\":", 1);
        assert!(bad(&two_keys).contains("not a single key"));
        assert!(bad(&"x".repeat(MAX_LAYOUT_FILE_BYTES + 1)).contains("too large"));
    }

    #[test]
    fn imported_layouts_are_kept_listed_and_loaded() {
        let dir = std::env::temp_dir().join(format!("ekushe-layouts-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut mine = FixedLayout::unijoy();
        mine.name = "My Office Layout".into();
        mine.short = String::new();
        let info = import_layout_into(&dir, &mine.to_file_text("Rahim", "")).unwrap();
        assert_eq!(info.id, "my-office-layout");
        assert_eq!(info.short, "M");
        let listed = list_custom_layouts_in(&dir);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].author, "Rahim");
        let loaded = load_custom_layout_from(&dir, "my-office-layout").unwrap();
        assert_eq!(word(&loaded, "jgk"), "ক্ত");
        assert!(load_custom_layout_from(&dir, "../etc/passwd").is_err());
        assert!(load_custom_layout_from(&dir, "missing").is_err());
        // Bangla-only names get a stable id
        assert_eq!(layout_id_for("আমার লেআউট"), layout_id_for("আমার লেআউট"));
        assert!(layout_id_for("আমার লেআউট").starts_with("layout-"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
