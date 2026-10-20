use ekushe_core::{
    config::InputLanguage,
    conjuncts::search_signs_and_conjuncts,
    convert::{bijoy_to_unicode, unicode_to_bijoy},
    phonetic::PhoneticEngine,
    Ekushe,
};

#[test]
fn golden_phonetic_words_and_conjuncts() {
    let eng = PhoneticEngine::new();
    let cases: &[(&str, &str)] = &[
        // Classic sentences
        ("ami banglay gan gai.", "আমি বাংলায় গান গাই।"),
        ("amar sOnar bangla, ami tOmay valObasi.", "আমার সোনার বাংলা, আমি তোমায় ভালোবাসি।"),
        // Vowels & Kars
        ("o a i I u U rri e OI O OU", "অ আ ই ঈ উ ঊ ঋ এ ঐ ও ঔ"),
        ("ko ka ki kI ku kU krri ke kOI kO kOU", "ক কা কি কী কু কূ কৃ কে কৈ কো কৌ"),
        ("kee koo koi kou", "কী কূ কৈ কৌ"),
        // Diacritic marks & punctuation
        ("cha^d", "চাঁদ"),
        ("bangla", "বাংলা"),
        ("du:kho", "দুঃখ"),
        ("hothat``", "হঠাৎ"),
        ("hothat/", "হঠাৎ"),
        ("bak,,", "বাক্‌"),
        ("100$", "১০০৳"),
        ("...", "..."),
        ("..", "."),
        // স্ব, শ্ব, ধ্ব, দ্ব, ত্ব, জ্ব (Ba-fola connected letters)
        ("swadhin", "স্বাধীন"),
        ("sbadhin", "স্বাধীন"),
        ("s+badhin", "স্বাধীন"),
        ("swapno", "স্বপ্ন"),
        ("sbopno", "স্বপ্ন"),
        ("swadesh", "স্বদেশ"),
        ("swagatam", "স্বাগতম"),
        ("bishwo", "বিশ্ব"),
        ("bishbo", "বিশ্ব"),
        ("dhwoni", "ধ্বনি"),
        ("dhboni", "ধ্বনি"),
        ("dwitIyo", "দ্বিতীয়"),
        ("dbitIyo", "দ্বিতীয়"),
        ("gurutwo", "গুরুত্ব"),
        ("gurutbo", "গুরুত্ব"),
        ("jwor", "জ্বর"),
        ("jbor", "জ্বর"),
        // Other major connected letters (যুক্তবর্ণ)
        ("shikkha", "শিক্ষা"),
        ("shikSha", "শিক্ষা"),
        ("k+Sha", "ক্ষা"),
        ("ggan", "জ্ঞান"),
        ("jNGan", "জ্ঞান"),
        ("biggan", "বিজ্ঞান"),
        ("rokto", "রক্ত"),
        ("k+to", "ক্ত"),
        ("prothom", "প্রথম"),
        ("chatro", "ছাত্র"),
        ("bakyo", "বাক্য"),
        ("jonyo", "জন্য"),
        ("anondo", "আনন্দ"),
        ("bondhu", "বন্ধু"),
        ("shanto", "শান্ত"),
        ("montro", "মন্ত্র"),
        ("somporko", "সম্পর্ক"),
        ("lomba", "লম্বা"),
        ("nishchoy", "নিশ্চয়"),
        ("koshTo", "কষ্ট"),
        ("shreShTho", "শ্রেষ্ঠ"),
        ("uShNo", "উষ্ণ"),
        ("rasta", "রাস্তা"),
        ("obostha", "অবস্থা"),
        ("brohmo", "ব্রহ্ম"),
        ("onchol", "অঞ্চল"),
        ("gonj", "গঞ্জ"),
        ("bongo", "বঙ্গ"),
        ("onko", "অঙ্ক"),
        ("uttor", "উত্তর"),
        ("zuddho", "যুদ্ধ"),
        // Single `r` before a consonant is a full র; `rr` forces রেফ
        ("korte", "করতে"),
        ("dorkar", "দরকার"),
        ("korrmo", "কর্ম"),
        ("shorrbo", "শর্ব"),
        // Explicit breaker
        ("k`h", "কহ"),
        ("s`w", "সও"),
    ];

    for &(roman, expected) in cases {
        let actual = eng.transliterate(roman);
        assert_eq!(
            actual, expected,
            "Failed on roman input '{roman}': got '{actual}', expected '{expected}'"
        );
    }
}

#[test]
fn golden_sign_and_conjunct_search_includes_sw() {
    let hits = search_signs_and_conjuncts("স্ব");
    assert!(!hits.is_empty());
    assert_eq!(hits[0].bangla, "স্ব");
    assert_eq!(hits[0].primary_key, "sw");
    assert!(hits[0].alt_keys.contains(&"sb"));
}

#[test]
fn golden_session_workflow_and_bijoy() {
    let mut ek = Ekushe::default();
    assert_eq!(ek.language(), InputLanguage::Bangla);

    // Type `/sal` snippet -> commits `আসসালামু আলাইকুম `
    for ch in ['/','s','a','l'] {
        ek.push_char(ch, false);
    }
    let out = ek.push_char(' ', false);
    assert_eq!(out.committed.as_deref(), Some("আসসালামু আলাইকুম "));

    // Bijoy round-trip with স্ব, ক্ষ, জ্ঞ
    let uni = "স্বাধীন বাংলাদেশে শিক্ষা ও বিজ্ঞান।";
    let bijoy = unicode_to_bijoy(uni);
    let back = bijoy_to_unicode(&bijoy);
    assert_eq!(back, uni);
}
