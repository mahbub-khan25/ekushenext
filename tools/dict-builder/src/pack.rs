//! `ekushe-dict pack --lexicon lexicon.tsv [--english english-matches.tsv] [--pairs pairs.tsv]
//!                   [--vulgar curated/vulgar-prefixes.tsv] --out ekushe.dict`

use ekushe_core::dict::{DictBuilder, WordClass, WordRecord, FLAG_CONFIDENT, FLAG_CORE, FLAG_DISTINCT, FLAG_SINGLE_SOURCE, FLAG_SUSPECT};
use std::collections::HashMap;
use std::path::Path;

pub fn pack(lexicon: &str, english: Option<&str>, pairs: Option<&str>, vulgar: Option<&str>, out: &str) {
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
    let (bytes, keys) = b.write(Path::new(out)).expect("write");
    eprintln!("wrote {out}: {} words, {keys} keys, {} pairs, {:.1} MB", b.words.len(), b.pairs.len(), bytes as f64 / 1e6);
}
