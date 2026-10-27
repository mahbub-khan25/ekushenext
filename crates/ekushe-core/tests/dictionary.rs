//! Engine behaviour with Ekushe's full dictionary (`data/dictionary/ekushe.dict` or `ekushe.dict.z`).
//! Skipped (passes trivially) when the dictionary file has not been built.

use ekushe_core::dict::Dictionary;
use ekushe_core::Ekushe;
use std::path::Path;

fn dict() -> Option<&'static Dictionary> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dictionary");
    let Some(p) = ["ekushe.dict", "ekushe.dict.z"]
        .iter()
        .map(|f| base.join(f))
        .find(|p| p.exists())
    else {
        eprintln!("ekushe.dict not built; skipping");
        return None;
    };
    Some(Box::leak(Box::new(
        Dictionary::load(&p).expect("load dict"),
    )))
}

fn engine(d: &'static Dictionary) -> Ekushe {
    let mut ek = Ekushe::default();
    ek.suggestions_engine_mut().set_dictionary(Some(d));
    ek
}

fn type_words(ek: &mut Ekushe, text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if let Some(c) = ek.push_char(ch, false).committed {
            out.push_str(&c);
        }
    }
    if let Some(c) = ek.push_char(' ', false).committed {
        out.push_str(&c);
    }
    out
}

#[test]
fn sentences_commit_dictionary_words() {
    let Some(d) = dict() else { return };
    let mut ek = engine(d);
    assert_eq!(
        type_words(&mut ek, "ami tomake valobashi"),
        "আমি তোমাকে ভালোবাসি "
    );
    assert_eq!(
        type_words(&mut ek, "amra sobai bangladesher manush"),
        "আমরা সবাই বাংলাদেশের মানুষ "
    );
    assert_eq!(type_words(&mut ek, "kormo"), "কর্ম ");
}

#[test]
fn standard_spelling_and_colloquial_forms() {
    let Some(d) = dict() else { return };
    let mut ek = engine(d);
    assert_eq!(type_words(&mut ek, "sorkari"), "সরকারি ");
    let out = ek.push_char('k', false);
    assert!(!out.preedit.is_empty());
    for c in "orsi".chars() {
        ek.push_char(c, false);
    }
    let out = ek.push_char('i', false); // "korsii" is not needed; check the candidate list instead
    let _ = out;
    ek.reset_composition();
    let mut cands = Vec::new();
    for c in "korsi".chars() {
        cands = ek.push_char(c, false).candidates;
    }
    let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
    assert!(texts.contains(&"করছি"), "{texts:?}");
}

#[test]
fn preedit_matches_what_space_commits() {
    let Some(d) = dict() else { return };
    let mut ek = engine(d);
    let mut last = Default::default();
    for c in "porikkha".chars() {
        last = ek.push_char(c, false);
    }
    let committed = ek.push_char(' ', false).committed.unwrap();
    assert_eq!(format!("{} ", last.preedit), committed);
    assert_eq!(committed, "পরীক্ষা ");
}

#[test]
fn backspace_undo_still_works() {
    let Some(d) = dict() else { return };
    let mut ek = engine(d);
    type_words(&mut ek, "swadhin");
    assert_eq!(ek.backspace().committed.as_deref(), Some("swadhin"));
}

#[test]
fn spell_checker_uses_dictionary() {
    let Some(d) = dict() else { return };
    let ek = engine(d);
    let sc = ek.suggestions_engine().spell_checker();
    assert!(sc.is_correct("বাংলাদেশ"));
    assert!(!sc.is_correct("কিনতু"));
    assert_eq!(
        sc.suggest("কিনতু", 3).first().map(String::as_str),
        Some("কিন্তু")
    );
    let issues = sc.check_text("সরকারী অফিসে কিনতু");
    assert_eq!(issues.len(), 2, "{issues:?}");
    assert_eq!(issues[0].kind, "non_standard");
    assert_eq!(issues[0].suggestions, vec!["সরকারি".to_string()]);
    assert_eq!(issues[1].kind, "misspelling");
    // precomposed য় (from other keyboards) is accepted
    assert!(sc.is_correct("\u{09AE}\u{09C7}\u{09DF}\u{09C7}"));
}

#[test]
fn bijoy_round_trip_every_dictionary_word() {
    use ekushe_core::convert::{bijoy_to_unicode, normalize_bangla_unicode as n, unicode_to_bijoy};
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dictionary/lexicon.tsv");
    let Ok(text) = std::fs::read_to_string(&p) else {
        return;
    };
    let mut bad = Vec::new();
    let mut total = 0;
    for w in text.lines().skip(1).filter_map(|l| l.split('\t').next()) {
        total += 1;
        let back = bijoy_to_unicode(&unicode_to_bijoy(w));
        if n(&back) != n(w) {
            bad.push(format!("{w} → {} → {back}", unicode_to_bijoy(w)));
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {total} words do not survive Unicode → Bijoy → Unicode: {:?}",
        bad.len(),
        &bad[..bad.len().min(10)]
    );
}

/// Backspace right after a word reopens it with the very suggestions it had (same context, the
/// English spelling among them), and it leaves the recent-words list.
#[test]
fn backspace_reopens_a_word_with_the_same_suggestions() {
    let Some(d) = dict() else { return };
    let mut ek = engine(d);
    type_words(&mut ek, "kemon");
    let mut first = Default::default();
    for ch in "ase".chars() {
        first = ek.push_char(ch, false);
    }
    let first: ekushe_core::SessionOutput = first;
    let committed = ek.push_char(' ', false).committed.unwrap();
    assert!(ek.recent_words().iter().any(|r| r.roman == "ase"));

    let again = ek.reopen_last_word().expect("reopened");
    assert_eq!(again.delete_before, committed.encode_utf16().count());
    let texts = |o: &ekushe_core::SessionOutput| -> Vec<String> {
        o.candidates.iter().map(|c| c.text.clone()).collect()
    };
    assert_eq!(
        texts(&again),
        texts(&first),
        "same context, same suggestions"
    );
    assert!(!ek.recent_words().iter().any(|r| r.roman == "ase"));

    // The English as typed is one of the choices
    let mut ek = engine(d);
    type_words(&mut ek, "facebook");
    let again = ek.reopen_last_word().expect("reopened");
    assert!(
        texts(&again).iter().any(|t| t == "facebook"),
        "{:?}",
        texts(&again)
    );
}
