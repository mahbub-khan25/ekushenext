//! zip_once MODEL_DIR file.wav : read one recording with the live model (and test the detector)
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    eprintln!("loading model");
    let m = ekushe_voice::live::ZipModel::load(std::path::Path::new(&a[0]), 2).expect("model");
    eprintln!("model ok");
    let mut r = hound::WavReader::open(&a[1]).expect("wav");
    let pcm: Vec<f32> = r
        .samples::<i16>()
        .map(|x| x.unwrap() as f32 / 32768.0)
        .collect();
    println!("{}", m.read(&pcm[..pcm.len().min(16000 * 8)]));
    eprintln!("detector");
    let p = ekushe_voice::live::Phrases::new(0.45).expect("vad");
    let mut all = vec![];
    for c in pcm.chunks(640) {
        p.feed(c);
        all.extend(p.finished());
    }
    println!(
        "{} phrases: {:?}",
        all.len(),
        all.iter()
            .map(|(a, b)| format!("{:.1}-{:.1}", *a as f32 / 16000.0, *b as f32 / 16000.0))
            .collect::<Vec<_>>()
    );
}
