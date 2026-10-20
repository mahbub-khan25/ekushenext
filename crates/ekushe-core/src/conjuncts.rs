//! Bangla Signs (কার / চিহ্ন), Folas (ফলা), and Connected Letters (যুক্তবর্ণ) Assistant.
//!
//! Makes typing vowel signs, diacritics, and conjuncts effortless for both beginners
//! and advanced users by providing:
//! 1. Multi-alias phonetic shortcuts (e.g., `sw`/`sb`/`s+b` -> `স্ব`; `kkh`/`kSh` -> `ক্ষ`;
//!    `gg`/`jNG` -> `জ্ঞ`; `+` explicitly joins any two consonants `k+t` -> `ক্ত`).
//! 2. Instant contextual "Next-Key Hints" when the user types a base consonant or prefix,
//!    showing how to add কার (vowel signs), ফলা (r/y/w/m/l/n fola), or যুক্তবর্ণ (conjuncts).
//! 3. Searchable series-organized breakdown of every Bangla sign and connected letter with
//!    rich real-world word examples (Bangla + Roman phonetic spelling + English meaning).

use serde::{Deserialize, Serialize};

/// Category of a Bangla sign or connected letter helper entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignKind {
    /// Vowel sign attached to a consonant (া, ি, ী, ু, ূ, ৃ, ে, ৈ, ো, ৌ)
    Kar,
    /// Diacritic / modifier mark (ঁ চন্দ্রবিন্দু, ং অনুস্বার, ঃ বিসর্গ, ৎ খণ্ড-ত, ্ হসন্ত, । দাড়ি, ৳ টাকা)
    Mark,
    /// Consonant appendage (য-ফলা ্য, র-ফলা ্র, রেফ র্, ব-ফলা ্ব, ম-ফলা ্ম, ল-ফলা ্ল, ন-ফলা ্ন)
    Fola,
    /// Connected consonant cluster (যুক্তবর্ণ, e.g. ক্ত, ক্ষ, ঙ্ক, ঙ্গ, চ্চ, চ্ছ, জ্ঞ, ষ্ট, স্ব...)
    Conjunct,
}

/// A real-world example word for a conjunct, fola, or kar.
#[derive(Debug, Clone, Serialize)]
pub struct WordExample {
    /// Bangla word (e.g. "রক্ত")
    pub bangla: &'static str,
    /// How to type in Ekushe phonetic (e.g. "rokto")
    pub roman: &'static str,
    /// English meaning (e.g. "Blood")
    pub meaning: &'static str,
}

/// A rich entry describing how to write a Bangla sign, fola, or connected letter.
#[derive(Debug, Clone, Serialize)]
pub struct SignGuideEntry {
    /// Actual Bangla sequence inserted when attached (e.g., "ক্ত", "ক্ষ", "স্ব", "্র", "ী")
    pub bangla: &'static str,
    /// Complete shaped Bangla glyph for UI display (never starts with a bare combining mark,
    /// so it never renders a broken dotted circle `◌` in any font).
    pub display_bangla: &'static str,
    /// Series identifier for filtering in the Helper Page (e.g., "K-Series", "Ch-Series", "T-Series", "N-Series", "P-Series", "Sh-Series", "D-Series", "L-Series", "Fola", "Kar", "Mark")
    pub series: &'static str,
    /// Summary example string
    pub example: &'static str,
    /// Primary easiest Roman key sequence (e.g., "kt", "kkh", "sw")
    pub primary_key: &'static str,
    /// Alternative Roman sequences (e.g., &["k+t"])
    pub alt_keys: &'static [&'static str],
    /// Clean visual letter-by-letter formula without bare combining marks after spaces
    /// (e.g., "ক + ত = ক্ত (k + ta)")
    pub decomposition: &'static str,
    /// Human-readable Bangla + English name
    pub name: &'static str,
    /// Category
    pub kind: SignKind,
    /// Structured list of 5+ example words with Bangla, Roman spelling, and English meaning
    pub words: &'static [WordExample],
}

/// Interactive hint shown in the candidate bar when the user is mid-word.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignHint {
    /// Resulting Bangla cluster or syllable if completed (e.g., "স্ব", "ক্ষ", "কী", "ক্র")
    pub result_bangla: String,
    /// Roman suffix to type next or full shortcut (e.g., "sw", "kkh", "krri")
    pub key_hint: String,
    /// Short explanation without bare combining marks (e.g., "স+ব", "ক+ষ", "ঋ-কার", "র-ফলা")
    pub label: String,
    /// Category
    pub kind: SignKind,
}

