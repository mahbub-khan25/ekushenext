//! Show the ranking for one input: cargo run --release -p ekushe-core --example rank -- data/dictionary/ekushe.dict.z aso কেমন [context_w]
use ekushe_core::dict::{Dictionary, RankParams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::load(std::path::Path::new(&a[1])).unwrap();
    let prev = if a[3].is_empty() {
        None
    } else {
        Some(a[3].as_str())
    };
    let cw: f32 = a.get(4).map(|x| x.parse().unwrap()).unwrap_or(0.35);
    for r in d.rank(
        &a[2],
        prev,
        8,
        &RankParams {
            context_w: cw,
            ..Default::default()
        },
    ) {
        println!(
            "{:8.3} {} {:?} comp={}",
            r.score, r.word, r.class, r.completion
        );
    }
}
