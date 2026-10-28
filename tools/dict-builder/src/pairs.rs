//! `ekushe-dict pairs --vocab VOCAB --out FILE [--min N]`: count adjacent word pairs (bigrams) for
//! context, keeping pairs seen at least N times (default 3). `merge-pairs` sums the files counted
//! one corpus file at a time, so a build never needs the whole corpus on disk.
//!
//! Only words in VOCAB (one per line) are counted. Two words form a pair only when separated by
//! whitespace alone; punctuation, digits, Latin text or line breaks end the context.

use crate::norm;
use rustc_hash::FxHashMap;
use std::io::{self, BufRead, BufWriter, Read, Write};

pub fn pairs(vocab_path: &str, out_path: &str, min: u32, cap: usize) {
    let mut vocab: FxHashMap<String, u32> = FxHashMap::default();
    let mut words: Vec<String> = Vec::new();
    for line in io::BufReader::new(std::fs::File::open(vocab_path).expect("vocab")).lines() {
        let w = line.unwrap();
        if !w.is_empty() && !vocab.contains_key(&w) {
            vocab.insert(w.clone(), words.len() as u32);
            words.push(w);
        }
    }
    let mut counts: FxHashMap<u64, u32> = FxHashMap::default();
    let mut stdin = io::stdin().lock();
    let mut chunk = vec![0u8; 1 << 22];
    let mut carry: Vec<u8> = Vec::new();
    let mut pruned = 0u32;
    let mut total: u64 = 0;
    loop {
        let n = stdin.read(&mut chunk).expect("read");
        let eof = n == 0;
        let mut buf = std::mem::take(&mut carry);
        buf.extend_from_slice(&chunk[..n]);
        let cut = if eof { buf.len() } else {
            match buf.iter().rposition(|&b| b == b'\n' || b == 0x1e) { Some(p) => p + 1, None => { carry = buf; continue; } }
        };
        carry.extend_from_slice(&buf[cut..]);
        let text = String::from_utf8_lossy(&buf[..cut]);
        let mut prev: Option<u32> = None;
        let mut start: Option<usize> = None;
        let mut broken = false;
        for (idx, c) in text.char_indices().chain(std::iter::once((text.len(), '\n'))) {
            if norm::is_word_char(c) {
                if start.is_none() { start = Some(idx); }
                continue;
            }
            if let Some(s) = start.take() {
                let id = if c.is_alphanumeric() || ('\u{0980}'..='\u{09FF}').contains(&c) { None }
                         else { norm::normalize_word(&text[s..idx]).and_then(|w| vocab.get(&w).copied()) };
                if let (Some(a), Some(b)) = (prev, id) {
                    if !broken {
                        *counts.entry(((a as u64) << 32) | b as u64).or_insert(0) += 1;
                        total += 1;
                    }
                }
                prev = id;
                broken = false;
            }
            if c == ' ' || c == '\t' || c == '\u{a0}' {
                continue;
            }
            prev = None;           // newline, punctuation, digits, Latin: context ends
            broken = false;
        }
        // Memory: past `cap` distinct pairs, those seen once so far are dropped
        if counts.len() > cap {
            counts.retain(|_, v| *v > 1);
            pruned += 1;
            eprintln!("pruned -> {}", counts.len());
        }
        if eof { break; }
    }
    let mut rows: Vec<(u64, u32)> = counts.into_iter().filter(|(_, c)| *c >= min).collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    let mut w = BufWriter::new(std::fs::File::create(out_path).unwrap());
    writeln!(w, "#pairs\t{total}\tpruned={pruned}").unwrap();
    for (k, c) in rows {
        writeln!(w, "{}\t{}\t{}", words[(k >> 32) as usize], words[(k & 0xffff_ffff) as usize], c).unwrap();
    }
    eprintln!("pairs counted={total}");
}

/// Sum pair files written by `pairs` (`#pairs TOTAL ...` header, then `a TAB b TAB count`) and keep
/// the pairs seen at least `min` times in all.
pub fn merge(files: &[String], out_path: &str, min: u32) {
    let mut ids: FxHashMap<String, u32> = FxHashMap::default();
    let mut words: Vec<String> = Vec::new();
    let mut counts: FxHashMap<u64, u32> = FxHashMap::default();
    let mut total: u64 = 0;
    for f in files {
        let file = io::BufReader::new(std::fs::File::open(f).expect("pairs file"));
        for line in file.lines() {
            let line = line.unwrap();
            let mut p = line.split('\t');
            let (a, b, c) = (p.next().unwrap_or(""), p.next().unwrap_or(""), p.next().unwrap_or(""));
            if a == "#pairs" {
                total += b.parse::<u64>().unwrap_or(0);
                continue;
            }
            let Ok(c) = c.parse::<u32>() else { continue };
            let mut id = |w: &str| -> u32 {
                if let Some(&i) = ids.get(w) {
                    return i;
                }
                let i = words.len() as u32;
                ids.insert(w.to_string(), i);
                words.push(w.to_string());
                i
            };
            let k = ((id(a) as u64) << 32) | id(b) as u64;
            let e = counts.entry(k).or_insert(0);
            *e = e.saturating_add(c);
        }
        eprintln!("merged {f}: {} pairs so far", counts.len());
    }
    let mut rows: Vec<(u64, u32)> = counts.into_iter().filter(|(_, c)| *c >= min).collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    let mut w = BufWriter::new(std::fs::File::create(out_path).unwrap());
    writeln!(w, "#pairs\t{total}\tmerged={}", files.len()).unwrap();
    for (k, c) in &rows {
        writeln!(w, "{}\t{}\t{}", words[(k >> 32) as usize], words[(k & 0xffff_ffff) as usize], c).unwrap();
    }
    eprintln!("wrote {out_path}: {} pairs (>= {min})", rows.len());
}
