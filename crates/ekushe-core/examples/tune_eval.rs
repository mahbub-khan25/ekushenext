//! Score several test sets at once (Smart mode, what Space commits), with ranking weights overridden on
//! the command line. Used to tune `RankParams` without overfitting one set:
//! cargo run --release -p ekushe-core --example tune_eval -- DICT set1.tsv,set2.tsv [freq_w=0.55 ...] [--fails]
use ekushe_core::config::{Config, TypingMode};
use ekushe_core::dict::{Dictionary, RankParams};
use ekushe_core::Ekushe;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d: &'static Dictionary = Box::leak(Box::new(Dictionary::load(Path::new(&a[1])).unwrap()));
    let mut p = RankParams::default();
    let fails = a.iter().any(|x| x == "--fails");
    for kv in a.iter().skip(3).filter(|x| x.contains('=')) {
        let (k, v) = kv.split_once('=').unwrap();
        let v: f32 = v.parse().unwrap();
        match k {
            "freq_w" => p.freq_w = v,
            "exact" => p.exact = v,
            "case" => p.case = v,
            "lift" => p.lift = v,
            "final_vowel" => p.final_vowel = v,
            "context_w" => p.context_w = v,
            "completion" => p.completion = v,
            "ctx_max" => p.ctx_max = v,
            "sc_pair" => p.sc_pair = v,
            _ => panic!("unknown parameter {k}"),
        }
    }
    let mut total = (0, 0);
    let mut line = Vec::new();
    for set in a[2].split(',') {
        let tests: Vec<(String, Vec<String>)> = std::fs::read_to_string(set)
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
        let mut ok = 0;
        for (typed, acc) in &tests {
            let mut ek = Ekushe::new(Config {
                typing_mode: TypingMode::Smart,
                ..Default::default()
            });
            ek.suggestions_engine_mut().set_dictionary(Some(d));
            ek.suggestions_engine_mut().set_rank_params(p);
            for c in typed.chars() {
                ek.push_char(c, false);
            }
            let got = ek.push_char(' ', false).committed.unwrap_or_default();
            let got = got.trim_end().to_string();
            if acc.contains(&got) {
                ok += 1;
            } else if fails {
                println!(
                    "  {:10} {typed:24} want {:12} got {got}",
                    Path::new(set).file_stem().unwrap().to_string_lossy(),
                    acc[0]
                );
            }
        }
        total.0 += ok;
        total.1 += tests.len();
        line.push(format!(
            "{}={}/{}",
            Path::new(set).file_stem().unwrap().to_string_lossy(),
            ok,
            tests.len()
        ));
    }
    println!("TOTAL {}/{}  {}", total.0, total.1, line.join("  "));
}
