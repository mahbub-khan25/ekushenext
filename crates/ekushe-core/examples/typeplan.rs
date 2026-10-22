//! For each "roman<TAB>target" pair (one word per line, blank line = Enter), type the roman letters
//! through the live session, report where the target sits in the candidate list, and commit it.
//! Lines starting with '@' are raw text typed in passthrough (punctuation the phonetic layer can't make).
use ekushe_core::{config::Config, Ekushe};
fn main() {
    let path = std::env::args().nth(1).expect("plan file");
    let mut ek = Ekushe::new(Config::default());
    let mut doc = String::new();
    let mut bad = 0;
    for line in std::fs::read_to_string(path).unwrap().lines() {
        if line.is_empty() { doc.push('\n'); continue; }
        if let Some(raw) = line.strip_prefix('@') { doc.push_str(raw); continue; }
        let (roman, target) = line.split_once('\t').unwrap();
        let mut out = Default::default();
        for c in roman.chars() { out = ek.push_char(c, false); }
        let out: ekushe_core::SessionOutput = out;
        let texts: Vec<String> = out.candidates.iter().map(|c| c.text.clone()).collect();
        let idx = texts.iter().position(|t| t == target);
        match idx {
            Some(i) => {
                if i > 0 { println!("TAB x{i}: {roman} -> {target}   list={texts:?}"); }
                let o = ek.select_candidate(i, Some(' '));
                doc.push_str(&o.committed.unwrap());
            }
            None => {
                bad += 1;
                println!("MISSING: {roman} -> {target}   list={texts:?}");
                let o = ek.select_candidate(0, Some(' '));
                doc.push_str(&o.committed.unwrap());
            }
        }
    }
    println!("---- bad={bad}\n{doc}");
}
