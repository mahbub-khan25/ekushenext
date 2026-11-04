//! live_pack MODEL_DIR OUT.ekm : pack the live model's files into one download
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let names = [
        "encoder.int8.onnx",
        "decoder.int8.onnx",
        "joiner.int8.onnx",
        "tokens.txt",
        "CREDITS.txt",
    ];
    ekushe_voice::live::pack(
        std::path::Path::new(&a[0]),
        &names,
        std::path::Path::new(&a[1]),
    )
    .expect("pack");
    println!("{} bytes", std::fs::metadata(&a[1]).unwrap().len());
}
