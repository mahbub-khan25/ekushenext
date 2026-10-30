//! Bijoy (SutonnyMJ ANSI) <-> Unicode, syllable by syllable.
//!
//! Bijoy text stores what is drawn, left to right: a pre-base kar (`ি` `ে` `ৈ`) comes before its
//! consonant cluster, `ো`/`ৌ` are split around it (`‡Kv` = কো), reph (`©`) comes after the cluster,
//! and many conjuncts are single glyphs or a half letter plus a sign (`¯^` = স্ব, `š¿` = ন্ত্র).
//! Unicode stores the spoken order. The glyph tables come from bijoy_helper (MIT); the reordering
//! here is Ekushe's own.

#[rustfmt::skip]
mod tables {
    // Generated from the bijoy_helper tables (MIT, Copyright (c) 2022 Muhammad Fahid Sarker); see
    // THIRD_PARTY_NOTICES.md. Bijoy text -> Unicode, one glyph (or glyph pair) at a time.
    pub(crate) const DECODE: &[(&str, &str)] = &[
        ("ÿ", "ক্ষ"), // Bijoy 52 / SutonnyMJ ক্ষ (also ¶)
        ("i¨", "র\u{200D}্য"), // র-য-ফলা (র‍্যাব), not reph + য
        ("Av", "আ"),
        ("„„", "ৃ"),
        ("&", "্"), // visible hasanta between letters with no ligature
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
        ("^", "্ব"),
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
        ("v", "া"),
        ("w", "ি"),
        ("x", "ী"),
        ("y", "ু"),
        ("z", "ু"),
        ("|", "।"),
        ("~", "ূ"),
        ("¡", "্ব"),
        ("¢", "্ভ"),
        ("£", "্ভ্র"),
        ("¤", "ম্"),
        ("¥", "্ম"),
        ("¦", "্ব"),
        ("§", "্ম"),
        ("¨", "্য"),
        ("©", "র্"),
        ("ª", "্র"),
        ("«", "্র"),
        ("¬", "্ল"),
        ("\u{AD}", "্ল"),
        ("®", "ষ্"),
        ("¯", "স্"),
        ("°", "ক্ক"),
        ("±", "ক্ট"),
        ("²", "ক্ষ্ম"),
        ("³", "ক্ত"),
        ("´", "ক্ম"),
        ("µ", "ক্র"),
        ("¶", "ক্ষ"),
        ("·", "ক্স"),
        ("¸", "গু"),
        ("¹", "জ্ঞ"),
        ("º", "গ্দ"),
        ("»", "গ্ধ"),
        ("¼", "ঙ্ক"),
        ("½", "ঙ্গ"),
        ("¾", "জ্জ"),
        ("¿", "্ত্র"),
        ("À", "জ্ঝ"),
        ("Á", "জ্ঞ"),
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
        ("Í", "ত্ম"),
        ("Î", "ত্র"),
        ("Ï", "দ্দ"),
        ("Ð", "–"),
        ("Ñ", "—"),
        ("Ò", "“"),
        ("Ó", "”"),
        ("Ô", "‘"),
        ("Õ", "’"),
        ("Ö", "্র"),
        ("×", "দ্ধ"),
        ("Ø", "দ্ব"),
        ("Ù", "দ্ম"),
        ("Ú", "ন্ঠ"),
        ("Û", "ন্ড"),
        ("Ü", "ন্ধ"),
        ("Ý", "ন্স"),
        ("Þ", "প্ট"),
        ("ß", "প্ত"),
        ("à", "প্প"),
        ("á", "প্স"),
        ("â", "ব্জ"),
        ("ã", "ব্দ"),
        ("ä", "ব্ধ"),
        ("å", "ভ্র"),
        ("æ", "ম্ন"),
        ("ç", "ম্ফ"),
        ("è", "্ণ"),
        ("é", "ল্ক"),
        ("ê", "ল্গ"),
        ("ë", "ল্ট"),
        ("ì", "ল্ড"),
        ("í", "ল্প"),
        ("î", "ল্ফ"),
        ("ï", "শু"),
        ("ð", "শ্চ"),
        ("ñ", "শ্ছ"),
        ("ò", "ষ্ণ"),
        ("ó", "ষ্ট"),
        ("ô", "ষ্ঠ"),
        ("õ", "ষ্ফ"),
        ("ö", "স্খ"),
        ("÷", "স্ট"),
        ("ø", "স্ন"),
        ("ù", "স্ফ"),
        ("ú", "্প"),
        ("û", "হু"),
        ("ü", "হৃ"),
        ("ý", "হ্ন"),
        ("þ", "হ্ম"),
        ("Œ", "্ক্র"),
        ("œ", "্ন"),
        ("Š", "ৗ"),
        ("š", "ন্"),
        ("Ÿ", "্ব"),
        ("ƒ", "ূ"),
        ("ˆ", "ৈ"),
        ("˜", "দ্"),
        ("–", "ু"),
        ("—", "্ত"),
        ("‘", "্তু"),
        ("’", "্থ"),
        ("‚", "ূ"),
        ("“", "ু"),
        ("”", "চ্"),
        ("„", "ৃ"),
        ("†", "ে"),
        ("‡", "ে"),
        ("•", "ঙ্"),
        ("…", "ৃ"),
        ("‰", "ৈ"),
        ("‹", "্ক"),
        ("›", "ন্"),
        ("™", "দ্"),
    ];

