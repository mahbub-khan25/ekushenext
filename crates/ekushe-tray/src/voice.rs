//! Voice typing on Windows: runs the `ekushe-voice.exe` helper (next to ekushe.exe), which keeps the
//! Bangla speech model loaded, records from the microphone and returns the text. The tray thread is
//! told about each event with a window message and types the text into the app in front.
//!
//! Files: the helper ships with Ekushe; a speech model (large, small or live, see [`MODELS`]) is downloaded
//! once into `%APPDATA%\Ekushe\voice\`, and can be removed again from Studio. What the person
//! corrected after dictating is in `voice-learned.tsv` there.

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// A Bangla speech model Ekushe can download (whisper.cpp q5_0, from Ekushe's own releases)
pub struct Model {
    /// "large", "small" or "live" (the `voice_model` setting)
    pub id: &'static str,
    /// The live model (Zipformer): typed while the person speaks, run by its own helper
    pub live: bool,
    pub file: &'static str,
    pub url: &'static str,
    /// Size of the file in bytes (checked after download)
    pub bytes: u64,
}

/// Live (the default): a Bangla Zipformer (CC BY-SA 4.0), typed while the person speaks, fast on any
/// computer, less accurate. Large: Bengali.AI Whisper-medium (Apache-2.0), more accurate, needs a
/// strong graphics card to keep up. Small: a Whisper-small Bangla fine-tune (not released yet).
pub const MODELS: [Model; 3] = [
    Model {
        id: "live",
        live: true,
        file: "bn-live-1.ekm",
        url: "https://github.com/mahbub-khan25/ekushenext/releases/download/voice-model-live-1/bn-live-1.ekm",
        bytes: 73_635_876,
    },
    Model {
        id: "large",
        live: false,
        file: "bn-medium-q5_0.bin",
        url: "https://github.com/mahbub-khan25/ekushenext/releases/download/voice-model-1/bn-medium-q5_0.bin",
        bytes: 539_257_671,
    },
    Model {
        id: "small",
        live: false,
        file: SMALL_FILE,
        url: SMALL_URL,
        bytes: SMALL_BYTES,
    },
];
const SMALL_FILE: &str = "bn-small-q5_0.bin";
const SMALL_URL: &str =
    "https://github.com/mahbub-khan25/ekushenext/releases/download/voice-model-small-1/bn-small-q5_0.bin";
const SMALL_BYTES: u64 = 0; // set once the small model release is built

pub fn model(id: &str) -> &'static Model {
    MODELS.iter().find(|m| m.id == id).unwrap_or(&MODELS[0])
}

/// The helper program for this version, signed, from this release's files (when it isn't installed
/// next to ekushe.exe)
fn helper_url(name: &str) -> String {
    format!(
        "https://github.com/mahbub-khan25/ekushenext/releases/download/{}/{name}",
        crate::update_check::current_version()
    )
}

/// The helper for the graphics card (Vulkan) and the one for the processor only. The GPU one is
/// tried first; if it cannot start on this PC (no Vulkan driver) Ekushe remembers that and uses the
/// other from then on.
const GPU_EXE: &str = "ekushe-voice-gpu.exe";
const CPU_EXE: &str = "ekushe-voice.exe";
/// The helper for the live model (processor only; it is fast enough everywhere)
const LIVE_EXE: &str = "ekushe-voice-live.exe";

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Ready,
    Listening,
    Level(f32),
    Thinking,
    /// Heard but not typed yet (the live model), for the banner
    Live(String),
    /// `more`: a piece typed while the person goes on speaking
    Text {
        heard: String,
        text: String,
        more: bool,
        /// seconds of speech and seconds it took to recognize
        secs: f32,
        took: f32,
    },
    /// Finished; everything was already typed piece by piece
    Done,
    Nothing,
    Error(String),
    Exited,
}

struct Helper {
    child: Child,
    stdin: ChildStdin,
}

