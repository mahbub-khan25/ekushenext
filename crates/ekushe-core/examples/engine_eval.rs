//! Type every pilot word through the full engine and score what Space commits.
//! cargo run --release -p ekushe-core --example engine_eval -- data/dictionary/ekushe.dict.z tools/dict-builder/eval/pilot.tsv [--fails]
use ekushe_core::config::{Config, TypingMode};
use ekushe_core::dict::Dictionary;
use ekushe_core::Ekushe;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d: &'static Dictionary = Box::leak(Box::new(Dictionary::load(Path::new(&a[1])).unwrap()));
    let tests: Vec<(String, Vec<String>)> = std::fs::read_to_string(&a[2]).unwrap().lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| { let f: Vec<&str> = l.split('\t').collect(); (f[0].to_string(), f[1].split('|').map(String::from).collect()) })
        .collect();
    for (name, mode) in [("Smart (default)", TypingMode::Smart), ("PhoneticFirst", TypingMode::PhoneticFirst)] {
        let mut ok = 0; let mut fails = Vec::new();
        let mut worst = std::time::Duration::ZERO; let mut total = std::time::Duration::ZERO; let mut keys = 0;
        for (typed, acc) in &tests {
            let mut cfg = Config::default(); cfg.typing_mode = mode;
            let mut ek = Ekushe::new(cfg);
            ek.suggestions_engine_mut().set_dictionary(Some(d));
            for c in typed.chars() {
                let t = std::time::Instant::now();
                ek.push_char(c, false);
                let el = t.elapsed(); total += el; worst = worst.max(el); keys += 1;
            }
            let got = ek.push_char(' ', false).committed.unwrap_or_default();
            let got = got.trim_end().to_string();
            if acc.contains(&got) { ok += 1 } else { fails.push(format!("  {typed:16} want {:12} got {got}", acc[0])); }
        }
        println!("{name:24} committed correctly: {ok}/{} = {:.1}%   keystroke latency mean {:.2} ms, worst {:.2} ms",
                 tests.len(), 100.0 * ok as f64 / tests.len() as f64, total.as_secs_f64() * 1000.0 / keys as f64, worst.as_secs_f64() * 1000.0);
        if a.iter().any(|x| x == "--fails") { for f in &fails { println!("{f}"); } }
    }
}
