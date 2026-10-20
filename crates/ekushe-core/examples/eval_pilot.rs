//! cargo run --release -p ekushe-core --example eval_pilot -- data/dictionary/ekushe.dict.z tools/dict-builder/eval/pilot.tsv [--fails]
use ekushe_core::dict::{Dictionary, RankParams};
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::load(Path::new(&a[1])).expect("dict");
    let p = RankParams::default();
    let mut stats: std::collections::BTreeMap<String, (u32, u32, u32)> = Default::default();
    let mut fails = Vec::new();
    let mut total = std::time::Duration::ZERO;
    let mut worst = std::time::Duration::ZERO;
    for line in std::fs::read_to_string(&a[2]).unwrap().lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let (typed, acc, style) = (f[0], f[1].split('|').collect::<Vec<_>>(), f[2]);
        let (ctx, word) = match typed.rsplit_once(' ') {
            Some((c, w)) => (Some(c), w),
            None => (None, typed),
        };
        let prev = ctx.map(|c| {
            let r = d.rank(c.split(' ').next_back().unwrap(), None, 1, &p);
            r.first().map(|x| x.word.clone()).unwrap_or_default()
        });
        let t = std::time::Instant::now();
        let res = d.rank(word, prev.as_deref(), 3, &p);
        let el = t.elapsed();
        total += el;
        worst = worst.max(el);
        let top1 = res
            .first()
            .map(|r| acc.contains(&r.word.as_str()))
            .unwrap_or(false);
        let top3 = res.iter().any(|r| acc.contains(&r.word.as_str()));
        for st in [style, "all"] {
            let e = stats.entry(st.to_string()).or_default();
            e.0 += 1;
            e.1 += top1 as u32;
            e.2 += top3 as u32;
        }
        if !top1 {
            fails.push(format!(
                "  {typed:16} want {:12} got {}",
                acc[0],
                res.iter()
                    .map(|r| r.word.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    }
    for (k, (n, t1, t3)) in &stats {
        println!(
            "{k:7} n={n:3}  top1={:5.1}%  top3={:5.1}%",
            100.0 * *t1 as f64 / *n as f64,
            100.0 * *t3 as f64 / *n as f64
        );
    }
    let n = stats["all"].0;
    println!(
        "latency: mean {:.2} ms, worst {:.2} ms",
        total.as_secs_f64() * 1000.0 / n as f64,
        worst.as_secs_f64() * 1000.0
    );
    if a.iter().any(|x| x == "--fails") {
        for f in fails {
            println!("{f}");
        }
    }
}
