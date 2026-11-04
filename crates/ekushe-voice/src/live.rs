//! Live voice typing with a fast Bangla Zipformer (sherpa-onnx, Apache-2.0).
//!
//! The model reads a few seconds of speech in a few hundredths of a second, so the last few seconds
//! of speech (the "window") are simply read again and again while the person speaks:
//! - words that two readings in a row agree on are typed at once, and never changed afterwards;
//! - when the window gets long, its start moves up to the first word not yet typed, so every
//!   reading stays short: no slowdown however long the dictation, and nothing piles up (one reading
//!   at a time; a slower computer just reads less often);
//! - at a pause (the Silero speech detector), the window is read once more and the rest is typed.

use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineTransducerModelConfig, SileroVadModelConfig,
    VadModelConfig, VoiceActivityDetector,
};
use std::path::Path;

/// The Bangla speech model: a folder with encoder/decoder/joiner ONNX files and tokens.txt
pub struct ZipModel {
    rec: OfflineRecognizer,
}

fn s(p: &Path) -> Option<String> {
    Some(p.to_string_lossy().into_owned())
}

impl ZipModel {
    pub fn load(dir: &Path, threads: usize) -> Result<Self, String> {
        let pick = |a: &str, b: &str| {
            let p = dir.join(a);
            if p.exists() {
                p
            } else {
                dir.join(b)
            }
        };
        // a missing file crashes the engine instead of failing politely: check first
        for f in ["encoder", "decoder", "joiner"] {
            if !pick(&format!("{f}.int8.onnx"), &format!("{f}.onnx")).exists() {
                return Err(format!("the speech model is incomplete ({f} is missing)"));
            }
        }
        if !dir.join("tokens.txt").exists() {
            return Err("the speech model is incomplete (tokens.txt is missing)".into());
        }
        let mut c = OfflineRecognizerConfig::default();
        c.model_config.transducer = OfflineTransducerModelConfig {
            encoder: s(&pick("encoder.int8.onnx", "encoder.onnx")),
            decoder: s(&pick("decoder.int8.onnx", "decoder.onnx")),
            joiner: s(&pick("joiner.int8.onnx", "joiner.onnx")),
        };
        c.model_config.tokens = s(&dir.join("tokens.txt"));
        c.model_config.num_threads = threads.clamp(1, 8) as i32;
        c.model_config.provider = Some("cpu".into());
        c.decoding_method = Some("greedy_search".into());
        let rec = OfflineRecognizer::create(&c).ok_or("the speech model could not be loaded")?;
        Ok(Self { rec })
    }

    /// Text for 16 kHz mono samples
    pub fn read(&self, pcm: &[f32]) -> String {
        if pcm.len() < 1600 {
            return String::new();
        }
        let st = self.rec.create_stream();
        st.accept_waveform(16_000, pcm);
        self.rec.decode(&st);
        st.get_result()
            .map(|r| r.text.trim().to_string())
            .unwrap_or_default()
    }
}

/// The model is shipped as one file (easy to download and check): a short header, then the files
/// of the model folder one after another. [`unpack`] puts them in a folder next to it, once.
const PACK_MAGIC: &[u8; 6] = b"EKZM1\n";

/// Pack the files of a model folder (encoder, decoder, joiner, tokens.txt, credits) into one file
pub fn pack(dir: &Path, names: &[&str], out: &Path) -> Result<(), String> {
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(out).map_err(|e| e.to_string())?);
    let mut head = PACK_MAGIC.to_vec();
    head.extend((names.len() as u32).to_le_bytes());
    for n in names {
        let len = std::fs::metadata(dir.join(n))
            .map_err(|e| format!("{n}: {e}"))?
            .len();
        head.extend((n.len() as u16).to_le_bytes());
        head.extend(n.as_bytes());
        head.extend(len.to_le_bytes());
    }
    f.write_all(&head).map_err(|e| e.to_string())?;
    for n in names {
        let mut src = std::fs::File::open(dir.join(n)).map_err(|e| e.to_string())?;
        std::io::copy(&mut src, &mut f).map_err(|e| e.to_string())?;
    }
    f.flush().map_err(|e| e.to_string())
}

