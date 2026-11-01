//! Correct OCR output files: ocr_correct IN_DIR OUT_DIR (every *.txt except *.B.txt)
fn main() {
    let a: Vec<String> = std::env::args().collect();
    std::fs::create_dir_all(&a[2]).unwrap();
    for e in std::fs::read_dir(&a[1]).unwrap().flatten() {
        let p = e.path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".txt") || name.ends_with(".B.txt") {
            continue;
        }
        let t = std::fs::read_to_string(&p).unwrap();
        let env = |k: &str, d: f32| {
            std::env::var(k)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(d)
        };
        let d = ekushe_core::ocr::OcrParams::default();
        let p = ekushe_core::ocr::OcrParams {
            margin: env("MARGIN", d.margin),
            channel: env("CHANNEL", d.channel),
            context: env("CONTEXT", d.context),
            unknown: env("UNKNOWN", d.unknown),
        };
        let r = ekushe_core::ocr::correct_with(&t, p);
        std::fs::write(format!("{}/{}", a[2], name), r.text).unwrap();
    }
}