static HELPER: Mutex<Option<Helper>> = Mutex::new(None);
/// The newest microphone level (for redrawing the banner between level events)
pub static LAST_LEVEL: Mutex<Option<f32>> = Mutex::new(None);
static EVENTS: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());
/// Dictation in progress (from `listen` until text, nothing or an error)
pub static LISTENING: AtomicBool = AtomicBool::new(false);
/// A dictation is waiting for its text (from `listen` until text, nothing or an error)
static WAITING: AtomicBool = AtomicBool::new(false);
/// The `voice_use_gpu` setting
pub static USE_GPU: AtomicBool = AtomicBool::new(true);
/// Text came back in the dictation in progress
static GOT_TEXT: AtomicBool = AtomicBool::new(false);
/// Model download: bytes so far (u64::MAX = failed); `DOWNLOADING` while it runs
pub static DOWNLOADED: AtomicU64 = AtomicU64::new(0);
pub static DOWNLOADING: AtomicBool = AtomicBool::new(false);

pub fn voice_dir() -> PathBuf {
    ekushe_core::config::Config::voice_dir()
}

pub fn model_path(id: &str) -> PathBuf {
    if model(id).live && custom_live().exists() {
        return custom_live();
    }
    voice_dir().join(model(id).file)
}

/// A live model the person trained themselves (`voice\live-custom.ekm`, for testing), used instead of
/// the downloaded one while it is there
pub fn custom_live() -> PathBuf {
    voice_dir().join("live-custom.ekm")
}

pub fn learned_path() -> PathBuf {
    voice_dir().join("voice-learned.tsv")
}

fn find_exe(name: &str) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let next = exe.parent()?.join(name);
    if next.exists() {
        return Some(next);
    }
    let inv = voice_dir().join(name);
    inv.exists().then_some(inv)
}

fn gpu_off_marker() -> PathBuf {
    voice_dir().join("gpu-off")
}

/// The helper program (next to ekushe.exe, or in the voice folder): the GPU one unless it failed here
pub fn helper_path() -> Option<PathBuf> {
    if USE_GPU.load(Ordering::SeqCst) && !gpu_off_marker().exists() {
        if let Some(g) = find_exe(GPU_EXE) {
            return Some(g);
        }
    }
    find_exe(CPU_EXE)
}

/// The helper program that runs this model
pub fn helper_for(model_id: &str) -> Option<PathBuf> {
    if model(model_id).live {
        find_exe(LIVE_EXE)
    } else {
        helper_path()
    }
}

/// This model is in place (and has the right size)
pub fn model_ready(id: &str) -> bool {
    let m = model(id);
    if m.live && custom_live().exists() {
        return true;
    }
    m.bytes > 0
        && std::fs::metadata(voice_dir().join(m.file))
            .map(|f| f.len() == m.bytes)
            .unwrap_or(false)
}

/// The model to use: the chosen one, or the other if only that one is downloaded
pub fn active_model(preferred: &str) -> Option<&'static str> {
    let first = model(preferred).id;
    std::iter::once(first)
        .chain(MODELS.iter().map(|m| m.id).filter(move |id| *id != first))
        .find(|id| model_ready(id))
}

