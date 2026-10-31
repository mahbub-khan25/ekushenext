//! Ekushe Next binary dictionary (`ekushe.dict`) and the Roman → Bangla ranker.
//!
//! Built by `tools/dict-builder` (`ekushe-dict pack`) from the open-licensed corpora; see
//! `tools/dict-builder/README.md`. Loading is zero-copy: the file is one byte buffer, sections are
//! located once at load, and every lookup reads little-endian integers from it.
//!
//! Ranking (ported from `tools/dict-builder/scripts/rank.py`, weights tuned on `eval/pilot.tsv`):
//! ```text
//! score = cheapest(romanization cost + weighted edit distance(typed, romanization) − exact bonus)
//!         − freq_w × log10(frequency per million) + class penalty − context bonus
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use once_cell::sync::OnceCell;

use crate::roman::{romanize, skeleton};

const MAGIC: &[u8; 8] = b"EKDICT\x00\x01";

/// Word classes (see `tools/dict-builder/RULES.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WordClass {
    Standard = 0,
    Variant = 1,
    Colloquial = 2,
    Sadhu = 3,
    Typo = 4,
    Vulgar = 5,
}

impl WordClass {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Variant,
            2 => Self::Colloquial,
            3 => Self::Sadhu,
            4 => Self::Typo,
            5 => Self::Vulgar,
            _ => Self::Standard,
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "variant" => Self::Variant,
            "colloquial" => Self::Colloquial,
            "sadhu" => Self::Sadhu,
            "typo" => Self::Typo,
            "vulgar" => Self::Vulgar,
            _ => Self::Standard,
        }
    }
}

pub const FLAG_SUSPECT: u8 = 1;
pub const FLAG_DISTINCT: u8 = 2;
pub const FLAG_CORE: u8 = 4;
pub const FLAG_SINGLE_SOURCE: u8 = 8;
/// The non-standard → standard decision is backed by a curated entry or an Academy rule (high confidence).
pub const FLAG_CONFIDENT: u8 = 16;

#[derive(Debug, Clone, Copy)]
pub struct WordInfo {
    /// log10(frequency per million words)
    pub log_ppm: f32,
    pub class: WordClass,
    pub flags: u8,
    pub preferred: Option<u32>,
}

#[derive(Debug)]
pub struct DictError(pub String);

impl std::fmt::Display for DictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ekushe.dict: {}", self.0)
    }
}
impl std::error::Error for DictError {}

#[derive(Default, Clone, Copy)]
struct Sec {
    off: usize,
    #[allow(dead_code)]
    len: usize,
}

pub struct Dictionary {
    bytes: Cow<'static, [u8]>,
    n_words: usize,
    n_keys: usize,
    word_offs: Sec,
    word_blob: Sec,
    meta: Sec,
    sorted: Sec,
    key_offs: Sec,
    key_blob: Sec,
    post_offs: Sec,
    postings: Sec,
    rom_offs: Sec,
    rom_blob: Sec,
    pair_offs: Sec,
    pair_next: Sec,
    pair_count: Sec,
}

impl std::fmt::Debug for Dictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dictionary({} words, {} keys, {} bytes)",
            self.n_words,
            self.n_keys,
            self.bytes.len()
        )
    }
}

#[inline]
fn rd_u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}
#[inline]
fn rd_u16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

static GLOBAL: OnceCell<Option<Dictionary>> = OnceCell::new();

