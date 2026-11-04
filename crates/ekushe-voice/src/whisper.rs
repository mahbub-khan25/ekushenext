//! The Whisper path (whisper.cpp): accurate on a fast computer, slow on a weak one.

use crate::{Quality, VoiceError};
use std::path::Path;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

/// A loaded Bangla speech model. Loading takes a few seconds, so keep one around.
pub struct Recognizer {
    ctx: WhisperContext,
    /// Working memory (on the graphics card when there is one), kept from one piece to the next
    state: Option<WhisperState>,
    threads: i32,
    pub quality: Quality,
    /// Keep the working memory from one piece to the next (faster; off = a fresh one each time)
    pub reuse_state: bool,
    /// Decode again at higher temperatures when the first try looks unsure (whisper's fallback)
    pub fallback: bool,
}

impl Recognizer {
    /// Load a whisper.cpp model file (e.g. `bn-medium-q5_0.bin`).
    pub fn load(model: &Path, threads: usize, use_gpu: bool) -> Result<Self, VoiceError> {
        let mut p = WhisperContextParameters::default();
        p.use_gpu(use_gpu);
        // (flash attention stays off: the well-tested setting; it was on while pieces went wrong)
        p.flash_attn(false);
        let ctx = WhisperContext::new_with_params(model, p)
            .map_err(|e| VoiceError::Model(e.to_string()))?;
        Ok(Self {
            ctx,
            state: None,
            threads: threads.max(1) as i32,
            quality: Quality::Fast,
            // A kept working memory corrupted later pieces (77% vs 33% words wrong on our test
            // pieces, on the processor too): a fresh one each time
            reuse_state: false,
            fallback: true,
        })
    }

    /// Recognize Bangla speech in 16 kHz mono samples (-1.0..1.0). Returns the heard text, untidied.
    pub fn transcribe(&mut self, pcm: &[f32]) -> Result<String, VoiceError> {
        if pcm.len() < 16_000 / 4 {
            return Ok(String::new()); // under a quarter second: nothing said
        }
        if !self.reuse_state {
            self.state = None;
        }
        if self.state.is_none() {
            self.state = Some(
                self.ctx
                    .create_state()
                    .map_err(|e| VoiceError::Recognize(e.to_string()))?,
            );
        }
        let strategy = match self.quality {
            Quality::Fast => SamplingStrategy::Greedy { best_of: 1 },
            Quality::Best => SamplingStrategy::BeamSearch {
                beam_size: 5,
                patience: -1.0,
            },
        };
        let mut fp = FullParams::new(strategy);
        fp.set_language(Some("bn"));
        fp.set_translate(false);
        fp.set_no_timestamps(true); // the Bangla model was trained without them; with them it cuts the first word
        fp.set_n_threads(self.threads);
        fp.set_print_progress(false);
        fp.set_print_realtime(false);
        fp.set_print_special(false);
        fp.set_print_timestamps(false);
        fp.set_suppress_blank(true);
        fp.set_no_context(true);
        // the encoder only looks at as much audio as was said (much faster for short phrases)
        let secs = pcm.len() as f32 / 16_000.0;
        if secs < 29.0 {
            fp.set_audio_ctx(audio_ctx(secs));
        }
        // Bangla speech is about 27 tokens a second (at most ~50 in our tests); a model stuck
        // repeating itself stops there instead of running to the end of its window
        fp.set_max_tokens(max_tokens(secs));
        // no re-decoding at higher temperatures: on a stuck piece that multiplied the time by five
        // (8 s of speech took 12-16 s and everything said afterwards waited)
        // Retry an unsure piece at most twice (temperatures 0.4, 0.8); with the token cap above a
        // stuck piece can no longer take many seconds
        fp.set_temperature_inc(if self.fallback { 0.4 } else { 0.0 });
        let st = self.state.as_mut().expect("state was just created");
        st.full(fp, pcm)
            .map_err(|e| VoiceError::Recognize(e.to_string()))?;
        let n = st.full_n_segments();
        let mut text = String::new();
        for i in 0..n {
            if let Some(seg) = st.get_segment(i) {
                if let Ok(t) = seg.to_str_lossy() {
                    text.push_str(t.trim());
                    text.push(' ');
                }
            }
        }
        Ok(text.trim().to_string())
    }
}

/// Encoder frames for `secs` of audio (1500 = 30 s), with room to spare; at least 256
fn audio_ctx(secs: f32) -> i32 {
    ((secs * 50.0) as i32 + 128).clamp(256, 1500)
}

/// Most tokens to decode for `secs` of speech
fn max_tokens(secs: f32) -> i32 {
    ((secs * 55.0) as i32 + 30).clamp(64, 440)
}

/// Recognize and tidy in one step, as typed into an app
pub fn dictate(
    rec: &mut Recognizer,
    pcm: &[f32],
    opts: &ekushe_core::voice::VoiceOptions,
    learned: &std::collections::HashMap<String, String>,
) -> Result<String, VoiceError> {
    let heard = rec.transcribe(pcm)?;
    Ok(ekushe_core::voice::tidy(&heard, opts, learned))
}

#[cfg(test)]
mod tests {
    #[test]
    fn audio_ctx_grows_with_length() {
        assert_eq!(super::audio_ctx(1.0), 256);
        assert_eq!(super::audio_ctx(6.0), 428);
        assert_eq!(super::audio_ctx(40.0), 1500);
    }

    #[test]
    fn token_cap() {
        assert_eq!(super::max_tokens(0.5), 64);
        assert_eq!(super::max_tokens(5.0), 305);
        assert_eq!(super::max_tokens(30.0), 440);
    }
}
