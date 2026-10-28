//! ekushe-dict: Ekushe Next dictionary builder.
//!
//! `ekushe-dict count --out FILE` reads raw text on stdin (documents separated by U+001E),
//! tokenises Bangla words, normalises them, and writes `word<TAB>count<TAB>doc_count` sorted by count.

mod norm;
mod pairs;
mod pack;

use rustc_hash::FxHashMap;
use std::io::{self, BufRead, BufWriter, Read, Write};

struct Entry {
    count: u64,
    docs: u32,
    last_doc: u32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("count") => {
            let out = args
                .iter()
                .position(|a| a == "--out")
                .and_then(|i| args.get(i + 1))
                .expect("--out FILE required");
            count(out);
        }
        Some("pairs") => {
            let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
            let min = get("--min").and_then(|m| m.parse().ok()).unwrap_or(3);
            pairs::pairs(&get("--vocab").expect("--vocab"), &get("--out").expect("--out"), min);
        }
        Some("merge-pairs") => {
            // ekushe-dict merge-pairs --out FILE [--min N] A.tsv B.tsv ...: sum per-file pair counts
            let mut out = None;
            let mut min = 3u32;
            let mut files = Vec::new();
            let mut it = args.iter().skip(2);
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--out" => out = it.next().cloned(),
                    "--min" => min = it.next().and_then(|m| m.parse().ok()).unwrap_or(3),
                    f => files.push(f.to_string()),
                }
            }
            pairs::merge(&files, &out.expect("--out"), min);
        }
        Some("pack") => {
            let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
            pack::pack(&get("--lexicon").expect("--lexicon"), get("--english").as_deref(), get("--pairs").as_deref(), get("--phrases").as_deref(),
                       get("--vulgar").as_deref(), get("--spoken").as_deref(), &get("--out").expect("--out"));
        }
        Some("roman") => {
            // Print Ekushe's romanizations for words on stdin (parity check with romanize.py)
            for line in io::stdin().lock().lines() {
                let w = line.unwrap();
                let rs = ekushe_core::roman::romanize(w.trim(), 8);
                let joined: Vec<String> = rs.iter().map(|(c, r)| format!("{c:.2}:{r}")).collect();
                let sk = rs.first().map(|(_, r)| ekushe_core::roman::skeleton(r)).unwrap_or_default();
                println!("{}\t{}\t{}", w.trim(), joined.join("|"), sk);
            }
        }
        Some("norm") => {
            // Normalise words from stdin one per line (debug helper)
            for line in io::stdin().lock().lines() {
                let l = line.unwrap();
                println!("{}\t{:?}", l, norm::normalize_word(l.trim()));
            }
        }
        _ => eprintln!("usage: ekushe-dict count --out FILE < text"),
    }
}

fn count(out_path: &str) {
    let mut map: FxHashMap<String, Entry> = FxHashMap::default();
    let mut stdin = io::stdin().lock();
    let mut buf = Vec::with_capacity(1 << 24);
    let mut carry = Vec::new();
    let mut doc: u32 = 0;
    let mut tokens: u64 = 0;
    let mut rejected: u64 = 0;
    let mut mixed: u64 = 0;
    let mut pruned = false;
    let mut chunk = vec![0u8; 1 << 22];

    loop {
        let n = stdin.read(&mut chunk).expect("read stdin");
        let eof = n == 0;
        buf.clear();
        buf.extend_from_slice(&carry);
        buf.extend_from_slice(&chunk[..n]);
        carry.clear();
        // keep incomplete UTF-8 / trailing partial line for next round
        let cut = if eof {
            buf.len()
        } else {
            match buf.iter().rposition(|&b| b == b'\n' || b == 0x1e) {
                Some(p) => p + 1,
                None => {
                    carry.extend_from_slice(&buf);
                    continue;
                }
            }
        };
        carry.extend_from_slice(&buf[cut..]);
        let text = String::from_utf8_lossy(&buf[..cut]);

        let mut start: Option<usize> = None;
        let mut prev_char = ' ';
        let mut iter = text.char_indices().peekable();
        while let Some((idx, c)) = iter.next() {
            if c == '\u{1e}' {
                doc += 1;
            }
            if norm::is_word_char(c) {
                if start.is_none() {
                    start = Some(idx);
                    // token glued to Latin letters/digits or another script -> mixed
                    if prev_char.is_alphanumeric() && !norm::is_word_char(prev_char) && !('\u{09E6}'..='\u{09EF}').contains(&prev_char) {
                        start = Some(usize::MAX);
                    }
                }
            } else if let Some(s) = start.take() {
                let next_mixed = c.is_alphanumeric() && !('\u{09E6}'..='\u{09EF}').contains(&c);
                if s == usize::MAX || next_mixed || ('\u{0980}'..='\u{09FF}').contains(&c) && !('\u{09E6}'..='\u{09EF}').contains(&c) && !matches!(c, '\u{09F2}' | '\u{09F3}') {
                    // mixed-script, or broken by an unsupported Bengali-block char (ৰ ৱ ৺ ...)
                    if s == usize::MAX || next_mixed { mixed += 1; } else { rejected += 1; }
                    // swallow the rest of a broken token
                    if !(s == usize::MAX || next_mixed) {
                        while let Some(&(_, nc)) = iter.peek() {
                            if norm::is_word_char(nc) || ('\u{0980}'..='\u{09FF}').contains(&nc) { iter.next(); } else { break; }
                        }
                    }
                } else {
                    add(&mut map, &text[s..idx], doc, &mut tokens, &mut rejected);
                }
            }
            prev_char = c;
        }
        if let Some(s) = start.take() {
            if s != usize::MAX {
                add(&mut map, &text[s..], doc, &mut tokens, &mut rejected);
            }
        }

        if map.len() > 30_000_000 {
            map.retain(|_, e| e.count > 1);
            pruned = true;
            eprintln!("pruned singletons -> {} entries", map.len());
        }
        if eof {
            break;
        }
    }

    let mut rows: Vec<(String, Entry)> = map.into_iter().collect();
    rows.sort_unstable_by(|a, b| b.1.count.cmp(&a.1.count).then_with(|| a.0.cmp(&b.0)));
    let f = std::fs::File::create(out_path).expect("create out");
    let mut w = BufWriter::new(f);
    writeln!(w, "#tokens\t{tokens}\t{}", doc + 1).unwrap();
    for (word, e) in &rows {
        writeln!(w, "{word}\t{}\t{}", e.count, e.docs).unwrap();
    }
    eprintln!(
        "docs={} tokens={} unique={} rejected={} mixed={} pruned={}",
        doc + 1,
        tokens,
        rows.len(),
        rejected,
        mixed,
        pruned
    );
}

#[inline]
fn add(map: &mut FxHashMap<String, Entry>, raw: &str, doc: u32, tokens: &mut u64, rejected: &mut u64) {
    let Some(word) = norm::normalize_word(raw) else {
        *rejected += 1;
        return;
    };
    *tokens += 1;
    if let Some(e) = map.get_mut(&word) {
        e.count += 1;
        if e.last_doc != doc {
            e.docs += 1;
            e.last_doc = doc;
        }
    } else {
        map.insert(word, Entry { count: 1, docs: 1, last_doc: doc });
    }
}
