//! Audio helpers: mono 16 kHz conversion and a simple speech/silence detector.

/// Mix to mono and resample to 16 kHz (linear interpolation after a short averaging filter, plenty
/// for speech recognition).
pub fn to_16k_mono(samples: &[f32], channels: usize, rate: u32) -> Vec<f32> {
    let ch = channels.max(1);
    let mono: Vec<f32> = samples
        .chunks(ch)
        .map(|f| f.iter().sum::<f32>() / f.len() as f32)
        .collect();
    if rate == 16_000 {
        return mono;
    }
    let ratio = rate as f64 / 16_000.0;
    // average over the step so higher frequencies don't fold back (a simple low-pass)
    let win = ratio.round().max(1.0) as usize;
    let smooth: Vec<f32> = if win > 1 {
        let mut acc = 0.0f32;
        let mut out = Vec::with_capacity(mono.len());
        for i in 0..mono.len() {
            acc += mono[i];
            if i >= win {
                acc -= mono[i - win];
            }
            out.push(acc / win.min(i + 1) as f32);
        }
        out
    } else {
        mono
    };
    let n = (smooth.len() as f64 / ratio) as usize;
    (0..n)
        .map(|i| {
            let x = i as f64 * ratio;
            let j = x as usize;
            let f = (x - j as f64) as f32;
            let a = smooth[j.min(smooth.len() - 1)];
            let b = smooth[(j + 1).min(smooth.len() - 1)];
            a + (b - a) * f
        })
        .collect()
}

/// Notices speech and the silence after it, 30 ms at a time. The threshold follows the room's own
/// noise level, so a fan or traffic doesn't count as speech.
#[derive(Debug, Clone)]
pub struct Endpoint {
    noise: f32,
    speech_ms: u32,
    silence_ms: u32,
    /// Silence that ends the dictation once speech was heard
    pub stop_after_ms: u32,
    /// Give up if nothing is said for this long
    pub give_up_ms: u32,
    waited_ms: u32,
    /// Speech heard since the last piece was taken (for typing piece by piece)
    pub piece_speech_ms: u32,
    /// The last frame was speech
    pub last_loud: bool,
    /// Loudness of the last few seconds of frames
    recent: std::collections::VecDeque<f32>,
}

/// About five seconds of 30 ms frames
const RECENT_FRAMES: usize = 170;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointState {
    Waiting,
    Speaking,
    Done,
    NothingSaid,
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            noise: 0.004,
            speech_ms: 0,
            silence_ms: 0,
            stop_after_ms: 1400,
            give_up_ms: 8000,
            waited_ms: 0,
            piece_speech_ms: 0,
            last_loud: false,
            recent: std::collections::VecDeque::new(),
        }
    }
}

impl Endpoint {
    /// Feed one frame of 16 kHz audio (any length; 30 ms = 480 samples is typical)
    pub fn push(&mut self, frame: &[f32]) -> EndpointState {
        if frame.is_empty() {
            return self.state();
        }
        let ms = (frame.len() as u32 * 1000) / 16_000;
        let rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();
        let loud = rms > (self.noise * 3.0).max(0.01);
        self.last_loud = loud;
        // The background level is the quietest sound of the last few seconds (the gaps between
        // words), so a steady fan or traffic is learned even though it never goes quiet, and soft
        // speech cannot raise it step by step.
        self.recent.push_back(rms);
        if self.recent.len() > RECENT_FRAMES {
            self.recent.pop_front();
        }
        if self.recent.len() >= 10 {
            let mut v: Vec<f32> = self.recent.iter().copied().collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            self.noise = v[v.len() / 10].clamp(0.0005, 0.05);
        }
        if loud {
            self.piece_speech_ms += ms;
            self.speech_ms += ms;
            self.silence_ms = 0;
        } else if self.speech_ms >= 200 {
            self.silence_ms += ms;
        } else {
            self.waited_ms += ms;
        }
        self.state()
    }

    /// Length of the current pause after speech (milliseconds)
    pub fn pause_ms(&self) -> u32 {
        self.silence_ms
    }

    pub fn state(&self) -> EndpointState {
        if self.speech_ms >= 200 && self.silence_ms >= self.stop_after_ms {
            EndpointState::Done
        } else if self.speech_ms >= 200 {
            EndpointState::Speaking
        } else if self.waited_ms >= self.give_up_ms {
            EndpointState::NothingSaid
        } else {
            EndpointState::Waiting
        }
    }
}

