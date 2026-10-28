//! `ekushe-dict pack --lexicon lexicon.tsv [--english english-matches.tsv] [--pairs pairs.tsv]
//!                   [--vulgar curated/vulgar-prefixes.tsv] --out ekushe.dict`

use ekushe_core::dict::{DictBuilder, WordClass, WordRecord, FLAG_CONFIDENT, FLAG_CORE, FLAG_DISTINCT, FLAG_SINGLE_SOURCE, FLAG_SUSPECT};
use std::collections::HashMap;
use std::path::Path;

/// The count each chat-phrase word pair gets (the corpus pairs are capped at 65,535 in the file)
const CHAT_PAIR_COUNT: u32 = 60_000;

pub fn pack(lexicon: &str, english: Option<&str>, pairs: Option<&str>, phrases: Option<&str>, vulgar: Option<&str>,
            spoken: Option<&str>, out: &str) {
    let mut extra: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(p) = english {
        for line in std::fs::read_to_string(p).expect("english").lines() {
            if line.starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() >= 2 {
                extra.entry(f[0].to_string()).or_default().push(f[1].to_string());
            }
        }
    }
    let vulgar: Vec<String> = vulgar
        .map(|p| {
            std::fs::read_to_string(p).expect("vulgar").lines()
                .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                .map(|l| l.split('\t').next().unwrap().trim().to_string())
                .collect()
        })
        .unwrap_or_default();
    let mut b = DictBuilder { max_followers: 16, ..Default::default() };
    let text = std::fs::read_to_string(lexicon).expect("lexicon");
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 10 {
            continue;
        }
        let mut class = WordClass::parse(f[3]);
        let mut flags = 0u8;
        if f[2] == "core" {
            flags |= FLAG_CORE;
        }
        if f[9].contains("suspect_typo_of") {
            flags |= FLAG_SUSPECT;
        }
        if f[9].contains("distinct_word") {
            flags |= FLAG_DISTINCT;
        }
        if f[9].contains("single_source") {
            flags |= FLAG_SINGLE_SOURCE;
        }
        if f[8] == "high" && (f[7].starts_with("curated") || f[7].starts_with("academy")) {
            flags |= FLAG_CONFIDENT;
        }
        if vulgar.iter().any(|v| f[0].starts_with(v.as_str())) {
            class = WordClass::Vulgar;
        }
        let extra_roms = if class == WordClass::Vulgar {
            Vec::new()
        } else {
            extra.get(f[0]).map(|v| v.iter().map(|e| (0.5f32, e.clone())).collect()).unwrap_or_default()
        };
        b.words.push(WordRecord {
            word: f[0].to_string(),
            ppm: f[1].parse().unwrap_or(0.0),
            class,
            flags,
            preferred: if f[4].is_empty() { None } else { Some(f[4].to_string()) },
            extra_roms,
        });
    }
    if let Some(p) = spoken {
        apply_spoken(&mut b, p);
    }
    if let Some(p) = pairs {
        for line in std::fs::read_to_string(p).expect("pairs").lines() {
            if line.starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() == 3 {
                let c: u32 = f[2].parse().unwrap_or(0);
                if c >= 5 {
                    b.pairs.push((f[0].to_string(), f[1].to_string(), c));
                }
            }
        }
    }
    // Everyday chat phrases (curated/chat-phrases.txt): each pair of neighbouring words gets a strong
    // count, so conversation is ranked well even though the source texts are mostly formal
    if let Some(p) = phrases {
        let mut chat: std::collections::HashMap<(String, String), u32> = std::collections::HashMap::new();
        for line in std::fs::read_to_string(p).expect("phrases").lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let words: Vec<String> = line
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| "।,?!.;:\"'".contains(c)).to_string())
                .filter(|w| !w.is_empty())
                .collect();
            for w in words.windows(2) {
                chat.insert((w[0].clone(), w[1].clone()), CHAT_PAIR_COUNT);
            }
        }
        let n = chat.len();
        for (a, b_, c) in b.pairs.iter_mut() {
            if let Some(cc) = chat.remove(&(a.clone(), b_.clone())) {
                *c = (*c).max(cc);
            }
        }
        b.pairs.extend(chat.into_iter().map(|((a, b_), c)| (a, b_, c)));
        eprintln!("chat phrases: {n} word pairs");
    }
    let (bytes, keys) = b.write(Path::new(out)).expect("write");
    eprintln!("wrote {out}: {} words, {keys} keys, {} pairs, {:.1} MB", b.words.len(), b.pairs.len(), bytes as f64 / 1e6);
}

/// How often an everyday spoken word is assumed to occur at least (per million words). The source
/// texts are mostly written Bangla, where chat forms like খাইছো or আসোনি are almost absent.
const SPOKEN_PPM: f64 = 3.0;
/// Typing cost of a spelling listed for a spoken word (cheap enough to count as an exact match)
const SPOKEN_KEY_COST: f32 = 0.1;

/// `curated/spoken.tsv` (`word, typed spellings, standard form [, ppm]`): everyday spoken words get a
/// realistic frequency and the spellings people type for them. A word with a standard form becomes a
/// colloquial form of it, so the standard spelling is offered first and the spoken one right after.
fn apply_spoken(b: &mut DictBuilder, path: &str) {
    let mut index: HashMap<String, usize> =
        b.words.iter().enumerate().map(|(i, w)| (w.word.clone(), i)).collect();
    let (mut n, mut added) = (0, 0);
    for line in std::fs::read_to_string(path).expect("spoken").lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').map(str::trim).collect();
        let word = f[0];
        let keys: Vec<&str> = f.get(1).map(|k| k.split(',').map(str::trim).filter(|k| !k.is_empty()).collect()).unwrap_or_default();
        let standard = f.get(2).copied().unwrap_or("");
        let floor = f.get(3).and_then(|p| p.parse::<f64>().ok()).unwrap_or(SPOKEN_PPM);
        let mut ensure = |b: &mut DictBuilder, w: &str| -> usize {
            if let Some(&i) = index.get(w) {
                return i;
            }
            b.words.push(WordRecord {
                word: w.to_string(),
                ppm: 0.0,
                class: WordClass::Standard,
                flags: FLAG_CORE,
                preferred: None,
                extra_roms: Vec::new(),
            });
            index.insert(w.to_string(), b.words.len() - 1);
            added += 1;
            b.words.len() - 1
        };
        let i = ensure(b, word);
        if b.words[i].class == WordClass::Vulgar {
            continue;
        }
        let r = &mut b.words[i];
        r.ppm = r.ppm.max(floor);
        r.flags = (r.flags | FLAG_CORE) & !FLAG_SUSPECT;
        r.extra_roms.extend(keys.iter().map(|k| (SPOKEN_KEY_COST, k.to_string())));
        if !standard.is_empty() && standard != word {
            r.class = WordClass::Colloquial;
            r.preferred = Some(standard.to_string());
            let s = ensure(b, standard);
            let sr = &mut b.words[s];
            sr.ppm = sr.ppm.max(floor);
            sr.flags = (sr.flags | FLAG_CORE) & !FLAG_SUSPECT;
        }
        n += 1;
    }
    eprintln!("spoken words: {n} ({added} not in the lexicon)");
}
