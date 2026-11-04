//! Voice typing: tidy the text a speech model heard into what a person would have typed.
//!
//! - spoken punctuation: "দাঁড়ি" → ।, "কমা" → ,, "নতুন লাইন" → a line break …
//! - numbers said in words become digits ("দুই হাজার পাঁচশ টাকা" → "২৫০০ টাকা", "উনিশশ একাত্তর" → "১৯৭১");
//!   small counts on their own stay words ("এক কাপ চা")
//! - the person's own corrections (learned from what they fixed after dictating) are applied
//! - spacing around punctuation is cleaned up
//!
//! Everything here is plain text work, so it is the same on every platform.

use std::collections::HashMap;

/// How the heard text is tidied
#[derive(Debug, Clone)]
pub struct VoiceOptions {
    /// Numbers come out as ০-৯ (true) or 0-9 (false)
    pub bangla_digits: bool,
    /// Turn spoken punctuation words into punctuation
    pub spoken_punctuation: bool,
    /// Turn numbers said in words into digits
    pub numbers_as_digits: bool,
}

impl Default for VoiceOptions {
    fn default() -> Self {
        Self {
            bangla_digits: true,
            spoken_punctuation: true,
            numbers_as_digits: true,
        }
    }
}

const UNITS: [&str; 100] = [
    "শূন্য",
    "এক",
    "দুই",
    "তিন",
    "চার",
    "পাঁচ",
    "ছয়",
    "সাত",
    "আট",
    "নয়",
    "দশ",
    "এগারো",
    "বারো",
    "তেরো",
    "চৌদ্দ",
    "পনেরো",
    "ষোলো",
    "সতেরো",
    "আঠারো",
    "উনিশ",
    "বিশ",
    "একুশ",
    "বাইশ",
    "তেইশ",
    "চব্বিশ",
    "পঁচিশ",
    "ছাব্বিশ",
    "সাতাশ",
    "আটাশ",
    "ঊনত্রিশ",
    "ত্রিশ",
    "একত্রিশ",
    "বত্রিশ",
    "তেত্রিশ",
    "চৌত্রিশ",
    "পঁয়ত্রিশ",
    "ছত্রিশ",
    "সাঁইত্রিশ",
    "আটত্রিশ",
    "ঊনচল্লিশ",
    "চল্লিশ",
    "একচল্লিশ",
    "বিয়াল্লিশ",
    "তেতাল্লিশ",
    "চুয়াল্লিশ",
    "পঁয়তাল্লিশ",
    "ছেচল্লিশ",
    "সাতচল্লিশ",
    "আটচল্লিশ",
    "ঊনপঞ্চাশ",
    "পঞ্চাশ",
    "একান্ন",
    "বাহান্ন",
    "তিপ্পান্ন",
    "চুয়ান্ন",
    "পঞ্চান্ন",
    "ছাপ্পান্ন",
    "সাতান্ন",
    "আটান্ন",
    "ঊনষাট",
    "ষাট",
    "একষট্টি",
    "বাষট্টি",
    "তেষট্টি",
    "চৌষট্টি",
    "পঁয়ষট্টি",
    "ছেষট্টি",
    "সাতষট্টি",
    "আটষট্টি",
    "ঊনসত্তর",
    "সত্তর",
    "একাত্তর",
    "বাহাত্তর",
    "তিয়াত্তর",
    "চুয়াত্তর",
    "পঁচাত্তর",
    "ছিয়াত্তর",
    "সাতাত্তর",
    "আটাত্তর",
    "ঊনআশি",
    "আশি",
    "একাশি",
    "বিরাশি",
    "তিরাশি",
    "চুরাশি",
    "পঁচাশি",
    "ছিয়াশি",
    "সাতাশি",
    "আটাশি",
    "ঊননব্বই",
    "নব্বই",
    "একানব্বই",
    "বিরানব্বই",
    "তিরানব্বই",
    "চুরানব্বই",
    "পঁচানব্বই",
    "ছিয়ানব্বই",
    "সাতানব্বই",
    "আটানব্বই",
    "নিরানব্বই",
];