/// The folder with the model's files for a packed model file (unpacked next to it the first time,
/// and again if a file there is missing or has the wrong size). A folder is used as it is.
pub fn unpack(file: &Path) -> Result<std::path::PathBuf, String> {
    use std::io::{Read, Seek, SeekFrom};
    if file.is_dir() {
        return Ok(file.to_path_buf());
    }
    let bad = || "the speech model file is damaged; remove it and download it again".to_string();
    let mut f = std::fs::File::open(file).map_err(|e| e.to_string())?;
    let mut magic = [0u8; 6];
    f.read_exact(&mut magic).map_err(|_| bad())?;
    if &magic != PACK_MAGIC {
        return Err(bad());
    }
    let mut b4 = [0u8; 4];
    f.read_exact(&mut b4).map_err(|_| bad())?;
    let count = u32::from_le_bytes(b4) as usize;
    if count > 64 {
        return Err(bad());
    }
    let mut entries = vec![];
    for _ in 0..count {
        let mut b2 = [0u8; 2];
        f.read_exact(&mut b2).map_err(|_| bad())?;
        let mut name = vec![0u8; u16::from_le_bytes(b2) as usize];
        f.read_exact(&mut name).map_err(|_| bad())?;
        let name = String::from_utf8(name).map_err(|_| bad())?;
        // plain file names only
        if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
            return Err(bad());
        }
        let mut b8 = [0u8; 8];
        f.read_exact(&mut b8).map_err(|_| bad())?;
        entries.push((name, u64::from_le_bytes(b8)));
    }
    let data_at = f.stream_position().map_err(|e| e.to_string())?;
    let total: u64 = entries.iter().map(|e| e.1).sum();
    if f.metadata().map_err(|e| e.to_string())?.len() != data_at + total {
        return Err(bad());
    }
    let dir = file.with_extension("d");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // which pack the files came from: a new pack with the same file sizes (another training of the
    // same model) is unpacked again
    let meta = f.metadata().map_err(|e| e.to_string())?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let from = format!("{} {mtime}", meta.len());
    let stamp = dir.join(".from");
    let same_pack = std::fs::read_to_string(&stamp)
        .map(|s| s == from)
        .unwrap_or(false);
    let mut at = data_at;
    for (name, len) in &entries {
        let dst = dir.join(name);
        let ok = same_pack
            && std::fs::metadata(&dst)
                .map(|m| m.len() == *len)
                .unwrap_or(false);
        if !ok {
            f.seek(SeekFrom::Start(at)).map_err(|e| e.to_string())?;
            let part = dir.join(format!("{name}.part"));
            let mut out = std::fs::File::create(&part).map_err(|e| e.to_string())?;
            std::io::copy(&mut (&mut f).take(*len), &mut out).map_err(|e| e.to_string())?;
            drop(out);
            std::fs::rename(&part, &dst).map_err(|e| e.to_string())?;
        }
        at += len;
    }
    if !same_pack {
        std::fs::write(&stamp, from).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

/// A heard word and when it starts (seconds from the start of the audio that was read)
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub at: f32,
}

impl ZipModel {
    /// The words in 16 kHz mono samples, with their start times
    pub fn words(&self, pcm: &[f32]) -> Vec<Word> {
        if pcm.len() < 1600 {
            return vec![];
        }
        let st = self.rec.create_stream();
        st.accept_waveform(16_000, pcm);
        self.rec.decode(&st);
        let Some(r) = st.get_result() else {
            return vec![];
        };
        let times = r.timestamps.unwrap_or_default();
        let mut out: Vec<Word> = vec![];
        for (i, t) in r.tokens.iter().enumerate() {
            let at = times.get(i).copied().unwrap_or(0.0);
            // a token starting with a space starts a new word
            if t.starts_with(' ') || t.starts_with('▁') || out.is_empty() {
                out.push(Word {
                    text: t.trim_start_matches([' ', '▁']).to_string(),
                    at,
                });
            } else if let Some(w) = out.last_mut() {
                w.text.push_str(t);
            }
        }
        out.retain(|w| !w.text.is_empty());
        out
    }
}