impl Dictionary {
    /// Parse a dictionary image (embedded bytes or a file read into memory).
    pub fn from_bytes(bytes: Cow<'static, [u8]>) -> Result<Self, DictError> {
        let b: &[u8] = &bytes;
        if b.len() < 16 || &b[..8] != MAGIC {
            return Err(DictError("not an Ekushe dictionary (bad magic)".into()));
        }
        let mut pos = 8usize;
        let mut take = |len: usize| -> Result<Sec, DictError> {
            if pos + len > b.len() {
                return Err(DictError("truncated".into()));
            }
            let s = Sec { off: pos, len };
            pos += len;
            Ok(s)
        };
        let hdr = take(8)?;
        let n_words = rd_u32(b, hdr.off) as usize;
        let n_keys = rd_u32(b, hdr.off + 4) as usize;
        let word_offs = take((n_words + 1) * 4)?;
        let wlen = rd_u32(b, word_offs.off + n_words * 4) as usize;
        let word_blob = take(wlen)?;
        let meta = take(n_words * 8)?;
        let sorted = take(n_words * 4)?;
        let key_offs = take((n_keys + 1) * 4)?;
        let klen = rd_u32(b, key_offs.off + n_keys * 4) as usize;
        let key_blob = take(klen)?;
        let post_offs = take((n_keys + 1) * 4)?;
        let plen = rd_u32(b, post_offs.off + n_keys * 4) as usize;
        let postings = take(plen * 4)?;
        let rom_offs = take((n_words + 1) * 4)?;
        let rlen = rd_u32(b, rom_offs.off + n_words * 4) as usize;
        let rom_blob = take(rlen)?;
        let pair_offs = take((n_words + 1) * 4)?;
        let pn = rd_u32(b, pair_offs.off + n_words * 4) as usize;
        let pair_next = take(pn * 4)?;
        let pair_count = take(pn * 2)?;
        Ok(Self {
            bytes,
            n_words,
            n_keys,
            word_offs,
            word_blob,
            meta,
            sorted,
            key_offs,
            key_blob,
            post_offs,
            postings,
            rom_offs,
            rom_blob,
            pair_offs,
            pair_next,
            pair_count,
        })
    }

    /// Load `ekushe.dict` (raw) or `ekushe.dict.z` (zlib-compressed); the format is detected.
    pub fn load(path: &Path) -> Result<Self, DictError> {
        let data =
            std::fs::read(path).map_err(|e| DictError(format!("{}: {e}", path.display())))?;
        if data.starts_with(MAGIC) {
            Self::from_bytes(Cow::Owned(data))
        } else {
            Self::from_compressed(&data)
        }
    }

    /// Parse a zlib-compressed dictionary image.
    pub fn from_compressed(data: &[u8]) -> Result<Self, DictError> {
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(data)
            .map_err(|e| DictError(format!("decompress: {e:?}")))?;
        Self::from_bytes(Cow::Owned(raw))
    }

