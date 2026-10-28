//! Debug: why a word ranks where it does. `why_rank DICT typed [typed...]`
use ekushe_core::dict::{Dictionary, RankParams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::from_compressed(&std::fs::read(&a[1]).unwrap()).unwrap();
    let p = RankParams::default();
    for typed in &a[2..] {
        println!("── {typed}");
        for r in d.rank(typed, None, 6, &p) {
            let info = d.info(r.id);
            let roms: Vec<String> = d
                .romanizations(r.id)
                .into_iter()
                .take(6)
                .map(|(c, s)| format!("{s}:{c:.2}"))
                .collect();
            println!(
                "  {:>6.2}  {:14} ppm={:>8.2} class={:?}  roms={}",
                r.score,
                r.word,
                10f32.powf(info.log_ppm),
                info.class,
                roms.join(" ")
            );
        }
    }
}
