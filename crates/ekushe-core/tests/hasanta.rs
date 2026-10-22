//! An explicit hasanta (`,,`) is a deliberate spelling: its exact output must be the first
//! candidate (what Space commits), not a dictionary word without the hasanta.
use ekushe_core::{config::Config, Ekushe};

#[test]
fn explicit_hasanta_leads() {
    for (roman, want) in [("allah,,", "আল্লাহ্\u{200C}"), ("bak,,", "বাক্\u{200C}")]
    {
        let mut ek = Ekushe::new(Config::default());
        let mut out = None;
        for c in roman.chars() {
            out = Some(ek.push_char(c, false));
        }
        let first = out
            .unwrap()
            .candidates
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();
        assert_eq!(first, want, "first candidate for {roman}");
    }
}