    // Unicode -> Bijoy: letters, conjuncts and signs, matched longest first, from the same tables.
    pub(crate) const ENCODE: &[(&str, &str)] = &[
        ("ড\u{09BC}", "o"),
        ("ঢ\u{09BC}", "p"),
        ("য\u{09BC}", "q"),
        ("র\u{200D}্য", "i¨"),
        ("।", "|"),
        ("‘", "Ô"),
        ("’", "Õ"),
        ("“", "Ò"),
        ("”", "Ó"),
        ("্র্য", "ª¨"),
        ("ম্প্র", "¤cÖ"),
        ("র\u{200C}্য", "i¨"),
        ("ক্ষ্ম", "²"),
        ("ক্ক", "°"),
        ("ক্ট", "±"),
        ("ক্ত", "³"),
        ("ক্ব", "K¡"),
        ("স্ক্র", "¯Œ"),
        ("ক্র", "µ"),
        ("ক্ল", "K¬"),
        ("ক্ষ", "ÿ"),
        ("ক্স", "·"),
        ("গু", "¸"),
        ("গ্ধ", "»"),
        ("গ্ন", "Mœ"),
        ("গ্ম", "M¥"),
        ("গ্ল", "M\u{AD}"),
        ("গ্রু", "Mªy"),
        ("ঙ্ক", "¼"),
        ("ঙ্ক্ষ", "•¶"),
        ("ঙ্খ", "•L"),
        ("ঙ্গ", "½"),
        ("ঙ্ঘ", "•N"),
        ("চ্ছ্ব", "”Q¡"),
        ("চ্চ", "”P"),
        ("চ্ছ", "”Q"),
        ("চ্ঞ", "”T"),
        ("জ্জ্ব", "¾¡"),
        ("জ্জ", "¾"),
        ("জ্ঝ", "À"),
        ("জ্ঞ", "Á"),
        ("জ্ব", "R¡"),
        ("ঞ্চ", "Â"),
        ("ঞ্ছ", "Ã"),
        ("ঞ্জ", "Ä"),
        ("ঞ্ঝ", "Å"),
        ("ট্ট", "Æ"),
        ("ট্ব", "U¡"),
        ("ট্ম", "U¥"),
        ("ড্ড", "Ç"),
        ("ণ্ট", "È"),
        ("ণ্ঠ", "É"),
        ("ন্স", "Ý"),
        ("ণ্ড", "Ê"),
        ("ন্তু", "š‘"),
        ("ণ্ব", "Y^"),
        ("ত্ত্ব", "Ë¡"),
        ("ত্ত", "Ë"),
        ("ত্থ", "Ì"),
        ("ত্ন", "Zœ"),
        ("ত্ম", "Z¥"),
        ("ন্ত্ব", "š—¡"),
        ("ত্ব", "Z¡"),
        ("থ্ব", "_¡"),
        ("দ্গ", "˜M"),
        ("দ্ঘ", "˜N"),
        ("দ্দ", "Ï"),
        ("দ্ধ", "×"),
        ("দ্ব", "Ø"),
        ("দ্ভ", "™¢"),
        ("দ্ম", "Ù"),
        ("দ্রু", "`ª“"),
        ("ধ্ব", "aŸ"),
        ("ধ্ম", "a¥"),
        ("ন্ট", "›U"),
        ("ন্ঠ", "Ú"),
        ("ন্ড", "Û"),
        ("ন্ত্র", "š¿"),
        ("ন্ত", "š—"),
        ("স্ত্র", "¯¿"),
        ("ত্র", "Î"),
        ("ন্থ", "š’"),
        ("ন্দ", "›`"),
        ("ন্দ্ব", "›Ø"),
        ("ন্ধ", "Ü"),
        ("ন্ন", "bœ"),
        ("ন্ব", "š^"),
        ("ন্ম", "b¥"),
        ("প্ট", "Þ"),
        ("প্ত", "ß"),
        ("প্ন", "cœ"),
        ("প্প", "à"),
        ("প্ল", "c\u{AD}"),
        ("প্স", "á"),
        ("ফ্ল", "d¬"),
        ("ব্জ", "â"),
        ("ব্দ", "ã"),
        ("ব্ধ", "ä"),
        ("ব্ব", "eŸ"),
        ("ব্ল", "e\u{AD}"),
        ("ভ্র", "å"),
        ("ম্ন", "gœ"),
        ("ম্প", "¤ú"),
        ("ম্ফ", "ç"),
        ("ম্ব", "¤^"),
        ("ম্ভ", "¤¢"),
        ("ম্ভ্র", "¤£"),
        ("ম্ম", "¤§"),
        ("ম্ল", "¤\u{AD}"),
        ("্র", "ª"),
        ("রু", "i“"),
        ("রূ", "iƒ"),
        ("ল্ক", "é"),
        ("ল্গ", "ê"),
        ("ল্ট", "ë"),
        ("ল্ড", "ì"),
        ("ল্প", "í"),
        ("ল্ফ", "î"),
        ("ল্ব", "j¦"),
        ("ল্ম", "j¥"),
        ("ল্ল", "j\u{AD}"),
        ("শু", "ï"),
        ("শ্চ", "ð"),
        ("শ্ন", "kœ"),
        ("শ্ব", "k¦"),
        ("শ্ম", "k¥"),
        ("শ্ল", "k\u{AD}"),
        ("ষ্ক", "®‹"),
        ("ষ্ক্র", "®Œ"),
        ("ষ্ট", "ó"),
        ("ষ্ঠ", "ô"),
        ("ষ্ণ", "ò"),
        ("ষ্প", "®ú"),
        ("ষ্ফ", "õ"),
        ("ষ্ম", "®§"),
        ("স্ক", "¯‹"),
        ("স্ট", "÷"),
        ("স্খ", "ö"),
        ("স্ত", "¯—"),
        ("স্তু", "¯‘"),
        ("স্থ", "¯’"),
        ("স্ন", "mœ"),
        ("স্প", "¯ú"),
        ("স্ফ", "ù"),
        ("স্ব", "¯^"),
        ("স্ম", "¯§"),
        ("স্ল", "¯\u{AD}"),
        ("হু", "û"),
        ("হ্ণ", "nè"),
        ("হ্ব", "nŸ"),
        ("হ্ন", "ý"),
        ("হ্ম", "þ"),
        ("হ্ল", "n¬"),
        ("হৃ", "ü"),
        ("র্", "©"),
        ("্য", "¨"),
        ("্", "&"),
        ("আ", "Av"),
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
        ("ড়", "o"),
        ("ঢ়", "p"),
        ("য়", "q"),
        ("ৎ", "r"),
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
        ("া", "v"),
        ("ি", "w"),
        ("ী", "x"),
        ("ু", "y"),
        ("ূ", "~"),
        ("ৃ", "…"),
        ("ে", "‡"),
        ("ৈ", "‰"),
        ("ৗ", "Š"),
        ("ং", "s"),
        ("ঃ", "t"),
        ("ঁ", "u"),
    ];
}