/// Other spellings people (and speech models) use for the same numbers
const UNIT_VARIANTS: &[(&str, u32)] = &[
    ("দুয়ে", 2),
    ("ছ", 6),
    ("ছয়", 6),
    ("নয়", 9),
    ("নও", 9),
    ("এগার", 11),
    ("বার", 12),
    ("তের", 13),
    ("চোদ্দ", 14),
    ("চৌদ্দো", 14),
    ("পনের", 15),
    ("ষোল", 16),
    ("সতের", 17),
    ("আঠার", 18),
    ("উনত্রিশ", 29),
    ("উনচল্লিশ", 39),
    ("উনপঞ্চাশ", 49),
    ("উনষাট", 59),
    ("উনসত্তর", 69),
    ("উনআশি", 79),
    ("উননব্বই", 89),
    ("বিয়াল্লিশ", 42),
    ("ছেচল্লিশ", 46),
    ("তিয়াত্তর", 73),
    ("বাহাত্তর", 72),
];

/// One spelling for ড় ঢ় য় (the single code point and the letter + nukta are the same letter)
fn nk(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\u{9DC}' => o.push_str("\u{9A1}\u{9BC}"),
            '\u{9DD}' => o.push_str("\u{9A2}\u{9BC}"),
            '\u{9DF}' => o.push_str("\u{9AF}\u{9BC}"),
            _ => o.push(c),
        }
    }
    o
}

fn unit_value(w: &str) -> Option<u32> {
    if let Some(i) = UNITS.iter().position(|u| nk(u) == w) {
        return Some(i as u32);
    }
    UNIT_VARIANTS
        .iter()
        .find(|(s, _)| nk(s) == w)
        .map(|(_, v)| *v)
}

/// A word that is (part of) a number: a unit (0-99), a unit with শ/শো/শত glued on (দুইশ = 200), or a
/// multiplier word.
#[derive(Debug, Clone, Copy, PartialEq)]
enum NumWord {
    Unit(u32),
    Hundreds(u32), // দুইশ, পাঁচশো: the unit times 100 (একশ = 100)
    Hundred,       // শ / শো / শত on its own
    Thousand,
    Lakh,
    Crore,
    Half(u32), // দেড় (1½ → 3 halves), আড়াই (2½ → 5 halves)
    AndHalf,   // সাড়ে: the following number plus a half
}

fn num_word(w: &str) -> Option<NumWord> {
    match w {
        "শ" | "শো" | "শত" => return Some(NumWord::Hundred),
        "হাজার" => return Some(NumWord::Thousand),
        "লাখ" | "লক্ষ" => return Some(NumWord::Lakh),
        "কোটি" => return Some(NumWord::Crore),
        "দে\u{9A1}\u{9BC}" => return Some(NumWord::Half(3)),
        "আ\u{9A1}\u{9BC}াই" => return Some(NumWord::Half(5)),
        "সা\u{9A1}\u{9BC}ে" => return Some(NumWord::AndHalf),
        _ => {}
    }
    if let Some(v) = unit_value(w) {
        return Some(NumWord::Unit(v));
    }
    for suf in ["শো", "শত", "শ"] {
        if let Some(stem) = w.strip_suffix(suf) {
            if let Some(v) = unit_value(stem) {
                return Some(NumWord::Hundreds(v));
            }
        }
    }
    None
}

/// Words that may follow a number glued to it (দশটা, পাঁচজন): kept on the digits (১০টা, ৫জন)
const COUNT_SUFFIXES: [&str; 6] = ["টা", "টি", "জন", "খানা", "খানি", "বার"];

/// Words after which even a small number is written in digits (৫ টাকা, ৩ তারিখ)
const DIGIT_CONTEXT: [&str; 16] = [
    "টাকা",
    "তারিখ",
    "সাল",
    "সালে",
    "নম্বর",
    "নং",
    "শতাংশ",
    "ভাগ",
    "মিনিট",
    "সেকেন্ড",
    "কেজি",
    "গ্রাম",
    "কিলোমিটার",
    "মিটার",
    "লিটার",
    "পয়সা",
];

