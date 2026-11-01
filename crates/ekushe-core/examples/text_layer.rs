//! Check a PDF page's text: `text_layer FILE` prints what Ekushe would use instead of OCR.
fn main() {
    let s = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    println!(
        "dict: {}",
        ekushe_core::dict::Dictionary::global().is_some()
    );
    match ekushe_core::ocr::text_layer(&s) {
        Some(t) => print!("[{}]\n{}", t.kind, t.text),
        None => println!("(not usable: read the picture)"),
    }
}
