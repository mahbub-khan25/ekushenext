//! Transcribe WAV files with a Bangla speech model and tidy the text.
//! cargo run -p ekushe-voice --release --example voice_file -- MODEL.bin a.wav [b.wav …]
use ekushe_core::voice::VoiceOptions;
use std::{collections::HashMap, path::Path, time::Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: voice_file MODEL.bin file.wav …");
        std::process::exit(2);
    }
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let t = Instant::now();
    let mut rec =
        ekushe_voice::Recognizer::load(Path::new(&args[0]), threads, true).expect("model");
    eprintln!("model loaded in {:.1} s", t.elapsed().as_secs_f32());
    rec.reuse_state = std::env::var("FRESH").is_err();
    rec.fallback = std::env::var("FALLBACK").is_ok();
    for f in &args[1..] {
        let mut r = hound::WavReader::open(f).expect("wav");
        let spec = r.spec();
        let s: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => r.samples::<f32>().map(|x| x.unwrap()).collect(),
            hound::SampleFormat::Int => r
                .samples::<i32>()
                .map(|x| x.unwrap() as f32 / (1 << (spec.bits_per_sample - 1)) as f32)
                .collect(),
        };
        let pcm = ekushe_voice::audio::to_16k_mono(&s, spec.channels as usize, spec.sample_rate);
        let t = Instant::now();
        let heard = rec.transcribe(&pcm).expect("transcribe");
        let tidy = ekushe_core::voice::tidy(&heard, &VoiceOptions::default(), &HashMap::new());
        println!("{f}\t{:.2}s\t{heard}\t{tidy}", t.elapsed().as_secs_f32());
    }
}
