//! Type sentences word by word through the real engine (Smart mode, the dictionary), print what Space commits.
use ekushe_core::config::{Config, TypingMode};
use ekushe_core::dict::Dictionary;
use ekushe_core::Ekushe;
use std::path::Path;
fn main() {
    let d: &'static Dictionary = Box::leak(Box::new(
        Dictionary::load(Path::new("data/dictionary/ekushe.dict.z")).unwrap(),
    ));
    for line in std::env::args().skip(1) {
        let mut ek = Ekushe::new(Config {
            typing_mode: TypingMode::Smart,
            ..Default::default()
        });
        ek.suggestions_engine_mut().set_dictionary(Some(d));
        let mut doc = String::new();
        for c in line.chars() {
            let out = ek.push_char(c, false);
            for _ in 0..out.delete_before {
                doc.pop();
            }
            if let Some(s) = out.committed {
                doc.push_str(&s);
            }
        }
        let out = ek.commit_default(None);
        if let Some(s) = out.committed {
            doc.push_str(&s);
        }
        println!("{line}\n  → {doc}");
    }
}
