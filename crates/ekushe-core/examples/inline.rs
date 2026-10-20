//! Show the inline (preedit) text after each keystroke: cargo run --release -p ekushe-core --example inline -- data/dictionary/ekushe.dict.z swadhin ...
use ekushe_core::dict::Dictionary;
use ekushe_core::Ekushe;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d: &'static Dictionary = Box::leak(Box::new(Dictionary::load(std::path::Path::new(&a[1])).unwrap()));
    for w in &a[2..] {
        let mut ek = Ekushe::default();
        ek.suggestions_engine_mut().set_dictionary(Some(d));
        let steps: Vec<String> = w.chars().map(|c| ek.push_char(c, false).preedit).collect();
        println!("{w}: {}", steps.join(" → "));
    }
}
