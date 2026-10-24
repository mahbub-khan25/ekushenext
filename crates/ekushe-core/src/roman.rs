//! Ekushe Next Bangla → Roman generator and the forgiving lookup key ("skeleton").
//!
//! Clean-room design (see `tools/dict-builder/ROMAN.md`). Every Bangla word gets several weighted
//! romanizations covering precise phonetic typing (`T` ট, `D` ড, `N` ণ, `Sh` ষ, `R` ড়, `O` ও, `I` ঈ, `U` ঊ)
//! and casual Banglish (`amar`, `valobashi`, `ase`, `kmn`), each with a small cost. [`skeleton`] maps any
//! Roman string to a vowel-free, aspiration-free, case-free key so different spellings meet in one bucket.
//!
//! This module is the single source of truth: the dictionary builder uses it to build the index and the
//! engine uses it at typing time. `tools/dict-builder/scripts/romanize.py` is the research prototype.

const HC: char = '\u{09CD}';
const NUKTA: char = '\u{09BC}';
const ZWJ: char = '\u{200D}';
const RRA: char = '\u{09DC}';
const RHA: char = '\u{09DD}';
const YYA: char = '\u{09DF}';
/// Cost of pronouncing a word-final inherent vowel (bol → bolo). Tuned on the pilot set.
pub const FINAL_O: f64 = 0.3;

#[derive(Clone, Copy, PartialEq)]
enum Opt {
    S(&'static str, f64),
    /// Repeat the previous consonant (bakko for বাক্য)
    Double(f64),
}
use Opt::{Double, S};

fn cons(c: char) -> Option<&'static [Opt]> {
    Some(match c {
        'ক' => &[S("k", 0.0)],
        'খ' => &[S("kh", 0.0), S("k", 0.6)],
        'গ' => &[S("g", 0.0)],
        'ঘ' => &[S("gh", 0.0), S("g", 0.6)],
        'ঙ' => &[S("ng", 0.0), S("Ng", 0.2)],
        'চ' => &[S("c", 0.0), S("ch", 0.0)],
        'ছ' => &[S("ch", 0.0), S("chh", 0.2), S("s", 0.3)],
        'জ' => &[S("j", 0.0), S("z", 0.5)],
        'ঝ' => &[S("jh", 0.0), S("j", 0.6)],
        'ঞ' => &[S("NG", 0.0), S("n", 0.3)],
        'ট' => &[S("T", 0.0), S("t", 0.1)],
        'ঠ' => &[S("Th", 0.0), S("th", 0.1), S("t", 0.6)],
        'ড' => &[S("D", 0.0), S("d", 0.1)],
        'ঢ' => &[S("Dh", 0.0), S("dh", 0.1), S("d", 0.6)],
        'ণ' => &[S("N", 0.0), S("n", 0.1)],
        'ত' => &[S("t", 0.0)],
        'থ' => &[S("th", 0.0), S("t", 0.6)],
        'দ' => &[S("d", 0.0)],
        'ধ' => &[S("dh", 0.0), S("d", 0.6)],
        'ন' => &[S("n", 0.0)],
        'প' => &[S("p", 0.0)],
        'ফ' => &[S("f", 0.0), S("ph", 0.2)],
        'ব' => &[S("b", 0.0), S("v", 0.7)],
        'ভ' => &[S("bh", 0.0), S("v", 0.1)],
        'ম' => &[S("m", 0.0)],
        'য' => &[S("z", 0.0), S("j", 0.1), S("y", 0.6)],
        'র' => &[S("r", 0.0)],
        'ল' => &[S("l", 0.0)],
        'শ' => &[S("sh", 0.0), S("s", 0.2)],
        'ষ' => &[S("Sh", 0.0), S("sh", 0.1), S("s", 0.3)],
        'স' => &[S("s", 0.0), S("sh", 0.3)],
        'হ' => &[S("h", 0.0)],
        RRA => &[S("R", 0.0), S("r", 0.1), S("rh", 0.5)],
        RHA => &[S("Rh", 0.0), S("rh", 0.1), S("r", 0.4)],
        YYA => &[S("y", 0.0), S("", 0.4)],
        'ৎ' => &[S("t", 0.0), S("t``", 0.3)],
        _ => return None,
    })
}