/// Value of a run of number words, or None when they don't make one number. Returns the value in
/// halves (so দেড় হাজার = 3000 halves = 1500) and how many words were used.
#[allow(clippy::manual_is_multiple_of)] // keeps building with Rust 1.85
fn parse_number(words: &[&str]) -> Option<(u64, usize)> {
    let mut total: u64 = 0; // in halves
    let mut cur: u64 = 0; // in halves
    let mut used = 0;
    let mut and_half = false;
    let mut last: Option<NumWord> = None;
    for w in words {
        let Some(n) = num_word(w) else { break };
        match n {
            NumWord::Unit(v) => {
                // two plain units in a row (এক দুই) are two numbers, except a year said in halves
                // (উনিশশ একাত্তর is handled by Hundreds); stop here
                if matches!(last, Some(NumWord::Unit(_)) | Some(NumWord::Half(_))) {
                    break;
                }
                if matches!(last, Some(NumWord::Hundreds(_)) | Some(NumWord::Hundred))
                    && cur % 200 != 0
                {
                    break;
                }
                cur += 2 * v as u64;
            }
            NumWord::Hundreds(v) => {
                if matches!(
                    last,
                    Some(NumWord::Unit(_)) | Some(NumWord::Hundreds(_)) | Some(NumWord::Hundred)
                ) {
                    break;
                }
                cur += 2 * 100 * v as u64;
            }
            NumWord::Hundred => {
                if !matches!(last, Some(NumWord::Unit(_)) | Some(NumWord::Half(_))) {
                    break;
                }
                cur *= 100;
            }
            NumWord::Thousand | NumWord::Lakh | NumWord::Crore => {
                let m: u64 = match n {
                    NumWord::Thousand => 1_000,
                    NumWord::Lakh => 100_000,
                    _ => 10_000_000,
                };
                if cur == 0 {
                    if last.is_none() {
                        break; // "হাজার" alone is a word (হাজার মানুষ), not ১০০০
                    }
                    cur = 2;
                }
                if and_half {
                    cur += 1;
                    and_half = false;
                }
                if m == 10_000_000 {
                    total = (total + cur) * m;
                } else {
                    total += cur * m;
                }
                cur = 0;
            }
            NumWord::Half(h) => {
                if last.is_some() {
                    break;
                }
                cur = h as u64;
            }
            NumWord::AndHalf => {
                if last.is_some() {
                    break;
                }
                and_half = true;
            }
        }
        last = Some(n);
        used += 1;
    }
    if used == 0 || (used == 1 && matches!(last, Some(NumWord::AndHalf))) {
        return None;
    }
    if and_half {
        if cur >= 200 && matches!(last, Some(NumWord::Hundreds(_)) | Some(NumWord::Hundred)) {
            cur += 100; // সাড়ে তিনশ = 350
        } else {
            return None; // "সাড়ে তিন" (time): leave as words
        }
    }
    let v = total + cur;
    if v % 2 != 0 {
        return None; // a half left over (দেড় on its own): not a whole number
    }
    Some((v, used))
}

fn digits(n: u64, bangla: bool) -> String {
    let s = n.to_string();
    if !bangla {
        return s;
    }
    s.chars()
        .map(|c| char::from_u32(c as u32 - '0' as u32 + 0x09E6).unwrap_or(c))
        .collect()
}

/// Spoken punctuation: words (or two-word phrases) → the mark
const PUNCT: &[(&str, &str)] = &[
    ("নতুন অনুচ্ছেদ", "\n\n"),
    ("নতুন প্যারা", "\n\n"),
    ("নতুন লাইন", "\n"),
    ("নতুন লাইক", "\n"),
    ("প্রশ্নবোধক চিহ্ন", "?"),
    ("প্রশ্ন চিহ্ন", "?"),
    ("প্রশ্নচিহ্ন", "?"),
    ("বিস্ময়সূচক চিহ্ন", "!"),
    ("বিস্ময় চিহ্ন", "!"),
    ("বিস্ময়চিহ্ন", "!"),
    ("কোটেশন শুরু", "“"),
    ("কোটেশন শেষ", "”"),
    ("উদ্ধৃতি শুরু", "“"),
    ("উদ্ধৃতি শেষ", "”"),
    ("ব্র্যাকেট শুরু", "("),
    ("ব্র্যাকেট শেষ", ")"),
    ("বন্ধনী শুরু", "("),
    ("বন্ধনী শেষ", ")"),
    ("দাঁড়ি", "।"),
    ("দাড়ি", "।"),
    // how the speech model often hears "দাঁড়ি"
    ("দারি", "।"),
    ("দারী", "।"),
    ("ফুলস্টপ", "।"),
    ("পূর্ণচ্ছেদ", "।"),
    ("কমা", ","),
    ("কোলন", ":"),
    ("সেমিকোলন", ";"),
    ("হাইফেন", "-"),
    ("ড্যাশ", "—"),
];

