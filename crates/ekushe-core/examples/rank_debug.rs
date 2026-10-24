//! Show the ranker's top words and scores for typed text, with an optional previous word:
//! cargo run --release -p ekushe-core --example rank_debug -- data/dictionary/ekushe.dict.z ase কেমন
use ekushe_core::dict::{Dictionary, RankParams};
use std::path::Path;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::load(Path::new(&a[1])).unwrap();
    let prev = a.get(3).map(|s| s.as_str());
    for r in d.rank(&a[2], prev, 6, &RankParams::default()) {
        println!(
            "{:8.3}  {}{}  {:?}",
            r.score,
            r.word,
            if r.completion { " (completion)" } else { "" },
            r.class
        );
    }
    if let Some(p) = prev {
        if let Some(id) = d.id_of(p) {
            let f: Vec<String> = d
                .followers(id)
                .into_iter()
                .take(16)
                .map(|(w, c)| format!("{}:{}", d.word(w), c))
                .collect();
            println!("followers of {p}: {}", f.join(" "));
        }
    }
}