/// Complete reference table organized by Series (`K-Series`, `Ch-Series`, `T-Series`, `N-Series`,
/// `P-Series`, `Sh-Series`, `D-Series`, `L-Series`, `Fola`, `Kar`, `Mark`) with rich word examples.
pub static SIGN_AND_CONJUNCT_GUIDE: &[SignGuideEntry] = &[
    // ════════════════════════════════════════════════════════════════════════
    // 1. K-Series (ক, গ, ঙ)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "ক্ত",
        display_bangla: "ক্ত",
        series: "K-Series",
        example: "রক্ত (rokto), ভক্ত (bhokto), শক্তি (shokti)",
        primary_key: "kt",
        alt_keys: &["k+t"],
        decomposition: "ক + ত = ক্ত (k + ta)",
        name: "ক্ত (ক+ত যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "রক্ত",
                roman: "rokto",
                meaning: "Blood",
            },
            WordExample {
                bangla: "ভক্ত",
                roman: "bhokto",
                meaning: "Devotee",
            },
            WordExample {
                bangla: "শক্তি",
                roman: "shokti",
                meaning: "Power/Energy",
            },
            WordExample {
                bangla: "মুক্তি",
                roman: "mukti",
                meaning: "Freedom",
            },
            WordExample {
                bangla: "ব্যক্ত",
                roman: "byokto",
                meaning: "Expressed",
            },
            WordExample {
                bangla: "যুক্ত",
                roman: "jukto",
                meaning: "Connected",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ক্ষ",
        display_bangla: "ক্ষ",
        series: "K-Series",
        example: "শিক্ষা (shikkha), ক্ষমা (khoma), লক্ষ্য (lokkho)",
        primary_key: "kkh",
        alt_keys: &["kSh", "k+Sh"],
        decomposition: "ক + ষ = ক্ষ (k + ssa)",
        name: "ক্ষ (ক+ষ যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "শিক্ষা",
                roman: "shikkha",
                meaning: "Education",
            },
            WordExample {
                bangla: "ক্ষমা",
                roman: "khoma / kkhoma",
                meaning: "Forgiveness",
            },
            WordExample {
                bangla: "ক্ষতি",
                roman: "khoti / kkhoti",
                meaning: "Harm/Loss",
            },
            WordExample {
                bangla: "লক্ষ্য",
                roman: "lokkho / lokkhyo",
                meaning: "Goal/Target",
            },
            WordExample {
                bangla: "দক্ষ",
                roman: "dokkho",
                meaning: "Skilled",
            },
            WordExample {
                bangla: "পরীক্ষা",
                roman: "porikkha",
                meaning: "Exam/Test",
            },
            WordExample {
                bangla: "বৃক্ষ",
                roman: "briksho / brrikkho",
                meaning: "Tree",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ঙ্ক",
        display_bangla: "ঙ্ক",
        series: "K-Series",
        example: "অঙ্ক (onko), ব্যাংক (bank), আশঙ্কা (ashongka)",
        primary_key: "nk",
        alt_keys: &["ngk", "NGk", "NG+k"],
        decomposition: "ঙ + ক = ঙ্ক (ng + ka)",
        name: "ঙ্ক (ঙ+ক যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "অঙ্ক",
                roman: "onko",
                meaning: "Math/Digit",
            },
            WordExample {
                bangla: "ব্যাংক",
                roman: "bank / byank",
                meaning: "Bank",
            },
            WordExample {
                bangla: "আশঙ্কা",
                roman: "ashongka",
                meaning: "Anxiety/Fear",
            },
            WordExample {
                bangla: "অহঙ্কার",
                roman: "ahongkar / ohongkar",
                meaning: "Arrogance",
            },
            WordExample {
                bangla: "কঙ্কাল",
                roman: "kongkal",
                meaning: "Skeleton",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ঙ্গ",
        display_bangla: "ঙ্গ",
        series: "K-Series",
        example: "সঙ্গী (shonggi), বাঙালি (bangali), মঙ্গল (monggol)",
        primary_key: "ngg",
        alt_keys: &["NGg", "NG+g"],
        decomposition: "ঙ + গ = ঙ্গ (ng + ga)",
        name: "ঙ্গ (ঙ+গ যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "সঙ্গী",
                roman: "shonggi / songi",
                meaning: "Companion",
            },
            WordExample {
                bangla: "বাঙালি",
                roman: "bangali",
                meaning: "Bengali person",
            },
            WordExample {
                bangla: "রঙ্গিন",
                roman: "ronggin",
                meaning: "Colorful",
            },
            WordExample {
                bangla: "অঙ্গ",
                roman: "onggo",
                meaning: "Body part",
            },
            WordExample {
                bangla: "মঙ্গল",
                roman: "monggol",
                meaning: "Welfare/Tuesday",
            },
            WordExample {
                bangla: "বঙ্গ",
                roman: "bonggo / bongo",
                meaning: "Bengal",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ক্ল",
        display_bangla: "ক্ল",
        series: "K-Series",
        example: "ক্লান্ত (klanto), ক্লাস (klas)",
        primary_key: "kl",
        alt_keys: &["k+l"],
        decomposition: "ক + ল = ক্ল (k + la)",
        name: "ক্ল (ক+ল যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "ক্লান্ত",
                roman: "klanto",
                meaning: "Tired",
            },
            WordExample {
                bangla: "ক্লাস",
                roman: "klas",
                meaning: "Class",
            },
            WordExample {
                bangla: "ক্লাব",
                roman: "klab",
                meaning: "Club",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ক্স",
        display_bangla: "ক্স",
        series: "K-Series",
        example: "বাক্স (bakso), ট্যাক্সি (tyaksi)",
        primary_key: "ks",
        alt_keys: &["x", "k+s"],
        decomposition: "ক + স = ক্স (k + sa)",
        name: "ক্স (ক+স যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "বাক্স",
                roman: "bakso",
                meaning: "Box",
            },
            WordExample {
                bangla: "ট্যাক্সি",
                roman: "tyaksi",
                meaning: "Taxi",
            },
            WordExample {
                bangla: "কক্সবাজার",
                roman: "koksobajar",
                meaning: "Cox's Bazar",
            },
        ],
    },
    SignGuideEntry {
        bangla: "গ্ধ",
        display_bangla: "গ্ধ",
        series: "K-Series",
        example: "মুগ্ধ (mugdho), দুগ্ধ (dugdho)",
        primary_key: "gdh",
        alt_keys: &["g+dh"],
        decomposition: "গ + ধ = গ্ধ (g + dha)",
        name: "গ্ধ (গ+ধ যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "মুগ্ধ",
                roman: "mugdho",
                meaning: "Fascinated/Charmed",
            },
            WordExample {
                bangla: "দুগ্ধ",
                roman: "dugdho",
                meaning: "Milk",
            },
            WordExample {
                bangla: "দগ্ধ",
                roman: "dogdho",
                meaning: "Burned",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 2. Ch-Series (চ, জ, ঞ)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "চ্চ",
        display_bangla: "চ্চ",
        series: "Ch-Series",
        example: "উচ্চ (uccho), বাচ্চা (bacca), উচ্চারণ (uccaron)",
        primary_key: "cc",
        alt_keys: &["chch", "c+c"],
        decomposition: "চ + চ = চ্চ (ch + cha)",
        name: "চ্চ (চ+চ যুক্তবর্ণ · Ch-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "উচ্চ",
                roman: "uccho / ucco",
                meaning: "High",
            },
            WordExample {
                bangla: "বাচ্চা",
                roman: "bacca / bachcha",
                meaning: "Child",
            },
            WordExample {
                bangla: "উচ্চারণ",
                roman: "uccaron",
                meaning: "Pronunciation",
            },
            WordExample {
                bangla: "সচ্চরিত্র",
                roman: "soccoritro",
                meaning: "Good character",
            },
            WordExample {
                bangla: "বিচ্ছিন্ন",
                roman: "bicchinno",
                meaning: "Disconnected",
            },
        ],
    },
    SignGuideEntry {
        bangla: "চ্ছ",
        display_bangla: "চ্ছ",
        series: "Ch-Series",
        example: "ইচ্ছা (iccha), আচ্ছা (accha), স্বচ্ছ (shoccho)",
        primary_key: "cch",
        alt_keys: &["chchh", "c+chh"],
        decomposition: "চ + ছ = চ্ছ (ch + chha)",
        name: "চ্ছ (চ+ছ যুক্তবর্ণ · Ch-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "ইচ্ছা",
                roman: "iccha",
                meaning: "Wish/Desire",
            },
            WordExample {
                bangla: "আচ্ছা",
                roman: "accha",
                meaning: "Okay/Alright",
            },
            WordExample {
                bangla: "স্বচ্ছ",
                roman: "shoccho / swoccho",
                meaning: "Transparent/Clear",
            },
            WordExample {
                bangla: "উচ্ছেদ",
                roman: "ucched",
                meaning: "Eviction",
            },
            WordExample {
                bangla: "আচ্ছাদন",
                roman: "acchadon",
                meaning: "Covering",
            },
            WordExample {
                bangla: "শুভেচ্ছা",
                roman: "shubheccha",
                meaning: "Greetings",
            },
        ],
    },
    SignGuideEntry {
        bangla: "জ্ঞ",
        display_bangla: "জ্ঞ",
        series: "Ch-Series",
        example: "জ্ঞান (gyan / ggan), বিজ্ঞান (bigyan), প্রতিজ্ঞা (protigga)",
        primary_key: "gg",
        alt_keys: &["gy", "jNG", "j+NG"],
        decomposition: "জ + ঞ = জ্ঞ (j + nyo)",
        name: "জ্ঞ (জ+ঞ যুক্তবর্ণ · Ch-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "জ্ঞান",
                roman: "gyan / ggan",
                meaning: "Knowledge",
            },
            WordExample {
                bangla: "বিজ্ঞান",
                roman: "bigyan / biggan",
                meaning: "Science",
            },
            WordExample {
                bangla: "অজ্ঞ",
                roman: "oggo",
                meaning: "Ignorant",
            },
            WordExample {
                bangla: "বিজ্ঞাপন",
                roman: "biggapon",
                meaning: "Advertisement",
            },
            WordExample {
                bangla: "প্রতিজ্ঞা",
                roman: "protigga",
                meaning: "Promise",
            },
            WordExample {
                bangla: "জিজ্ঞাসা",
                roman: "jiggasha",
                meaning: "Question/Inquiry",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ঞ্চ",
        display_bangla: "ঞ্চ",
        series: "Ch-Series",
        example: "পঞ্চ (poncho), অঞ্চল (onchol), চঞ্চল (chonchol)",
        primary_key: "nch",
        alt_keys: &["NGch", "NG+ch"],
        decomposition: "ঞ + চ = ঞ্চ (nyo + cha)",
        name: "ঞ্চ (ঞ+চ যুক্তবর্ণ · Ch-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "পঞ্চ",
                roman: "poncho",
                meaning: "Five",
            },
            WordExample {
                bangla: "অঞ্চল",
                roman: "onchol",
                meaning: "Region/Zone",
            },
            WordExample {
                bangla: "পঞ্চম",
                roman: "ponchom",
                meaning: "Fifth",
            },
            WordExample {
                bangla: "চঞ্চল",
                roman: "chonchol",
                meaning: "Restless",
            },
            WordExample {
                bangla: "বঞ্চনা",
                roman: "bonchona",
                meaning: "Deprivation",
            },
            WordExample {
                bangla: "মঞ্চ",
                roman: "moncho",
                meaning: "Stage",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ঞ্জ",
        display_bangla: "ঞ্জ",
        series: "Ch-Series",
        example: "পাঞ্জাবি (panjabi), গঞ্জ (gonj)",
        primary_key: "nj",
        alt_keys: &["NGj", "NG+j"],
        decomposition: "ঞ + জ = ঞ্জ (nyo + ja)",
        name: "ঞ্জ (ঞ+জ যুক্তবর্ণ · Ch-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "পাঞ্জাবি",
                roman: "panjabi",
                meaning: "Panjabi attire",
            },
            WordExample {
                bangla: "গঞ্জ",
                roman: "gonj",
                meaning: "Market town",
            },
            WordExample {
                bangla: "রঞ্জন",
                roman: "ronjon",
                meaning: "Coloring/Pleasing",
            },
        ],
    },
    SignGuideEntry {
        bangla: "জ্জ",
        display_bangla: "জ্জ",
        series: "Ch-Series",
        example: "লজ্জা (lojja), উজ্জ্বল (ujjwol)",
        primary_key: "jj",
        alt_keys: &["j+j"],
        decomposition: "জ + জ = জ্জ (j + ja)",
        name: "জ্জ (জ+জ যুক্তবর্ণ · Ch-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "লজ্জা",
                roman: "lojja",
                meaning: "Shyness/Shame",
            },
            WordExample {
                bangla: "সজ্জা",
                roman: "shojja",
                meaning: "Decoration",
            },
            WordExample {
                bangla: "উজ্জ্বল",
                roman: "ujjwol",
                meaning: "Bright/Radiant",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 3. T-Series (ত, ট, ড)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "ট্ট",
        display_bangla: "ট্ট",
        series: "T-Series",
        example: "চট্টগ্রাম (chottogram), অট্টালিকা (ottalika)",
        primary_key: "TT",
        alt_keys: &["T+T"],
        decomposition: "ট + ট = ট্ট (tt + tta)",
        name: "ট্ট (ট+ট যুক্তবর্ণ · T-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "চট্টগ্রাম",
                roman: "chottogram",
                meaning: "Chittagong",
            },
            WordExample {
                bangla: "অট্টালিকা",
                roman: "ottalika",
                meaning: "Building/Mansion",
            },
            WordExample {
                bangla: "ভট্টাচার্য",
                roman: "bhottachorjo",
                meaning: "Bhattacharya (surname)",
            },
            WordExample {
                bangla: "টট্টর",
                roman: "tottor",
                meaning: "Chattering sound",
            },
            WordExample {
                bangla: "ঠাট্টা",
                roman: "thaTTa",
                meaning: "Joke/Jest",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ত্ত",
        display_bangla: "ত্ত",
        series: "T-Series",
        example: "উত্তেজনা (uttejna), উত্তর (uttor), সম্পত্তি (shompotti)",
        primary_key: "tt",
        alt_keys: &["t+t"],
        decomposition: "ত + ত = ত্ত (t + ta)",
        name: "ত্ত (ত+ত যুক্তবর্ণ · T-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "উত্তেজনা",
                roman: "uttejna / uttejona",
                meaning: "Excitement",
            },
            WordExample {
                bangla: "উত্তর",
                roman: "uttor",
                meaning: "Answer/North",
            },
            WordExample {
                bangla: "সম্পত্তি",
                roman: "shompotti",
                meaning: "Property",
            },
            WordExample {
                bangla: "উত্তম",
                roman: "uttom",
                meaning: "Excellent",
            },
            WordExample {
                bangla: "বিপত্তি",
                roman: "bipotti",
                meaning: "Adversity",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ত্র",
        display_bangla: "ত্র",
        series: "T-Series",
        example: "পত্র (potro), ছাত্র (chatro), মিত্র (mitro)",
        primary_key: "tr",
        alt_keys: &["t+r"],
        decomposition: "ত + র = ত্র (t + ra-phala)",
        name: "ত্র (ত+র যুক্তবর্ণ · T-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "পত্র",
                roman: "potro",
                meaning: "Letter/Page",
            },
            WordExample {
                bangla: "ছাত্র",
                roman: "chatro",
                meaning: "Student",
            },
            WordExample {
                bangla: "মিত্র",
                roman: "mitro",
                meaning: "Friend",
            },
            WordExample {
                bangla: "ত্রাণ",
                roman: "tran",
                meaning: "Relief/Rescue",
            },
            WordExample {
                bangla: "রাত্রি",
                roman: "ratri",
                meaning: "Night",
            },
            WordExample {
                bangla: "চিত্র",
                roman: "chitro",
                meaning: "Picture/Art",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ত্ব",
        display_bangla: "ত্ব",
        series: "T-Series",
        example: "গুরুত্ব (gurutwo), নেতৃত্ব (netritwo), ব্যক্তিত্ব (byoktitwo)",
        primary_key: "tw",
        alt_keys: &["tb", "t+b"],
        decomposition: "ত + ব = ত্ব (t + wa-phala)",
        name: "ত্ব (ত+ব যুক্তবর্ণ · T-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "গুরুত্ব",
                roman: "gurutwo",
                meaning: "Importance",
            },
            WordExample {
                bangla: "সত্যত্ব",
                roman: "sottwo",
                meaning: "Truthfulness",
            },
            WordExample {
                bangla: "নেতৃত্ব",
                roman: "netritwo",
                meaning: "Leadership",
            },
            WordExample {
                bangla: "ঘনত্ব",
                roman: "ghonotwo",
                meaning: "Density",
            },
            WordExample {
                bangla: "ব্যক্তিত্ব",
                roman: "byoktitwo",
                meaning: "Personality",
            },
            WordExample {
                bangla: "দায়িত্ব",
                roman: "dayitwo",
                meaning: "Responsibility",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ত্ম",
        display_bangla: "ত্ম",
        series: "T-Series",
        example: "আত্মা (atma), মহাত্মা (mohatma)",
        primary_key: "tm",
        alt_keys: &["t+m"],
        decomposition: "ত + ম = ত্ম (t + ma)",
        name: "ত্ম (ত+ম যুক্তবর্ণ · T-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "আত্মা",
                roman: "atma",
                meaning: "Soul/Spirit",
            },
            WordExample {
                bangla: "মহাত্মা",
                roman: "mohatma",
                meaning: "Great soul",
            },
            WordExample {
                bangla: "আত্মবিশ্বাস",
                roman: "atmobishwas",
                meaning: "Self-confidence",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ত্ন",
        display_bangla: "ত্ন",
        series: "T-Series",
        example: "যত্ন (jotno), রত্ন (rotno)",
        primary_key: "tn",
        alt_keys: &["t+n"],
        decomposition: "ত + ন = ত্ন (t + na)",
        name: "ত্ন (ত+ন যুক্তবর্ণ · T-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "যত্ন",
                roman: "jotno",
                meaning: "Care",
            },
            WordExample {
                bangla: "রত্ন",
                roman: "rotno",
                meaning: "Gem/Jewel",
            },
            WordExample {
                bangla: "প্রযত্ন",
                roman: "projotno",
                meaning: "Effort",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 4. N-Series (ন, ণ)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "ণ্ট",
        display_bangla: "ণ্ট",
        series: "N-Series",
        example: "ঘণ্টা (ghonta), কণ্টক (kontok), প্যান্ট (pant)",
        primary_key: "NT",
        alt_keys: &["nT", "N+T"],
        decomposition: "ণ + ট = ণ্ট (nn + tta)",
        name: "ণ্ট (ণ+ট যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "ঘণ্টা",
                roman: "ghonta",
                meaning: "Hour/Bell",
            },
            WordExample {
                bangla: "কণ্টক",
                roman: "kontok",
                meaning: "Thorn",
            },
            WordExample {
                bangla: "প্যান্ট",
                roman: "pant",
                meaning: "Pants",
            },
            WordExample {
                bangla: "কাউন্টার",
                roman: "kauntar",
                meaning: "Counter",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ণ্ঠ",
        display_bangla: "ণ্ঠ",
        series: "N-Series",
        example: "কণ্ঠ (kontho), লণ্ঠন (lonthon), কুণ্ঠা (kuntha)",
        primary_key: "NTh",
        alt_keys: &["nTh", "N+Th"],
        decomposition: "ণ + ঠ = ণ্ঠ (nn + thha)",
        name: "ণ্ঠ (ণ+ঠ যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "কণ্ঠ",
                roman: "kontho",
                meaning: "Voice/Throat",
            },
            WordExample {
                bangla: "লণ্ঠন",
                roman: "lonthon",
                meaning: "Lantern",
            },
            WordExample {
                bangla: "কুণ্ঠা",
                roman: "kuntha",
                meaning: "Hesitation",
            },
            WordExample {
                bangla: "উৎকণ্ঠা",
                roman: "utkontha",
                meaning: "Anxiety",
            },
            WordExample {
                bangla: "উপকণ্ঠ",
                roman: "upokontho",
                meaning: "Outskirts",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ন্দ",
        display_bangla: "ন্দ",
        series: "N-Series",
        example: "সুন্দর (shundor), আনন্দ (anondo), মন্দির (mondir)",
        primary_key: "nd",
        alt_keys: &["n+d"],
        decomposition: "ন + দ = ন্দ (n + da)",
        name: "ন্দ (ন+দ যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "সুন্দর",
                roman: "shundor",
                meaning: "Beautiful",
            },
            WordExample {
                bangla: "আনন্দ",
                roman: "anondo",
                meaning: "Joy",
            },
            WordExample {
                bangla: "মন্দির",
                roman: "mondir",
                meaning: "Temple",
            },
            WordExample {
                bangla: "ছন্দ",
                roman: "chhondo",
                meaning: "Rhythm",
            },
            WordExample {
                bangla: "আন্দোলন",
                roman: "andolon",
                meaning: "Movement/Protest",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ন্ধ",
        display_bangla: "ন্ধ",
        series: "N-Series",
        example: "অন্ধ (ondho), বন্ধু (bondhu), সন্ধ্যা (shondhya)",
        primary_key: "ndh",
        alt_keys: &["n+dh"],
        decomposition: "ন + ধ = ন্ধ (n + dha)",
        name: "ন্ধ (ন+ধ যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "অন্ধ",
                roman: "ondho",
                meaning: "Blind",
            },
            WordExample {
                bangla: "বন্ধু",
                roman: "bondhu",
                meaning: "Friend",
            },
            WordExample {
                bangla: "সন্ধ্যা",
                roman: "shondhya",
                meaning: "Evening",
            },
            WordExample {
                bangla: "বন্ধ",
                roman: "bondho",
                meaning: "Closed",
            },
            WordExample {
                bangla: "গন্ধ",
                roman: "gondho",
                meaning: "Smell/Fragrance",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ন্ত",
        display_bangla: "ন্ত",
        series: "N-Series",
        example: "কিন্তু (kintu), শান্ত (shanto), বসন্ত (boshonto)",
        primary_key: "nt",
        alt_keys: &["n+t"],
        decomposition: "ন + ত = ন্ত (n + ta)",
        name: "ন্ত (ন+ত যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "কিন্তু",
                roman: "kintu",
                meaning: "But",
            },
            WordExample {
                bangla: "শান্ত",
                roman: "shanto",
                meaning: "Calm",
            },
            WordExample {
                bangla: "অন্তত",
                roman: "ontoto",
                meaning: "At least",
            },
            WordExample {
                bangla: "বসন্ত",
                roman: "boshonto",
                meaning: "Spring",
            },
            WordExample {
                bangla: "অনন্ত",
                roman: "ononto",
                meaning: "Endless",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ন্ত্র",
        display_bangla: "ন্ত্র",
        series: "N-Series",
        example: "মন্ত্র (montro), যন্ত্র (jontro), গণতন্ত্র (gonotontro)",
        primary_key: "ntr",
        alt_keys: &["n+t+r"],
        decomposition: "ন + ত + র = ন্ত্র (n + t + ra)",
        name: "ন্ত্র (ন+ত+র যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "মন্ত্র",
                roman: "montro",
                meaning: "Mantra/Chant",
            },
            WordExample {
                bangla: "যন্ত্র",
                roman: "jontro",
                meaning: "Machine/Instrument",
            },
            WordExample {
                bangla: "গণতন্ত্র",
                roman: "gonotontro",
                meaning: "Democracy",
            },
            WordExample {
                bangla: "নিয়ন্ত্রণ",
                roman: "niyontron",
                meaning: "Control",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ণ্ড",
        display_bangla: "ণ্ড / ন্ড",
        series: "N-Series",
        example: "ঠান্ডা (thanda), প্রচণ্ড (prochondo), কাণ্ড (kaNDo)",
        primary_key: "ND",
        alt_keys: &["nD", "N+D"],
        decomposition: "ণ/ন + ড = ণ্ড/ন্ড (nn/n + dda)",
        name: "ণ্ড / ন্ড (ণ+ড যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "ঠান্ডা",
                roman: "thanda / thanDa",
                meaning: "Cold",
            },
            WordExample {
                bangla: "প্রচণ্ড",
                roman: "prochondo",
                meaning: "Intense/Extreme",
            },
            WordExample {
                bangla: "কাণ্ড",
                roman: "kaNDo",
                meaning: "Incident/Trunk",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ন্ন",
        display_bangla: "ন্ন",
        series: "N-Series",
        example: "অন্ন (onno), রান্না (ranna), সম্পন্ন (shomponno)",
        primary_key: "nn",
        alt_keys: &["n+n"],
        decomposition: "ন + ন = ন্ন (n + na)",
        name: "ন্ন (ন+ন যুক্তবর্ণ · N-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "অন্ন",
                roman: "onno",
                meaning: "Rice/Food",
            },
            WordExample {
                bangla: "রান্না",
                roman: "ranna",
                meaning: "Cooking",
            },
            WordExample {
                bangla: "সম্পন্ন",
                roman: "shomponno",
                meaning: "Completed/Accomplished",
            },
            WordExample {
                bangla: "ভিন্ন",
                roman: "bhinno",
                meaning: "Different",
            },
            WordExample {
                bangla: "উন্নতি",
                roman: "unnoti",
                meaning: "Progress",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 5. P-Series (প, ব, ম)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "প্ত",
        display_bangla: "প্ত",
        series: "P-Series",
        example: "প্রাপ্ত (prapto), সপ্ত (sopto), গুপ্ত (gupto), সপ্তাহ (soptaho)",
        primary_key: "pt",
        alt_keys: &["p+t"],
        decomposition: "প + ত = প্ত (p + ta)",
        name: "প্ত (প+ত যুক্তবর্ণ · P-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "প্রাপ্ত",
                roman: "prapto",
                meaning: "Received",
            },
            WordExample {
                bangla: "সপ্ত",
                roman: "sopto",
                meaning: "Seven",
            },
            WordExample {
                bangla: "গুপ্ত",
                roman: "gupto",
                meaning: "Hidden/Secret",
            },
            WordExample {
                bangla: "সুপ্ত",
                roman: "supto",
                meaning: "Dormant",
            },
            WordExample {
                bangla: "সপ্তাহ",
                roman: "soptaho",
                meaning: "Week",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ম্প",
        display_bangla: "ম্প",
        series: "P-Series",
        example: "কম্পিউটার (kompiutar), সম্পর্ক (shomporko), সম্পূর্ণ (shompurno)",
        primary_key: "mp",
        alt_keys: &["m+p"],
        decomposition: "ম + প = ম্প (m + pa)",
        name: "ম্প (ম+প যুক্তবর্ণ · P-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "কম্পিউটার",
                roman: "kompiutar",
                meaning: "Computer",
            },
            WordExample {
                bangla: "সম্পর্ক",
                roman: "shomporko",
                meaning: "Relationship",
            },
            WordExample {
                bangla: "সম্পূর্ণ",
                roman: "shompurno",
                meaning: "Complete",
            },
            WordExample {
                bangla: "কম্পন",
                roman: "kompon",
                meaning: "Vibration/Tremor",
            },
            WordExample {
                bangla: "সম্পদ",
                roman: "shompod",
                meaning: "Wealth/Asset",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ম্ব",
        display_bangla: "ম্ব",
        series: "P-Series",
        example: "লম্বা (lomba), কম্বল (kombol), বিলম্ব (bilombo)",
        primary_key: "mb",
        alt_keys: &["m+b"],
        decomposition: "ম + ব = ম্ব (m + ba)",
        name: "ম্ব (ম+ব যুক্তবর্ণ · P-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "লম্বা",
                roman: "lomba",
                meaning: "Tall/Long",
            },
            WordExample {
                bangla: "কম্বল",
                roman: "kombol",
                meaning: "Blanket",
            },
            WordExample {
                bangla: "চৌম্বক",
                roman: "choumbok",
                meaning: "Magnet",
            },
            WordExample {
                bangla: "অম্বল",
                roman: "ombol",
                meaning: "Acidity/Sour dish",
            },
            WordExample {
                bangla: "বিলম্ব",
                roman: "bilombo",
                meaning: "Delay",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ম্ভ",
        display_bangla: "ম্ভ",
        series: "P-Series",
        example: "সম্ভব (shombhob), গম্ভীর (gombhir), সম্ভাবনা (shombhabona)",
        primary_key: "mbh",
        alt_keys: &["mv", "m+bh"],
        decomposition: "ম + ভ = ম্ভ (m + bha)",
        name: "ম্ভ (ম+ভ যুক্তবর্ণ · P-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "সম্ভব",
                roman: "shombhob",
                meaning: "Possible",
            },
            WordExample {
                bangla: "গম্ভীর",
                roman: "gombhir",
                meaning: "Serious/Deep",
            },
            WordExample {
                bangla: "অসম্ভব",
                roman: "oshombhob",
                meaning: "Impossible",
            },
            WordExample {
                bangla: "স্তম্ভ",
                roman: "stombho",
                meaning: "Pillar",
            },
            WordExample {
                bangla: "সম্ভাবনা",
                roman: "shombhabona",
                meaning: "Probability/Potential",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ম্ম",
        display_bangla: "ম্ম",
        series: "P-Series",
        example: "সম্মান (shomman), সম্মতি (shommoti), আম্মা (amma)",
        primary_key: "mm",
        alt_keys: &["m+m"],
        decomposition: "ম + ম = ম্ম (m + ma)",
        name: "ম্ম (ম+ম যুক্তবর্ণ · P-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "সম্মান",
                roman: "shomman",
                meaning: "Respect/Honor",
            },
            WordExample {
                bangla: "সম্মতি",
                roman: "shommoti",
                meaning: "Consent",
            },
            WordExample {
                bangla: "আম্মা",
                roman: "amma",
                meaning: "Mother",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ব্দ",
        display_bangla: "ব্দ",
        series: "P-Series",
        example: "শব্দ (shobdo), শতাব্দী (shotabdI)",
        primary_key: "bd",
        alt_keys: &["b+d"],
        decomposition: "ব + দ = ব্দ (b + da)",
        name: "ব্দ (ব+দ যুক্তবর্ণ · P-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "শব্দ",
                roman: "shobdo",
                meaning: "Word/Sound",
            },
            WordExample {
                bangla: "শতাব্দী",
                roman: "shotabdI",
                meaning: "Century",
            },
            WordExample {
                bangla: "জব্দ",
                roman: "jobdo",
                meaning: "Subdued/Confiscated",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 6. Sh-Series (শ, ষ, স)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "শ্চ",
        display_bangla: "শ্চ",
        series: "Sh-Series",
        example: "নিশ্চয় (nishchoy), পশ্চিম (poshchim), আশ্চর্য (ashchorjo)",
        primary_key: "shch",
        alt_keys: &["sh+ch"],
        decomposition: "শ + চ = শ্চ (sh + cha)",
        name: "শ্চ (শ+চ যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "নিশ্চয়",
                roman: "nishchoy",
                meaning: "Definitely",
            },
            WordExample {
                bangla: "পশ্চিম",
                roman: "poshchim",
                meaning: "West",
            },
            WordExample {
                bangla: "আশ্চর্য",
                roman: "ashchorjo",
                meaning: "Surprised/Wonderful",
            },
            WordExample {
                bangla: "দুশ্চিন্তা",
                roman: "dushchinta",
                meaning: "Anxiety/Worry",
            },
            WordExample {
                bangla: "দুশ্চরিত্র",
                roman: "dushchoritro",
                meaning: "Bad character",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ষ্ট",
        display_bangla: "ষ্ট",
        series: "Sh-Series",
        example: "কষ্ট (koshto), মিষ্টি (mishti), বৃষ্টি (brishti), সৃষ্টি (srishti)",
        primary_key: "ShT",
        alt_keys: &["shT", "Sh+T"],
        decomposition: "ষ + ট = ষ্ট (ss + tta)",
        name: "ষ্ট (ষ+ট যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "কষ্ট",
                roman: "koshto / koshTo",
                meaning: "Trouble/Pain",
            },
            WordExample {
                bangla: "মিষ্টি",
                roman: "mishti / miShTi",
                meaning: "Sweet",
            },
            WordExample {
                bangla: "নষ্ট",
                roman: "noshto / noShTo",
                meaning: "Ruined/Spoiled",
            },
            WordExample {
                bangla: "বৃষ্টি",
                roman: "brishti / brriShTi",
                meaning: "Rain",
            },
            WordExample {
                bangla: "সৃষ্টি",
                roman: "srishti / srriShTi",
                meaning: "Creation",
            },
            WordExample {
                bangla: "দৃষ্টি",
                roman: "drishti / drriShTi",
                meaning: "Vision/Sight",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ষ্ঠ",
        display_bangla: "ষ্ঠ",
        series: "Sh-Series",
        example: "শ্রেষ্ঠ (shreshtho), অনুষ্ঠান (onushthan), পৃষ্ঠা (prishtha)",
        primary_key: "ShTh",
        alt_keys: &["shTh", "Sh+Th"],
        decomposition: "ষ + ঠ = ষ্ঠ (ss + thha)",
        name: "ষ্ঠ (ষ+ঠ যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "শ্রেষ্ঠ",
                roman: "shreshtho",
                meaning: "Best",
            },
            WordExample {
                bangla: "অনুষ্ঠান",
                roman: "onushthan",
                meaning: "Event/Program",
            },
            WordExample {
                bangla: "প্রতিষ্ঠিত",
                roman: "protishthit",
                meaning: "Established",
            },
            WordExample {
                bangla: "পৃষ্ঠা",
                roman: "prishtha / prriShTha",
                meaning: "Page",
            },
            WordExample {
                bangla: "নিষ্ঠা",
                roman: "nishtha",
                meaning: "Dedication",
            },
        ],
    },
    SignGuideEntry {
        bangla: "স্ক",
        display_bangla: "স্ক",
        series: "Sh-Series",
        example: "নমস্কার (nomoshkar), স্কুল (skul), পুরস্কার (puroshkar)",
        primary_key: "sk",
        alt_keys: &["s+k"],
        decomposition: "স + ক = স্ক (s + ka)",
        name: "স্ক (স+ক যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "নমস্কার",
                roman: "nomoshkar",
                meaning: "Hello/Greetings",
            },
            WordExample {
                bangla: "স্কুল",
                roman: "skul",
                meaning: "School",
            },
            WordExample {
                bangla: "পুরস্কার",
                roman: "puroshkar",
                meaning: "Reward/Award",
            },
            WordExample {
                bangla: "তিরস্কার",
                roman: "tiroshkar",
                meaning: "Scolding/Rebuke",
            },
            WordExample {
                bangla: "ভাস্কর",
                roman: "bhaskor",
                meaning: "Sculptor",
            },
        ],
    },
    SignGuideEntry {
        bangla: "স্ত",
        display_bangla: "স্ত",
        series: "Sh-Series",
        example: "রাস্তা (rasta), সস্তা (shoshta), ব্যস্ত (byosto), সমস্ত (shomosto)",
        primary_key: "st",
        alt_keys: &["s+t"],
        decomposition: "স + ত = স্ত (s + ta)",
        name: "স্ত (স+ত যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "রাস্তা",
                roman: "rasta",
                meaning: "Road",
            },
            WordExample {
                bangla: "সস্তা",
                roman: "shoshta / sosta",
                meaning: "Cheap",
            },
            WordExample {
                bangla: "ব্যস্ত",
                roman: "byosto",
                meaning: "Busy",
            },
            WordExample {
                bangla: "সমস্ত",
                roman: "shomosto",
                meaning: "All",
            },
            WordExample {
                bangla: "হস্ত",
                roman: "hosto",
                meaning: "Hand",
            },
        ],
    },
    SignGuideEntry {
        bangla: "স্থ",
        display_bangla: "স্থ",
        series: "Sh-Series",
        example: "স্থান (sthan), অবস্থা (obostha), সুস্থ (shustho), উপস্থিত (uposthit)",
        primary_key: "sth",
        alt_keys: &["s+th"],
        decomposition: "স + থ = স্থ (s + tha)",
        name: "স্থ (স+থ যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "স্থান",
                roman: "sthan",
                meaning: "Place",
            },
            WordExample {
                bangla: "অবস্থা",
                roman: "obostha",
                meaning: "Condition/Situation",
            },
            WordExample {
                bangla: "সুস্থ",
                roman: "shustho / sustho",
                meaning: "Healthy",
            },
            WordExample {
                bangla: "উপস্থিত",
                roman: "uposthit",
                meaning: "Present",
            },
            WordExample {
                bangla: "স্থায়ী",
                roman: "sthayi",
                meaning: "Permanent",
            },
        ],
    },
    SignGuideEntry {
        bangla: "স্প",
        display_bangla: "স্প",
        series: "Sh-Series",
        example: "স্পষ্ট (shoposhto), স্পর্শ (shoporsho), স্পন্দন (spondon)",
        primary_key: "sp",
        alt_keys: &["s+p"],
        decomposition: "স + প = স্প (s + pa)",
        name: "স্প (স+প যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "স্পষ্ট",
                roman: "shoposhto / sposhto",
                meaning: "Clear",
            },
            WordExample {
                bangla: "স্পর্শ",
                roman: "shoporsho / sporsho",
                meaning: "Touch",
            },
            WordExample {
                bangla: "স্পন্দন",
                roman: "spondon",
                meaning: "Pulse/Beat",
            },
            WordExample {
                bangla: "স্পঞ্জ",
                roman: "sponjo",
                meaning: "Sponge",
            },
            WordExample {
                bangla: "পরস্পর",
                roman: "porospor",
                meaning: "Each other",
            },
        ],
    },
    SignGuideEntry {
        bangla: "স্ব",
        display_bangla: "স্ব",
        series: "Sh-Series",
        example: "স্বাধীন (swadhin), স্বপ্ন (swapno), স্বদেশ (swadesh)",
        primary_key: "sw",
        alt_keys: &["sb", "s+b"],
        decomposition: "স + ব = স্ব (s + wa/ba)",
        name: "স্ব (স+ব যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "স্বাধীন",
                roman: "swadhin / sbadhin",
                meaning: "Independent",
            },
            WordExample {
                bangla: "স্বপ্ন",
                roman: "swapno / sbopno",
                meaning: "Dream",
            },
            WordExample {
                bangla: "স্বদেশ",
                roman: "swadesh",
                meaning: "Homeland",
            },
            WordExample {
                bangla: "স্বাগতম",
                roman: "swagatam",
                meaning: "Welcome",
            },
            WordExample {
                bangla: "স্বভাব",
                roman: "swobhab",
                meaning: "Nature/Habit",
            },
        ],
    },
    SignGuideEntry {
        bangla: "শ্ব",
        display_bangla: "শ্ব",
        series: "Sh-Series",
        example: "বিশ্ব (bishwo), বিশ্বাস (bishwas), ঈশ্বর (ishwor)",
        primary_key: "shw",
        alt_keys: &["shb", "sh+b"],
        decomposition: "শ + ব = শ্ব (sh + wa/ba)",
        name: "শ্ব (শ+ব যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "বিশ্ব",
                roman: "bishwo / biswo",
                meaning: "World",
            },
            WordExample {
                bangla: "বিশ্বাস",
                roman: "bishwas / bissas",
                meaning: "Belief/Trust",
            },
            WordExample {
                bangla: "নিঃশ্বাস",
                roman: "ni:shwas",
                meaning: "Breath",
            },
            WordExample {
                bangla: "ঈশ্বর",
                roman: "Ishwor",
                meaning: "God",
            },
        ],
    },
    SignGuideEntry {
        bangla: "স্ম",
        display_bangla: "স্ম",
        series: "Sh-Series",
        example: "স্মৃতি (smriti / smrriti), স্মরণ (smoron), বিস্ময় (bismoy)",
        primary_key: "sm",
        alt_keys: &["s+m"],
        decomposition: "স + ম = স্ম (s + ma)",
        name: "স্ম (স+ম যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "স্মৃতি",
                roman: "smriti / smrriti",
                meaning: "Memory",
            },
            WordExample {
                bangla: "স্মৃতিসৌধ",
                roman: "smritishoudho",
                meaning: "National Memorial",
            },
            WordExample {
                bangla: "স্মরণ",
                roman: "smoron",
                meaning: "Remembrance",
            },
            WordExample {
                bangla: "বিস্ময়",
                roman: "bismoy",
                meaning: "Wonder/Surprise",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ষ্ণ",
        display_bangla: "ষ্ণ",
        series: "Sh-Series",
        example: "উষ্ণ (ushno), কৃষ্ণ (krishno), তৃষ্ণা (trishna)",
        primary_key: "ShN",
        alt_keys: &["shN", "Sh+N"],
        decomposition: "ষ + ণ = ষ্ণ (ss + nna)",
        name: "ষ্ণ (ষ+ণ যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "উষ্ণ",
                roman: "ushno / uShNo",
                meaning: "Warm",
            },
            WordExample {
                bangla: "কৃষ্ণ",
                roman: "krishno / krriShNo",
                meaning: "Krishna/Black",
            },
            WordExample {
                bangla: "তৃষ্ণা",
                roman: "trishna / trriShNa",
                meaning: "Thirst",
            },
        ],
    },
    SignGuideEntry {
        bangla: "শ্ন",
        display_bangla: "শ্ন",
        series: "Sh-Series",
        example: "প্রশ্ন (proshno)",
        primary_key: "shn",
        alt_keys: &["sh+n"],
        decomposition: "শ + ন = শ্ন (sh + na)",
        name: "শ্ন (শ+ন যুক্তবর্ণ · Sh-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "প্রশ্ন",
                roman: "proshno",
                meaning: "Question",
            },
            WordExample {
                bangla: "প্রশ্নপত্র",
                roman: "proshnopotro",
                meaning: "Question paper",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 7. D-Series & L/H-Series (দ, ধ, ল, হ — Everyday High-Frequency)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "দ্ধ",
        display_bangla: "দ্ধ",
        series: "D-Series",
        example: "যুদ্ধ (zuddho), বুদ্ধি (buddhi), শুদ্ধ (shuddho)",
        primary_key: "ddh",
        alt_keys: &["d+dh"],
        decomposition: "দ + ধ = দ্ধ (d + dha)",
        name: "দ্ধ (দ+ধ যুক্তবর্ণ · D-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "যুদ্ধ",
                roman: "zuddho",
                meaning: "War",
            },
            WordExample {
                bangla: "বুদ্ধি",
                roman: "buddhi",
                meaning: "Intelligence",
            },
            WordExample {
                bangla: "শুদ্ধ",
                roman: "shuddho",
                meaning: "Pure/Correct",
            },
            WordExample {
                bangla: "শ্রদ্ধা",
                roman: "shroddha",
                meaning: "Respect",
            },
            WordExample {
                bangla: "বৃদ্ধ",
                roman: "briddho / brriddho",
                meaning: "Elderly",
            },
        ],
    },
    SignGuideEntry {
        bangla: "দ্ব",
        display_bangla: "দ্ব",
        series: "D-Series",
        example: "দ্বিতীয় (dwitiyo), দ্বার (dwar), দ্বীপ (dwIp)",
        primary_key: "dw",
        alt_keys: &["db", "d+b"],
        decomposition: "দ + ব = দ্ব (d + wa)",
        name: "দ্ব (দ+ব যুক্তবর্ণ · D-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "দ্বিতীয়",
                roman: "dwitiyo / dwitIyo",
                meaning: "Second",
            },
            WordExample {
                bangla: "দ্বার",
                roman: "dwar",
                meaning: "Door/Gate",
            },
            WordExample {
                bangla: "দ্বীপ",
                roman: "dwIp",
                meaning: "Island",
            },
            WordExample {
                bangla: "দ্বন্দ্ব",
                roman: "dwondwo",
                meaning: "Conflict",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ধ্ব",
        display_bangla: "ধ্ব",
        series: "D-Series",
        example: "ধ্বনি (dhwoni), ধ্বংস (dhwongsho)",
        primary_key: "dhw",
        alt_keys: &["dhb", "dh+b"],
        decomposition: "ধ + ব = ধ্ব (dh + wa)",
        name: "ধ্ব (ধ+ব যুক্তবর্ণ · D-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "ধ্বনি",
                roman: "dhwoni / dhboni",
                meaning: "Sound",
            },
            WordExample {
                bangla: "ধ্বংস",
                roman: "dhwongsho",
                meaning: "Destruction",
            },
            WordExample {
                bangla: "প্রতিধ্বনি",
                roman: "protidhwoni",
                meaning: "Echo",
            },
        ],
    },
    SignGuideEntry {
        bangla: "দ্দ",
        display_bangla: "দ্দ",
        series: "D-Series",
        example: "উদ্দেশ্য (uddeshyo), পদ্ধতি (poddhoti)",
        primary_key: "dd",
        alt_keys: &["d+d"],
        decomposition: "দ + দ = দ্দ (d + da)",
        name: "দ্দ (দ+দ যুক্তবর্ণ · D-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "উদ্দেশ্য",
                roman: "uddeshyo",
                meaning: "Purpose/Aim",
            },
            WordExample {
                bangla: "পদ্ধতি",
                roman: "poddhoti",
                meaning: "Method/System",
            },
        ],
    },
    SignGuideEntry {
        bangla: "দ্ম",
        display_bangla: "দ্ম",
        series: "D-Series",
        example: "পদ্ম (podmo), পদ্মা (podma)",
        primary_key: "dm",
        alt_keys: &["d+m"],
        decomposition: "দ + ম = দ্ম (d + ma)",
        name: "দ্ম (দ+ম যুক্তবর্ণ · D-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "পদ্ম",
                roman: "podmo",
                meaning: "Lotus",
            },
            WordExample {
                bangla: "পদ্মা",
                roman: "podma",
                meaning: "Padma River",
            },
            WordExample {
                bangla: "ছদ্মবেশ",
                roman: "chhodmobesh",
                meaning: "Disguise",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ল্প",
        display_bangla: "ল্প",
        series: "L-Series",
        example: "গল্প (golpo), অল্প (olpo), কল্পনা (kolpona), শিল্প (shilpo)",
        primary_key: "lp",
        alt_keys: &["l+p"],
        decomposition: "ল + প = ল্প (l + pa)",
        name: "ল্প (ল+প যুক্তবর্ণ · L-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "গল্প",
                roman: "golpo",
                meaning: "Story",
            },
            WordExample {
                bangla: "অল্প",
                roman: "olpo",
                meaning: "Little/Few",
            },
            WordExample {
                bangla: "কল্পনা",
                roman: "kolpona",
                meaning: "Imagination",
            },
            WordExample {
                bangla: "শিল্প",
                roman: "shilpo",
                meaning: "Art/Industry",
            },
            WordExample {
                bangla: "বিকল্প",
                roman: "bikolpo",
                meaning: "Alternative",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ল্ল",
        display_bangla: "ল্ল",
        series: "L-Series",
        example: "উল্লাস (ullash), পল্লী (pollI), আল্লাহ (allah)",
        primary_key: "ll",
        alt_keys: &["l+l"],
        decomposition: "ল + ল = ল্ল (l + la)",
        name: "ল্ল (ল+ল যুক্তবর্ণ · L-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "উল্লাস",
                roman: "ullash",
                meaning: "Joy/Cheer",
            },
            WordExample {
                bangla: "পল্লী",
                roman: "pollI",
                meaning: "Rural village",
            },
            WordExample {
                bangla: "আল্লাহ",
                roman: "allah",
                meaning: "Allah",
            },
        ],
    },
    SignGuideEntry {
        bangla: "হ্ম",
        display_bangla: "হ্ম",
        series: "L-Series",
        example: "ব্রহ্ম (brohmo), ব্রাহ্মণ (brahmoN)",
        primary_key: "hm",
        alt_keys: &["h+m"],
        decomposition: "হ + ম = হ্ম (h + ma)",
        name: "হ্ম (হ+ম যুক্তবর্ণ · L-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "ব্রহ্ম",
                roman: "brohmo",
                meaning: "Brahma",
            },
            WordExample {
                bangla: "ব্রহ্মপুত্র",
                roman: "brohmoputro",
                meaning: "Brahmaputra River",
            },
            WordExample {
                bangla: "ব্রাহ্মণ",
                roman: "brahmoN",
                meaning: "Brahmin",
            },
        ],
    },
    SignGuideEntry {
        bangla: "হ্ন",
        display_bangla: "হ্ন / হ্ণ",
        series: "L-Series",
        example: "চিহ্ন (chihno), অপরাহ্ণ (oporahNo)",
        primary_key: "hn",
        alt_keys: &["hN", "h+n"],
        decomposition: "হ + ন/ণ = হ্ন/হ্ণ (h + na)",
        name: "হ্ন / হ্ণ (হ+ন/ণ যুক্তবর্ণ · L-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "চিহ্ন",
                roman: "chihno",
                meaning: "Sign/Symbol",
            },
            WordExample {
                bangla: "মধ্যাহ্ন",
                roman: "modhyahno",
                meaning: "Noon",
            },
            WordExample {
                bangla: "অপরাহ্ণ",
                roman: "oporahNo",
                meaning: "Afternoon",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ক্ষ্ম",
        display_bangla: "ক্ষ্ম",
        series: "K-Series",
        example: "লক্ষ্মী (lokkhmi / lokShmI), সূক্ষ্ম (sUkShmo)",
        primary_key: "kkhm",
        alt_keys: &["kShm"],
        decomposition: "ক + ষ + ম = ক্ষ্ম (k + ssa + ma)",
        name: "ক্ষ্ম (ক+ষ+ম যুক্তবর্ণ · K-Series)",
        kind: SignKind::Conjunct,
        words: &[
            WordExample {
                bangla: "লক্ষ্মী",
                roman: "lokShmI",
                meaning: "Lakshmi",
            },
            WordExample {
                bangla: "সূক্ষ্ম",
                roman: "sUkShmo",
                meaning: "Subtle/Fine",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 8. Folas (ফলা — ্র, ্য, র্, ্ব, ্ম, ্ল, ্ন)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "্র",
        display_bangla: "ক্র / প্র",
        series: "Fola",
        example: "প্রথম (prothom), প্রাণ (pran), গ্রাম (gram), প্রেম (prem)",
        primary_key: "r",
        alt_keys: &["+r"],
        decomposition: "ব্যঞ্জন + র = র-ফলা (ক্র / প্র)",
        name: "র-ফলা (Ra-fola)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "প্রথম",
                roman: "prothom",
                meaning: "First",
            },
            WordExample {
                bangla: "প্রাণ",
                roman: "praN",
                meaning: "Life/Soul",
            },
            WordExample {
                bangla: "গ্রাম",
                roman: "gram",
                meaning: "Village",
            },
            WordExample {
                bangla: "প্রেম",
                roman: "prem",
                meaning: "Love",
            },
            WordExample {
                bangla: "প্রিয়",
                roman: "priyo",
                meaning: "Dear/Favorite",
            },
        ],
    },
    SignGuideEntry {
        bangla: "্য",
        display_bangla: "ক্য / ব্য",
        series: "Fola",
        example: "বাক্য (bakyo), সত্য (shotyo), জন্য (jonyo), ব্যবসা (byobsha)",
        primary_key: "y",
        alt_keys: &["Z", "+z"],
        decomposition: "ব্যঞ্জন + য = য-ফলা (ক্য / ব্য)",
        name: "য-ফলা (Ja-fola)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "বাক্য",
                roman: "bakyo",
                meaning: "Sentence",
            },
            WordExample {
                bangla: "সত্য",
                roman: "shotyo",
                meaning: "Truth",
            },
            WordExample {
                bangla: "জন্য",
                roman: "jonyo",
                meaning: "For",
            },
            WordExample {
                bangla: "শূন্য",
                roman: "shUnyo",
                meaning: "Zero/Empty",
            },
            WordExample {
                bangla: "ব্যবসা",
                roman: "byobsha",
                meaning: "Business",
            },
        ],
    },
    SignGuideEntry {
        bangla: "র্",
        display_bangla: "র্ক / র্ম",
        series: "Fola",
        example: "কর্ম (kormo), ধর্ম (dhormo), সূর্য (shurjo), গর্ব (gorbo)",
        primary_key: "r",
        alt_keys: &["rr"],
        decomposition: "র + ব্যঞ্জন = রেফ (র্ক / র্ম)",
        name: "রেফ (Reph)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "কর্ম",
                roman: "kormo",
                meaning: "Work/Deed",
            },
            WordExample {
                bangla: "ধর্ম",
                roman: "dhormo",
                meaning: "Religion",
            },
            WordExample {
                bangla: "সূর্য",
                roman: "shurjo / sUrjo",
                meaning: "Sun",
            },
            WordExample {
                bangla: "গর্ব",
                roman: "gorbo",
                meaning: "Pride",
            },
            WordExample {
                bangla: "বর্ণ",
                roman: "borNo",
                meaning: "Letter/Color",
            },
        ],
    },
    SignGuideEntry {
        bangla: "্ব",
        display_bangla: "স্ব / ক্ব",
        series: "Fola",
        example: "স্বাধীন (swadhin / sbadhin), বিশ্ব (bishwo)",
        primary_key: "w",
        alt_keys: &["b", "+b"],
        decomposition: "ব্যঞ্জন + ব = ব-ফলা (স্ব)",
        name: "ব-ফলা (Ba-fola)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "স্বাধীন",
                roman: "swadhin",
                meaning: "Independent",
            },
            WordExample {
                bangla: "বিশ্ব",
                roman: "bishwo",
                meaning: "World",
            },
            WordExample {
                bangla: "ধ্বনি",
                roman: "dhwoni",
                meaning: "Sound",
            },
        ],
    },
    SignGuideEntry {
        bangla: "্ম",
        display_bangla: "পদ্ম / ক্ম",
        series: "Fola",
        example: "পদ্ম (podmo), স্মৃতি (smriti), আত্মা (atma)",
        primary_key: "m",
        alt_keys: &["+m"],
        decomposition: "ব্যঞ্জন + ম = ম-ফলা (পদ্ম / স্ম)",
        name: "ম-ফলা (Ma-fola)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "পদ্ম",
                roman: "podmo",
                meaning: "Lotus",
            },
            WordExample {
                bangla: "স্মৃতি",
                roman: "smriti / smrriti",
                meaning: "Memory",
            },
            WordExample {
                bangla: "আত্মা",
                roman: "atma",
                meaning: "Soul",
            },
        ],
    },
    SignGuideEntry {
        bangla: "্ল",
        display_bangla: "ক্ল / প্ল",
        series: "Fola",
        example: "ক্লান্ত (klanto), বিপ্লব (biplob)",
        primary_key: "l",
        alt_keys: &["+l"],
        decomposition: "ব্যঞ্জন + ল = ল-ফলা (ক্ল / প্ল)",
        name: "ল-ফলা (La-fola)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "ক্লান্ত",
                roman: "klanto",
                meaning: "Tired",
            },
            WordExample {
                bangla: "বিপ্লব",
                roman: "biplob",
                meaning: "Revolution",
            },
        ],
    },
    SignGuideEntry {
        bangla: "্ন",
        display_bangla: "প্রশ্ন / ক্ন",
        series: "Fola",
        example: "প্রশ্ন (proshno), স্বপ্ন (swapno)",
        primary_key: "n",
        alt_keys: &["+n"],
        decomposition: "ব্যঞ্জন + ন = ন-ফলা (প্রশ্ন)",
        name: "ন-ফলা (Na-fola)",
        kind: SignKind::Fola,
        words: &[
            WordExample {
                bangla: "প্রশ্ন",
                roman: "proshno",
                meaning: "Question",
            },
            WordExample {
                bangla: "স্বপ্ন",
                roman: "swapno",
                meaning: "Dream",
            },
            WordExample {
                bangla: "যত্ন",
                roman: "jotno",
                meaning: "Care",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 9. Vowel Signs (কার — া, ি, ী, ু, ূ, ৃ, ে, ৈ, ো, ৌ)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "ৃ",
        display_bangla: "কৃ / সৃ / স্মৃ",
        series: "Kar",
        example: "সৃষ্টি (srishti / srriShTi), স্মৃতি (smriti / smrriti), কৃষি (krishi)",
        primary_key: "rri",
        alt_keys: &["R"],
        decomposition: "ব্যঞ্জন + ঋ = ঋ-কার (কৃ / সৃ / স্মৃ)",
        name: "ঋ-কার (Rri-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "সৃষ্টি",
                roman: "srishti / srriShTi",
                meaning: "Creation",
            },
            WordExample {
                bangla: "স্মৃতি",
                roman: "smriti / smrriti",
                meaning: "Memory",
            },
            WordExample {
                bangla: "কৃষি",
                roman: "krishi / krriShi",
                meaning: "Agriculture",
            },
            WordExample {
                bangla: "পৃথিবী",
                roman: "prithibi / prrithibI",
                meaning: "Earth",
            },
            WordExample {
                bangla: "হৃদয়",
                roman: "hridoy / hrridoy",
                meaning: "Heart",
            },
        ],
    },
    SignGuideEntry {
        bangla: "া",
        display_bangla: "কা",
        series: "Kar",
        example: "মা (ma), বাবা (baba), ভাষা (bhasha)",
        primary_key: "a",
        alt_keys: &["A"],
        decomposition: "ক + আ = কা (আ-কার)",
        name: "আ-কার (Aa-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "মা",
                roman: "ma",
                meaning: "Mother",
            },
            WordExample {
                bangla: "বাবা",
                roman: "baba",
                meaning: "Father",
            },
            WordExample {
                bangla: "ভাষা",
                roman: "bhasha",
                meaning: "Language",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ি",
        display_bangla: "কি",
        series: "Kar",
        example: "দিন (din), চিঠি (chithi), মিষ্টি (mishti)",
        primary_key: "i",
        alt_keys: &[],
        decomposition: "ক + ই = কি (হ্রস্ব ই-কার)",
        name: "হ্রস্ব ই-কার (Short i-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "দিন",
                roman: "din",
                meaning: "Day",
            },
            WordExample {
                bangla: "চিঠি",
                roman: "chiThi",
                meaning: "Letter",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ী",
        display_bangla: "কী",
        series: "Kar",
        example: "নদী (nodI), জীবন (jibon / jIbon), শহীদ (shohid)",
        primary_key: "I",
        alt_keys: &["ee"],
        decomposition: "ক + ঈ = কী (দীর্ঘ ঈ-কার)",
        name: "দীর্ঘ ঈ-কার (Long ii-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "জীবন",
                roman: "jIbon / jibon",
                meaning: "Life",
            },
            WordExample {
                bangla: "নদী",
                roman: "nodI / nodee",
                meaning: "River",
            },
            WordExample {
                bangla: "শহীদ",
                roman: "shohId / shohid",
                meaning: "Martyr",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ু",
        display_bangla: "কু",
        series: "Kar",
        example: "ফুল (phul), সুখ (shukh), মুখ (mukh)",
        primary_key: "u",
        alt_keys: &[],
        decomposition: "ক + উ = কু (হ্রস্ব উ-কার)",
        name: "হ্রস্ব উ-কার (Short u-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "ফুল",
                roman: "phul / ful",
                meaning: "Flower",
            },
            WordExample {
                bangla: "সুখ",
                roman: "shukh",
                meaning: "Happiness",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ূ",
        display_bangla: "কূ",
        series: "Kar",
        example: "সূর্য (sUrjo / shurjo), রূপ (rUp / roop)",
        primary_key: "U",
        alt_keys: &["oo"],
        decomposition: "ক + ঊ = কূ (দীর্ঘ ঊ-কার)",
        name: "দীর্ঘ ঊ-কার (Long uu-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "সূর্য",
                roman: "sUrjo / shurjo",
                meaning: "Sun",
            },
            WordExample {
                bangla: "রূপ",
                roman: "rUp / roop",
                meaning: "Beauty/Form",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ে",
        display_bangla: "কে",
        series: "Kar",
        example: "দেশ (desh), মেঘ (megh)",
        primary_key: "e",
        alt_keys: &[],
        decomposition: "ক + এ = কে (এ-কার)",
        name: "এ-কার (E-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "দেশ",
                roman: "desh",
                meaning: "Country",
            },
            WordExample {
                bangla: "মেঘ",
                roman: "megh",
                meaning: "Cloud",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ৈ",
        display_bangla: "কৈ",
        series: "Kar",
        example: "বৈশাখ (boishakh), শৈশব (shoishob)",
        primary_key: "OI",
        alt_keys: &["oi"],
        decomposition: "ক + ঐ = কৈ (ঐ-কার)",
        name: "ঐ-কার (Oi-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "বৈশাখ",
                roman: "boishakh",
                meaning: "Boishakh (1st Bangla month)",
            },
            WordExample {
                bangla: "শৈশব",
                roman: "shoishob",
                meaning: "Childhood",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ো",
        display_bangla: "কো",
        series: "Kar",
        example: "ভোর (bhOr), চোখ (chOkh)",
        primary_key: "O",
        alt_keys: &["o"],
        decomposition: "ক + ও = কো (ও-কার)",
        name: "ও-কার (O-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "ভোর",
                roman: "bhOr",
                meaning: "Dawn",
            },
            WordExample {
                bangla: "চোখ",
                roman: "chOkh",
                meaning: "Eye",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ৌ",
        display_bangla: "কৌ",
        series: "Kar",
        example: "নৌকা (nouka), গৌরব (gourobo)",
        primary_key: "OU",
        alt_keys: &["ou"],
        decomposition: "ক + ঔ = কৌ (ঔ-কার)",
        name: "ঔ-কার (Ou-kar)",
        kind: SignKind::Kar,
        words: &[
            WordExample {
                bangla: "নৌকা",
                roman: "nouka",
                meaning: "Boat",
            },
            WordExample {
                bangla: "গৌরব",
                roman: "gourob",
                meaning: "Glory",
            },
        ],
    },
    // ════════════════════════════════════════════════════════════════════════
    // 10. Diacritics & Special Marks (চিহ্ন)
    // ════════════════════════════════════════════════════════════════════════
    SignGuideEntry {
        bangla: "ঁ",
        display_bangla: "চাঁ",
        series: "Mark",
        example: "চাঁদ (cha^d), বাঁশ (ba^sh), হাসিমুখ (ha^shi)",
        primary_key: "^",
        alt_keys: &["~"],
        decomposition: "চন্দ্রবিন্দু (চাঁদ)",
        name: "চন্দ্রবিন্দু (Chandrabindu)",
        kind: SignKind::Mark,
        words: &[
            WordExample {
                bangla: "চাঁদ",
                roman: "cha^d",
                meaning: "Moon",
            },
            WordExample {
                bangla: "বাঁশি",
                roman: "ba^shi",
                meaning: "Flute",
            },
            WordExample {
                bangla: "ফাঁকা",
                roman: "pha^ka",
                meaning: "Empty",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ং",
        display_bangla: "বাং",
        series: "Mark",
        example: "বাংলা (bangla), রং (rong), এবং (ebong)",
        primary_key: "ng",
        alt_keys: &["M"],
        decomposition: "অনুস্বার (বাংলা)",
        name: "অনুস্বার (Anusvara)",
        kind: SignKind::Mark,
        words: &[
            WordExample {
                bangla: "বাংলা",
                roman: "bangla",
                meaning: "Bengali",
            },
            WordExample {
                bangla: "রং",
                roman: "rong",
                meaning: "Color",
            },
            WordExample {
                bangla: "এবং",
                roman: "ebong",
                meaning: "And",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ঃ",
        display_bangla: "দুঃ",
        series: "Mark",
        example: "দুঃখ (du:kho), নিঃশ্বাস (ni:shwas)",
        primary_key: ":",
        alt_keys: &["H"],
        decomposition: "বিসর্গ (দুঃখ)",
        name: "বিসর্গ (Visarga)",
        kind: SignKind::Mark,
        words: &[
            WordExample {
                bangla: "দুঃখ",
                roman: "du:kho",
                meaning: "Sorrow",
            },
            WordExample {
                bangla: "নিঃশ্বাস",
                roman: "ni:shwas",
                meaning: "Breath",
            },
        ],
    },
    SignGuideEntry {
        bangla: "ৎ",
        display_bangla: "ৎ",
        series: "Mark",
        example: "হঠাৎ (hothat``), উৎসব (ut``shob)",
        primary_key: "t``",
        alt_keys: &["t/", "T``"],
        decomposition: "খণ্ড ত (হঠাৎ)",
        name: "খণ্ড ত (Khanda Ta)",
        kind: SignKind::Mark,
        words: &[
            WordExample {
                bangla: "হঠাৎ",
                roman: "hothat``",
                meaning: "Suddenly",
            },
            WordExample {
                bangla: "উৎসব",
                roman: "ut``shob",
                meaning: "Festival",
            },
        ],
    },
    SignGuideEntry {
        bangla: "্‌",
        display_bangla: "ক্‌",
        series: "Mark",
        example: "বাক্‌ (bak,,)",
        primary_key: ",,",
        alt_keys: &["+"],
        decomposition: "হসন্ত / যুক্তবর্ণ যোজক",
        name: "হসন্ত / যুক্ত যোজক (Hasanta / Joiner)",
        kind: SignKind::Mark,
        words: &[WordExample {
            bangla: "ক+ত = ক্ত",
            roman: "k+t",
            meaning: "Explicitly join any 2 letters",
        }],
    },
    SignGuideEntry {
        bangla: "।",
        display_bangla: "।",
        series: "Mark",
        example: "আমি যাই। (ami zai.)",
        primary_key: ".",
        alt_keys: &["|"],
        decomposition: "। (পূর্ণচ্ছেদ দাড়ি)",
        name: "দাড়ি (Dari / Full stop)",
        kind: SignKind::Mark,
        words: &[WordExample {
            bangla: "আমি যাই।",
            roman: "ami zai.",
            meaning: "Full stop at sentence end",
        }],
    },
    SignGuideEntry {
        bangla: "৳",
        display_bangla: "৳",
        series: "Mark",
        example: "১০০৳ (100$)",
        primary_key: "$",
        alt_keys: &[],
        decomposition: "৳ (বাংলাদেশী টাকা চিহ্ন)",
        name: "টাকা চিহ্ন (Taka sign)",
        kind: SignKind::Mark,
        words: &[WordExample {
            bangla: "১০০৳",
            roman: "100$",
            meaning: "100 Bangladeshi Taka",
        }],
    },
];

/// Decompose any Bangla word or cluster into human-readable base characters + signs,
/// e.g. `"ক্ষ"` -> `"ক + ্ + ষ"`, `"কী"` -> `"ক + ী"`.
pub fn decompose_bangla(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    chars
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Given the current Roman input and transliterated Bangla preedit, returns contextual
/// "Next-Key Hints" showing the user how to attach vowel signs (কার), ফলা, or যুক্তবর্ণ
/// to the last consonant they typed.
pub fn contextual_sign_hints(roman: &str, bangla_preedit: &str) -> Vec<SignHint> {
    let Some(last_char) = bangla_preedit.chars().last() else {
        return Vec::new();
    };
    if !is_bangla_consonant(last_char) {
        return Vec::new();
    }

    let base = last_char.to_string();
    let mut hints = Vec::new();

    // Add consonant-specific common connected letters (যুক্তবর্ণ) FIRST so they are immediately visible!
    let specific: &[(&str, &str, &str)] = match last_char {
        'ক' => &[
            ("ক্ষ", "kkh", "ক+ষ"),
            ("ক্ত", "kt", "ক+ত"),
            ("ক্ল", "kl", "ক+ল"),
            ("ক্স", "ks", "ক+স"),
        ],
        'গ' => &[
            ("গ্ধ", "gdh", "গ+ধ"),
            ("গ্ল", "gl", "গ+ল"),
            ("গ্ন", "gn", "গ+ন"),
        ],
        'ঙ' => &[("ঙ্ক", "nk", "ঙ+ক"), ("ঙ্গ", "ngg", "ঙ+গ")],
        'চ' => &[("চ্চ", "cc", "চ+চ"), ("চ্ছ", "cch", "চ+ছ")],
        'জ' => &[
            ("জ্ঞ", "gg", "জ+ঞ"),
            ("জ্জ", "jj", "জ+জ"),
            ("জ্ব", "jw", "জ+ব"),
        ],
        'ঞ' => &[("ঞ্চ", "nch", "ঞ+চ"), ("ঞ্জ", "nj", "ঞ+জ")],
        'ট' => &[("ট্ট", "TT", "ট+ট")],
        'ণ' => &[
            ("ণ্ড", "ND", "ণ+ড"),
            ("ণ্ট", "NT", "ণ+ট"),
            ("ণ্ঠ", "NTh", "ণ+ঠ"),
        ],
        'ত' => &[
            ("ত্ব", "tw", "ত+ব"),
            ("ত্ত", "tt", "ত+ত"),
            ("ত্র", "tr", "ত+র"),
            ("ত্ন", "tn", "ত+ন"),
            ("ত্ম", "tm", "ত+ম"),
        ],
        'দ' => &[
            ("দ্ব", "dw", "দ+ব"),
            ("দ্ধ", "ddh", "দ+ধ"),
            ("দ্দ", "dd", "দ+দ"),
            ("দ্ম", "dm", "দ+ম"),
        ],
        'ধ' => &[
            ("ধ্ব", "dhw", "ধ+ব"),
            ("ধ্র", "dhr", "ধ+র"),
            ("ধ্ন", "dhn", "ধ+ন"),
        ],
        'ন' => &[
            ("ন্দ", "nd", "ন+দ"),
            ("ন্ধ", "ndh", "ন+ধ"),
            ("ন্ত", "nt", "ন+ত"),
            ("ন্ত্র", "ntr", "ন+ত+র"),
            ("ন্ন", "nn", "ন+ন"),
        ],
        'প' => &[
            ("প্ত", "pt", "প+ত"),
            ("প্প", "pp", "প+প"),
            ("প্ল", "pl", "প+ল"),
        ],
        'ব' => &[
            ("ব্ধ", "bdh", "ব+ধ"),
            ("ব্দ", "bd", "ব+দ"),
            ("ব্ব", "bb", "ব+ব"),
        ],
        'ম' => &[
            ("ম্প", "mp", "ম+প"),
            ("ম্ব", "mb", "ম+ব"),
            ("ম্ভ", "mbh", "ম+ভ"),
            ("ম্ম", "mm", "ম+ম"),
        ],
        'ল' => &[
            ("ল্ল", "ll", "ল+ল"),
            ("ল্প", "lp", "ল+প"),
            ("ল্ট", "lT", "ল+ট"),
        ],
        'শ' => &[
            ("শ্ব", "shw", "শ+ব"),
            ("শ্চ", "shch", "শ+চ"),
            ("শ্ন", "shn", "শ+ন"),
            ("শ্ল", "shl", "শ+ল"),
        ],
        'ষ' => &[
            ("ষ্ট", "ShT", "ষ+ট"),
            ("ষ্ঠ", "ShTh", "ষ+ঠ"),
            ("ষ্ণ", "ShN", "ষ+ণ"),
            ("ষ্প", "Shp", "ষ+প"),
        ],
        'স' => &[
            ("স্ব", "sw", "স+ব"),
            ("স্ম", "sm", "স+ম"),
            ("স্ত", "st", "স+ত"),
            ("স্থ", "sth", "স+থ"),
            ("স্প", "sp", "স+প"),
            ("স্ক", "sk", "স+ক"),
        ],
        'হ' => &[
            ("হ্ম", "hm", "হ+ম"),
            ("হ্ণ", "hN", "হ+ণ"),
            ("হ্ন", "hn", "হ+ন"),
        ],
        _ => &[],
    };

    for (cluster, shortcut, breakdown) in specific {
        hints.push(SignHint {
            result_bangla: (*cluster).to_string(),
            key_hint: (*shortcut).to_string(),
            label: (*breakdown).to_string(),
            kind: SignKind::Conjunct,
        });
    }

    // Folas & Vowel Signs (without bare combining marks in `label`)
    hints.extend([
        SignHint {
            result_bangla: format!("{base}্ব"),
            key_hint: format!("{roman}w"),
            label: "ব-ফলা".to_string(),
            kind: SignKind::Fola,
        },
        SignHint {
            result_bangla: format!("{base}্র"),
            key_hint: format!("{roman}r"),
            label: "র-ফলা".to_string(),
            kind: SignKind::Fola,
        },
        SignHint {
            result_bangla: format!("{base}্য"),
            key_hint: format!("{roman}y"),
            label: "য-ফলা".to_string(),
            kind: SignKind::Fola,
        },
        SignHint {
            result_bangla: format!("{base}া"),
            key_hint: format!("{roman}a"),
            label: "আ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ি"),
            key_hint: format!("{roman}i"),
            label: "ই-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ী"),
            key_hint: format!("{roman}I"),
            label: "ঈ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ু"),
            key_hint: format!("{roman}u"),
            label: "উ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ূ"),
            key_hint: format!("{roman}U"),
            label: "ঊ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ৃ"),
            key_hint: format!("{roman}rri"),
            label: "ঋ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ে"),
            key_hint: format!("{roman}e"),
            label: "এ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ৈ"),
            key_hint: format!("{roman}OI"),
            label: "ঐ-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ো"),
            key_hint: format!("{roman}O"),
            label: "ও-কার".to_string(),
            kind: SignKind::Kar,
        },
        SignHint {
            result_bangla: format!("{base}ৌ"),
            key_hint: format!("{roman}OU"),
            label: "ঔ-কার".to_string(),
            kind: SignKind::Kar,
        },
    ]);

    hints
}