/// Delete a downloaded model (the helper is stopped first if it uses it)
pub fn remove_model(id: &str) -> Result<(), String> {
    if RUNNING_MODEL.lock().as_deref() == Some(model(id).id) {
        shutdown();
    }
    let p = model_path(id);
    if p.exists() {
        std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    // the live model's files, unpacked next to it by the helper
    let unpacked = p.with_extension("d");
    if unpacked.is_dir() {
        std::fs::remove_dir_all(&unpacked).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Take the events that arrived since the last call
pub fn take_events() -> Vec<Event> {
    EVENTS.lock().drain(..).collect()
}

fn push(e: Event) {
    EVENTS.lock().push_back(e);
}

/// Options given to the helper when it starts
#[derive(Clone)]
pub struct Options {
    /// The `voice_model` setting ("large" or "small")
    pub model: String,
    pub bangla_digits: bool,
    pub spoken_punctuation: bool,
    pub numbers_as_digits: bool,
    pub best_quality: bool,
    pub stop_after_ms: u32,
    /// End by itself after a pause (otherwise until the shortcut or Esc)
    pub auto_stop: bool,
}

/// How the helper was last started, and what to call when it reports
type Started = (Options, fn());
static LAST_START: Mutex<Option<Started>> = Mutex::new(None);
/// The model the running helper has loaded
static RUNNING_MODEL: Mutex<Option<&'static str>> = Mutex::new(None);
/// The last dictation's speed: (model, seconds of speech, seconds to recognize)
pub static LAST_SPEED: Mutex<Option<(&'static str, f32, f32)>> = Mutex::new(None);

/// Start the helper if it isn't running. `notify` is called (from a reader thread) whenever events
/// are waiting; it should post a message to the tray window.
pub fn ensure_started(opts: &Options, notify: fn()) -> Result<(), String> {
    let use_model = active_model(&opts.model).ok_or("model not downloaded")?;
    let mut h = HELPER.lock();
    if let Some(hp) = h.as_mut() {
        if hp.child.try_wait().ok().flatten().is_none() {
            if *RUNNING_MODEL.lock() == Some(use_model) {
                return Ok(());
            }
            // another model was chosen: start again with that one
            let _ = writeln!(hp.stdin, "quit");
            let _ = hp.stdin.flush();
            let _ = hp.child.kill();
        }
    }
    let exe = helper_for(use_model).ok_or("ekushe-voice.exe not found")?;
    let live = model(use_model).live;
    *LAST_START.lock() = Some((opts.clone(), notify));
    let gpu = exe.file_name().map(|n| n == GPU_EXE).unwrap_or(false);
    let mut cmd = Command::new(&exe);
    if !gpu && !live {
        cmd.arg("--no-gpu");
    }
    // test builds: a file named `log-text` next to ekushe.exe puts what was heard into helper.log
    if std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join("log-text").exists()))
        .unwrap_or(false)
    {
        cmd.env("EKUSHE_VOICE_LOG_TEXT", "1");
    }
    if live {
        // two processor cores: quick enough, and leaves the rest of the computer free
        cmd.arg("--serve")
            .arg("--live-model")
            .arg(model_path(use_model));
        cmd.args(["--threads", "2"]);
    } else {
        cmd.arg("--serve").arg("--model").arg(model_path(use_model));
    }
    *RUNNING_MODEL.lock() = Some(use_model);
    cmd.arg("--learned").arg(learned_path());
    if !opts.bangla_digits {
        cmd.args(["--digits", "en"]);
    }
    if !opts.spoken_punctuation {
        cmd.arg("--no-punctuation");
    }
    if !opts.numbers_as_digits {
        cmd.arg("--no-number-digits");
    }
    if opts.best_quality && !live {
        cmd.args(["--quality", "best"]);
    }
    if opts.auto_stop {
        cmd.args(["--stop-after", &opts.stop_after_ms.to_string()]);
    }
    // whisper.cpp's own messages and timings go to a log next to the model
    let log = std::fs::File::create(voice_dir().join("helper.log"))
        .map(Stdio::from)
        .unwrap_or_else(|_| Stdio::null());
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(log);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let stdin = child.stdin.take().ok_or("no stdin")?;
    let stdout = child.stdout.take().ok_or("no stdout")?;
    std::thread::spawn(move || {
        let mut ready = false;
        for line in BufReader::new(stdout).lines() {
            let Ok(l) = line else { break };
            if let Some(e) = parse_event(&l) {
                match &e {
                    Event::Ready => ready = true,
                    Event::Text {
                        secs, took, more, ..
                    } => {
                        GOT_TEXT.store(true, Ordering::SeqCst);
                        if *secs > 0.0 {
                            *LAST_SPEED.lock() = Some((use_model, *secs, *took));
                        }
                        if !*more {
                            LISTENING.store(false, Ordering::SeqCst);
                            WAITING.store(false, Ordering::SeqCst);
                        }
                    }
                    Event::Done | Event::Nothing | Event::Error(_) => {
                        LISTENING.store(false, Ordering::SeqCst);
                        WAITING.store(false, Ordering::SeqCst);
                    }
                    _ => {}
                }
                push(e);
                notify();
            }
        }
        // the GPU helper could not start, or stopped in the middle: use the processor-only one
        if gpu && (!ready || WAITING.load(Ordering::SeqCst)) && find_exe(CPU_EXE).is_some() {
            let _ = std::fs::write(gpu_off_marker(), "the GPU voice helper did not work here\n");
            HELPER.lock().take();
            let start = LAST_START.lock().clone();
            if let Some((o, n)) = start {
                if ensure_started(&o, n).is_ok()
                    && WAITING.load(Ordering::SeqCst)
                    && !GOT_TEXT.load(Ordering::SeqCst)
                {
                    // nothing was typed yet: listen again with the other helper
                    send("listen");
                    return;
                }
            }
        }
        LISTENING.store(false, Ordering::SeqCst);
        // stopped in the middle of a dictation: say so instead of going quiet
        if WAITING.swap(false, Ordering::SeqCst) {
            push(Event::Error(
                "ভয়েস টাইপিং হঠাৎ বন্ধ হয়ে গেছে, আবার চেষ্টা করুন".into(),
            ));
        }
        push(Event::Exited);
        notify();
    });
    *h = Some(Helper { child, stdin });
    Ok(())
}

fn send(cmd: &str) -> bool {
    let mut h = HELPER.lock();
    match h.as_mut() {
        Some(hp) => writeln!(hp.stdin, "{cmd}")
            .and_then(|_| hp.stdin.flush())
            .is_ok(),
        None => false,
    }
}

/// Voice typing needs AVX2 (Intel since 2013, AMD since 2015); the helper is built for it
pub fn cpu_supported() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        true
    }
}