/// Cuts live speech into pieces that can be recognized and typed while the person goes on, always
/// in a gap between words: a pause of `piece_pause_ms`; or, when someone speaks on and on, the
/// first short gap (150 ms) after 5 s; or by 9 s the longest gap seen since 3 s. (Longer pieces made
/// the text come late.) (Cutting at any slightly
/// quieter moment split words in two, and the model guessed odd letters at both ends.)
#[derive(Debug, Clone, Default)]
pub struct Cutter {
    pub endpoint: Endpoint,
    /// All audio so far, 16 kHz mono
    pub pcm: Vec<f32>,
    /// A pause this long after a second of speech ends a piece (0 = no pieces)
    pub piece_pause_ms: u32,
    last_cut: usize,
    taken: usize,
    ready: std::collections::VecDeque<usize>,
    speech_ms: u32,
    quiet_frames: usize,
    quiet_start: usize,
    /// the longest gap since the piece got long: (middle, frames)
    best_gap: Option<(usize, usize)>,
}

const FRAME_MS: usize = 30;

impl Cutter {
    /// Feed one 30 ms frame of 16 kHz audio
    pub fn push(&mut self, frame: &[f32]) -> EndpointState {
        let st = self.endpoint.push(frame);
        let start = self.pcm.len();
        self.pcm.extend_from_slice(frame);
        if self.endpoint.last_loud {
            self.quiet_frames = 0;
            self.speech_ms += (frame.len() * 1000 / 16_000) as u32;
        } else {
            if self.quiet_frames == 0 {
                self.quiet_start = start;
            }
            self.quiet_frames += 1;
        }
        if self.piece_pause_ms == 0 || self.speech_ms < 1000 {
            return st;
        }
        let len = self.pcm.len() - self.last_cut;
        let gap_mid = self.quiet_start + (self.pcm.len() - self.quiet_start) / 2;
        if self.quiet_frames * FRAME_MS >= self.piece_pause_ms as usize {
            // a real pause: the piece ends here (the pause goes with it)
            self.cut(self.pcm.len());
        } else if len > 6 * 16_000 {
            if self.quiet_frames >= 3
                && self
                    .best_gap
                    .map(|(_, n)| self.quiet_frames > n)
                    .unwrap_or(true)
            {
                self.best_gap = Some((gap_mid, self.quiet_frames));
            }
            if len > 8 * 16_000 && self.quiet_frames >= 6 {
                self.cut(gap_mid); // a short gap between words (180 ms)
            } else if len > 12 * 16_000 {
                let at = self.best_gap.map(|(m, _)| m).unwrap_or(self.pcm.len());
                self.cut(at);
            }
        }
        st
    }

    fn cut(&mut self, at: usize) {
        let at = at.clamp(self.last_cut, self.pcm.len());
        self.ready.push_back(at);
        self.last_cut = at;
        self.speech_ms = 0;
        self.best_gap = None;
        // speech after the cut (when cutting inside a gap) belongs to the next piece
        self.quiet_frames = 0;
    }

    /// A finished piece, if one is ready
    pub fn take_piece(&mut self) -> Option<Vec<f32>> {
        let end = self.ready.pop_front()?;
        let start = self.taken;
        self.taken = end;
        Some(self.pcm[start..end].to_vec())
    }

    /// Speech heard since the last cut (milliseconds)
    pub fn rest_speech_ms(&self) -> u32 {
        self.speech_ms
    }

