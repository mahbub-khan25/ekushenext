//! `ekushe-voice`: Ekushe's offline voice typing helper. The keyboard app starts it once and keeps it
//! running (the model stays loaded); each dictation is one `listen` command.
//!
//! ```text
//! ekushe-voice --serve --model bn-medium-q5_0.bin [--learned voice-learned.tsv] [--digits en]
//!              [--no-punctuation] [--stop-after 1400] [--piece-pause 600] [--quality best]
//!              [--threads 8] [--no-gpu]
//! ekushe-voice --model bn-medium-q5_0.bin file.wav …     (transcribe files and print the text)
//! ```
//!
//! Serve protocol, one line per message. In (stdin): `listen`, `stop` (finish now), `cancel`,
//! `reload-learned`, `quit`. Out (stdout, JSON): `{"event":"ready"}`, `{"event":"listening"}`,
//! `{"event":"level","v":0.42}` (about ten a second), `{"event":"thinking"}`,
//! `{"event":"text","heard":"…","text":"…"}` (with `"more":true` for a piece typed while the person
//! goes on speaking), `{"event":"done"}` (finished, everything already typed), `{"event":"nothing"}`,
//! `{"event":"error","message":"…"}`.

use ekushe_core::voice::{tidy, VoiceLearned, VoiceOptions};
#[cfg(feature = "whisper")]
use ekushe_voice::segment::{vad_model_file, Cut, Segmenter};
#[cfg(feature = "whisper")]
use ekushe_voice::Recognizer;
use ekushe_voice::{Quality, Recorder};
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

struct Args {
    model: PathBuf,
    learned: Option<PathBuf>,
    opts: VoiceOptions,
    stop_after: u32,
    piece_pause: u32,
    quality: Quality,
    threads: usize,
    gpu: bool,
    serve: bool,
    files: Vec<String>,
    /// live voice typing with the Zipformer model in this folder
    live_model: Option<PathBuf>,
    /// test this computer: dictate these recordings as if spoken into the microphone, then report
    bench: bool,
}

fn args() -> Args {
    let mut a = Args {
        model: PathBuf::new(),
        learned: None,
        opts: VoiceOptions::default(),
        stop_after: 0,
        piece_pause: 400,
        quality: Quality::Fast,
        threads: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8),
        gpu: true,
        serve: false,
        files: vec![],
        live_model: None,
        bench: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "--model" => a.model = it.next().unwrap_or_default().into(),
            "--learned" => a.learned = it.next().map(PathBuf::from),
            "--digits" => a.opts.bangla_digits = it.next().as_deref() != Some("en"),
            "--no-punctuation" => a.opts.spoken_punctuation = false,
            "--no-number-digits" => a.opts.numbers_as_digits = false,
            "--stop-after" => a.stop_after = it.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            "--piece-pause" => {
                a.piece_pause = it.next().and_then(|v| v.parse().ok()).unwrap_or(600)
            }
            "--quality" => {
                a.quality = if it.next().as_deref() == Some("best") {
                    Quality::Best
                } else {
                    Quality::Fast
                }
            }
            "--threads" => a.threads = it.next().and_then(|v| v.parse().ok()).unwrap_or(a.threads),
            "--no-gpu" => a.gpu = false,
            "--serve" => a.serve = true,
            "--live-model" => a.live_model = it.next().map(PathBuf::from),
            "--bench" => a.bench = true,
            f => a.files.push(f.to_string()),
        }
    }
    a
}

/// The self-test (`--bench`): events are kept here with their time instead of printed
static BENCH_LOG: std::sync::Mutex<Option<Vec<(f32, serde_json::Value)>>> =
    std::sync::Mutex::new(None);
static BENCH_T0: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);
/// The recording the self-test is playing as the microphone
#[cfg(feature = "live")]
static BENCH_WAV: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn say(v: serde_json::Value) {
    if let Some(log) = BENCH_LOG.lock().unwrap().as_mut() {
        let ev = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
        let mut t0 = BENCH_T0.lock().unwrap();
        if ev == "listening" {
            *t0 = Some(Instant::now());
        }
        let t = t0.map(|t| t.elapsed().as_secs_f32()).unwrap_or(0.0);
        if ev != "level" {
            log.push((t, v));
        }
        return;
    }
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{v}");
    let _ = out.flush();
}

fn load_learned(p: &Option<PathBuf>) -> HashMap<String, String> {
    p.as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| VoiceLearned::parse(&s).active(1))
        .unwrap_or_default()
}