/// Start a dictation (the helper says `listening`)
pub fn listen() -> bool {
    let ok = send("listen");
    if ok {
        LISTENING.store(true, Ordering::SeqCst);
        WAITING.store(true, Ordering::SeqCst);
        GOT_TEXT.store(false, Ordering::SeqCst);
    }
    ok
}

/// Finish the dictation now (the helper recognizes what was said so far)
pub fn stop() {
    send("stop");
}

pub fn cancel() {
    send("cancel");
    LISTENING.store(false, Ordering::SeqCst);
    WAITING.store(false, Ordering::SeqCst);
}

/// The learned corrections changed on disk
pub fn reload_learned() {
    send("reload-learned");
}

/// Stop the helper (settings changed, or Ekushe is closing)
pub fn shutdown() {
    WAITING.store(false, Ordering::SeqCst);
    *RUNNING_MODEL.lock() = None;
    if let Some(mut hp) = HELPER.lock().take() {
        let _ = writeln!(hp.stdin, "quit");
        let _ = hp.stdin.flush();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = hp.child.kill();
    }
    LISTENING.store(false, Ordering::SeqCst);
}

/// One event line from the helper
pub fn parse_event(line: &str) -> Option<Event> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    Some(match v.get("event")?.as_str()? {
        "ready" => Event::Ready,
        "listening" => Event::Listening,
        "level" => {
            let l = v.get("v").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
            *LAST_LEVEL.lock() = Some(l);
            Event::Level(l)
        }
        "thinking" => Event::Thinking,
        "live" => Event::Live(s("text")),
        "text" => Event::Text {
            heard: s("heard"),
            text: s("text"),
            more: v.get("more").and_then(|x| x.as_bool()).unwrap_or(false),
            secs: v.get("secs").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            took: v.get("took").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
        },
        "done" => Event::Done,
        "nothing" => Event::Nothing,
        "error" => Event::Error(s("message")),
        _ => return None,
    })
}

/// The banner shown while listening: a small level meter
pub fn level_banner(level: f32, hotkey: &str) -> String {
    let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇"];
    let n = ((level.clamp(0.0, 1.0)) * 6.0).round() as usize;
    let meter: String = (0..7).map(|i| if i <= n { bars[i] } else { "·" }).collect();
    // say which model is listening, so a test of the live model can't silently use another one
    let live = RUNNING_MODEL.lock().map(|m| model(m).live).unwrap_or(false);
    let tag = if live { "লাইভ · " } else { "" };
    format!("🔴 {tag}শুনছি, বলুন {meter}  ·  শেষ: {hotkey}  ·  বাতিল: Esc")
}