/// After these words "দাঁড়ি" means a beard, not the full stop
const BEARD_NEXT: [&str; 12] = [
    "কাটা",
    "কাটে",
    "কাটি",
    "কাটলাম",
    "কামানো",
    "কামায়",
    "রাখা",
    "রাখে",
    "রেখেছি",
    "রেখেছে",
    "রেখেছেন",
    "পেকেছে",
];

/// A word that tidying may join with the next one: the first word of a spoken mark ("নতুন" of
/// "নতুন লাইন") or a number word ("দুই" of "দুই হাজার"). Live typing waits for the next word
/// before typing such a word, so the two are tidied together.
pub fn may_join_next(word: &str) -> bool {
    let w = nk(word);
    let starts_mark = PUNCT.iter().any(|(p, _)| {
        let p = nk(p);
        let mut parts = p.split(' ');
        parts.next() == Some(w.as_str()) && parts.next().is_some()
    });
    starts_mark || num_word(&w).is_some()
}

/// How the speech model often hears "দাঁড়ি" right before a pause. These are real words, so only
/// the live typing uses this, and only for the last word of a phrase.
pub fn is_misheard_full_stop(word: &str) -> bool {
    matches!(nk(word).as_str(), "তারই" | "তারি" | "তাড়ি")
}

/// Starts with a mark that belongs to the word before it (no space in front)
pub fn starts_with_mark(text: &str) -> bool {
    text.chars()
        .next()
        .map(|c| is_punct_mark(c.encode_utf8(&mut [0; 4])))
        .unwrap_or(false)
}

fn is_punct_mark(s: &str) -> bool {
    matches!(s, "।" | "," | "?" | "!" | ":" | ";" | "”" | ")")
}

/// Tidy what the speech model heard. `learned` holds the person's own corrections (heard → wanted),
/// whole words or phrases, applied first.
pub fn tidy(heard: &str, opts: &VoiceOptions, learned: &HashMap<String, String>) -> String {
    let mut text = nk(&collapse_repeats(heard));
    // the person's own corrections, longest first so phrases win over single words
    if !learned.is_empty() {
        let mut keys: Vec<&String> = learned.keys().collect();
        keys.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
        for k in keys {
            text = replace_words(&text, &nk(k), &learned[k]);
        }
    }
    // models sometimes add their own punctuation; keep it, but split it off so the words can be read
    let mut words: Vec<String> = Vec::new();
    for w in text.split(' ') {
        let mut w = w.to_string();
        let mut tail = Vec::new();
        while let Some(c) = w.chars().last() {
            if "।,?!:;".contains(c) && w.chars().count() > 1 {
                w.pop();
                tail.push(c.to_string());
            } else {
                break;
            }
        }
        if !w.is_empty() {
            words.push(w);
        }
        words.extend(tail.into_iter().rev());
    }
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        // spoken punctuation (one or two words)
        if opts.spoken_punctuation {
            let two = if i + 1 < words.len() {
                format!("{} {}", words[i], words[i + 1])
            } else {
                String::new()
            };
            if let Some((_, mark)) = PUNCT.iter().find(|(p, _)| !two.is_empty() && nk(p) == two) {
                out.push(mark.to_string());
                i += 2;
                continue;
            }
            if let Some((p, mark)) = PUNCT.iter().find(|(p, _)| nk(p) == words[i]) {
                let next = words.get(i + 1).map(|s| s.as_str()).unwrap_or("");
                let beard = p.contains("াঁ") || p.starts_with("দাড়") || p.starts_with("দা\u{9A1}");
                let beard = beard && p.starts_with("দা") && BEARD_NEXT.iter().any(|b| nk(b) == next);
                if !beard {
                    out.push(mark.to_string());
                    i += 1;
                    continue;
                }
            }
        }
        // numbers said in words
        if opts.numbers_as_digits {
            let slice: Vec<&str> = words[i..].iter().map(|s| s.as_str()).collect();
            // a count suffix glued to the last number word (দশটা, পাঁচশো জন)
            let (mut found, mut suffix) = (parse_number(&slice), "");
            if found.is_none() || found.map(|(_, u)| u) == Some(0) {
                found = None;
            }
            if let Some((_, used)) = found {
                // a longer number may end in a glued suffix: try the next word with its suffix cut off
                if let Some(next) = slice.get(used) {
                    for s in COUNT_SUFFIXES {
                        if let Some(stem) = next.strip_suffix(s) {
                            let mut tryv: Vec<&str> = slice[..used].to_vec();
                            tryv.push(stem);
                            if let Some((v2, u2)) = parse_number(&tryv) {
                                if u2 == used + 1 {
                                    found = Some((v2, u2));
                                    suffix = s;
                                }
                            }
                            break;
                        }
                    }
                }
            } else {
                for s in COUNT_SUFFIXES {
                    if let Some(stem) = slice[0].strip_suffix(s) {
                        if let Some((v, 1)) = parse_number(&[stem]) {
                            found = Some((v, 1));
                            suffix = s;
                        }
                        break;
                    }
                }
            }
            if let Some((halves, used)) = found {
                let value = halves / 2;
                let compound = used > 1;
                let next = slice.get(used).copied().unwrap_or("");
                let context = DIGIT_CONTEXT.iter().any(|d| nk(d) == next);
                if compound || value >= 10 || context || (!suffix.is_empty() && value >= 2) {
                    out.push(format!("{}{}", digits(value, opts.bangla_digits), suffix));
                    i += used;
                    continue;
                }
            }
        }
        out.push(words[i].clone());
        i += 1;
    }
    join_tidy(&out)
}

