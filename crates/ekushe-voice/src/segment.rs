//! Cutting live speech into short pieces with a real speech detector (Silero VAD, MIT licence, run by
//! whisper.cpp). The Bangla model was trained on short clips: given 20 seconds it writes the first
//! sentence and stops, so each piece must be about one sentence (here at most ~9 s), ending in a
//! pause. The energy-based detector used before mistook steady speech for background noise on quiet
//! Bluetooth microphones and cut nothing at all.

use std::path::Path;
use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

/// The speech detector model, built into the program (885 KB)
pub const VAD_MODEL: &[u8] = include_bytes!("../assets/ggml-silero-v5.1.2.bin");

/// Write the built-in speech detector to a file whisper.cpp can open (once per version)
pub fn vad_model_file() -> Result<std::path::PathBuf, String> {
    let p = std::env::temp_dir().join("ekushe-silero-v5.1.2.bin");
    if std::fs::metadata(&p)
        .map(|m| m.len() as usize != VAD_MODEL.len())
        .unwrap_or(true)
    {
        std::fs::write(&p, VAD_MODEL).map_err(|e| e.to_string())?;
    }
    Ok(p)
}

/// Splits the audio that has not been recognized yet
pub struct Segmenter {
    vad: WhisperVadContext,
    /// A pause this long after speech ends a piece (milliseconds)
    pub pause_ms: f32,
    /// Pieces are cut by this length at the latest (seconds), at a pause if there is one
    pub max_piece_secs: f32,
}

/// What to do with the audio not yet recognized
#[derive(Debug, Clone, PartialEq)]
pub enum Cut {
    /// Not yet: keep listening
    Wait,
    /// Recognize `start..end` (speech with a little silence around it); the audio before `end` is done
    Piece { start: usize, end: usize },
    /// Only silence up to `end`: drop it
    Silence { end: usize },
}

const SR: f32 = 16_000.0;

impl Segmenter {
    pub fn new(model: &Path, threads: usize) -> Result<Self, String> {
        let mut p = WhisperVadContextParams::default();
        // one thread: the recognizer needs the processor more (the detector is tiny)
        let _ = threads;
        p.set_n_threads(1);
        p.set_use_gpu(false);
        let vad = WhisperVadContext::new(&model.to_string_lossy(), p).map_err(|e| e.to_string())?;
        Ok(Self {
            vad,
            pause_ms: 350.0,
            max_piece_secs: 9.0,
        })
    }

    /// Speech segments in `audio` as (start, end) sample positions
    fn segments(&mut self, audio: &[f32]) -> Vec<(usize, usize)> {
        let mut vp = WhisperVadParams::default();
        vp.set_threshold(0.5);
        vp.set_min_speech_duration(200);
        vp.set_min_silence_duration(120);
        vp.set_max_speech_duration(self.max_piece_secs - 1.0);
        vp.set_speech_pad(60);
        vp.set_samples_overlap(0.0);
        match self.vad.segments_from_samples(vp, audio) {
            Ok(segs) => segs
                .map(|s| {
                    let a = ((s.start / 100.0) * SR) as usize;
                    let b = ((s.end / 100.0) * SR) as usize;
                    (a.min(audio.len()), b.min(audio.len()))
                })
                .filter(|(a, b)| b > a)
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Whether there is any speech in `audio`
    pub fn has_speech(&mut self, audio: &[f32]) -> bool {
        audio.len() >= (0.3 * SR) as usize && !self.segments(audio).is_empty()
    }

    /// Decide about `audio`, everything heard since the last piece; `done` = the dictation ended.
    /// Only the first `max_piece_secs` + 3 s are ever examined, so a check costs the same however
    /// far behind the recognizer is. After `done`, call again until the audio is used up: long
    /// leftovers come out as sentence-sized pieces too.
    pub fn cut(&mut self, audio: &[f32], done: bool) -> Cut {
        let len = audio.len();
        if len == 0 || (len < (0.4 * SR) as usize && !done) {
            return Cut::Wait;
        }
        let max = (self.max_piece_secs * SR) as usize;
        let window = len.min(max + (3.0 * SR) as usize);
        let segs = self.segments(&audio[..window]);
        let pad = (0.15 * SR) as usize;
        if segs.is_empty() {
            // nothing said: let go of old silence, keep the last second (speech may be starting)
            return if done {
                if window == len {
                    Cut::Silence { end: len }
                } else {
                    Cut::Silence { end: window }
                }
            } else if len > (3.0 * SR) as usize {
                Cut::Silence {
                    end: window.min(len - SR as usize),
                }
            } else {
                Cut::Wait
            };
        }
        let start = segs[0].0.saturating_sub(pad);
        if done && window == len && len - start <= max + (2.0 * SR) as usize {
            return Cut::Piece { start, end: len };
        }
        let pause = (self.pause_ms / 1000.0 * SR) as usize;
        // segments followed by a long enough pause are finished
        let finished: Vec<&(usize, usize)> = segs
            .iter()
            .filter(|(_, b)| b + pause <= window || (done && window == len))
            .collect();
        if let Some(&&(_, last_end)) = finished.last() {
            // cut after the last finished one that keeps the piece within the limit
            let within = finished
                .iter()
                .rev()
                .find(|(_, b)| b - start <= max)
                .map(|s| s.1);
            let end = within.unwrap_or(last_end);
            if end - start <= max + (2.0 * SR) as usize {
                return Cut::Piece {
                    start,
                    end: (end + pad).min(len),
                };
            }
        }
        if window - start > max || window < len {
            // talking on and on without a pause: cut between two segments, or at the limit
            let gap = segs
                .windows(2)
                .map(|w| w[0].1)
                .rfind(|e| *e > start && e - start <= max);
            let end = gap.unwrap_or(start + max).min(len);
            return Cut::Piece { start, end };
        }
        Cut::Wait
    }
}
