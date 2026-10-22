//! For Roman words that use an explicit spelling mark, show the exact (literal) output and
//! where it sits in the candidate list. cargo run --example marks -- word...
use ekushe_core::{config::Config, phonetic::PhoneticEngine, Ekushe};
fn main() {
    for w in std::env::args().skip(1) {
        let mut ek = Ekushe::new(Config::default());
        let mut out = None;
        for c in w.chars() {
            out = Some(ek.push_char(c, false));
        }
        let out = out.unwrap();
        let lit = &PhoneticEngine::new().transliterate_word(&w);
        let pos = out.candidates.iter().position(|c| &c.text == lit);
        let top: Vec<_> = out
            .candidates
            .iter()
            .take(4)
            .map(|c| c.text.clone())
            .collect();
        let tips: Vec<_> = out
            .sign_hints
            .iter()
            .filter(|h| out.candidates.iter().any(|c| c.text == h.result_bangla))
            .map(|h| format!("{} ← {}", h.result_bangla, h.key_hint))
            .collect();
        println!(
            "{w:<14} literal={lit:<12} rank={:<5} top={}  tips=[{}]",
            pos.map(|p| (p + 1).to_string()).unwrap_or("-".into()),
            top.join(" | "),
            tips.join(", ")
        );
    }
}