    /// Everything not yet handed out as a piece
    pub fn rest(&mut self) -> Vec<f32> {
        let start = self.taken.min(self.pcm.len());
        self.taken = self.pcm.len();
        self.ready.clear();
        self.pcm[start..].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_48k_stereo() {
        let s: Vec<f32> = (0..48_000 * 2)
            .map(|i| ((i / 2) as f32 / 48.0).sin() * 0.5)
            .collect();
        let o = to_16k_mono(&s, 2, 48_000);
        assert!((o.len() as i32 - 16_000).abs() <= 1);
        assert!(o.iter().all(|x| x.abs() <= 0.51));
    }

    #[test]
    fn endpoint_speech_then_silence() {
        let mut e = Endpoint::default();
        let quiet = vec![0.001f32; 480];
        let loud: Vec<f32> = (0..480).map(|i| ((i as f32) * 0.3).sin() * 0.2).collect();
        for _ in 0..20 {
            assert_eq!(e.push(&quiet), EndpointState::Waiting);
        }
        for _ in 0..30 {
            e.push(&loud);
        }
        assert_eq!(e.state(), EndpointState::Speaking);
        let mut st = EndpointState::Speaking;
        for _ in 0..60 {
            st = e.push(&quiet);
        }
        assert_eq!(st, EndpointState::Done);
    }

    #[test]
    fn long_dictation_keeps_hearing_speech() {
        // ten minutes of speech with soft parts and short pauses, in a quiet room
        let mut e = Endpoint {
            stop_after_ms: 45_000,
            ..Default::default()
        };
        let tone =
            |amp: f32| -> Vec<f32> { (0..480).map(|i| ((i as f32) * 0.3).sin() * amp).collect() };
        let (normal, soft, room) = (tone(0.07), tone(0.025), tone(0.003));
        for _ in 0..(10 * 60 * 1000 / 30 / 40) {
            for k in 0..40 {
                let f = match k % 10 {
                    0..=5 => &normal,
                    6..=7 => &soft,
                    _ => &room,
                };
                e.push(f);
            }
        }
        e.push(&normal);
        assert!(
            e.last_loud,
            "normal speech must still count after ten minutes (noise {})",
            e.noise
        );
        assert!(e.noise < 0.01);
    }

    #[test]
    fn steady_fan_is_background_not_speech() {
        let mut e = Endpoint {
            stop_after_ms: 45_000,
            ..Default::default()
        };
        let tone = |amp: f32, f: f32| -> Vec<f32> {
            (0..480).map(|i| ((i as f32) * f).sin() * amp).collect()
        };
        let fan = tone(0.02, 0.05); // louder than the default speech threshold
        let speech = tone(0.12, 0.3);
        for _ in 0..100 {
            e.push(&fan);
        }
        for _ in 0..50 {
            e.push(&speech);
        }
        // a 600 ms pause with only the fan: must count as a pause
        for _ in 0..20 {
            e.push(&fan);
        }
        assert!(
            !e.last_loud,
            "the fan alone is not speech (noise {})",
            e.noise
        );
        assert!(e.pause_ms() >= 500);
        e.push(&speech);
        assert!(e.last_loud);
    }

    #[test]
    fn cuts_only_in_gaps_between_words() {
        let tone =
            |amp: f32| -> Vec<f32> { (0..480).map(|i| ((i as f32) * 0.3).sin() * amp).collect() };
        let (word, soft, room) = (tone(0.1), tone(0.02), tone(0.002));
        let mut c = Cutter {
            piece_pause_ms: 500,
            ..Default::default()
        };
        for _ in 0..30 {
            c.push(&room);
        }
        // 20 s of talking without a real pause: words of 450 ms with soft consonants inside,
        // and a 210 ms gap between words
        let mut word_frames = vec![];
        for w in 0..44 {
            for k in 0..15 {
                let f = if k == 7 { &soft } else { &word };
                c.push(f);
                word_frames.push((c.pcm.len() - 480, c.pcm.len()));
            }
            for _ in 0..7 {
                c.push(&room);
            }
            let _ = w;
        }
        let mut cuts = vec![];
        let mut pos = 0;
        while let Some(p) = c.take_piece() {
            pos += p.len();
            cuts.push(pos);
        }
        assert!(cuts.len() >= 2, "long speech is cut into pieces: {cuts:?}");
        for at in &cuts {
            // never inside a word
            assert!(
                !word_frames.iter().any(|(a, b)| at > a && at < b),
                "cut at {at} is inside a word"
            );
            assert!(
                *at - cuts.iter().filter(|x| *x < at).max().copied().unwrap_or(0) <= 13 * 16_000
            );
        }
    }

    #[test]
    fn endpoint_gives_up() {
        let mut e = Endpoint {
            give_up_ms: 600,
            ..Default::default()
        };
        let quiet = vec![0.0f32; 480];
        let mut st = EndpointState::Waiting;
        for _ in 0..25 {
            st = e.push(&quiet);
        }
        assert_eq!(st, EndpointState::NothingSaid);
    }
}
