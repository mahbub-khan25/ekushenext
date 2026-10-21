//! Run Ekushe Studio on any computer (macOS, Linux, Windows) without the keyboard app, for design
//! work. Uses the real engine, dictionary and Studio page. Settings you save are written to the
//! normal settings file.
//!
//! ```sh
//! cargo run -p ekushe-ffi --example studio_preview --features "studio embedded-dict"
//! # then open http://127.0.0.1:45219/
//! ```

fn main() -> std::io::Result<()> {
    let port: u16 = std::env::var("EKUSHE_PREVIEW_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(45219);
    println!("Ekushe Studio preview: http://127.0.0.1:{port}/");
    ekushe_ffi::studio::serve_blocking(port)
}