/// A piece of speech for the recognizing thread; `last` marks the end of the dictation
#[cfg(feature = "whisper")]
struct Job {
    pcm: Vec<f32>,
    last: bool,
}

#[cfg(feature = "whisper")]
enum Heard {
    Loaded,
    LoadFailed(String),
    Text {
        heard: String,
        secs: f32,
        took: f32,
        last: bool,
    },
    Failed(String),
}

fn main() {
    // `--mic-check`: open the microphone for three seconds and report (for the setup check)
    if std::env::args().any(|x| x == "--mic-check") {
        match Recorder::start() {
            Ok(r) => {
                let mut top = 0f32;
                for _ in 0..30 {
                    std::thread::sleep(Duration::from_millis(100));
                    top = top.max(r.level());
                }
                let n = r.stop().len();
                println!(
                    "MICROPHONE OK: {:.1} s recorded, loudest level {top:.2}",
                    n as f32 / 16_000.0
                );
                std::process::exit(if n > 8000 { 0 } else { 1 });
            }
            Err(e) => {
                println!("MICROPHONE PROBLEM: {e}");
                std::process::exit(1);
            }
        }
    }
    let a = args();
    #[cfg(feature = "live")]
    if let Some(dir) = a.live_model.clone() {
        live_main(&a, &dir);
        return;
    }
    #[cfg(feature = "whisper")]
    whisper_main(a);
    #[cfg(not(feature = "whisper"))]
    {
        let _ = a;
        eprintln!("usage: ekushe-voice --live-model DIR --serve");
        std::process::exit(2);
    }
}