fn sign(c: char) -> Option<&'static [Opt]> {
    Some(match c {
        'া' => &[S("a", 0.0), S("aa", 0.6)],
        'ি' => &[S("i", 0.0), S("e", 0.8)],
        'ী' => &[S("I", 0.0), S("i", 0.1), S("ee", 0.3)],
        'ু' => &[S("u", 0.0), S("o", 0.8)],
        'ূ' => &[S("U", 0.0), S("u", 0.1), S("oo", 0.4)],
        'ৃ' => &[S("rri", 0.0), S("ri", 0.2)],
        'ে' => &[S("e", 0.0), S("a", 0.7), S("ae", 0.9)],
        'ৈ' => &[S("OI", 0.0), S("oi", 0.1)],
        'ো' => &[S("O", 0.0), S("o", 0.1), S("oo", 0.7)],
        'ৌ' => &[S("OU", 0.0), S("ou", 0.1), S("ow", 0.5)],
        _ => return None,
    })
}

fn indep(c: char) -> Option<&'static [Opt]> {
    Some(match c {
        'অ' => &[S("o", 0.0), S("a", 0.5)],
        'আ' => &[S("a", 0.0), S("aa", 0.6)],
        'ই' => &[S("i", 0.0)],
        'ঈ' => &[S("I", 0.0), S("i", 0.1), S("ee", 0.3)],
        'উ' => &[S("u", 0.0)],
        'ঊ' => &[S("U", 0.0), S("u", 0.1)],
        'ঋ' => &[S("rri", 0.0), S("ri", 0.2)],
        'এ' => &[S("e", 0.0), S("a", 0.7)],
        'ঐ' => &[S("OI", 0.0), S("oi", 0.1)],
        'ও' => &[S("O", 0.0), S("o", 0.1)],
        'ঔ' => &[S("OU", 0.0), S("ou", 0.1)],
        _ => return None,
    })
}

fn modifier(c: char) -> Option<&'static [Opt]> {
    Some(match c {
        'ং' => &[S("ng", 0.0), S("n", 0.4)],
        'ঃ' => &[S(":", 0.0), S("", 0.2)],
        'ঁ' => &[S("^", 0.0), S("", 0.1)],
        _ => return None,
    })
}

fn special_cluster(cl: &[char]) -> Option<&'static [Opt]> {
    Some(match cl {
        ['ক', 'ষ'] => &[S("kkh", 0.0), S("kh", 0.2), S("kSh", 0.1), S("x", 0.6)],
        ['জ', 'ঞ'] => &[S("gg", 0.0), S("gy", 0.3), S("g", 0.4), S("jNG", 0.3)],
        ['ঞ', 'চ'] => &[S("nc", 0.0), S("nch", 0.1)],
        ['ঞ', 'জ'] => &[S("nj", 0.0)],
        ['ঙ', 'ক'] => &[S("nk", 0.0), S("ngk", 0.1)],
        ['ঙ', 'গ'] => &[S("ngg", 0.0), S("ng", 0.1)],
        ['হ', 'ম'] => &[S("hm", 0.0), S("mm", 0.5)],
        ['চ', 'ছ'] => &[S("cch", 0.0), S("chh", 0.1), S("ch", 0.2), S("ss", 0.6)],
        ['চ', 'চ'] => &[S("cc", 0.0), S("cch", 0.1), S("ch", 0.2)],
        _ => return None,
    })
}

enum Unit {
    /// Independent vowel (or nothing, for a stray mark) + marks
    V(Option<char>, Vec<char>),
    /// Consonant cluster + vowel sign + marks
    C(Vec<char>, Option<char>, Vec<char>),
}

fn is_cons(c: char) -> bool {
    cons(c).is_some()
}

fn parse(word: &str) -> Vec<Unit> {
    // Internally use precomposed ড় ঢ় য় so each is one consonant.
    let mut w: Vec<char> = Vec::with_capacity(word.len());
    for c in word.chars() {
        if c == ZWJ {
            continue;
        }
        if c == NUKTA {
            match w.last().copied() {
                Some('ড') => {
                    w.pop();
                    w.push(RRA);
                    continue;
                }
                Some('ঢ') => {
                    w.pop();
                    w.push(RHA);
                    continue;
                }
                Some('য') => {
                    w.pop();
                    w.push(YYA);
                    continue;
                }
                _ => {}
            }
        }
        w.push(c);
    }
    let n = w.len();
    let mut units = Vec::new();
    let mut i = 0;
    while i < n {
        let c = w[i];
        if indep(c).is_some() {
            i += 1;
            let mut mods = Vec::new();
            while i < n && modifier(w[i]).is_some() {
                mods.push(w[i]);
                i += 1;
            }
            units.push(Unit::V(Some(c), mods));
            continue;
        }
        if is_cons(c) {
            let mut cl = vec![c];
            i += 1;
            while i + 1 < n && w[i] == HC && is_cons(w[i + 1]) {
                cl.push(w[i + 1]);
                i += 2;
            }
            let mut sg = None;
            if i < n && sign(w[i]).is_some() {
                sg = Some(w[i]);
                i += 1;
            }
            if i < n && w[i] == HC {
                i += 1;
            }
            let mut mods = Vec::new();
            while i < n && modifier(w[i]).is_some() {
                mods.push(w[i]);
                i += 1;
            }
            units.push(Unit::C(cl, sg, mods));
            continue;
        }
        if modifier(c).is_some() {
            units.push(Unit::V(None, vec![c]));
            i += 1;
            continue;
        }
        i += 1;
    }
    units
}