use crate::conjuncts::is_bangla_consonant;
use tables::{DECODE, ENCODE};

/// Placeholder for a reph read from Bijoy (`©`) until it is moved in front of its cluster
const REPH: char = '\u{E000}';
const HASANTA: char = '্';
const NUKTA: char = '়';

fn is_prekar(c: char) -> bool {
    matches!(c, 'ি' | 'ে' | 'ৈ')
}

fn is_postkar(c: char) -> bool {
    matches!(c, 'া' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ৗ')
}

/// ড়, ঢ়, য় as letter + nukta (the form the dictionary and phonetic typing use)
fn decompose_nukta(s: &str) -> String {
    s.replace('\u{09DC}', "ড\u{09BC}")
        .replace('\u{09DD}', "ঢ\u{09BC}")
        .replace('\u{09DF}', "য\u{09BC}")
}

/// End of the consonant cluster starting at `s` (consonants joined by hasanta, each with an
/// optional nukta); `s` itself when there is no consonant there.
fn cluster_end(t: &[char], s: usize) -> usize {
    if !t.get(s).is_some_and(|&c| is_bangla_consonant(c)) {
        return s;
    }
    let mut j = s + 1;
    if t.get(j) == Some(&NUKTA) {
        j += 1;
    }
    // র‍্য: ra, a joiner, then ya-phala
    if t[s] == 'র'
        && matches!(t.get(j), Some('\u{200D}') | Some('\u{200C}'))
        && t.get(j + 1) == Some(&HASANTA)
        && t.get(j + 2) == Some(&'য')
    {
        j += 3;
    }
    while t.get(j) == Some(&HASANTA) && t.get(j + 1).is_some_and(|&c| is_bangla_consonant(c)) {
        j += 2;
        if t.get(j) == Some(&NUKTA) {
            j += 1;
        }
    }
    j
}

/// Bijoy (SutonnyMJ) text to Unicode Bangla.
pub fn to_unicode(input: &str) -> String {
    // 1. Glyphs to letters and signs, longest match first; reph kept as a placeholder
    let mut t: Vec<char> = Vec::with_capacity(input.len() * 2);
    let mut rest = input;
    'outer: while let Some(c) = rest.chars().next() {
        if c == '©' {
            t.push(REPH);
            rest = &rest[c.len_utf8()..];
            continue;
        }
        for &(b, u) in DECODE {
            if rest.starts_with(b) {
                t.extend(u.chars());
                rest = &rest[b.len()..];
                continue 'outer;
            }
        }
        t.push(c);
        rest = &rest[c.len_utf8()..];
    }
    // A half letter followed by a sign that starts with hasanta (স্ + ্ক): one hasanta
    let mut dedup: Vec<char> = Vec::with_capacity(t.len());
    for c in t {
        if c == HASANTA && dedup.last() == Some(&HASANTA) {
            continue;
        }
        dedup.push(c);
    }
    let t = dedup;

    // 2. Visual order to Unicode order, one syllable at a time
    let mut out: Vec<char> = Vec::with_capacity(t.len());
    let mut i = 0;
    while i < t.len() {
        let c = t[i];
        if is_prekar(c) {
            let end = cluster_end(&t, i + 1);
            if end == i + 1 {
                out.push(c);
                i += 1;
                continue;
            }
            let mut j = end;
            let mut kar = c;
            let mut reph = 0;
            while t.get(j) == Some(&REPH) {
                reph += 1;
                j += 1;
            }
            if kar == 'ে' {
                match t.get(j) {
                    Some('া') => {
                        kar = 'ো';
                        j += 1;
                    }
                    Some('ৗ') => {
                        kar = 'ৌ';
                        j += 1;
                    }
                    _ => {}
                }
            }
            while reph == 0 && t.get(j) == Some(&REPH) {
                reph += 1;
                j += 1;
            }
            for _ in 0..reph {
                out.extend(['র', HASANTA]);
            }
            out.extend(&t[i + 1..end]);
            out.push(kar);
            i = j;
            continue;
        }
        if is_bangla_consonant(c) {
            let end = cluster_end(&t, i);
            let rephs = |from: usize| t[from..].iter().take_while(|&&x| x == REPH).count();
            let n = rephs(end);
            if n > 0 {
                for _ in 0..n {
                    out.extend(['র', HASANTA]);
                }
                out.extend(&t[i..end]);
                i = end + n;
                continue;
            }
            if t.get(end).is_some_and(|&k| is_postkar(k)) && rephs(end + 1) > 0 {
                let n = rephs(end + 1);
                for _ in 0..n {
                    out.extend(['র', HASANTA]);
                }
                out.extend(&t[i..=end]);
                i = end + 1 + n;
                continue;
            }
            out.extend(&t[i..end]);
            i = end;
            continue;
        }
        if c == REPH {
            out.extend(['র', HASANTA]);
        } else {
            out.push(c);
        }
        i += 1;
    }

    // 3. Small fixes: অ + া = আ, and chandrabindu after the vowel sign
    let mut fixed: Vec<char> = Vec::with_capacity(out.len());
    let mut k = 0;
    while k < out.len() {
        let c = out[k];
        if c == 'অ' && out.get(k + 1) == Some(&'া') {
            fixed.push('আ');
            k += 2;
            continue;
        }
        if c == 'ঁ'
            && out
                .get(k + 1)
                .is_some_and(|&n| is_postkar(n) || n == 'ো' || n == 'ৌ')
        {
            fixed.push(out[k + 1]);
            fixed.push('ঁ');
            k += 2;
            continue;
        }
        fixed.push(c);
        k += 1;
    }
    fixed.into_iter().collect()
}