/// The Whisper path: one piece of speech at a time, read by whisper.cpp
#[cfg(feature = "whisper")]
fn whisper_main(a: Args) {
    if a.model.as_os_str().is_empty() {
        eprintln!("usage: ekushe-voice --model MODEL.bin (--serve | file.wav …)");
        std::process::exit(2);
    }
    if !a.serve {
        let mut rec = match Recognizer::load(&a.model, a.threads, a.gpu) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        };
        rec.quality = a.quality;
        let learned = load_learned(&a.learned);
        for f in &a.files {
            match read_wav(f) {
                Ok(pcm) => match rec.transcribe(&pcm) {
                    Ok(h) => println!("{f}\t{}", tidy(&h, &a.opts, &learned)),
                    Err(e) => eprintln!("{f}: {e}"),
                },
                Err(e) => eprintln!("{f}: {e}"),
            }
        }
        return;
    }

    // The model loads on its own thread and recognizes there, so the microphone can start at once
    // (nothing said while the model is still loading is lost) and keeps recording while a piece is
    // being recognized.
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (heard_tx, heard_rx) = mpsc::channel::<Heard>();
    {
        let (model, threads, gpu, quality) = (a.model.clone(), a.threads, a.gpu, a.quality);
        // test builds: what was heard goes into the local log too (EKUSHE_VOICE_LOG_TEXT=1)
        let log_text = std::env::var("EKUSHE_VOICE_LOG_TEXT").is_ok();
        std::thread::spawn(move || {
            let t0 = Instant::now();
            let mut rec = match Recognizer::load(&model, threads, gpu) {
                Ok(r) => r,
                Err(e) => {
                    let _ = heard_tx.send(Heard::LoadFailed(e.to_string()));
                    return;
                }
            };
            rec.quality = quality;
            eprintln!("model loaded in {:.1}s", t0.elapsed().as_secs_f32());
            let _ = heard_tx.send(Heard::Loaded);
            while let Ok(job) = job_rx.recv() {
                let t0 = Instant::now();
                let secs = job.pcm.len() as f32 / 16_000.0;
                let msg = match rec.transcribe(&job.pcm) {
                    Ok(h) => {
                        let took = t0.elapsed().as_secs_f32();
                        eprintln!("{:.1}s of speech recognized in {took:.2}s", secs);
                        if log_text {
                            eprintln!("  heard: {h}");
                        }
                        Heard::Text {
                            heard: h,
                            secs,
                            took,
                            last: job.last,
                        }
                    }
                    Err(e) => Heard::Failed(e.to_string()),
                };
                if heard_tx.send(msg).is_err() {
                    break;
                }
            }
        });
    }

    // commands from the keyboard app arrive on their own thread
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(l) = line else { break };
            if tx.send(l.trim().to_string()).is_err() {
                break;
            }
        }
        let _ = tx.send("quit".into());
    });
    say(json!({"event":"ready"}));
    let mut learned = load_learned(&a.learned);
    // the speech detector that cuts dictation into sentence-sized pieces
    let mut seg = match vad_model_file().and_then(|p| Segmenter::new(&p, a.threads)) {
        Ok(s) => s,
        Err(e) => {
            say(json!({"event":"error","message": format!("speech detector: {e}")}));
            std::process::exit(1);
        }
    };
    let mut load_error: Option<String> = None;
    // a result that is not part of a dictation (the model finished loading)
    let note = |h: &Heard, load_error: &mut Option<String>| match h {
        Heard::Loaded => say(json!({"event":"loaded"})),
        Heard::LoadFailed(e) => {
            *load_error = Some(e.clone());
            say(json!({"event":"error","message": e}));
        }
        _ => {}
    };
    while let Ok(cmd) = rx.recv() {
        while let Ok(h) = heard_rx.try_recv() {
            note(&h, &mut load_error);
        }
        match cmd.as_str() {
            "quit" => break,
            "reload-learned" => learned = load_learned(&a.learned),
            "listen" => {
                if let Some(e) = &load_error {
                    say(json!({"event":"error","message": e}));
                    continue;
                }
                // tests: EKUSHE_VOICE_FAKE_MIC=file.wav plays a recording instead of the microphone
                let fake = std::env::var("EKUSHE_VOICE_FAKE_MIC")
                    .ok()
                    .and_then(|f| read_wav(&f).ok());
                let r = match fake
                    .map(Recorder::from_samples)
                    .map(Ok)
                    .unwrap_or_else(Recorder::start)
                {
                    Ok(r) => r,
                    Err(e) => {
                        say(json!({"event":"error","message": e.to_string()}));
                        continue;
                    }
                };
                say(json!({"event":"listening"}));
                let started = Instant::now();
                let mut last_level = Instant::now();
                let mut last_check = Instant::now();
                let (mut cancelled, mut quit) = (false, false);
                let mut sent = 0usize; // pieces handed to the recognizer
                let mut typed_any = false;
                let mut nothing_said = false;
                let mut taken = 0usize; // audio before this was handed out or dropped
                let mut heard_speech = false;
                let mut last_speech = 0usize; // where the last piece ended
                let handle =
                    |h: Heard, typed_any: &mut bool, load_error: &mut Option<String>| -> bool {
                        // true = one of this dictation's pieces came back
                        match h {
                            Heard::Text {
                                heard,
                                secs,
                                took,
                                last,
                            } => {
                                if !heard.trim().is_empty() {
                                    let t = tidy(&heard, &a.opts, &learned);
                                    say(json!({"event":"text","heard": heard, "text": t,
                                    "more": !last, "secs": secs, "took": took}));
                                    *typed_any = true;
                                }
                                true
                            }
                            Heard::Failed(e) => {
                                eprintln!("recognizing failed: {e}");
                                true
                            }
                            other => {
                                note(&other, load_error);
                                false
                            }
                        }
                    };
                loop {
                    match rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(c) if c == "stop" => break,
                        Ok(c) if c == "cancel" => {
                            cancelled = true;
                            break;
                        }
                        Ok(c) if c == "quit" => {
                            cancelled = true;
                            quit = true;
                            break;
                        }
                        _ => {}
                    }
                    // every 300 ms: is a sentence finished? (the speech detector decides)
                    if last_check.elapsed() >= Duration::from_millis(300) {
                        last_check = Instant::now();
                        let audio = r.audio(taken);
                        match seg.cut(&audio, false) {
                            Cut::Piece { start, end } => {
                                let _ = job_tx.send(Job {
                                    pcm: audio[start..end].to_vec(),
                                    last: false,
                                });
                                sent += 1;
                                taken += end;
                                heard_speech = true;
                                last_speech = taken;
                            }
                            Cut::Silence { end } => taken += end,
                            Cut::Wait => {}
                        }
                        let now = r.len();
                        if a.stop_after > 0
                            && heard_speech
                            && now.saturating_sub(last_speech) > a.stop_after as usize * 16
                            && !seg.has_speech(&r.audio(taken))
                        {
                            break; // a pause ends the dictation (setting)
                        }
                        if !heard_speech && now > 20 * 16_000 && !seg.has_speech(&r.audio(taken)) {
                            nothing_said = true;
                            break;
                        }
                        if heard_speech && now.saturating_sub(last_speech) > 45 * 16_000 {
                            break; // a long silence ends it anyway
                        }
                    }
                    while let Ok(h) = heard_rx.try_recv() {
                        if handle(h, &mut typed_any, &mut load_error) {
                            sent -= 1;
                        }
                    }
                    if load_error.is_some() {
                        cancelled = true;
                        break;
                    }
                    // the level for the mic banner, about ten times a second
                    if last_level.elapsed() >= Duration::from_millis(100) {
                        last_level = Instant::now();
                        say(json!({"event":"level","v": (r.level() * 100.0).round() / 100.0}));
                    }
                    if started.elapsed() > Duration::from_secs(600) {
                        break; // ten minutes at most in one go
                    }
                }
                let all = r.stop();
                // what was said after the last piece, in sentence-sized pieces too
                let mut last_pieces = vec![];
                if !cancelled && !nothing_said {
                    let mut at = taken.min(all.len());
                    while at < all.len() {
                        match seg.cut(&all[at..], true) {
                            Cut::Piece { start, end } => {
                                last_pieces.push(all[at + start..at + end].to_vec());
                                at += end.max(1);
                            }
                            Cut::Silence { end } => at += end.max(1),
                            Cut::Wait => break,
                        }
                    }
                }
                if quit {
                    break;
                }
                if cancelled {
                    // pieces already being recognized are dropped
                    while sent > 0 {
                        match heard_rx.recv() {
                            Ok(Heard::Text { .. }) | Ok(Heard::Failed(..)) => sent -= 1,
                            Ok(other) => note(&other, &mut load_error),
                            Err(_) => break,
                        }
                    }
                    continue;
                }
                if nothing_said && !typed_any && sent == 0 {
                    // (nothing to wait for)
                    say(json!({"event":"nothing"}));
                    continue;
                }
                let n = last_pieces.len();
                for (i, pcm) in last_pieces.into_iter().enumerate() {
                    let _ = job_tx.send(Job {
                        pcm,
                        last: i + 1 == n,
                    });
                    sent += 1;
                }
                if sent > 0 {
                    say(json!({"event":"thinking"}));
                }
                while sent > 0 {
                    match heard_rx.recv() {
                        Ok(h) => {
                            if handle(h, &mut typed_any, &mut load_error) {
                                sent -= 1;
                            }
                        }
                        Err(_) => break,
                    }
                }
                if typed_any {
                    say(json!({"event":"done"}));
                } else {
                    say(json!({"event":"nothing"}));
                }
            }
            _ => {}
        }
    }
}