fn last_consonant(s: &str) -> String {
    // the final letter of the string if it is an ASCII consonant (y counts), lower-cased
    match s.as_bytes().last() {
        Some(&x)
            if x.is_ascii_alphabetic()
                && !matches!(x.to_ascii_lowercase(), b'a' | b'e' | b'i' | b'o' | b'u') =>
        {
            ((x as char).to_ascii_lowercase()).to_string()
        }
        _ => String::new(),
    }
}

/// Option lists for a consonant cluster (+ its vowel sign). Returns whether the sign was consumed.
fn cluster_options(cl: &[char], sg: Option<char>) -> (Vec<Vec<Opt>>, bool) {
    if let Some(sp) = special_cluster(cl) {
        return (vec![sp.to_vec()], false);
    }
    let mut lists: Vec<Vec<Opt>> = Vec::new();
    let mut consumed = false;
    for (j, &c) in cl.iter().enumerate() {
        if j == 0 {
            if c == 'র' && cl.len() > 1 {
                lists.push(vec![S("r", 0.0), S("rr", 0.3)]); // reph
            } else {
                lists.push(cons(c).unwrap().to_vec());
            }
            continue;
        }
        let prev = cl[j - 1];
        if j == 1 && cl[0] == 'র' {
            lists.push(cons(c).unwrap().to_vec()); // after reph: plain consonant
            continue;
        }
        if c == 'য' {
            if sg == Some('া') && j == cl.len() - 1 {
                lists.push(vec![S("ya", 0.0), S("a", 0.3), S("e", 0.5), S("ae", 0.6)]);
                consumed = true;
            } else {
                lists.push(vec![S("y", 0.0), Double(0.1), S("", 0.3)]); // য-ফলা
            }
        } else if c == 'ব' && prev != 'ম' {
            lists.push(vec![S("w", 0.0), S("b", 0.3), S("", 0.4), Double(0.4)]);
        // ব-ফলা
        } else if c == 'ম' && j == cl.len() - 1 && prev != 'ন' {
            lists.push(vec![S("m", 0.0), S("", 0.4), Double(0.5)]); // ম-ফলা
        } else if c == 'র' {
            lists.push(vec![S("r", 0.0)]);
        } else {
            lists.push(cons(c).unwrap().to_vec());
        }
    }
    (lists, consumed)
}

fn beam(lists: &[Vec<Opt>], k: usize) -> Vec<(f64, String)> {
    let mut beams: Vec<(f64, String)> = vec![(0.0, String::new())];
    for opts in lists {
        let mut nb: Vec<(f64, String)> = Vec::with_capacity(beams.len() * opts.len());
        for (cost, s) in &beams {
            for o in opts {
                let (r, c) = match *o {
                    S(r, c) => (r.to_string(), c),
                    Double(c) => (last_consonant(s), c),
                };
                nb.push((cost + c, format!("{s}{r}")));
            }
        }
        nb.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then_with(|| a.1.cmp(&b.1)));
        nb.dedup();
        nb.truncate(k);
        beams = nb;
    }
    beams
}