/// The Bijoy glyph for a phala (`্` + consonant) the tables have no ligature for, after `prev`.
fn phala(prev: char, next: char) -> Option<&'static str> {
    Some(match next {
        'য' => "¨",
        'র' if matches!(prev, 'ট' | 'ঠ' | 'ড' | 'ঢ' | 'দ') => "ª",
        'র' => "Ö",
        'ব' if matches!(prev, 'ধ' | 'ব' | 'হ') => "Ÿ",
        'ব' if matches!(prev, 'স' | 'ণ' | 'ম' | 'ন') => "^",
        'ব' if matches!(prev, 'শ' | 'ল') => "¦",
        'ব' => "¡",
        'ম' => "¥",
        'ন' => "œ",
        'ল' => "\u{AD}",
        'ণ' => "è",
        _ => return None,
    })
}

/// Encode a run of Unicode characters (a cluster, possibly with its vowel sign) longest match first.
fn encode_run(chars: &[char], out: &mut String) {
    let mut p = 0;
    while p < chars.len() {
        // Longest table entry starting here
        let mut best: Option<(usize, &str)> = None;
        for &(u, b) in ENCODE {
            let n = u.chars().count();
            if n > best.map_or(0, |x| x.0)
                && p + n <= chars.len()
                && u.chars().zip(&chars[p..]).all(|(a, &c)| a == c)
            {
                best = Some((n, b));
            }
        }
        // A phala without a bigger ligature: the glyph that fits the letter before it (ra-phala
        // is `Ö` after most letters, `ª` after ট ঠ ড ঢ দ) rather than hasanta + letter
        // ্র্য (ব্র্যাক): the ra-phala that fits, then ya-phala
        if chars[p] == HASANTA && chars[p + 1..].starts_with(&['র', HASANTA, 'য']) {
            if let Some(&prev) = p.checked_sub(1).and_then(|q| chars.get(q)) {
                out.push_str(phala(prev, 'র').unwrap_or("Ö"));
                out.push('¨');
                p += 4;
                continue;
            }
        }
        if chars[p] == HASANTA && best.map_or(0, |x| x.0) <= 2 {
            if let (Some(&prev), Some(&next)) = (
                p.checked_sub(1).and_then(|q| chars.get(q)),
                chars.get(p + 1),
            ) {
                if let Some(g) = phala(prev, next) {
                    out.push_str(g);
                    p += 2;
                    continue;
                }
            }
        }
        match best {
            Some((n, b)) => {
                out.push_str(b);
                p += n;
            }
            None => {
                out.push(chars[p]);
                p += 1;
            }
        }
    }
}

