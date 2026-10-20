//! Latency per typed prefix: cargo run --release -p ekushe-core --example latency -- data/dictionary/ekushe.dict.z tools/dict-builder/eval/pilot.tsv
use ekushe_core::dict::{Dictionary, RankParams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::load(std::path::Path::new(&a[1])).unwrap();
    let p = RankParams::default();
    let mut v = Vec::new();
    for l in std::fs::read_to_string(&a[2])
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let w = l.split('\t').next().unwrap();
        for i in 1..=w.len() {
            let pre = &w[..i];
            let t = std::time::Instant::now();
            for _ in 0..3 {
                d.rank(pre, None, 9, &p);
            }
            v.push((t.elapsed().as_secs_f64() * 1000.0 / 3.0, pre.to_string()));
        }
    }
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let n = v.len();
    let mean: f64 = v.iter().map(|x| x.0).sum::<f64>() / n as f64;
    let p95 = v[n / 20].0;
    println!("prefixes={n} mean={mean:.2}ms p95={p95:.2}ms");
    for (t, s) in v.iter().take(12) {
        println!("{t:6.2} ms  {s}");
    }
}