/// A short soft two-note sound (16 kHz WAV in memory): rising when the microphone opens, falling
/// when it closes
pub fn chime(rising: bool) -> Vec<u8> {
    let rate = 16_000u32;
    let notes: [f32; 2] = if rising {
        [660.0, 990.0]
    } else {
        [990.0, 660.0]
    };
    let per = (rate as f32 * 0.07) as usize;
    let mut pcm: Vec<i16> = Vec::with_capacity(per * 2);
    for (k, f) in notes.iter().enumerate() {
        for i in 0..per {
            let t = (k * per + i) as f32 / rate as f32;
            // gentle fade in and out of each note
            let x = i as f32 / per as f32;
            let env = (x * 10.0).min(1.0) * ((1.0 - x) * 4.0).min(1.0);
            let v = (2.0 * std::f32::consts::PI * f * t).sin() * env * 0.18;
            pcm.push((v * 32767.0) as i16);
        }
    }
    let data_len = (pcm.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + data_len as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data_len).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data_len.to_le_bytes());
    for s in pcm {
        w.extend_from_slice(&s.to_le_bytes());
    }
    w
}

/// Download the model in the background (Windows: WinHTTP through `update_check::download`).
/// Progress is in [`DOWNLOADED`]; `done` runs at the end with the outcome.
pub fn start_download(id: &str, done: impl FnOnce(Result<(), String>) + Send + 'static) {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }
    let m = model(id);
    *DOWNLOADING_MODEL.lock() = Some(m.id);
    DOWNLOADED.store(0, Ordering::SeqCst);
    std::thread::spawn(move || {
        let r = download_model(m);
        if r.is_err() {
            DOWNLOADED.store(u64::MAX, Ordering::SeqCst);
        }
        DOWNLOADING.store(false, Ordering::SeqCst);
        done(r);
    });
}

/// The model being (or last) downloaded
pub static DOWNLOADING_MODEL: Mutex<Option<&'static str>> = Mutex::new(None);

fn download_model(m: &'static Model) -> Result<(), String> {
    if m.bytes == 0 {
        return Err("this model is not available yet".into());
    }
    std::fs::create_dir_all(voice_dir()).map_err(|e| e.to_string())?;
    if m.live {
        if find_exe(LIVE_EXE).is_none() {
            let part = voice_dir().join("ekushe-voice-live.exe.part");
            crate::update_check::download(&helper_url("ekushe-voice-live-x64.exe"), &part, |_| {})?;
            std::fs::rename(&part, voice_dir().join(LIVE_EXE)).map_err(|e| e.to_string())?;
        }
    } else if find_exe(CPU_EXE).is_none() {
        let part = voice_dir().join("ekushe-voice.exe.part");
        crate::update_check::download(&helper_url("ekushe-voice-x64.exe"), &part, |_| {})?;
        std::fs::rename(&part, voice_dir().join(CPU_EXE)).map_err(|e| e.to_string())?;
    }
    if !m.live && find_exe(GPU_EXE).is_none() {
        // optional: faster on a graphics card; the processor-only helper works without it
        let part = voice_dir().join("ekushe-voice-gpu.exe.part");
        if crate::update_check::download(&helper_url("ekushe-voice-gpu-x64.exe"), &part, |_| {})
            .is_ok()
        {
            let _ = std::fs::rename(&part, voice_dir().join(GPU_EXE));
        } else {
            let _ = std::fs::remove_file(&part);
        }
    }
    let part = voice_dir().join(format!("{}.part", m.file));
    crate::update_check::download(m.url, &part, |n| DOWNLOADED.store(n, Ordering::SeqCst))?;
    let len = std::fs::metadata(&part).map_err(|e| e.to_string())?.len();
    if len != m.bytes {
        let _ = std::fs::remove_file(&part);
        return Err(format!("download incomplete ({len} of {} bytes)", m.bytes));
    }
    std::fs::rename(&part, voice_dir().join(m.file)).map_err(|e| e.to_string())
}