/// The words without the punctuation the model put on them ("লাইন।" → "লাইন"; a lone "।" goes)
pub fn without_marks(words: Vec<Word>) -> Vec<Word> {
    const MARKS: &[char] = &[
        '।', '॥', ',', '?', '!', ';', ':', '-', '—', '–', '"', '\'', '“', '”', '‘', '’', '.',
    ];
    words
        .into_iter()
        .filter_map(|mut w| {
            let t = w.text.trim_matches(MARKS);
            if t.is_empty() {
                return None;
            }
            w.text = t.to_string();
            Some(w)
        })
        .collect()
}

/// Decides what to type from repeated readings of a moving window of speech.
/// Positions are sample counts (16 kHz) from the start of the dictation.
pub struct Typist {
    /// where the window starts
    pub start: usize,
    prev: Vec<String>,
    typed: usize,
    /// the last few words typed (to find where typing stopped in a new reading)
    recent: Vec<String>,
    tail: String,
    /// move the window up once it is longer than this (seconds)
    pub win: f32,
    /// …keeping at least this much of its end (seconds) to read again
    pub keep: f32,
    /// never read more than this (seconds): the model fails on very long audio
    pub most: f32,
}

const SR: f32 = 16_000.0;

impl Default for Typist {
    fn default() -> Self {
        Self {
            start: 0,
            prev: vec![],
            typed: 0,
            recent: vec![],
            tail: String::new(),
            win: 6.0,
            keep: 2.0,
            most: 16.0,
        }
    }
}

impl Typist {
    /// A reading of the window up to now (`len` samples long). Returns the words to type now.
    pub fn partial(&mut self, w: &[Word], len: usize) -> Vec<String> {
        let now: Vec<String> = w.iter().map(|x| x.text.clone()).collect();
        let mut n = self
            .prev
            .iter()
            .zip(&now)
            .take_while(|(a, b)| a == b)
            .count();
        n = n.min(now.len().saturating_sub(1));
        let mut out = vec![];
        let p = self.align(&now, now.len());
        self.typed = p;
        if n > p {
            out.extend_from_slice(&now[p..n]);
            self.typed = n;
        }
        self.prev = now;
        let secs = len as f32 / SR;
        let mut moved = false;
        if secs > self.win && self.typed >= 1 {
            // move the start up to the first word not yet typed, keeping `keep` seconds
            let mut k = self.typed.min(w.len());
            while k > 0 && w[k - 1].at > secs - self.keep {
                k -= 1;
            }
            if k >= 1 && k < w.len() {
                self.start += ((w[k].at - 0.12).max(0.0) * SR) as usize;
                self.typed -= k;
                // the next reading starts elsewhere: wait for two new readings to agree
                self.prev.clear();
                moved = true;
            }
        }
        if !moved && secs > self.most {
            // no word boundary to move to: type what is there and start afresh near the end
            let keep_from = secs - self.keep;
            let cut = w.iter().position(|x| x.at >= keep_from).unwrap_or(w.len());
            if cut > self.typed {
                out.extend(w[self.typed..cut].iter().map(|x| x.text.clone()));
            }
            self.start += (keep_from * SR) as usize;
            self.typed = 0;
            self.prev.clear();
        }
        self.tail = self
            .prev
            .get(self.typed..)
            .map(|t| t.join(" "))
            .unwrap_or_default();
        self.remember(&out);
        out
    }

    /// Where the typed words end in a new reading. Usually where they ended in the last one, but
    /// a reading can hear a word more or less before that (most often the word at the window's
    /// start, read again after the window moved): match the last words typed, so no word is typed
    /// twice and none is skipped.
    fn align(&self, now: &[String], upto: usize) -> usize {
        let t = self.typed.min(upto);
        let score = |p: usize| {
            (1..=3)
                .take_while(|&j| {
                    p >= j
                        && self.recent.len() >= j
                        && now[p - j] == self.recent[self.recent.len() - j]
                })
                .count()
        };
        let mut best = (0, t);
        for p in t.saturating_sub(2)..=(t + 2).min(upto) {
            let sc = score(p);
            let closer = p.abs_diff(t) < best.1.abs_diff(t);
            if sc > best.0 || (sc == best.0 && sc > 0 && closer) {
                best = (sc, p);
            }
        }
        if best.0 == 0 {
            t
        } else {
            best.1
        }
    }

    fn remember(&mut self, typed: &[String]) {
        self.recent.extend(typed.iter().cloned());
        let n = self.recent.len();
        if n > 3 {
            self.recent.drain(..n - 3);
        }
    }