/// Search the sign & conjunct guide by Bangla substring, Roman shortcut, series, or word example.
/// Exact `bangla` or `primary_key` matches are ranked first.
pub fn search_signs_and_conjuncts(query: &str) -> Vec<&'static SignGuideEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return SIGN_AND_CONJUNCT_GUIDE.iter().collect();
    }
    let mut results: Vec<&'static SignGuideEntry> = SIGN_AND_CONJUNCT_GUIDE
        .iter()
        .filter(|e| {
            e.bangla.contains(&q)
                || e.display_bangla.contains(&q)
                || e.series.to_lowercase().contains(&q)
                || e.primary_key.to_lowercase().contains(&q)
                || e.alt_keys.iter().any(|k| k.to_lowercase().contains(&q))
                || e.name.to_lowercase().contains(&q)
                || e.example.to_lowercase().contains(&q)
                || e.decomposition.to_lowercase().contains(&q)
                || e.words.iter().any(|w| {
                    w.bangla.contains(&q)
                        || w.roman.to_lowercase().contains(&q)
                        || w.meaning.to_lowercase().contains(&q)
                })
        })
        .collect();
    results.sort_by_key(|e| {
        if e.bangla == q || e.primary_key.eq_ignore_ascii_case(&q) {
            0
        } else if e.alt_keys.iter().any(|k| k.eq_ignore_ascii_case(&q)) {
            1
        } else if e.bangla.contains(&q) || e.display_bangla.contains(&q) {
            2
        } else {
            3
        }
    });
    results
}

/// Returns true if `c` is a Bangla base consonant (ক..হ, ড়, ঢ়, য়).
#[inline]
pub fn is_bangla_consonant(c: char) -> bool {
    matches!(c, 'ক'..='হ' | '\u{09DC}' | '\u{09DD}' | '\u{09DF}')
}

/// Returns true if `c` is a Bangla dependent vowel sign (কার).
#[inline]
pub fn is_bangla_kar(c: char) -> bool {
    matches!(
        c,
        'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ' | 'ৗ'
    )
}