/// Live voice typing (Zipformer): words are typed while the person speaks.
///
/// Out, besides ready/loaded/listening/level/thinking/done/nothing/error:
/// `{"event":"text","text":"…","more":true}` words to type now, tidied (the app adds a space).
/// Typed words are never taken back, so the app only ever adds text.
/// `{"event":"live","text":"…"}` words heard but not typed yet (for the banner).
#[cfg(feature = "live")]
fn live_main(a: &Args, dir: &std::path::Path) {
    use ekushe_voice::live::{Phrases, Typist, Word, ZipModel};
    /// read samples `start..end` of the dictation; `fin` = the phrase ended there
    struct Job {
        pcm: Vec<f32>,
        start: usize,
        end: usize,
        fin: bool,
    }
    enum Done {
        Loaded(Result<(), String>),
        Read {
            words: Vec<Word>,
            start: usize,
            end: usize,
            fin: bool,
            took: f32,
        },
    }
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (done_tx, done_rx) = mpsc::channel::<Done>();
    {
        let (dir, threads) = (dir.to_path_buf(), a.threads);
        std::thread::spawn(move || {
            let t0 = Instant::now();
            let model =
                match ekushe_voice::live::unpack(&dir).and_then(|d| ZipModel::load(&d, threads)) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = done_tx.send(Done::Loaded(Err(e)));
                        return;
                    }
                };
            eprintln!("model loaded in {:.1}s", t0.elapsed().as_secs_f32());
            let _ = done_tx.send(Done::Loaded(Ok(())));
            while let Ok(j) = job_rx.recv() {
                let t = Instant::now();
                let words = model.words(&j.pcm);
                let took = t.elapsed().as_secs_f32();
                let msg = Done::Read {
                    words,
                    start: j.start,
                    end: j.end,
                    fin: j.fin,
                    took,
                };
                if done_tx.send(msg).is_err() {
                    break;
                }
            }
        });
    }
    let (tx, rx) = mpsc::channel::<String>();
    if a.bench {
        *BENCH_LOG.lock().unwrap() = Some(vec![]);
        let files = a.files.clone();
        std::thread::spawn(move || {
            // wait for the model, then play each recording as the microphone, in real time
            let logged = |ev: &str| {
                BENCH_LOG
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map(|l| l.iter().filter(|(_, v)| v["event"] == ev).count())
                    .unwrap_or(0)
            };
            while logged("loaded") == 0 && logged("error") == 0 {
                std::thread::sleep(Duration::from_millis(100));
            }
            for f in &files {
                let secs = read_wav(f)
                    .map(|p| p.len() as f32 / 16_000.0)
                    .unwrap_or(0.0);
                *BENCH_WAV.lock().unwrap() = Some(f.clone());
                let before = logged("stats");
                let _ = tx.send("listen".into());
                std::thread::sleep(Duration::from_secs_f32(secs + 0.3));
                let _ = tx.send("stop".into());
                while logged("stats") == before {
                    std::thread::sleep(Duration::from_millis(50));
                }
                std::thread::sleep(Duration::from_millis(300));
            }
            let _ = tx.send("quit".into());
        });
    } else {
        std::thread::spawn(move || {
            for line in std::io::stdin().lock().lines() {
                let Ok(l) = line else { break };
                if tx.send(l.trim().to_string()).is_err() {
                    break;
                }
            }
            let _ = tx.send("quit".into());
        });
    }
    say(json!({"event":"ready"}));
    let log_text = std::env::var("EKUSHE_VOICE_LOG_TEXT").is_ok();
    let mut learned = load_learned(&a.learned);
    let mut load_error: Option<String> = None;
    let pad = 2400; // 150 ms of audio kept after a phrase
    loop {
        // tell the app as soon as the model is loaded, even before any command
        let cmd = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(c) => c,
            Err(mpsc::RecvTimeoutError::Timeout) => String::new(),
            Err(_) => break,
        };
        while let Ok(d) = done_rx.try_recv() {
            if let Done::Loaded(Err(e)) = d {
                say(json!({"event":"error","message": e}));
                load_error = Some(e);
            } else if let Done::Loaded(Ok(())) = d {
                say(json!({"event":"loaded"}));
            }
        }
        match cmd.as_str() {
            "quit" => break,
            "reload-learned" => learned = load_learned(&a.learned),
            "listen" => {
                if let Some(e) = &load_error {
                    say(json!({"event":"error","message": e}));
                    continue;
                }
                let fake = BENCH_WAV
                    .lock()
                    .unwrap()
                    .clone()
                    .or_else(|| std::env::var("EKUSHE_VOICE_FAKE_MIC").ok())
                    .and_then(|f| read_wav(&f).ok());
                let r = match fake
                    .map(Recorder::from_samples)
                    .map(Ok)
                    .unwrap_or_else(Recorder::start)
                {
                    Ok(r) => r,
                    Err(e) => {
                        say(json!({"event":"error","message": e.to_string()}));
                        continue;
                    }
                };
                let phrases = match Phrases::new(0.45) {
                    Ok(p) => p,
                    Err(e) => {
                        say(json!({"event":"error","message": e}));
                        continue;
                    }
                };
                say(json!({"event":"listening"}));
                let started = Instant::now();
                let mut ty = Typist::default();
                // (for tuning on the test bench)
                let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
                ty.win = env("EKUSHE_LIVE_WIN").unwrap_or(ty.win);
                ty.keep = env("EKUSHE_LIVE_KEEP").unwrap_or(ty.keep);
                let (mut last_level, mut last_read) = (Instant::now(), Instant::now());
                let (mut fed, mut busy) = (0usize, false);
                let mut phrase_end: Option<usize> = None;
                let (mut typed_any, mut heard_speech) = (false, false);
                let mut last_speech_at = 0usize;
                let (mut cancelled, mut quit, mut nothing) = (false, false, false);
                let (mut reads, mut read_time, mut longest) = (0usize, 0f32, 0f32);
                // words held back: "নতুন" may become "নতুন লাইন", "দুই" may become "দুই হাজার"
                let held = std::cell::RefCell::new(Vec::<String>::new());
                let emit = |words: &[String], typed_any: &mut bool| {
                    if words.is_empty() {
                        return;
                    }
                    let t = tidy(&words.join(" "), &a.opts, &learned);
                    if !t.is_empty() {
                        // (the app adds the space after it, as for the other model)
                        say(json!({"event":"text","text": t, "more": true}));
                        *typed_any = true;
                    }
                };
                // a reading came back: type what is settled, show the rest
                let mut handle =
                    |d: Done, ty: &mut Typist, busy: &mut bool, typed_any: &mut bool| {
                        let Done::Read {
                            words,
                            start,
                            end,
                            fin,
                            took,
                        } = d
                        else {
                            return;
                        };
                        *busy = false;
                        reads += 1;
                        read_time += took;
                        longest = longest.max(took);
                        if start != ty.start {
                            return; // read before the window moved
                        }
                        // The model adds its own full stops and dashes at the end of phrases. With
                        // spoken punctuation on, the writer decides the marks: drop the model's.
                        let words = if a.opts.spoken_punctuation {
                            ekushe_voice::live::without_marks(words)
                        } else {
                            words
                        };
                        let mut new = if fin {
                            ty.finish(&words, end)
                        } else {
                            ty.partial(&words, end - start)
                        };
                        // "দাঁড়ি" said just before a pause is often heard as "তারই" or "তারি":
                        // at the very end of a phrase those mean the full stop
                        if fin && a.opts.spoken_punctuation {
                            if let Some(last) = new.last_mut() {
                                if ekushe_core::voice::is_misheard_full_stop(last) {
                                    *last = "দাঁড়ি".into();
                                }
                            }
                        }
                        if std::env::var("EKUSHE_VOICE_DEBUG").is_ok() {
                            let all: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
                            eprintln!(
                                "{:6.2}s read {:.1}-{:.1}s fin={fin}: {}",
                                started.elapsed().as_secs_f32(),
                                start as f32 / 16e3,
                                end as f32 / 16e3,
                                all.join(" ")
                            );
                        }
                        if fin && log_text {
                            let all: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
                            eprintln!(
                                "  phrase to {:.1}s: {}",
                                end as f32 / 16_000.0,
                                all.join(" ")
                            );
                        }
                        let out: Vec<String> = {
                            let mut h = held.borrow_mut();
                            h.extend(new);
                            // hold the last words back while they may still join the next one
                            let mut k = h.len();
                            while k > 0
                                && h.len() - k < 6
                                && ekushe_core::voice::may_join_next(&h[k - 1])
                            {
                                k -= 1;
                            }
                            if h.len() - k >= 6 {
                                k = h.len();
                            }
                            h.drain(..k).collect()
                        };
                        emit(&out, typed_any);
                        let shown = [held.borrow().join(" "), ty.tail().to_string()].join(" ");
                        say(json!({"event":"live","text": shown.trim()}));
                    };
                loop {
                    match rx.recv_timeout(Duration::from_millis(40)) {
                        Ok(c) if c == "stop" => break,
                        Ok(c) if c == "cancel" => {
                            cancelled = true;
                            break;
                        }
                        Ok(c) if c == "quit" => {
                            cancelled = true;
                            quit = true;
                            break;
                        }
                        _ => {}
                    }
                    let new = r.audio(fed);
                    if !new.is_empty() {
                        phrases.feed(&new);
                        fed += new.len();
                    }
                    if phrases.speaking() {
                        heard_speech = true;
                        last_speech_at = fed;
                    }
                    for (_, end) in phrases.finished() {
                        phrase_end = Some(end + pad);
                    }
                    while let Ok(d) = done_rx.try_recv() {
                        handle(d, &mut ty, &mut busy, &mut typed_any);
                    }
                    // one reading at a time: a slower computer just reads less often
                    if !busy {
                        if let Some(e) = phrase_end.take() {
                            let e = e.min(fed);
                            if e > ty.start + 1600 {
                                let pcm = r.audio(ty.start);
                                let n = (e - ty.start).min(pcm.len());
                                let _ = job_tx.send(Job {
                                    pcm: pcm[..n].to_vec(),
                                    start: ty.start,
                                    end: e,
                                    fin: true,
                                });
                                busy = true;
                                last_read = Instant::now();
                            } else {
                                ty.start = ty.start.max(e);
                            }
                        } else if phrases.speaking()
                            && last_read.elapsed() >= Duration::from_millis(250)
                            && fed > ty.start + 8000
                        {
                            let _ = job_tx.send(Job {
                                pcm: r.audio(ty.start),
                                start: ty.start,
                                end: fed,
                                fin: false,
                            });
                            busy = true;
                            last_read = Instant::now();
                        }
                    }
                    // keep only the audio still needed: 5 s before the reading window
                    r.forget_before(ty.start.saturating_sub(5 * 16_000));
                    if last_level.elapsed() >= Duration::from_millis(100) {
                        last_level = Instant::now();
                        say(json!({"event":"level","v": (r.level() * 100.0).round() / 100.0}));
                    }
                    if a.stop_after > 0
                        && heard_speech
                        && fed.saturating_sub(last_speech_at) > a.stop_after as usize * 16
                        && !phrases.speaking()
                    {
                        break;
                    }
                    if !heard_speech && fed > 20 * 16_000 {
                        nothing = true;
                        break;
                    }
                    // writers pause to think: without auto-stop only a long silence (10 min) ends it
                    let idle = if a.stop_after > 0 { 45 } else { 600 };
                    if heard_speech && fed.saturating_sub(last_speech_at) > idle * 16_000 {
                        break;
                    }
                    // a chapter can take a while: up to three hours in one go
                    if started.elapsed() > Duration::from_secs(3 * 3600) {
                        break;
                    }
                }
                let (kept_from, all) = r.stop_kept();
                let total = kept_from + all.len();
                if quit {
                    break;
                }
                if !cancelled && (busy || total > ty.start + 4800) {
                    say(json!({"event":"thinking"}));
                }
                // wait for the reading in progress, then read what is left
                let mut last_sent = false;
                loop {
                    if busy {
                        match done_rx.recv() {
                            Ok(d) if cancelled => {
                                if let Done::Read { .. } = d {
                                    busy = false;
                                }
                            }
                            Ok(d) => handle(d, &mut ty, &mut busy, &mut typed_any),
                            Err(_) => break,
                        }
                        continue;
                    }
                    if cancelled || nothing || last_sent || total <= ty.start + 4800 {
                        break;
                    }
                    let _ = job_tx.send(Job {
                        pcm: all[ty.start.saturating_sub(kept_from).min(all.len())..].to_vec(),
                        start: ty.start,
                        end: total,
                        fin: true,
                    });
                    busy = true;
                    last_sent = true;
                }
                // the words still held back
                if !cancelled {
                    let rest: Vec<String> = held.borrow_mut().drain(..).collect();
                    emit(&rest, &mut typed_any);
                }
                eprintln!(
                    "dictation: {:.0}s, read {reads} times in {read_time:.1}s (longest {longest:.2}s)",
                    total as f32 / 16_000.0
                );
                say(
                    json!({"event":"stats","secs": total as f32 / 16_000.0, "reads": reads, "read_time": read_time, "longest": longest}),
                );
                if cancelled {
                    continue;
                }
                say(json!({"event": if typed_any { "done" } else { "nothing" }}));
            }
            _ => {}
        }
    }
    if a.bench {
        bench_report(a);
    }
}

