//! Offline Bangla voice typing for Ekushe Next.
//!
//! - [`Recognizer`]: a Bangla speech model (whisper.cpp format) turns 16 kHz mono audio into text,
//!   entirely on this computer or phone.
//! - [`Recorder`] (feature `mic`): records from the default microphone and notices when the person
//!   has stopped speaking.
//! - The heard text is tidied with [`ekushe_core::voice::tidy`] (spoken punctuation, numbers, the
//!   person's own corrections) before it is typed.

pub mod audio;
#[cfg(feature = "live")]
pub mod live;
#[cfg(feature = "mic")]
pub mod mic;
#[cfg(feature = "whisper")]
pub mod segment;
#[cfg(feature = "mic")]
pub use mic::Recorder;

#[cfg(feature = "whisper")]
pub mod whisper;
#[cfg(feature = "whisper")]
pub use whisper::{dictate, Recognizer};

#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("the speech model could not be loaded: {0}")]
    Model(String),
    #[error("speech recognition failed: {0}")]
    Recognize(String),
    #[error("microphone: {0}")]
    Mic(String),
}

/// How hard the recognizer searches: `Fast` (greedy) or `Best` (beam search, slower, a little better)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Fast,
    Best,
}
