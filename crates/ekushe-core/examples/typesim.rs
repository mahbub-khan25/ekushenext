//! Simulate typing Roman text through the full Ekushe session (space commits top candidate,
//! newline = Enter commits then line break). Prints the resulting document.
use ekushe_core::{config::Config, Ekushe};
fn main() {
    let path = std::env::args().nth(1).expect("roman file");
    let text = std::fs::read_to_string(path).unwrap();
    let mut ek = Ekushe::new(Config::default());
    let mut doc = String::new();
    for c in text.chars() {
        let out = if c == '\n' {
            ek.commit_default(None)
        } else {
            ek.push_char(c, false)
        };
        for _ in 0..out.delete_before {
            doc.pop();
        }
        if let Some(s) = out.committed {
            doc.push_str(&s);
        }
        if c == '\n' {
            doc.push('\n');
        }
    }
    let out = ek.commit_default(None);
    if let Some(s) = out.committed {
        doc.push_str(&s);
    }
    print!("{doc}");
}
