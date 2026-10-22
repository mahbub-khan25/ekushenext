//! Deliberate spelling marks.
//!
//! When the user types one of Ekushe's explicit marks, they are telling us exactly what they want,
//! so the candidate that honours it must come first (that is the one Space commits):
//!
//! | Typed        | Asks for            | Example                    |
//! |--------------|---------------------|----------------------------|
//! | `,,`         | হসন্ত (exact output)  | `allah,,` → আল্লাহ্         |
//! | `+`          | যুক্তবর্ণ (exact)       | `k+t` → ক্ত                |
//! | `` ` ``      | no যুক্তবর্ণ (exact)    | `bol`le` → বললে            |
//! | `^` or `~`   | চন্দ্রবিন্দু ঁ            | `he^se` → হেঁসে             |
//! | `rri`        | ঋ / ঋ-কার ৃ           | `krrishi` → কৃষি           |
//! | `rr` + cons. | রেফ র্                | `dhorrmo` → ধর্ম           |
//! | `t```, `t/`  | খণ্ড-ত ৎ               | `hothat``` → হঠাৎ          |
//! | `:`          | বিসর্গ ঃ               | `du:kho` → দুঃখ            |
//! | `Z`          | য-ফলা ্য              | `bakZo` → বাক্য            |
//!
//! The reverse is also useful: when a word in the list carries a mark the user did not type
//! (they typed `hese` and want হেঁসে, option 3), [`mark_tips`] tells them how to type it directly.

use crate::conjuncts::{SignHint, SignKind};
use crate::phonetic::PhoneticEngine;
use crate::suggest::{Candidate, CandidateKind};

/// What the explicit marks in `roman` ask for: whether the exact phonetic output must lead, and
/// the signs a candidate must contain (each entry: any one of the alternatives).
fn requests(roman: &str) -> (bool, Vec<&'static [&'static str]>) {
    // Snippets (`;x`), English escapes (`/x`) and emoji codes (`:x:`) are not spellings
    if roman.is_empty() || roman.starts_with([';', '/', ':']) {
        return (false, Vec::new());
    }
    let no_khanda = roman.replace("t``", "").replace("T``", "");
    let exact = roman.contains(",,") || roman.contains('+') || no_khanda.contains('`');
    let mut need: Vec<&'static [&'static str]> = Vec::new();
    if roman.contains('^') || roman.contains('~') {
        need.push(&["ঁ"]);
    }
    if roman.contains("rri") {
        need.push(&["ঋ", "ৃ"]);
    }
    let b = roman.as_bytes();
    let reph = (0..b.len().saturating_sub(2)).any(|i| {
        b[i] == b'r'
            && b[i + 1] == b'r'
            && b[i + 2].is_ascii_alphabetic()
            && !b"aeiouAEIOUr".contains(&b[i + 2])
    });
    if reph {
        need.push(&["র্"]);
    }
    if roman.contains("t``")
        || roman.contains("T``")
        || roman.contains("t/")
        || roman.contains("T/")
    {
        need.push(&["ৎ"]);
    }
    if roman.replace(":`", "").contains(':') {
        need.push(&["ঃ"]);
    }
    if roman.contains('Z') {
        need.push(&["্য"]);
    }
    (exact, need)
}

/// Reorder `cands` so the spelling the marks in `roman` ask for comes first. `literal` is the exact
/// phonetic output of `roman` (it always honours the marks).
pub fn honor_marks(roman: &str, literal: &str, cands: &mut Vec<Candidate>) {
    let (exact, need) = requests(roman);
    if !exact && need.is_empty() {
        return;
    }
    let lit = || Candidate {
        text: literal.to_string(),
        kind: CandidateKind::Phonetic,
        annotation: None,
    };
    if exact {
        if let Some(p) = cands.iter().position(|c| c.text == literal) {
            let c = cands.remove(p);
            cands.insert(0, c);
        } else if !literal.is_empty() {
            cands.insert(0, lit());
        }
        return;
    }
    let ok = |t: &str| need.iter().all(|alts| alts.iter().any(|a| t.contains(a)));
    let (mut yes, no): (Vec<_>, Vec<_>) = cands.drain(..).partition(|c| ok(&c.text));
    if yes.is_empty() && !literal.is_empty() {
        yes.push(lit());
    }
    yes.extend(no);
    *cands = yes;
}