/// The self-test's report: how quickly words were typed, whether anything piled up, and the text
#[cfg(feature = "live")]
fn bench_report(a: &Args) {
    let log = BENCH_LOG.lock().unwrap().take().unwrap_or_default();
    let mut out = String::new();
    let mut all_ok = true;
    let threads = a.threads;
    out += &format!(
        "Ekushe live voice typing self-test\nprocessor threads used: {threads} (this computer has {})\n\n",
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0)
    );
    if let Some((_, e)) = log.iter().find(|(_, v)| v["event"] == "error") {
        out += &format!("ERROR: {}\n", e["message"]);
        all_ok = false;
    }
    // split into dictations at each "listening"
    let mut runs: Vec<Vec<(f32, serde_json::Value)>> = vec![];
    for (t, v) in log {
        if v["event"] == "listening" {
            runs.push(vec![]);
        }
        if let Some(r) = runs.last_mut() {
            r.push((t, v));
        }
    }
    for (f, run) in a.files.iter().zip(&runs) {
        let secs = read_wav(f)
            .map(|p| p.len() as f32 / 16_000.0)
            .unwrap_or(0.0);
        let texts: Vec<(f32, String)> = run
            .iter()
            .filter(|(_, v)| v["event"] == "text")
            .map(|(t, v)| (*t, v["text"].as_str().unwrap_or("").to_string()))
            .collect();
        let typed: String = texts
            .iter()
            .map(|(_, t)| {
                if t.ends_with('\n') {
                    t.clone()
                } else {
                    format!("{t} ")
                }
            })
            .collect();
        let words = typed.split_whitespace().count();
        let mut gaps: Vec<f32> = texts
            .windows(2)
            .filter(|w| w[1].0 < secs)
            .map(|w| w[1].0 - w[0].0)
            .collect();
        gaps.sort_by(|x, y| x.total_cmp(y));
        let median = gaps.get(gaps.len() / 2).copied().unwrap_or(f32::NAN);
        let max_gap = gaps.last().copied().unwrap_or(f32::NAN);
        let blocks = (secs / 30.0).floor() as usize;
        let per_block: Vec<usize> = (0..blocks)
            .map(|b| {
                texts
                    .iter()
                    .filter(|(t, _)| (*t / 30.0) as usize == b)
                    .map(|(_, x)| x.split_whitespace().count())
                    .sum()
            })
            .collect();
        let stats = run
            .iter()
            .find(|(_, v)| v["event"] == "stats")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let finished = run
            .iter()
            .find(|(_, v)| v["event"] == "done" || v["event"] == "nothing")
            .map(|(t, _)| t - (secs + 0.3))
            .unwrap_or(f32::NAN);
        let longest = stats["longest"].as_f64().unwrap_or(f64::NAN) as f32;
        let read_time = stats["read_time"].as_f64().unwrap_or(0.0) as f32;
        let avg = per_block.iter().sum::<usize>() as f32 / per_block.len().max(1) as f32;
        let last = per_block.last().copied().unwrap_or(0) as f32;
        let checks = [
            (
                "words typed while speaking (median gap at most 1.5 s)",
                median <= 1.5,
            ),
            (
                "no reading took long (longest at most 1.5 s)",
                longest <= 1.5,
            ),
            ("finished quickly after stop (at most 2 s)", finished <= 2.0),
            (
                "no slowdown: the last 30 s typed at least 40% of the average",
                blocks < 2 || last >= 0.4 * avg,
            ),
            ("words were typed", words > 0),
        ];
        let ok = checks.iter().all(|c| c.1);
        all_ok &= ok;
        out += &format!(
            "== {} ({secs:.0} s)\n{}\nwords typed: {words}; first words after {:.1} s; typing every {median:.2} s (longest wait {max_gap:.2} s)\n\
             words per 30 s: {per_block:?}\nreadings: {} taking {read_time:.1} s in all ({:.0}% of the time), longest {longest:.2} s\n\
             finished {finished:.2} s after stop\n",
            std::path::Path::new(f).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            if ok { "PASS" } else { "FAIL" },
            texts.first().map(|t| t.0).unwrap_or(f32::NAN),
            stats["reads"],
            100.0 * read_time / secs.max(1.0),
        );
        for (name, pass) in checks {
            out += &format!("  [{}] {name}\n", if pass { "ok" } else { "FAILED" });
        }
        out += &format!("typed text:\n{}\n\n", typed.trim_end());
    }
    if runs.len() < a.files.len() {
        out += "ERROR: not every recording was dictated\n";
        all_ok = false;
    }
    out += if all_ok {
        "RESULT: PASS\n"
    } else {
        "RESULT: FAIL\n"
    };
    print!("{out}");
    std::process::exit(if all_ok { 0 } else { 1 });
}

