//! Long-dictation test bench: feeds a recording through the same steps as live dictation (30 ms
//! frames, the piece cutter, the recognizer, tidy) on a simulated clock, and reports what was
//! dropped, accuracy-ready text, and how long after each piece the text would appear.
//!
//! voice_bench MODEL.bin VAD.bin recording.wav [threads] > out.tsv   (16 kHz mono WAV)
use ekushe_core::voice::{tidy, VoiceOptions};
use ekushe_voice::segment::{Cut, Segmenter};
use std::{collections::HashMap, path::Path, time::Instant};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let threads: usize = a.get(3).and_then(|t| t.parse().ok()).unwrap_or(4);
    let mut rec = ekushe_voice::Recognizer::load(Path::new(&a[0]), threads, false).expect("model");
    rec.reuse_state = false;
    rec.fallback = true;
    let mut seg = Segmenter::new(Path::new(&a[1]), threads).expect("vad");
    let mut r = hound::WavReader::open(&a[2]).expect("wav");
    let pcm: Vec<f32> = r
        .samples::<i16>()
        .map(|x| x.unwrap() as f32 / 32768.0)
        .collect();
    let frame = 480;
    let check_every = 4800; // look for a piece every 300 ms of audio
    let (mut taken, mut busy_until, mut since_check) = (0usize, 0f32, 0usize);
    let mut lags = vec![];
    let mut text = String::new();
    let mut pieces = 0;
    let mut pos = 0;
    let mut work = 0f32;
    while pos < pcm.len() {
        pos = (pos + frame).min(pcm.len());
        since_check += frame;
        let done = pos == pcm.len();
        if since_check < check_every && !done {
            continue;
        }
        since_check = 0;
        loop {
            let t = Instant::now();
            let c = seg.cut(&pcm[taken..pos], done);
            work += t.elapsed().as_secs_f32();
            match c {
                Cut::Wait => break,
                Cut::Silence { end } => taken += end,
                Cut::Piece { start, end } => {
                    let piece = &pcm[taken + start..taken + end];
                    let t = Instant::now();
                    let heard = rec.transcribe(piece).unwrap_or_default();
                    let took = t.elapsed().as_secs_f32();
                    work += took;
                    let now = pos as f32 / 16_000.0; // when the cut was made
                    let begin = busy_until.max(now);
                    busy_until = begin + took;
                    let spoken_end = (taken + end) as f32 / 16_000.0;
                    lags.push(busy_until - spoken_end);
                    pieces += 1;
                    let t = tidy(
                        &heard,
                        &VoiceOptions {
                            numbers_as_digits: false,
                            spoken_punctuation: false,
                            ..Default::default()
                        },
                        &HashMap::new(),
                    );
                    eprintln!("piece {pieces}: {:.1}s at {:.1}s, recognized in {took:.2}s, text after {:.2}s: {t}", piece.len() as f32 / 16_000.0, spoken_end, busy_until - spoken_end);
                    text.push_str(&heard);
                    text.push(' ');
                    taken += end;
                    if done && taken >= pcm.len() {
                        break;
                    }
                }
            }
            if done && taken >= pcm.len() {
                break;
            }
            if !done {
                break;
            }
        }
    }
    let secs = pcm.len() as f32 / 16_000.0;
    lags.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = lags.get(lags.len() / 2).copied().unwrap_or(0.0);
    let p90 = lags.get(lags.len() * 9 / 10).copied().unwrap_or(0.0);
    eprintln!("SUMMARY pieces {pieces}  work x{:.2} of real time  text delay median {med:.1}s p90 {p90:.1}s max {:.1}s", work / secs, lags.last().copied().unwrap_or(0.0));
    println!("{}", text.trim());
}