/// Tips for words in the list that carry a mark the user did not type: how to type each one
/// directly (`hese` → হেঁসে: `he^se`). Shown first in the popup's hint row. `first_for` returns
/// what typing a Roman text would put first; a tip is only offered when it really works.
pub fn mark_tips(
    roman: &str,
    cands: &[Candidate],
    phonetic: &PhoneticEngine,
    first_for: &dyn Fn(&str) -> Option<String>,
) -> Vec<SignHint> {
    let (exact, need) = requests(roman);
    if exact
        || !need.is_empty()
        || roman.len() < 2
        || !roman.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Vec::new();
    }
    let Some(first) = cands.first() else {
        return Vec::new();
    };
    let mut tips: Vec<SignHint> = Vec::new();
    let chars: Vec<char> = roman.chars().collect();
    // (sign, label, ways to put the mark into the Roman text)
    let marks: [(&str, &str, Spellings); 3] = [
        ("ঁ", "চন্দ্রবিন্দু", |w| {
            (1..=w.len()).map(|i| splice(w, i, "^")).collect()
        }),
        ("ৎ", "খণ্ড-ত", |w| {
            (0..w.len())
                .filter(|&i| w[i] == 't' || w[i] == 'T')
                .map(|i| splice(w, i + 1, "``"))
                .collect()
        }),
        ("ঃ", "বিসর্গ", |w| {
            (1..=w.len()).map(|i| splice(w, i, ":")).collect()
        }),
    ];
    for cand in cands.iter().skip(1).take(6) {
        if cand.kind == CandidateKind::Emoji || tips.iter().any(|t| t.result_bangla == cand.text) {
            continue;
        }
        for (sign, label, ways) in &marks {
            if !cand.text.contains(sign) || first.text.contains(sign) {
                continue;
            }
            // The mark goes where its exact output matches the word, allowing for letters that sound
            // alike (ba^s gives বাঁস; with the dictionary that is বাঁশ)
            let want = sound_key(&cand.text);
            let found = ways(&chars)
                .into_iter()
                .filter(|v| sound_key(&phonetic.transliterate_word(v)) == want)
                .take(2)
                .find(|v| first_for(v).as_deref() == Some(cand.text.as_str()));
            if let Some(v) = found {
                tips.push(SignHint {
                    result_bangla: cand.text.clone(),
                    key_hint: v,
                    label: (*label).to_string(),
                    kind: SignKind::Mark,
                });
                break;
            }
        }
    }
    tips
}

/// Fold letters that sound the same in Bangla typing (শ ষ স, ট ত, ড দ, ণ ন, য জ, long and short
/// vowels), so a tip can match a dictionary spelling the plain phonetic output only approximates.
fn sound_key(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\u{200C}')
        .map(|c| match c {
            'শ' | 'ষ' => 'স',
            'ট' => 'ত',
            'ঠ' => 'থ',
            'ড' => 'দ',
            'ঢ' => 'ধ',
            'ণ' => 'ন',
            'য' => 'জ',
            'ী' => 'ি',
            'ূ' => 'ু',
            'ঈ' => 'ই',
            'ঊ' => 'উ',
            '\u{09DC}' | '\u{09DD}' => 'র', // ড় ঢ় (precomposed)
            c => c,
        })
        .collect()
}

/// Ways to put a mark into a Roman word (each a full Roman spelling).
type Spellings = fn(&[char]) -> Vec<String>;

fn splice(w: &[char], at: usize, mark: &str) -> String {
    let mut s: String = w[..at].iter().collect();
    s.push_str(mark);
    s.extend(&w[at..]);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(t: &str) -> Candidate {
        Candidate {
            text: t.into(),
            kind: CandidateKind::Dictionary,
            annotation: None,
        }
    }

    #[test]
    fn a_requested_sign_moves_its_word_first() {
        let mut v = vec![c("করতে"), c("কীর্তি"), c("কৃতি")];
        honor_marks("kirrtti", "কির্ত্তি", &mut v);
        assert_eq!(v[0].text, "কীর্তি");
        let mut v = vec![c("হেসে"), c("হাসে"), c("হেঁসে")];
        honor_marks("he^se", "হেঁসে", &mut v);
        assert_eq!(v[0].text, "হেঁসে");
    }

    #[test]
    fn exact_marks_put_the_literal_first() {
        let mut v = vec![c("আল্লাহ"), c("আল্লাহর")];
        honor_marks("allah,,", "আল্লাহ্\u{200C}", &mut v);
        assert_eq!(v[0].text, "আল্লাহ্\u{200C}");
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn plain_words_and_codes_are_untouched() {
        let mut v = vec![c("আমার"), c("আমরা")];
        honor_marks("amar", "আমার", &mut v);
        honor_marks(":smile:", "", &mut v);
        assert_eq!(v[0].text, "আমার");
    }
}