fn read_wav(f: &str) -> Result<Vec<f32>, String> {
    // 16-bit PCM or float WAV, any rate and channel count
    let data = std::fs::read(f).map_err(|e| e.to_string())?;
    let (fmt, body) = wav_chunks(&data).ok_or("not a WAV file")?;
    let ch = u16::from_le_bytes([fmt[2], fmt[3]]) as usize;
    let rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
    let bits = u16::from_le_bytes([fmt[14], fmt[15]]);
    let tag = u16::from_le_bytes([fmt[0], fmt[1]]);
    let s: Vec<f32> = match (tag, bits) {
        (1, 16) => body
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
            .collect(),
        (3, 32) => body
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
        _ => return Err(format!("unsupported WAV format {tag}/{bits}")),
    };
    Ok(ekushe_voice::audio::to_16k_mono(&s, ch, rate))
}

fn wav_chunks(d: &[u8]) -> Option<(&[u8], &[u8])> {
    if d.len() < 12 || &d[0..4] != b"RIFF" || &d[8..12] != b"WAVE" {
        return None;
    }
    let (mut fmt, mut body) = (None, None);
    let mut i = 12;
    while i + 8 <= d.len() {
        let id = &d[i..i + 4];
        let n = u32::from_le_bytes([d[i + 4], d[i + 5], d[i + 6], d[i + 7]]) as usize;
        let end = (i + 8 + n).min(d.len());
        if id == b"fmt " {
            fmt = Some(&d[i + 8..end]);
        } else if id == b"data" {
            body = Some(&d[i + 8..end]);
        }
        i = end + (n & 1);
    }
    Some((fmt?, body?))
}