    /// The process-wide dictionary: the embedded copy (feature `embedded-dict`), else the first
    /// `ekushe.dict` or `ekushe.dict.z` found at `$EKUSHE_DICT`, next to the executable, in the per-user
    /// data folder (`%LOCALAPPDATA%\Ekushe`), or under `data/dictionary/` of the working directory.
    /// `None` if none is available (the engine then falls back to its small built-in word list).
    pub fn global() -> Option<&'static Dictionary> {
        GLOBAL
            .get_or_init(|| {
                #[cfg(feature = "embedded-dict")]
                {
                    static EMBED: &[u8] = include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../data/dictionary/ekushe.dict.z"
                    ));
                    if let Ok(d) = Dictionary::from_compressed(EMBED) {
                        return Some(d);
                    }
                }
                let mut paths = Vec::new();
                if let Ok(p) = std::env::var("EKUSHE_DICT") {
                    paths.push(std::path::PathBuf::from(p));
                }
                let mut dirs_to_try = Vec::new();
                if let Ok(exe) = std::env::current_exe() {
                    if let Some(dir) = exe.parent() {
                        dirs_to_try.push(dir.to_path_buf());
                    }
                }
                if let Some(d) = dirs::data_local_dir() {
                    dirs_to_try.push(d.join("Ekushe"));
                }
                dirs_to_try.push(std::path::PathBuf::from("data/dictionary"));
                for d in dirs_to_try {
                    paths.push(d.join("ekushe.dict"));
                    paths.push(d.join("ekushe.dict.z"));
                }
                paths.into_iter().find_map(|p| Dictionary::load(&p).ok())
            })
            .as_ref()
    }

    pub fn len(&self) -> usize {
        self.n_words
    }
    pub fn is_empty(&self) -> bool {
        self.n_words == 0
    }
    pub fn byte_size(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    fn b(&self) -> &[u8] {
        &self.bytes
    }

    pub fn word(&self, id: u32) -> &str {
        let b = self.b();
        let i = id as usize;
        let s = rd_u32(b, self.word_offs.off + i * 4) as usize;
        let e = rd_u32(b, self.word_offs.off + i * 4 + 4) as usize;
        std::str::from_utf8(&b[self.word_blob.off + s..self.word_blob.off + e]).unwrap_or("")
    }

    pub fn info(&self, id: u32) -> WordInfo {
        let b = self.b();
        let at = self.meta.off + id as usize * 8;
        let lf = i16::from_le_bytes([b[at], b[at + 1]]) as f32 / 1000.0;
        let class = WordClass::from_u8(b[at + 2]);
        let flags = b[at + 3];
        let p = rd_u32(b, at + 4);
        WordInfo {
            log_ppm: lf,
            class,
            flags,
            preferred: if p == u32::MAX { None } else { Some(p) },
        }
    }

    /// Find a word's id (binary search over the sorted id table).
    pub fn id_of(&self, word: &str) -> Option<u32> {
        let b = self.b();
        let (mut lo, mut hi) = (0usize, self.n_words);
        while lo < hi {
            let mid = (lo + hi) / 2;
            let id = rd_u32(b, self.sorted.off + mid * 4);
            match self.word(id).cmp(word) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return Some(id),
            }
        }
        None
    }

    /// The id at position `i` of the sorted table
    fn sorted_id(&self, i: usize) -> u32 {
        rd_u32(self.b(), self.sorted.off + i * 4)
    }

    /// Position in the sorted table of the first word not before `s`
    fn lower_bound(&self, s: &str) -> usize {
        let (mut lo, mut hi) = (0usize, self.n_words);
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.word(self.sorted_id(mid)) < s {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// Whether some word starts with `prefix` (so a word being typed may still become a real one).
    pub fn has_prefix(&self, prefix: &str) -> bool {
        let i = self.lower_bound(prefix);
        i < self.n_words && self.word(self.sorted_id(i)).starts_with(prefix)
    }

    /// Up to `n` everyday words that start with `prefix` and are longer than it, most common first
    /// (no typos or vulgar words). Looks at the first few thousand words in order, so it stays fast.
    pub fn completions(&self, prefix: &str, n: usize) -> Vec<String> {
        let mut found: Vec<(f32, u32)> = Vec::new();
        let start = self.lower_bound(prefix);
        for i in start..self.n_words.min(start + 4000) {
            let id = self.sorted_id(i);
            let w = self.word(id);
            if !w.starts_with(prefix) {
                break;
            }
            if w.len() == prefix.len() {
                continue;
            }
            let info = self.info(id);
            if matches!(info.class, WordClass::Typo | WordClass::Vulgar) {
                continue;
            }
            found.push((info.log_ppm, id));
        }
        found.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        found
            .into_iter()
            .take(n)
            .map(|(_, id)| self.word(id).to_string())
            .collect()
    }

    pub fn lookup(&self, word: &str) -> Option<WordInfo> {
        self.id_of(word).map(|id| self.info(id))
    }

    fn key(&self, k: usize) -> &str {
        let b = self.b();
        let s = rd_u32(b, self.key_offs.off + k * 4) as usize;
        let e = rd_u32(b, self.key_offs.off + k * 4 + 4) as usize;
        std::str::from_utf8(&b[self.key_blob.off + s..self.key_blob.off + e]).unwrap_or("")
    }

    fn key_lower_bound(&self, key: &str) -> usize {
        let (mut lo, mut hi) = (0usize, self.n_keys);
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.key(mid) < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    fn postings_at(&self, k: usize) -> impl Iterator<Item = u32> + '_ {
        let b = self.b();
        let s = rd_u32(b, self.post_offs.off + k * 4) as usize;
        let e = rd_u32(b, self.post_offs.off + k * 4 + 4) as usize;
        (s..e).map(move |i| rd_u32(b, self.postings.off + i * 4))
    }

    /// Words whose romanizations reduce to `key`, most frequent first.
    pub fn bucket(&self, key: &str) -> Vec<u32> {
        let k = self.key_lower_bound(key);
        if k < self.n_keys && self.key(k) == key {
            self.postings_at(k).collect()
        } else {
            Vec::new()
        }
    }

    fn bucket_len(&self, key: &str) -> Option<usize> {
        let k = self.key_lower_bound(key);
        if k < self.n_keys && self.key(k) == key {
            let b = self.b();
            Some(
                (rd_u32(b, self.post_offs.off + k * 4 + 4) - rd_u32(b, self.post_offs.off + k * 4))
                    as usize,
            )
        } else {
            None
        }
    }

    /// Stored romanizations (cost, roman). Core words store all of them; others store only extras
    /// such as English loanword spellings.
    fn stored_roms(&self, id: u32) -> Vec<(f32, &str)> {
        let b = self.b();
        let s = rd_u32(b, self.rom_offs.off + id as usize * 4) as usize;
        let e = rd_u32(b, self.rom_offs.off + id as usize * 4 + 4) as usize;
        let blob = &b[self.rom_blob.off + s..self.rom_blob.off + e];
        let mut out = Vec::new();
        let mut i = 0;
        while i < blob.len() {
            let cost = blob[i] as f32 / 20.0;
            let st = i + 1;
            let mut j = st;
            while j < blob.len() && blob[j] != 0 {
                j += 1;
            }
            out.push((cost, std::str::from_utf8(&blob[st..j]).unwrap_or("")));
            i = j + 1;
        }
        out
    }

    /// All romanizations of a word, cheapest first.
    pub fn romanizations(&self, id: u32) -> Vec<(f32, Cow<'_, str>)> {
        let info = self.info(id);
        let stored = self.stored_roms(id);
        let mut out: Vec<(f32, Cow<str>)> = Vec::new();
        if info.flags & FLAG_CORE == 0 {
            out.extend(
                romanize(self.word(id), 5)
                    .into_iter()
                    .map(|(c, r)| (c, Cow::Owned(r))),
            );
        }
        out.extend(stored.into_iter().map(|(c, r)| (c, Cow::Borrowed(r))));
        out
    }

    /// Most frequent words that follow `id` in running text, with their pair counts.
    pub fn followers(&self, id: u32) -> Vec<(u32, u16)> {
        let b = self.b();
        let s = rd_u32(b, self.pair_offs.off + id as usize * 4) as usize;
        let e = rd_u32(b, self.pair_offs.off + id as usize * 4 + 4) as usize;
        (s..e)
            .map(|i| {
                (
                    rd_u32(b, self.pair_next.off + i * 4),
                    rd_u16(b, self.pair_count.off + i * 2),
                )
            })
            .collect()
    }

    pub fn predict_next(&self, prev: &str, n: usize) -> Vec<String> {
        match self.id_of(prev) {
            Some(id) => self
                .followers(id)
                .into_iter()
                .filter(|(w, _)| self.info(*w).class == WordClass::Standard)
                .take(n)
                .map(|(w, _)| self.word(w).to_string())
                .collect(),
            None => Vec::new(),
        }
    }
}

// ------------------------------------------------------------------------------------------ ranking

/// Tunable ranking weights (defaults tuned on `tools/dict-builder/eval/pilot.tsv`).
#[derive(Debug, Clone, Copy)]
pub struct RankParams {
    pub freq_w: f32,
    pub exact: f32,
    pub case: f32,
    pub lift: f32,
    pub final_vowel: f32,
    pub context_w: f32,
    pub completion: f32,
    /// Largest log10 lift the context may add (scaled by `context_w`)
    pub ctx_max: f32,
    /// Typing cost of one Roman letter standing for another close one (s for ছ, v for ভ, z for জ …)
    pub sc_pair: f32,
}

impl Default for RankParams {
    fn default() -> Self {
        Self {
            freq_w: 0.55,
            exact: 0.8,
            case: 0.3,
            lift: 0.25,
            final_vowel: 0.8,
            context_w: 0.5,
            completion: 0.9,
            ctx_max: 3.0,
            sc_pair: 0.4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ranked {
    pub word: String,
    pub id: u32,
    pub class: WordClass,
    pub score: f32,
    /// Shown as a completion of a longer word (typed text is a prefix)
    pub completion: bool,
}

fn class_penalty(c: WordClass) -> f32 {
    match c {
        WordClass::Standard => 0.0,
        WordClass::Variant => 0.9,
        WordClass::Colloquial => 0.7,
        WordClass::Sadhu => 1.2,
        WordClass::Vulgar => 1.5,
        WordClass::Typo => 9.0,
    }
}

#[inline]
fn is_vowel(c: u8) -> bool {
    matches!(c, b'a' | b'e' | b'i' | b'o' | b'u')
}
#[inline]
fn cheap_char(c: u8) -> bool {
    matches!(c, b'h' | b'y' | b'w' | b'^' | b':' | b'`') || is_vowel(c)
}
#[inline]
fn cheap_pair(a: u8, b: u8) -> bool {
    matches!(
        (a.min(b), a.max(b)),
        (b'c', b's')
            | (b'j', b'z')
            | (b'b', b'v')
            | (b'j', b'y')
            | (b'b', b'w')
            | (b'f', b'p')
            | (b'i', b'y')
            | (b'e', b'y')
    )
}

/// Weighted edit distance tuned for Banglish (both inputs lower-case ASCII). With `prefix`, the end of
/// `b` is free, so it measures how well `a` matches the start of `b`.
fn wdist(
    a: &[u8],
    b: &[u8],
    final_vowel: f32,
    pair_cost: f32,
    prefix: bool,
    row: &mut Vec<f32>,
    cur: &mut Vec<f32>,
) -> f32 {
    let (la, lb) = (a.len(), b.len());
    row.clear();
    row.extend((0..=lb).map(|j| j as f32));
    for i in 1..=la {
        cur.clear();
        cur.push(i as f32);
        let ca = a[i - 1];
        for j in 1..=lb {
            let cb = b[j - 1];
            let sub = if ca == cb {
                0.0
            } else if is_vowel(ca) && is_vowel(cb) {
                if i == la && j == lb {
                    final_vowel
                } else {
                    0.35
                }
            } else if cheap_pair(ca, cb) {
                pair_cost
            } else {
                1.0
            };
            let ins_b = if cheap_char(cb) || (j > 1 && b[j - 2] == cb) {
                0.25
            } else {
                1.0
            };
            let del_a = if cheap_char(ca) || (i > 1 && a[i - 2] == ca) {
                0.25
            } else {
                1.0
            };
            let v = (row[j - 1] + sub)
                .min(row[j] + del_a)
                .min(cur[j - 1] + ins_b);
            cur.push(v);
        }
        std::mem::swap(row, cur);
    }
    if prefix {
        row.iter().copied().fold(f32::INFINITY, f32::min)
    } else {
        row[lb]
    }
}

const SKEL_ALPHABET: &[u8] = b"bcdfghjklmnprst";
/// log10 of the pair-counting corpus size in millions of words: all of Bangla Wikipedia, FineWeb-2 and
/// Sangraha's verified Bengali, 9,609 M (2026-10-28; was 1,680 M, 3.225). Must match the dictionary's
/// word pairs: tools/dict-builder/build.sh prints the word totals.
const PAIR_CORPUS_LOG10_M: f32 = 3.983;

impl Dictionary {
    /// Candidate ids for typed Roman text: its key's bucket, nearby keys (one consonant off) when the
    /// bucket is small, and completions (longer words sharing the key as a prefix).
    fn candidates(&self, sk: &str, limit: usize) -> (Vec<u32>, Vec<u32>) {
        let mut out: Vec<u32> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut push = |ids: Vec<u32>, out: &mut Vec<u32>, cap: usize| {
            for id in ids.into_iter().take(cap) {
                if seen.insert(id) {
                    out.push(id);
                }
            }
        };
        let main = self.bucket(sk);
        let small = main.len() < 20;
        push(main, &mut out, limit);
        // casual Banglish writes `s` for ছ (ase = আছে, kisu = কিছু): also read each `s` as ছ (key `c`)
        if sk.contains('s') {
            let positions: Vec<usize> = sk.match_indices('s').map(|(i, _)| i).collect();
            let mut alias: Vec<String> = positions
                .iter()
                .map(|&i| format!("{}c{}", &sk[..i], &sk[i + 1..]))
                .collect();
            if positions.len() > 1 {
                alias.push(sk.replace('s', "c"));
            }
            for k in alias {
                push(self.bucket(&k), &mut out, 40);
            }
        }
        let core_len = sk.trim_start_matches('@').len();
        if small && core_len >= 2 {
            let chars: Vec<u8> = sk.bytes().collect();
            let start = if chars.first() == Some(&b'@') { 1 } else { 0 };
            let mut near: Vec<String> = Vec::new();
            for i in start..chars.len() {
                let mut d = chars.clone();
                d.remove(i);
                near.push(String::from_utf8(d).unwrap());
                for &c in SKEL_ALPHABET {
                    if c != chars[i] {
                        let mut s = chars.clone();
                        s[i] = c;
                        near.push(String::from_utf8(s).unwrap());
                    }
                }
                if i + 1 < chars.len() {
                    let mut t = chars.clone();
                    t.swap(i, i + 1);
                    near.push(String::from_utf8(t).unwrap());
                }
            }
            for i in start..=chars.len() {
                for &c in SKEL_ALPHABET {
                    let mut s = chars.clone();
                    s.insert(i, c);
                    near.push(String::from_utf8(s).unwrap());
                }
            }
            for k in near {
                if self.bucket_len(&k).is_some() {
                    push(self.bucket(&k), &mut out, 40);
                }
            }
        }
        // completions
        let mut comps: Vec<u32> = Vec::new();
        if core_len >= 2 {
            let mut k = self.key_lower_bound(sk);
            let mut keys = 0;
            while k < self.n_keys && keys < 400 {
                let key = self.key(k);
                if !key.starts_with(sk) {
                    break;
                }
                if key.len() > sk.len() {
                    for id in self.postings_at(k).take(3) {
                        comps.push(id);
                    }
                    keys += 1;
                }
                k += 1;
            }
            comps.sort_by(|a, b| {
                self.info(*b)
                    .log_ppm
                    .partial_cmp(&self.info(*a).log_ppm)
                    .unwrap()
            });
            comps.retain(|id| !out.contains(id));
            comps.truncate(30);
        }
        (out, comps)
    }

    /// Rank Bangla words for typed Roman text. `prev` is the previous committed word (context).
    pub fn rank(&self, typed: &str, prev: Option<&str>, n: usize, p: &RankParams) -> Vec<Ranked> {
        if typed.is_empty() || !typed.bytes().any(|c| c.is_ascii_alphabetic()) {
            return Vec::new();
        }
        let tl = typed.to_ascii_lowercase();
        let tb = tl.as_bytes();
        let has_case = typed != tl;
        let sk = skeleton(typed);
        let (cands, comps) = self.candidates(&sk, 400);
        let prev_id = prev.and_then(|w| self.id_of(w));
        let prev_lp = prev_id.map(|id| self.info(id).log_ppm).unwrap_or(0.0);
        let ctx: HashMap<u32, u16> = prev_id
            .map(|id| self.followers(id).into_iter().collect())
            .unwrap_or_default();
        let (mut row, mut cur) = (Vec::with_capacity(32), Vec::with_capacity(32));
        let mut scores: HashMap<u32, (f32, bool)> = HashMap::new();
        let mut runtime_budget = 40usize;
        let best_rom = |id: u32,
                        prefix: bool,
                        budget: &mut usize,
                        row: &mut Vec<f32>,
                        cur: &mut Vec<f32>|
         -> Option<f32> {
            let info = self.info(id);
            if info.flags & FLAG_CORE == 0 {
                if *budget == 0 {
                    return None;
                }
                *budget -= 1;
            }
            let mut best = f32::INFINITY;
            for (c, r) in self.romanizations(id) {
                let rl = r.to_ascii_lowercase();
                let mut bonus = 0.0;
                if !prefix && rl == tl {
                    bonus += p.exact * if c <= 0.35 { 1.0 } else { 0.4 };
                    if has_case && r.as_ref() == typed {
                        bonus += p.case;
                    }
                }
                let d = wdist(
                    tb,
                    rl.as_bytes(),
                    p.final_vowel,
                    p.sc_pair,
                    prefix,
                    row,
                    cur,
                );
                best = best.min(c + d - bonus);
            }
            Some(best)
        };
        let consider = |id: u32,
                        prefix: bool,
                        scores: &mut HashMap<u32, (f32, bool)>,
                        budget: &mut usize,
                        row: &mut Vec<f32>,
                        cur: &mut Vec<f32>| {
            let info = self.info(id);
            if info.class == WordClass::Typo {
                return;
            }
            let Some(a) = best_rom(id, prefix, budget, row, cur) else {
                return;
            };
            if info.class == WordClass::Vulgar && (prefix || a > -0.3) {
                return; // vulgar words only on an exact spelling
            }
            // How much more likely the word is after `prev` than on its own (log10 lift, clipped):
            // P(w|prev) / P(w) ≈ count(prev,w) · 10^6 / (CORPUS_M · ppm(prev) · ppm(w))
            let ctx_bonus = |w: u32| {
                ctx.get(&w)
                    .map(|&c| {
                        let lift = (c as f32).log10() + 6.0
                            - PAIR_CORPUS_LOG10_M
                            - prev_lp
                            - self.info(w).log_ppm;
                        p.context_w * lift.clamp(0.0, p.ctx_max)
                    })
                    .unwrap_or(0.0)
            };
            let extra = if prefix { p.completion } else { 0.0 };
            let s = a + extra - p.freq_w * info.log_ppm + class_penalty(info.class) - ctx_bonus(id);
            let e = scores.entry(id).or_insert((f32::INFINITY, prefix));
            if s < e.0 {
                *e = (s, prefix);
            }
            // a non-standard form that matches well also brings in its standard form
            if let Some(pref) = info.preferred {
                let pi = self.info(pref);
                if pi.class != WordClass::Typo {
                    let s2 = a + extra + p.lift - p.freq_w * pi.log_ppm - ctx_bonus(pref);
                    let e2 = scores.entry(pref).or_insert((f32::INFINITY, prefix));
                    if s2 < e2.0 {
                        *e2 = (s2, prefix);
                    }
                }
            }
        };
        for id in cands {
            consider(
                id,
                false,
                &mut scores,
                &mut runtime_budget,
                &mut row,
                &mut cur,
            );
        }
        for id in comps {
            consider(
                id,
                true,
                &mut scores,
                &mut runtime_budget,
                &mut row,
                &mut cur,
            );
        }
        let mut v: Vec<(u32, f32, bool)> =
            scores.into_iter().map(|(id, (s, c))| (id, s, c)).collect();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then_with(|| a.0.cmp(&b.0)));
        // A spoken (colloquial) form is offered right after its standard form when both match, so
        // the standard spelling comes first and the spoken one is still at hand (খেয়েছো, খাইছো)
        let mut i = 0;
        while i < v.len() {
            let info = self.info(v[i].0);
            if info.class == WordClass::Colloquial {
                if let Some(pos) = info
                    .preferred
                    .and_then(|p| v[..i].iter().position(|x| x.0 == p))
                {
                    if pos + 1 < i {
                        let item = v.remove(i);
                        v.insert(pos + 1, item);
                    }
                }
            }
            i += 1;
        }
        v.truncate(n);
        v.into_iter()
            .map(|(id, score, completion)| Ranked {
                word: self.word(id).to_string(),
                id,
                class: self.info(id).class,
                score,
                completion,
            })
            .collect()
    }
}

// ------------------------------------------------------------------------------------------ writing

/// One word for [`DictBuilder`].
pub struct WordRecord {
    pub word: String,
    pub ppm: f64,
    pub class: WordClass,
    pub flags: u8,
    pub preferred: Option<String>,
    /// Extra romanizations (e.g. English loanword spellings) with their cost
    pub extra_roms: Vec<(f32, String)>,
}

/// Writes `ekushe.dict`. Core words (FLAG_CORE) get their romanizations stored; the rest are
/// romanized at typing time.
#[derive(Default)]
pub struct DictBuilder {
    pub words: Vec<WordRecord>,
    /// (previous word, next word, count)
    pub pairs: Vec<(String, String, u32)>,
    pub max_followers: usize,
}

impl DictBuilder {
    pub fn write(&self, path: &Path) -> std::io::Result<(usize, usize)> {
        let n = self.words.len();
        let ids: HashMap<&str, u32> = self
            .words
            .iter()
            .enumerate()
            .map(|(i, w)| (w.word.as_str(), i as u32))
            .collect();
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(MAGIC);
        // keys
        let mut index: std::collections::BTreeMap<String, Vec<(f64, u32)>> = Default::default();
        let mut roms_per_word: Vec<Vec<(f32, String)>> = Vec::with_capacity(n);
        for (i, w) in self.words.iter().enumerate() {
            let mut roms = Vec::new();
            let mut stored = Vec::new();
            if w.class != WordClass::Typo {
                let core = w.flags & FLAG_CORE != 0;
                let gen = romanize(&w.word, if core { 8 } else { 5 });
                if core {
                    stored.extend(gen.iter().cloned());
                }
                roms.extend(gen);
                roms.extend(w.extra_roms.iter().cloned());
                stored.extend(w.extra_roms.iter().cloned());
                let mut keys: Vec<String> = roms.iter().map(|(_, r)| skeleton(r)).collect();
                keys.sort();
                keys.dedup();
                for k in keys {
                    if !k.is_empty() {
                        index.entry(k).or_default().push((w.ppm, i as u32));
                    }
                }
            }
            roms_per_word.push(stored);
        }
        let n_keys = index.len();
        out.extend_from_slice(&(n as u32).to_le_bytes());
        out.extend_from_slice(&(n_keys as u32).to_le_bytes());
        // words
        let mut blob = Vec::new();
        for w in &self.words {
            out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
            blob.extend_from_slice(w.word.as_bytes());
        }
        out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
        out.extend_from_slice(&blob);
        // meta
        for w in &self.words {
            let lf = ((w.ppm.max(1e-3)).log10() * 1000.0)
                .round()
                .clamp(-32000.0, 32000.0) as i16;
            out.extend_from_slice(&lf.to_le_bytes());
            out.push(w.class as u8);
            out.push(w.flags);
            let p = w
                .preferred
                .as_deref()
                .and_then(|p| ids.get(p).copied())
                .unwrap_or(u32::MAX);
            out.extend_from_slice(&p.to_le_bytes());
        }
        // sorted ids
        let mut sorted: Vec<u32> = (0..n as u32).collect();
        sorted.sort_by(|a, b| {
            self.words[*a as usize]
                .word
                .cmp(&self.words[*b as usize].word)
        });
        for id in sorted {
            out.extend_from_slice(&id.to_le_bytes());
        }
        // keys + postings
        let mut kblob = Vec::new();
        let mut post: Vec<u32> = Vec::new();
        let mut poffs = Vec::new();
        for (k, v) in &index {
            out.extend_from_slice(&(kblob.len() as u32).to_le_bytes());
            kblob.extend_from_slice(k.as_bytes());
            poffs.push(post.len() as u32);
            let mut v = v.clone();
            v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then_with(|| a.1.cmp(&b.1)));
            post.extend(v.into_iter().map(|(_, id)| id));
        }
        out.extend_from_slice(&(kblob.len() as u32).to_le_bytes());
        out.extend_from_slice(&kblob);
        poffs.push(post.len() as u32);
        for o in poffs {
            out.extend_from_slice(&o.to_le_bytes());
        }
        for id in &post {
            out.extend_from_slice(&id.to_le_bytes());
        }
        // stored romanizations
        let mut rblob = Vec::new();
        for roms in &roms_per_word {
            out.extend_from_slice(&(rblob.len() as u32).to_le_bytes());
            for (c, r) in roms {
                rblob.push((c * 20.0).round().clamp(0.0, 255.0) as u8);
                rblob.extend_from_slice(r.as_bytes());
                rblob.push(0);
            }
        }
        out.extend_from_slice(&(rblob.len() as u32).to_le_bytes());
        out.extend_from_slice(&rblob);
        // pairs
        let mut per: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];
        for (a, b, c) in &self.pairs {
            if let (Some(&ia), Some(&ib)) = (ids.get(a.as_str()), ids.get(b.as_str())) {
                per[ia as usize].push((ib, *c));
            }
        }
        let maxf = if self.max_followers == 0 {
            16
        } else {
            self.max_followers
        };
        let mut next = Vec::new();
        let mut cnt = Vec::new();
        for list in per.iter_mut() {
            out.extend_from_slice(&(next.len() as u32).to_le_bytes());
            list.sort_by_key(|b| std::cmp::Reverse(b.1));
            for &(id, c) in list.iter().take(maxf) {
                next.push(id);
                cnt.push(c.min(65535) as u16);
            }
        }
        out.extend_from_slice(&(next.len() as u32).to_le_bytes());
        for id in &next {
            out.extend_from_slice(&id.to_le_bytes());
        }
        for c in &cnt {
            out.extend_from_slice(&c.to_le_bytes());
        }
        let mut f = std::fs::File::create(path)?;
        f.write_all(&out)?;
        // compressed copy for embedding / shipping: <path>.z
        let z = miniz_oxide::deflate::compress_to_vec_zlib(&out, 9);
        let mut zp = path.as_os_str().to_owned();
        zp.push(".z");
        std::fs::write(std::path::PathBuf::from(zp), &z)?;
        Ok((out.len(), n_keys))
    }
}