/// A speech model that gets stuck repeats a word or a phrase over and over ("আমি আমি আমি আমি…").
/// Three or more repeats in a row are cut back to one; a word said twice stays; a phrase of four or
/// more words said twice in a row is said once (people rarely repeat a whole clause word for word).
pub fn collapse_repeats(heard: &str) -> String {
    let mut w: Vec<&str> = heard.split_whitespace().collect();
    for n in 1..=10 {
        let mut out: Vec<&str> = Vec::with_capacity(w.len());
        let mut i = 0;
        while i < w.len() {
            if i + n <= w.len() {
                let mut reps = 1;
                while i + (reps + 1) * n <= w.len()
                    && w[i + reps * n..i + (reps + 1) * n] == w[i..i + n]
                {
                    reps += 1;
                }
                if reps >= 3 || (reps >= 2 && n >= 4) {
                    out.extend_from_slice(&w[i..i + n]);
                    i += reps * n;
                    continue;
                }
            }
            out.push(w[i]);
            i += 1;
        }
        w = out;
    }
    w.join(" ")
}

/// Join words and marks with the spacing a typed sentence has
fn join_tidy(parts: &[String]) -> String {
    let mut s = String::new();
    for p in parts {
        if p.starts_with('\n') {
            while s.ends_with(' ') {
                s.pop();
            }
            s.push_str(p);
            continue;
        }
        if is_punct_mark(p) {
            while s.ends_with(' ') {
                s.pop();
            }
            s.push_str(p);
            continue;
        }
        if !s.is_empty()
            && !s.ends_with('\n')
            && !s.ends_with('“')
            && !s.ends_with('(')
            && !s.ends_with(' ')
        {
            s.push(' ');
        }
        s.push_str(p);
    }
    // only spaces: a line break said at the end of a piece ("… নতুন লাইন", then a pause) must stay
    s.trim_matches(' ').to_string()
}

/// Replace whole words / phrases only (not inside longer words)
fn replace_words(text: &str, from: &str, to: &str) -> String {
    let words: Vec<&str> = text.split(' ').collect();
    let pat: Vec<&str> = from.split(' ').collect();
    if pat.is_empty() || words.len() < pat.len() {
        return text.to_string();
    }
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        if i + pat.len() <= words.len() && words[i..i + pat.len()] == pat[..] {
            out.push(to.to_string());
            i += pat.len();
        } else {
            out.push(words[i].to_string());
            i += 1;
        }
    }
    out.join(" ")
}

/// What the person fixed after dictating, learned for next time (Layer 1). Kept in
/// `<config dir>/voice-learned.tsv`: heard \t wanted \t count. A correction is used once it has been
/// made `min_count` times (1 by default: the person fixed it on purpose).
#[derive(Debug, Default, Clone)]
pub struct VoiceLearned {
    pub map: HashMap<String, (String, u32)>,
}