/// Up to `k` `(cost, roman)` pairs, cheapest first.
pub fn romanize(word: &str, k: usize) -> Vec<(f32, String)> {
    let units = parse(word);
    let mut lists: Vec<Vec<Opt>> = Vec::new();
    let nu = units.len();
    for (idx, u) in units.iter().enumerate() {
        let fin = idx == nu - 1;
        match u {
            Unit::V(v, mods) => {
                if let Some(v) = v {
                    lists.push(indep(*v).unwrap().to_vec());
                }
                for &m in mods {
                    lists.push(modifier(m).unwrap().to_vec());
                }
            }
            Unit::C(cl, sg, mods) => {
                let (cl_lists, consumed) = cluster_options(cl, *sg);
                lists.extend(cl_lists);
                if consumed {
                } else if let Some(s) = sg {
                    lists.push(sign(*s).unwrap().to_vec());
                } else if cl.as_slice() == ['ৎ'] {
                } else if cl.as_slice() == [YYA] && fin {
                    if let Unit::C(_, Some(ps), _) = &units[idx - 1.min(idx)] {
                        if idx > 0 && (*ps == 'ী' || *ps == 'ি') {
                            lists.push(vec![S("o", 0.0), S("", 0.2)]);
                        }
                    }
                } else if fin && cl.len() > 1 {
                    lists.push(vec![S("o", 0.0), S("", 0.3)]);
                } else if !mods.is_empty() {
                    lists.push(vec![S("o", 0.0), S("", 0.4)]);
                } else if fin {
                    lists.push(vec![S("", 0.0), S("o", FINAL_O)]);
                } else {
                    let next_is_c = matches!(units[idx + 1], Unit::C(..));
                    lists.push(if next_is_c {
                        vec![S("o", 0.0), S("", 0.3)]
                    } else {
                        vec![S("", 0.0), S("o", 0.3)]
                    });
                }
                for &m in mods {
                    lists.push(modifier(m).unwrap().to_vec());
                }
            }
        }
    }
    let mut out = beam(&lists, k * 3);
    // দুঃখ is typed "dukkho": visarga before a consonant doubles it
    let mut extra = Vec::new();
    for (cost, r) in &out {
        let b = r.as_bytes();
        if let Some(p) = (0..b.len().saturating_sub(1))
            .find(|&p| b[p] == b':' && b"bcdfghjklmnpqrstvwxz".contains(&b[p + 1]))
        {
            let c = b[p + 1] as char;
            extra.push((cost + 0.1, format!("{}{}{}", &r[..p], c, &r[p + 1..])));
        }
    }
    out.extend(extra);
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then_with(|| a.1.cmp(&b.1)));
    let mut res: Vec<(f32, String)> = Vec::new();
    for (cost, r) in out {
        if !r.is_empty() && !res.iter().any(|(_, x)| *x == r) {
            res.push((((cost * 100.0).round() / 100.0) as f32, r));
        }
        if res.len() >= k {
            break;
        }
    }
    res
}

/// Forgiving key: case-free, aspiration-free, vowel-free (a leading vowel leaves `@`), no doubled letters.
pub fn skeleton(roman: &str) -> String {
    let mut s = roman.to_lowercase();
    for (a, b) in [
        ("chh", "c"),
        ("ch", "c"),
        ("sh", "s"),
        ("ph", "f"),
        ("kkh", "k"),
        ("``", ""),
        ("ng", "n"),
    ] {
        s = s.replace(a, b);
    }
    // drop aspiration h after a consonant (judged on the unmodified string, like a regex look-behind)
    let chars: Vec<char> = s.chars().collect();
    let mut t = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        if c == 'h' && i > 0 && "bcdfgjklmnpqrstvwxz".contains(chars[i - 1]) {
            continue;
        }
        t.push(c);
    }
    let t = t
        .replace('v', "b")
        .replace('z', "j")
        .replace('q', "k")
        .replace('x', "ks")
        .replace('w', "b");
    let lead = matches!(t.chars().next(), Some('a' | 'e' | 'i' | 'o' | 'u' | 'y'));
    let mut out = String::with_capacity(t.len() + 1);
    if lead {
        out.push('@');
    }
    let mut last: Option<char> = None;
    for c in t.chars() {
        if "aeiouy^:".contains(c) {
            continue;
        }
        if Some(c) == last {
            continue;
        }
        out.push(c);
        last = Some(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn examples() {
        let r: Vec<String> = romanize("ভালোবাসি", 8).into_iter().map(|x| x.1).collect();
        assert_eq!(r[0], "bhalObasi");
        assert!(r.contains(&"valobasi".to_string()));
        assert_eq!(skeleton("valobashi"), "blbs");
        assert_eq!(skeleton("bhalObasi"), "blbs");
        assert_eq!(romanize("কর্ম", 1)[0].1, "kormo");
        assert_eq!(romanize("এবং", 1)[0].1, "ebong");
    }
}