fn glyph(c: char) -> String {
    let mut s = String::new();
    encode_run(&[c], &mut s);
    s
}

/// Unicode Bangla to Bijoy (SutonnyMJ) text.
pub fn from_unicode(input: &str) -> String {
    let u: Vec<char> = decompose_nukta(input).chars().collect();
    let mut out = String::with_capacity(input.len() * 2);
    let mut i = 0;
    while i < u.len() {
        // Reph: র + hasanta starting a syllable, before a consonant (র‍্য, with a joiner, is a ya-phala)
        let mut reph = 0;
        while u[i] == 'র'
            && u.get(i + 1) == Some(&HASANTA)
            && u.get(i + 2).is_some_and(|&c| is_bangla_consonant(c))
        {
            reph += 1;
            i += 2;
        }
        if !is_bangla_consonant(u[i]) {
            for _ in 0..reph {
                out.push('©');
            }
            // Vowels, digits, punctuation, explicit hasanta (্ + ZWNJ)
            if u[i] == HASANTA && matches!(u.get(i + 1), Some('\u{200C}')) {
                out.push('&');
                i += 2;
                continue;
            }
            let mut best: Option<(usize, &str)> = None;
            for &(k, b) in ENCODE {
                let n = k.chars().count();
                if n > best.map_or(0, |x| x.0)
                    && i + n <= u.len()
                    && k.chars().zip(&u[i..]).all(|(a, &c)| a == c)
                {
                    best = Some((n, b));
                }
            }
            match best {
                Some((n, b)) => {
                    out.push_str(b);
                    i += n;
                }
                None => {
                    out.push(u[i]);
                    i += 1;
                }
            }
            continue;
        }
        let end = cluster_end(&u, i);
        let cluster = &u[i..end];
        let mut j = end;
        let (mut pre, mut post) = (None, None);
        match u.get(j) {
            Some(&k) if is_prekar(k) => {
                pre = Some(k);
                j += 1;
            }
            Some('ো') => {
                pre = Some('ে');
                post = Some('া');
                j += 1;
            }
            Some('ৌ') => {
                pre = Some('ে');
                post = Some('ৗ');
                j += 1;
            }
            Some(&k) if is_postkar(k) => {
                post = Some(k);
                j += 1;
            }
            _ => {}
        }
        if let Some(k) = pre {
            out.push_str(&glyph(k));
        }
        if reph > 0 {
            encode_run(cluster, &mut out);
            for _ in 0..reph {
                out.push('©');
            }
            if let Some(k) = post {
                out.push_str(&glyph(k));
            }
        } else {
            let mut run: Vec<char> = cluster.to_vec();
            if let Some(k) = post {
                run.push(k);
            }
            encode_run(&run, &mut out);
        }
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real SutonnyMJ spellings (‡ and † are both ে; Ekushe writes ‡)
    const WORDS: &[(&str, &str)] = &[
        ("Avwg", "আমি"),
        ("evsjv", "বাংলা"),
        ("evsjv‡`k", "বাংলাদেশ"),
        ("Avgvi", "আমার"),
        ("‡mvbvi", "সোনার"),
        ("gvbyl", "মানুষ"),
        ("wkÿv", "শিক্ষা"),
        ("Kg©", "কর্ম"),
        ("cÖavbgš¿x", "প্রধানমন্ত্রী"),
        ("miKvi", "সরকার"),
        ("XvKv", "ঢাকা"),
        ("evsjv‡`‡ki", "বাংলাদেশের"),
        ("‡KŠkj", "কৌশল"),
        ("RvZxq", "জাতীয়"),
        ("msm`", "সংসদ"),
        ("wbe©vPb", "নির্বাচন"),
        ("¯^vaxbZv", "স্বাধীনতা"),
        ("cwiKíbv", "পরিকল্পনা"),
        ("Dbœqb", "উন্নয়ন"),
        ("A_©bxwZ", "অর্থনীতি"),
        ("¯‹yj", "স্কুল"),
        ("wek¦we`¨vjq", "বিশ্ববিদ্যালয়"),
        ("kZvãx", "শতাব্দী"),
        ("Øviv", "দ্বারা"),
        ("ï×", "শুদ্ধ"),
        ("MÖvg", "গ্রাম"),
        ("Kg©Pvix", "কর্মচারী"),
        ("‡Kv_vq", "কোথায়"),
        ("Pvu`", "চাঁদ"),
        ("j²x", "লক্ষ্মী"),
        ("ivóª", "রাষ্ট্র"),
        ("Bs‡iwR", "ইংরেজি"),
        ("ÔnvÕ", "‘হা’"),
        ("ÒAvgivÓ", "“আমরা”"),
    ];

    #[test]
    fn real_bijoy_words_convert_both_ways() {
        let n = |s: &str| decompose_nukta(s);
        for &(b, u) in WORDS {
            assert_eq!(to_unicode(b), n(u), "Bijoy {b}");
            let enc = from_unicode(u);
            assert_eq!(to_unicode(&enc), n(u), "{u} -> {enc}");
            assert!(!enc.contains('&'), "{u} -> {enc} uses a bare hasanta");
        }
        // Both forms of ে read the same
        assert_eq!(to_unicode("evsjv†`k"), "বাংলাদেশ");
    }

    #[test]
    fn every_ligature_in_the_tables_round_trips() {
        for &(u, b) in ENCODE {
            // Whole conjuncts only (not a lone reph or phala sign)
            let whole = u.chars().next().is_some_and(is_bangla_consonant) && !u.ends_with(HASANTA);
            if !whole || !u.chars().any(|c| c == HASANTA) || u.contains('\u{200C}') {
                continue;
            }
            assert_eq!(to_unicode(b), u, "Bijoy {b} should read as {u}");
            assert_eq!(to_unicode(&from_unicode(u)), u, "{u}");
        }
    }

    #[test]
    fn sentences_keep_their_spaces_and_punctuation() {
        let u = "আমি বাংলায় গান গাই। তুমি কেমন আছ?";
        let b = from_unicode(u);
        assert_eq!(to_unicode(&b), decompose_nukta(u));
        assert!(b.contains('|'));
    }
}
