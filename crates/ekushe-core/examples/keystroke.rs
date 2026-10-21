//! Whole-engine time per keystroke (what the user waits for): Ekushe::push_char with the built-in
//! dictionary, over every word of the pilot set, typed letter by letter.
//! cargo run --release -p ekushe-core --features embedded-dict --example keystroke -- tools/dict-builder/eval/pilot.tsv
use ekushe_core::{config::Config, Ekushe};
fn main() {
    let path = std::env::args().nth(1).expect("pilot.tsv");
    let mut ek = Ekushe::new(Config::default());
    let words: Vec<String> = std::fs::read_to_string(path).unwrap().lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split('\t').next().unwrap().to_string()).collect();
    // warm up (dictionary load, caches)
    for c in "amar sonar bangla ".chars() { if c == ' ' { ek.select_candidate(0, Some(' ')); } else { ek.push_char(c, false); } }
    let mut times = Vec::new();
    let t_all = std::time::Instant::now();
    for w in &words {
        for c in w.chars() {
            let t = std::time::Instant::now();
            ek.push_char(c, false);
            times.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        ek.select_candidate(0, Some(' '));
    }
    let total = t_all.elapsed().as_secs_f64();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = times.len();
    let mean: f64 = times.iter().sum::<f64>() / n as f64;
    println!("keystrokes={n} words={} mean={mean:.3}ms median={:.3}ms p95={:.3}ms max={:.3}ms total={total:.2}s", words.len(), times[n/2], times[n*95/100], times[n-1]);
}