/// Learn from a correction the person made in Studio's "a word was wrong" list: `heard` was typed,
/// `wanted` is right. Word-level differences are learned (Layer 1), then the helper reloads them.
pub fn learn_correction(heard: &str, wanted: &str) {
    let path = learned_path();
    let mut l = ekushe_core::voice::VoiceLearned::parse(
        &std::fs::read_to_string(&path).unwrap_or_default(),
    );
    for (h, w) in word_diffs(heard, wanted) {
        l.learn(&h, &w);
    }
    let _ = std::fs::create_dir_all(voice_dir());
    let _ = std::fs::write(&path, l.to_tsv());
    reload_learned();
}

/// The changed runs of words between two versions of a sentence (up to three words each)
pub fn word_diffs(a: &str, b: &str) -> Vec<(String, String)> {
    let x: Vec<&str> = a.split_whitespace().collect();
    let y: Vec<&str> = b.split_whitespace().collect();
    // common prefix and suffix, then everything between is one change; good for the usual case of
    // one fixed word or name. Several separate fixes are found by repeating on what is left.
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < x.len() && j < y.len() {
        if x[i] == y[j] {
            i += 1;
            j += 1;
            continue;
        }
        // find the next place where they agree again
        let mut best: Option<(usize, usize)> = None;
        'outer: for di in 0..=3 {
            for dj in 0..=3 {
                if (di, dj) == (0, 0) {
                    continue;
                }
                if i + di < x.len() && j + dj < y.len() && x[i + di] == y[j + dj] {
                    best = Some((di, dj));
                    break 'outer;
                }
            }
        }
        let (di, dj) = best.unwrap_or((x.len() - i, y.len() - j));
        if di <= 3 && dj <= 3 && di > 0 && dj > 0 {
            out.push((x[i..i + di].join(" "), y[j..j + dj].join(" ")));
        }
        i += di;
        j += dj;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events() {
        assert_eq!(parse_event(r#"{"event":"ready"}"#), Some(Event::Ready));
        assert_eq!(
            parse_event(r#"{"event":"level","v":0.5}"#),
            Some(Event::Level(0.5))
        );
        assert_eq!(
            parse_event(r#"{"event":"text","heard":"দুই হাজার","text":"২০০০"}"#),
            Some(Event::Text {
                heard: "দুই হাজার".into(),
                text: "২০০০".into(),
                more: false,
                secs: 0.0,
                took: 0.0
            })
        );
        assert!(matches!(
            parse_event(r#"{"event":"text","heard":"a","text":"a","more":true}"#),
            Some(Event::Text { more: true, .. })
        ));
        assert_eq!(parse_event(r#"{"event":"done"}"#), Some(Event::Done));
        assert_eq!(
            parse_event(r#"{"event":"live","text":"আমি বাং"}"#),
            Some(Event::Live("আমি বাং".into()))
        );
        assert!(model("live").live);
        assert!(!model("large").live);
        assert_eq!(parse_event("not json"), None);
        assert_eq!(parse_event(r#"{"event":"loaded"}"#), None);
        let w = chime(true);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(w.len(), 44 + 2 * 2 * 1120);
    }

    #[test]
    fn diffs() {
        assert_eq!(
            word_diffs("আমি রাহিম কে চিনি", "আমি রহিমকে চিনি"),
            vec![("রাহিম কে".to_string(), "রহিমকে".to_string())]
        );
        assert_eq!(
            word_diffs(
                "ঢাকা বিশ্ব বিদ্যালয় থেকে তানবীর এসেছে",
                "ঢাকা বিশ্ববিদ্যালয় থেকে তানভীর এসেছে"
            ),
            vec![
                ("বিশ্ব বিদ্যালয়".to_string(), "বিশ্ববিদ্যালয়".to_string()),
                ("তানবীর".to_string(), "তানভীর".to_string())
            ]
        );
        assert!(word_diffs("একই কথা", "একই কথা").is_empty());
    }

    #[test]
    fn banner() {
        assert!(level_banner(0.5, "Ctrl+Alt+Space").contains("Ctrl+Alt+Space"));
    }
}
