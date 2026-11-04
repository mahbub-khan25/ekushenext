//! Microphone recording (cpal: WASAPI on Windows, Core Audio on macOS, ALSA/PulseAudio/PipeWire on
//! Linux, AAudio on Android).
//!
//! The audio callback only hands the samples over (no locks, no work), so the sound card's buffer
//! never overflows while the computer is busy recognizing; a worker thread converts them to 16 kHz,
//! watches for pauses and cuts the speech into pieces that can be typed while the person goes on.

use crate::audio::to_16k_mono;
use crate::VoiceError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::mpsc;
use std::sync::Arc;

#[derive(Default)]
struct Shared {
    pcm: Vec<f32>, // the audio kept so far, 16 kHz mono
    level: f32,    // loudness of the last bit, 0..1, for the mic bubble
    /// samples already let go from the start (long dictations keep only what is still needed)
    dropped: usize,
}

fn shared_glitches() -> Arc<std::sync::atomic::AtomicU64> {
    Arc::new(std::sync::atomic::AtomicU64::new(0))
}

/// Records from the default microphone until stopped.
pub struct Recorder {
    stream: Option<cpal::Stream>,
    shared: Arc<Mutex<Shared>>,
    /// a test recording playing instead of the microphone
    fake: Option<Arc<std::sync::atomic::AtomicBool>>,
}

impl Recorder {
    /// Start recording from the default input device
    pub fn start() -> Result<Self, VoiceError> {
        let host = cpal::default_host();
        // the default recording device, or else the first one Windows lists (a headset can be
        // connected without being the default)
        let dev = match host.default_input_device() {
            Some(d) => d,
            None => {
                let mut all = host
                    .input_devices()
                    .map_err(|e| VoiceError::Mic(e.to_string()))?;
                match all.next() {
                    Some(d) => d,
                    None => {
                        return Err(VoiceError::Mic(
                            "no microphone found: is the headset connected and switched on?".into(),
                        ))
                    }
                }
            }
        };
        let name = dev
            .description()
            .map(|d| format!("{d:?}"))
            .unwrap_or_default();
        let cfg = dev
            .default_input_config()
            .map_err(|e| VoiceError::Mic(e.to_string()))?;
        let channels = cfg.channels() as usize;
        let rate: u32 = cfg.sample_rate();
        eprintln!("microphone: {name} ({rate} Hz, {channels} channel(s))");
        let shared = Arc::new(Mutex::new(Shared::default()));

        // the worker: device samples → 16 kHz mono in 30 ms blocks → endpoint and pieces
        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let sh = shared.clone();
        std::thread::spawn(move || {
            let block = (rate as usize * channels * 30) / 1000;
            let mut pending: Vec<f32> = Vec::new();
            while let Ok(data) = rx.recv() {
                pending.extend_from_slice(&data);
                while pending.len() >= block {
                    let chunk: Vec<f32> = pending.drain(..block).collect();
                    let m = to_16k_mono(&chunk, channels, rate);
                    let rms = (m.iter().map(|x| x * x).sum::<f32>() / m.len().max(1) as f32).sqrt();
                    let mut s = sh.lock();
                    s.level = (rms * 8.0).min(1.0);
                    s.pcm.extend_from_slice(&m);
                }
            }
        });

        // the sound system reports glitches (some microphones many times a second): count them
        let glitches = shared_glitches();
        let err = move |e: cpal::Error| {
            let n = glitches.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if n == 0 || n.is_power_of_two() {
                eprintln!("microphone: {e} ({} so far)", n + 1);
            }
        };
        let sc: cpal::StreamConfig = cfg.into();
        let stream = match cfg.sample_format() {
            cpal::SampleFormat::F32 => {
                let tx = tx.clone();
                dev.build_input_stream(
                    sc,
                    move |d: &[f32], _: &_| {
                        let _ = tx.send(d.to_vec());
                    },
                    err,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let tx = tx.clone();
                dev.build_input_stream(
                    sc,
                    move |d: &[i16], _: &_| {
                        let _ = tx.send(d.iter().map(|x| *x as f32 / 32768.0).collect());
                    },
                    err,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let tx = tx.clone();
                dev.build_input_stream(
                    sc,
                    move |d: &[u16], _: &_| {
                        let _ =
                            tx.send(d.iter().map(|x| (*x as f32 - 32768.0) / 32768.0).collect());
                    },
                    err,
                    None,
                )
            }
            other => {
                return Err(VoiceError::Mic(format!(
                    "unsupported sample format {other:?}"
                )))
            }
        }
        .map_err(|e| VoiceError::Mic(e.to_string()))?;
        drop(tx);
        stream.play().map_err(|e| VoiceError::Mic(e.to_string()))?;
        Ok(Self {
            stream: Some(stream),
            shared,
            fake: None,
        })
    }

    /// For tests: play 16 kHz mono samples as if spoken into the microphone, in real time
    pub fn from_samples(samples: Vec<f32>) -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (sh, st) = (shared.clone(), stop.clone());
        std::thread::spawn(move || {
            let t0 = std::time::Instant::now();
            for (i, chunk) in samples.chunks(480).enumerate() {
                if st.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let due = std::time::Duration::from_millis(i as u64 * 30);
                if let Some(wait) = due.checked_sub(t0.elapsed()) {
                    std::thread::sleep(wait);
                }
                let rms = (chunk.iter().map(|x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
                let mut s = sh.lock();
                s.level = (rms * 8.0).min(1.0);
                s.pcm.extend_from_slice(chunk);
            }
        });
        Self {
            stream: None,
            shared,
            fake: Some(stop),
        }
    }

    /// Loudness right now (0..1), for the mic bubble
    pub fn level(&self) -> f32 {
        self.shared.lock().level
    }

    /// Samples recorded so far (16 kHz)
    pub fn len(&self) -> usize {
        let s = self.shared.lock();
        s.dropped + s.pcm.len()
    }

    /// Let go of the audio before position `pos` (it will not be asked for again), so an
    /// hours-long dictation does not keep hundreds of megabytes of sound in memory
    pub fn forget_before(&self, pos: usize) {
        let mut s = self.shared.lock();
        let n = pos.saturating_sub(s.dropped).min(s.pcm.len());
        if n > 0 {
            s.pcm.drain(..n);
            s.dropped += n;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The audio from `from` on (16 kHz mono)
    pub fn audio(&self, from: usize) -> Vec<f32> {
        let s = self.shared.lock();
        let i = from.saturating_sub(s.dropped).min(s.pcm.len());
        s.pcm[i..].to_vec()
    }

    /// Stop recording; returns everything recorded (16 kHz mono)
    pub fn stop(self) -> Vec<f32> {
        if let Some(st) = &self.stream {
            let _ = st.pause();
        }
        if let Some(f) = &self.fake {
            f.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        drop(self.stream);
        // let the worker take in the last samples
        std::thread::sleep(std::time::Duration::from_millis(60));
        std::mem::take(&mut self.shared.lock().pcm)
    }

    /// Stop recording; returns where the kept audio starts and the kept audio
    pub fn stop_kept(self) -> (usize, Vec<f32>) {
        let dropped = self.shared.lock().dropped;
        (dropped, self.stop())
    }
}
