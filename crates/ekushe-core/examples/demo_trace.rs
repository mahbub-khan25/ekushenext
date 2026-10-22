//! Print the engine's real output for each keystroke of a phrase (used by the promo video).
//! cargo run --release -p ekushe-core --features embedded-dict --example demo_trace -- "amar bhasha"
use ekushe_core::{config::Config, Ekushe};
fn main() {
    let phrase = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "amar bhasha".into());
    let mut ek = Ekushe::new(Config::default());
    let mut steps = Vec::new();
    for c in phrase.chars() {
        let out = if c == ' ' {
            ek.select_candidate(0, Some(' '))
        } else {
            ek.push_char(c, false)
        };
        steps.push(serde_json::json!({
            "key": c.to_string(),
            "roman": out.roman_buffer,
            "preedit": out.preedit,
            "candidates": out.candidates.iter().take(5).map(|x| x.text.clone()).collect::<Vec<_>>(),
            "committed": out.committed,
        }));
    }
    let end = ek.select_candidate(0, None);
    steps.push(serde_json::json!({"key": "⏎", "committed": end.committed}));
    println!("{}", serde_json::to_string_pretty(&steps).unwrap());
}