impl VoiceLearned {
    pub fn parse(tsv: &str) -> Self {
        let mut map = HashMap::new();
        for line in tsv.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() >= 2 && !f[0].is_empty() && !f[1].is_empty() {
                let n = f.get(2).and_then(|x| x.parse().ok()).unwrap_or(1);
                map.insert(f[0].to_string(), (f[1].to_string(), n));
            }
        }
        Self { map }
    }

    pub fn to_tsv(&self) -> String {
        let mut rows: Vec<_> = self.map.iter().collect();
        rows.sort();
        rows.iter()
            .map(|(k, (v, n))| format!("{k}\t{v}\t{n}\n"))
            .collect()
    }

    /// Learn from one fix: the person changed `heard` into `wanted`. Only short fixes (up to three
    /// words each way) are learned, so rewriting a sentence doesn't teach odd rules.
    pub fn learn(&mut self, heard: &str, wanted: &str) {
        let (h, w) = (heard.trim(), wanted.trim());
        if h.is_empty()
            || w.is_empty()
            || h == w
            || h.split(' ').count() > 3
            || w.split(' ').count() > 3
        {
            return;
        }
        let e = self.map.entry(h.to_string()).or_insert((w.to_string(), 0));
        if e.0 != w {
            *e = (w.to_string(), 0);
        }
        e.1 += 1;
    }

    /// Forget a learned fix (the person undid it)
    pub fn forget(&mut self, heard: &str) {
        self.map.remove(heard.trim());
    }

    /// The corrections to apply (made at least `min_count` times)
    pub fn active(&self, min_count: u32) -> HashMap<String, String> {
        self.map
            .iter()
            .filter(|(_, (_, n))| *n >= min_count)
            .map(|(k, (v, _))| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> String {
        tidy(s, &VoiceOptions::default(), &HashMap::new())
    }

    #[test]
    fn spoken_punctuation() {
        assert_eq!(
            t("আমি আসছি দাঁড়ি তুমি কোথায় প্রশ্নবোধক চিহ্ন"),
            "আমি আসছি। তুমি কোথায়?"
        );
        assert_eq!(
            t("প্রিয় ভাইয়া কমা আশা করি ভালো আছেন দাঁড়ি"),
            "প্রিয় ভাইয়া, আশা করি ভালো আছেন।"
        );
        assert_eq!(t("বাজারের তালিকা কোলন নতুন লাইন চাল কমা ডাল"), "বাজারের তালিকা:\nচাল, ডাল");
        assert_eq!(t("কাল ছুটি নতুন অনুচ্ছেদ সবাই আসবে"), "কাল ছুটি\n\nসবাই আসবে");
        assert_eq!(collapse_repeats("আমি আমি আমি আমি যাব"), "আমি যাব");
        assert_eq!(collapse_repeats("ধীরে ধীরে চলো"), "ধীরে ধীরে চলো");
        assert_eq!(
            collapse_repeats("ক্ষমতায় গেলে আপনি মন্ত্রী হবেন ক্ষমতায় গেলে আপনি মন্ত্রী হবেন তাই"),
            "ক্ষমতায় গেলে আপনি মন্ত্রী হবেন তাই"
        );
        assert_eq!(collapse_repeats("আমি যাব আমি যাব"), "আমি যাব আমি যাব");
        assert_eq!(
            collapse_repeats("সে বলল আমি যাব আমি যাব আমি যাব আমি যাব"),
            "সে বলল আমি যাব"
        );
        // said on its own or at the end of a piece (typing goes piece by piece at pauses)
        assert_eq!(t("নতুন লাইন"), "\n");
        assert_eq!(t("গ্রেপ্তার করা হয়নি দারি"), "গ্রেপ্তার করা হয়নি।");
        assert_eq!(t("কারণ নতুন লাইক আজ"), "কারণ\nআজ");
        assert!(super::is_misheard_full_stop("তারই"));
        assert!(super::may_join_next("নতুন"));
        assert!(super::may_join_next("দুই"));
        assert!(super::may_join_next("প্রশ্নবোধক"));
        assert!(!super::may_join_next("বাংলা"));
        assert!(super::starts_with_mark("। আজ"));
        assert!(!super::starts_with_mark("আজ"));
        assert_eq!(t("আজ এই পর্যন্ত দাঁড়ি নতুন অনুচ্ছেদ"), "আজ এই পর্যন্ত।\n\n");
        assert_eq!(
            t("উনি বললেন কোটেশন শুরু আমি আসব কোটেশন শেষ"),
            "উনি বললেন “আমি আসব”"
        );
    }

    #[test]
    fn beard_is_not_a_full_stop() {
        assert_eq!(t("তিনি দাঁড়ি রেখেছেন"), "তিনি দাঁড়ি রেখেছেন");
    }

    #[test]
    fn numbers_become_digits() {
        assert_eq!(
            t("আজ থেকে প্রায় দুইশ পঁয়ত্রিশ বছর আগের কথা"),
            "আজ থেকে প্রায় ২৩৫ বছর আগের কথা"
        );
        assert_eq!(t("দাম দুই হাজার পাঁচশো টাকা"), "দাম ২৫০০ টাকা");
        assert_eq!(t("উনিশশ একাত্তর সালে"), "১৯৭১ সালে");
        assert_eq!(t("এক লাখ পঞ্চাশ হাজার টাকার কিস্তি"), "১৫০০০০ টাকার কিস্তি");
        assert_eq!(t("দেড় হাজার মানুষ"), "১৫০০ মানুষ");
        assert_eq!(t("সাড়ে তিনশ টাকা"), "৩৫০ টাকা");
        assert_eq!(t("আমাদের ক্লাসে পঁয়তাল্লিশ জন"), "আমাদের ক্লাসে ৪৫ জন");
        assert_eq!(t("রাত প্রায় দশটা"), "রাত প্রায় ১০টা");
    }

    #[test]
    fn small_counts_stay_words() {
        assert_eq!(t("এক কাপ চা দাও"), "এক কাপ চা দাও");
        assert_eq!(t("তিন দিন পরে আসব"), "তিন দিন পরে আসব");
        assert_eq!(t("হাজার মানুষ এসেছিল"), "হাজার মানুষ এসেছিল");
        assert_eq!(t("সাড়ে তিনটায় দেখা হবে"), "সাড়ে তিনটায় দেখা হবে");
        // but a small number before টাকা or তারিখ is written in digits
        assert_eq!(t("পাঁচ টাকা দাও"), "৫ টাকা দাও");
    }

    #[test]
    fn either_spelling_of_nukta_letters() {
        // ড় as one code point (U+09DC) or as ড + nukta: both are read
        assert_eq!(t("দে\u{9DC} হাজার"), "১৫০০");
        assert_eq!(t("দে\u{9A1}\u{9BC} হাজার"), "১৫০০");
        assert_eq!(t("সে দা\u{9DC}ি রেখেছে"), "সে দা\u{9A1}\u{9BC}ি রেখেছে");
    }

    #[test]
    fn english_digits_when_asked() {
        let o = VoiceOptions {
            bangla_digits: false,
            ..Default::default()
        };
        assert_eq!(tidy("দুই হাজার পঁচিশ সালে", &o, &HashMap::new()), "2025 সালে");
    }

    #[test]
    fn model_punctuation_is_kept_and_spaced() {
        assert_eq!(t("কেমন আছো ? আমি ভালো ।"), "কেমন আছো? আমি ভালো।");
    }

    #[test]
    fn learned_corrections() {
        let mut l = VoiceLearned::default();
        l.learn("রাহিম", "রহিম");
        l.learn("ঢাকা বিশ্ব বিদ্যালয়", "ঢাকা বিশ্ববিদ্যালয়");
        l.learn("এই পুরো বাক্যটা বদলে দিলাম আজ", "অন্য কিছু"); // too long: not learned
        let a = l.active(1);
        assert_eq!(a.len(), 2);
        let s = tidy("রাহিম ঢাকা বিশ্ব বিদ্যালয় এ পড়ে", &VoiceOptions::default(), &a);
        assert_eq!(s, "রহিম ঢাকা বিশ্ববিদ্যালয় এ পড়ে");
        // a word inside a longer word is not touched
        assert_eq!(replace_words("রাহিমের বই", "রাহিম", "রহিম"), "রাহিমের বই");
        let back = VoiceLearned::parse(&l.to_tsv());
        assert_eq!(back.active(1), a);
        l.forget("রাহিম");
        assert_eq!(l.active(1).len(), 1);
    }
}
