//! Studio's "picture to text" (Bangla OCR), shared by the Windows app (this crate) and by Studio on
//! macOS and Linux (`crates/ekushe-ffi`, which includes this file).
//!
//! The page reads the picture itself with Tesseract compiled to WebAssembly (tesseract.js), so the
//! picture never leaves the computer. This module serves the OCR files from memory and cleans the
//! text it reads with Ekushe's dictionary:
//!
//! - `GET  /ocr/<file>`        tesseract.js, its worker and core, the Bangla model, and pdf.js
//!   (scanned PDF pages are drawn in the page, then read)
//! - `POST /api/ocr/correct`   body `{"lines": [[["word", confidence], ...], null, ...], "junk": true}`
//!   (OCR's lines, `null` between paragraphs; or `{"text": "..."}`) → `{"text", "changes", "dropped"}`
//! - `POST /api/ocr/sample`   a page picture as read (`{"id", "page", "jpg": base64, "lines"}`) or the
//!   corrected text (`{"id", "corrected", "original"}`), saved under `<config dir>/ocr-samples/<id>/`
//!   on this computer, as material to train the reader on later
//! - `POST /api/ocr/textlayer` body `{"text": "..."}` (a PDF page's own text) → `{"use": true,
//!   "text", "kind"}` when it is real Bangla or Bijoy text, so the page needs no OCR

const FILES: &[(&str, &str, &[u8])] = &[
    (
        "tesseract.min.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../../assets/ocr/tesseract.min.js"),
    ),
    (
        "worker.min.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../../assets/ocr/worker.min.js"),
    ),
    (
        "tesseract-core-simd-lstm.wasm.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../../assets/ocr/tesseract-core-simd-lstm.wasm.js"),
    ),
    (
        "tesseract-core-lstm.wasm.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../../assets/ocr/tesseract-core-lstm.wasm.js"),
    ),
    (
        "pdf.min.mjs",
        "text/javascript; charset=utf-8",
        include_bytes!("../../../assets/ocr/pdf.min.mjs"),
    ),
    (
        "pdf.worker.min.mjs",
        "text/javascript; charset=utf-8",
        include_bytes!("../../../assets/ocr/pdf.worker.min.mjs"),
    ),
    // pdf.js image decoders: JBIG2 and JPEG 2000 (common in scanned book PDFs), colour profiles
    (
        "wasm/jbig2.wasm",
        "application/wasm",
        include_bytes!("../../../assets/ocr/wasm/jbig2.wasm"),
    ),
    (
        "wasm/openjpeg.wasm",
        "application/wasm",
        include_bytes!("../../../assets/ocr/wasm/openjpeg.wasm"),
    ),
    (
        "wasm/qcms_bg.wasm",
        "application/wasm",
        include_bytes!("../../../assets/ocr/wasm/qcms_bg.wasm"),
    ),
    (
        "ben.traineddata",
        "application/octet-stream",
        include_bytes!("../../../assets/ocr/ben.traineddata"),
    ),
    // Tesseract's own Bangla model: reads old print and archive scans better than the fine-tuned
    // one, which reads modern fonts better; Tesseract tries both on each unsure word
    // Tesseract's model fine-tuned on real scans (Mozhi-Bengali book pages, CC BY 4.0, and the
    // British Library's 1713-1914 Bengali books, public domain): the main reader for real pages
    (
        "benreal.traineddata",
        "application/octet-stream",
        include_bytes!("../../../assets/ocr/benreal.traineddata"),
    ),
    (
        "benold.traineddata",
        "application/octet-stream",
        include_bytes!("../../../assets/ocr/benold.traineddata"),
    ),
];

/// A file under `/ocr/`: its content type and bytes.
pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    let name = path.strip_prefix("/ocr/")?;
    FILES
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, ct, b)| (*ct, *b))
}

/// `POST /api/ocr/correct`: the cleaned text, every word that was changed, and what was left out
/// as pictures or smudges, as JSON.
pub fn correct(body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let junk = v["junk"].as_bool().unwrap_or(true);
    let (text, dropped) = match v["lines"].as_array() {
        Some(lines) => {
            let lines: Vec<Option<Vec<ekushe_core::ocr::OcrWord>>> = lines
                .iter()
                .map(|l| {
                    l.as_array().map(|ws| {
                        ws.iter()
                            .filter_map(|w| {
                                Some((
                                    w[0].as_str()?.to_string(),
                                    w[1].as_f64().unwrap_or(0.0) as f32,
                                ))
                            })
                            .collect()
                    })
                })
                .collect();
            if junk {
                ekushe_core::ocr::clean_lines(&lines)
            } else {
                let text = lines
                    .iter()
                    .map(|l| match l {
                        Some(ws) => ws
                            .iter()
                            .map(|(w, _)| w.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                        None => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                (text, Vec::new())
            }
        }
        None => (
            v["text"].as_str().unwrap_or_default().to_string(),
            Vec::new(),
        ),
    };
    let r = ekushe_core::ocr::correct(&text);
    serde_json::json!({ "text": r.text, "changes": r.changes, "dropped": dropped }).to_string()
}

/// `POST /api/ocr/textlayer`: whether a PDF page's own text can be used instead of reading it.
pub fn text_layer(body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    match ekushe_core::ocr::text_layer(v["text"].as_str().unwrap_or_default()) {
        Some(t) => serde_json::json!({ "use": true, "text": t.text, "kind": t.kind }).to_string(),
        None => r#"{"use":false}"#.into(),
    }
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        if c == b'=' {
            break;
        }
        if c.is_ascii_whitespace() {
            continue;
        }
        let v = T.iter().position(|&t| t == c)? as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

/// `POST /api/ocr/sample`: keep a page as read, or its corrected text, for training later.
pub fn save_sample(body: &str) -> String {
    let fail = |m: &str| serde_json::json!({ "ok": false, "error": m }).to_string();
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let id = v["id"].as_str().unwrap_or_default();
    if id.is_empty() || id.len() > 40 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return fail("bad id");
    }
    let dir = ekushe_core::config::Config::default_dir()
        .join("ocr-samples")
        .join(id);
    if std::fs::create_dir_all(&dir).is_err() {
        return fail("can't create the folder");
    }
    let write = |name: &str, data: &[u8]| std::fs::write(dir.join(name), data).is_ok();
    let ok = if let Some(jpg) = v["jpg"].as_str() {
        let page = v["page"].as_u64().unwrap_or(1).min(9999);
        match base64_decode(jpg) {
            Some(bytes) if bytes.starts_with(&[0xFF, 0xD8]) => {
                write(&format!("page-{page:04}.jpg"), &bytes)
                    && write(
                        &format!("page-{page:04}.json"),
                        v["lines"].to_string().as_bytes(),
                    )
            }
            _ => return fail("not a JPEG picture"),
        }
    } else {
        write(
            "corrected.txt",
            v["corrected"].as_str().unwrap_or_default().as_bytes(),
        ) && write(
            "original.txt",
            v["original"].as_str().unwrap_or_default().as_bytes(),
        )
    };
    if !ok {
        return fail("can't write the files");
    }
    serde_json::json!({ "ok": true, "dir": dir.to_string_lossy() }).to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_decodes() {
        assert_eq!(super::base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(super::base64_decode("/9j/").unwrap(), [0xFF, 0xD8, 0xFF]);
        assert!(super::base64_decode("a*b").is_none());
    }
}