    /// A last reading of the window when the phrase ended (a pause, or stop): type the rest and
    /// start the next window at `next`.
    pub fn finish(&mut self, w: &[Word], next: usize) -> Vec<String> {
        let now: Vec<String> = w.iter().map(|x| x.text.clone()).collect();
        let p = self.align(&now, now.len());
        let out: Vec<String> = now[p..].to_vec();
        self.remember(&out);
        self.start = next.max(self.start);
        self.prev.clear();
        self.typed = 0;
        self.tail.clear();
        out
    }

    /// Heard but not typed yet (for the banner)
    pub fn tail(&self) -> &str {
        &self.tail
    }
}

/// The Silero speech detector (MIT), built into the program
pub const VAD_ONNX: &[u8] = include_bytes!("../assets/silero_vad.onnx");

/// Finds where phrases start and end, fed as the audio comes in
pub struct Phrases {
    vad: VoiceActivityDetector,
}

impl Phrases {
    /// `pause` = the silence (seconds) that ends a phrase
    pub fn new(pause: f32) -> Result<Self, String> {
        let p = std::env::temp_dir().join("ekushe-silero-vad.onnx");
        if std::fs::metadata(&p)
            .map(|m| m.len() as usize != VAD_ONNX.len())
            .unwrap_or(true)
        {
            std::fs::write(&p, VAD_ONNX).map_err(|e| e.to_string())?;
        }
        let cfg = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: s(&p),
                threshold: 0.45,
                min_silence_duration: pause,
                min_speech_duration: 0.2,
                window_size: 512,
                // the model was trained on clips of up to 20 s; longer speech is cut at a pause
                max_speech_duration: 12.0,
            },
            sample_rate: 16_000,
            num_threads: 1,
            provider: Some("cpu".into()),
            debug: false,
            ..Default::default()
        };
        let vad = VoiceActivityDetector::create(&cfg, 60.0).ok_or("speech detector failed")?;
        Ok(Self { vad })
    }

    pub fn feed(&self, samples: &[f32]) {
        self.vad.accept_waveform(samples);
    }

    /// Someone is speaking right now
    pub fn speaking(&self) -> bool {
        self.vad.detected()
    }

    /// Finished phrases as (start, end) sample positions since the beginning
    pub fn finished(&self) -> Vec<(usize, usize)> {
        let mut out = vec![];
        while !self.vad.is_empty() {
            if let Some(seg) = self.vad.front() {
                let a = seg.start().max(0) as usize;
                out.push((a, a + seg.n().max(0) as usize));
            }
            self.vad.pop();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{Typist, Word};
    fn ws(s: &str, step: f32) -> Vec<Word> {
        s.split_whitespace()
            .enumerate()
            .map(|(i, t)| Word {
                text: t.into(),
                at: i as f32 * step,
            })
            .collect()
    }

    #[test]
    fn types_agreed_words_once() {
        let mut t = Typist::default();
        assert!(t.partial(&ws("আমি বাং", 0.4), 16000).is_empty());
        assert_eq!(t.partial(&ws("আমি বাংলায়", 0.4), 20000), vec!["আমি"]);
        assert_eq!(t.partial(&ws("আমি বাংলায় গান", 0.4), 24000), vec!["বাংলায়"]);
        assert_eq!(t.tail(), "গান");
        assert_eq!(
            t.finish(&ws("আমি বাংলায় গান গাই", 0.4), 30000),
            vec!["গান", "গাই"]
        );
        assert_eq!(t.start, 30000);
    }

    #[test]
    fn pack_and_unpack() {
        let d = std::env::temp_dir().join(format!("ekzm-test-{}", std::process::id()));
        let src = d.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("a.onnx"), b"hello").unwrap();
        std::fs::write(src.join("tokens.txt"), "x 1\ny 2\n").unwrap();
        let f = d.join("m.ekm");
        super::pack(&src, &["a.onnx", "tokens.txt"], &f).unwrap();
        let out = super::unpack(&f).unwrap();
        assert_eq!(std::fs::read(out.join("a.onnx")).unwrap(), b"hello");
        assert_eq!(
            std::fs::read_to_string(out.join("tokens.txt")).unwrap(),
            "x 1\ny 2\n"
        );
        // another training with the same file sizes is unpacked again
        std::thread::sleep(std::time::Duration::from_millis(1100));
        std::fs::write(src.join("a.onnx"), b"howdy").unwrap();
        super::pack(&src, &["a.onnx", "tokens.txt"], &f).unwrap();
        let out = super::unpack(&f).unwrap();
        assert_eq!(std::fs::read(out.join("a.onnx")).unwrap(), b"howdy");
        // a cut-off download is refused
        let b = std::fs::read(&f).unwrap();
        std::fs::write(&f, &b[..b.len() - 1]).unwrap();
        assert!(super::unpack(&f).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn word_read_again_after_the_window_moved_is_not_typed_twice() {
        let mut t = Typist::default();
        let long = "এক দুই তিন চার পাঁচ ছয় সাত আট নয় দশ এগারো বারো তেরো চোদ্দ পনেরো";
        t.partial(&ws(long, 0.5), 7 * 16000);
        t.partial(&ws(long, 0.5), 7 * 16000 + 4000); // typed এক … চোদ্দ, window moved to বারো
                                                     // the new window hears the end of এগারো again before বারো
        let again = "এগারো বারো তেরো চোদ্দ পনেরো ষোলো";
        assert!(t.partial(&ws(again, 0.5), 3 * 16000).is_empty());
        let more = "এগারো বারো তেরো চোদ্দ পনেরো ষোলো সতেরো";
        assert_eq!(
            t.partial(&ws(more, 0.5), 3 * 16000 + 4000),
            vec!["পনেরো", "ষোলো"]
        );
    }

    #[test]
    fn a_said_twice_word_stays_twice() {
        let mut t = Typist::default();
        t.partial(&ws("কারো কারো হয়তো", 0.4), 20000);
        assert_eq!(
            t.partial(&ws("কারো কারো হয়তো ছোট", 0.4), 24000),
            vec!["কারো", "কারো", "হয়তো"]
        );
        assert_eq!(
            t.finish(&ws("কারো কারো হয়তো ছোট ভাই", 0.4), 30000),
            vec!["ছোট", "ভাই"]
        );
    }

    #[test]
    fn model_marks_dropped() {
        let w = super::without_marks(ws("নতুন লাইন। একটি মেয়ে-- । এহন, ঠিক আছে? দাড়ি-দাঁড়িয়ে", 0.3));
        let t: Vec<&str> = w.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            t,
            [
                "নতুন",
                "লাইন",
                "একটি",
                "মেয়ে",
                "এহন",
                "ঠিক",
                "আছে",
                "দাড়ি-দাঁড়িয়ে"
            ]
        );
    }

    #[test]
    fn shorter_reading_is_safe() {
        let mut t = Typist::default();
        let long = "এক দুই তিন চার পাঁচ ছয় সাত আট নয় দশ";
        t.partial(&ws(long, 0.3), 3 * 16000);
        t.partial(&ws(long, 0.3), 3 * 16000 + 4000);
        // the model changes its mind and hears fewer words in a longer window
        assert!(t.partial(&ws("এক দুই", 0.3), 7 * 16000).is_empty());
        assert!(t.partial(&ws("এক", 0.3), 17 * 16000).is_empty());
    }

    #[test]
    fn window_moves_up() {
        let mut t = Typist::default();
        let long = "এক দুই তিন চার পাঁচ ছয় সাত আট নয় দশ এগারো বারো তেরো চোদ্দ পনেরো";
        t.partial(&ws(long, 0.5), 7 * 16000);
        let typed = t.partial(&ws(long, 0.5), 7 * 16000 + 4000);
        assert_eq!(typed.len(), 14);
        // window is 7.25 s: the start moves to the first word in the last 2 s (at 5.5 s)
        assert_eq!(t.start, ((5.5f32 - 0.12) * 16000.0) as usize);
        // the next readings start there: their words are not typed again
        let rest = "বারো তেরো চোদ্দ পনেরো ষোলো";
        assert!(t.partial(&ws(rest, 0.5), 3 * 16000).is_empty());
        assert_eq!(
            t.partial(&ws("বারো তেরো চোদ্দ পনেরো ষোলো সতেরো", 0.5), 3 * 16000 + 4000),
            vec!["পনেরো", "ষোলো"]
        );
    }
}
