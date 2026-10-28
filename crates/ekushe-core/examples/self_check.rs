//! For the most common words: type each word's cheapest romanization and see whether the word comes
//! back first / in the top 3. `self_check DICT WORDS.txt`
use ekushe_core::dict::{Dictionary, RankParams};
use ekushe_core::roman::romanize;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::from_compressed(&std::fs::read(&a[1]).unwrap()).unwrap();
    let p = RankParams::default();
    let (mut n, mut t1, mut t3) = (0, 0, 0);
    let mut misses = Vec::new();
    for w in std::fs::read_to_string(&a[2]).unwrap().lines() {
        let Some((_, r)) = romanize(w, 1).into_iter().next() else {
            continue;
        };
        let got = d.rank(&r, None, 3, &p);
        n += 1;
        if got.first().is_some_and(|g| g.word == w) {
            t1 += 1
        } else if misses.len() < 40 {
            misses.push(format!(
                "{r}→{}≠{w}",
                got.first().map(|g| g.word.as_str()).unwrap_or("-")
            ))
        }
        if got.iter().any(|g| g.word == w) {
            t3 += 1
        }
    }
    println!(
        "n={n} top1={t1} ({:.1}%) top3={t3} ({:.1}%)",
        100.0 * t1 as f64 / n as f64,
        100.0 * t3 as f64 / n as f64
    );
    if a.len() > 3 {
        println!("{}", misses.join("  "));
    }
}
