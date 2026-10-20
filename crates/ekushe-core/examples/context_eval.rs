//! Context pilot: cargo run --release -p ekushe-core --example context_eval -- data/dictionary/ekushe.dict.z tools/dict-builder/eval/context.tsv [w1,w2,...]
//! Type each phrase through the engine (Smart = default, and Phonetic-First); score the last committed word.
use ekushe_core::config::{Config, TypingMode};
use ekushe_core::dict::{Dictionary, RankParams};
use ekushe_core::Ekushe;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d: &'static Dictionary = Box::leak(Box::new(Dictionary::load(Path::new(&a[1])).unwrap()));
    let tests: Vec<(String, Vec<String>)> = std::fs::read_to_string(&a[2])
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (
                f[0].to_string(),
                f[1].split('|').map(String::from).collect(),
            )
        })
        .collect();
    let ctx_ws: Vec<f32> = a
        .get(3)
        .map(|s| s.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![RankParams::default().context_w]);
    for cw in ctx_ws {
        for (name, mode) in [
            ("smart", TypingMode::Smart),
            ("phonetic-first", TypingMode::PhoneticFirst),
        ] {
            let mut ok = 0;
            let mut fails = Vec::new();
            for (typed, acc) in &tests {
                let cfg = Config {
                    typing_mode: mode,
                    ..Default::default()
                };
                let mut ek = Ekushe::new(cfg);
                ek.suggestions_engine_mut().set_dictionary(Some(d));
                ek.suggestions_engine_mut().set_rank_params(RankParams {
                    context_w: cw,
                    ..Default::default()
                });
                let mut last = String::new();
                for c in typed.chars().chain(std::iter::once(' ')) {
                    if let Some(cm) = ek.push_char(c, false).committed {
                        last = cm;
                    }
                }
                let got = last.trim_end().to_string();
                if acc.contains(&got) {
                    ok += 1
                } else {
                    fails.push(format!("{typed} → {got} (want {})", acc[0]));
                }
            }
            println!(
                "context_w={cw:.2} {name:8} {ok}/{}  misses: {}",
                tests.len(),
                fails.join("; ")
            );
        }
    }
}
