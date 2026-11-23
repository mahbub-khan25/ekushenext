//! The Bengali fonts that ship inside Ekushe (from `assets/fonts/`): served to Studio at `/fonts/<file>`,
//! and on Windows also registered for the app and installed for the user. Shared by the Windows tray app
//! and the Linux/macOS Studio (`ekushe-ffi`), so both carry the same list.
//!
//! Shishir Bangla (four weights) is the default Bengali font, [`ekushe_core::config::DEFAULT_BENGALI_FONT`].

/// One bundled font file.
pub struct BundledFont {
    /// File name, in `assets/fonts/`, in Studio's `/fonts/` and in the font folder it is installed to
    pub file: &'static str,
    /// The name Windows lists it under (`HKCU\...\Fonts` value name)
    #[cfg_attr(not(windows), allow(dead_code))]
    pub windows_name: &'static str,
    pub bytes: &'static [u8],
}

macro_rules! font {
    ($file:literal, $name:literal) => {
        BundledFont {
            file: $file,
            windows_name: $name,
            bytes: include_bytes!(concat!("../../../assets/fonts/", $file)),
        }
    };
}

/// Every bundled font, the default family first.
pub static FONTS: &[BundledFont] = &[
    font!("ShishirBangla-Regular.ttf", "Shishir Bangla (TrueType)"),
    font!(
        "ShishirBangla-Medium.ttf",
        "Shishir Bangla Medium (TrueType)"
    ),
    font!(
        "ShishirBangla-SemiBold.ttf",
        "Shishir Bangla SemiBold (TrueType)"
    ),
    font!("ShishirBangla-Bold.ttf", "Shishir Bangla Bold (TrueType)"),
    font!("Kalpurush.ttf", "Kalpurush (TrueType)"),
    font!("SolaimanLipi.ttf", "SolaimanLipi (TrueType)"),
    font!("SiyamRupali.ttf", "Siyam Rupali (TrueType)"),
    font!("HindSiliguri-Regular.ttf", "Hind Siliguri (TrueType)"),
    font!("HindSiliguri-Bold.ttf", "Hind Siliguri Bold (TrueType)"),
];

/// The fonts that came with Ekushe before Shishir Bangla (what an install without a record of its own
/// fonts is taken to have installed).
#[cfg_attr(not(windows), allow(dead_code))]
pub const LEGACY_FILES: &[&str] = &[
    "Kalpurush.ttf",
    "SolaimanLipi.ttf",
    "SiyamRupali.ttf",
    "HindSiliguri-Regular.ttf",
    "HindSiliguri-Bold.ttf",
];

/// Shishir Bangla's licence (SIL OFL 1.1, with its upstream credits), shipped next to the font files
#[cfg_attr(not(windows), allow(dead_code))]
pub const SHISHIR_BANGLA_LICENCE: (&str, &str) = (
    "OFL-1.1-ShishirBangla.txt",
    include_str!("../../../assets/fonts/LICENSES/OFL-1.1-ShishirBangla.txt"),
);

/// The bytes of the bundled font file `name` (for Studio's `/fonts/<name>`).
pub fn font_file(name: &str) -> Option<&'static [u8]> {
    FONTS.iter().find(|f| f.file == name).map(|f| f.bytes)
}

/// A TrueType font's own version: its `head` table's font revision (16.16 fixed point), or `None`
/// when `bytes` is not a readable font.
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub fn font_revision(bytes: &[u8]) -> Option<u32> {
    let be32 = |at: usize| -> Option<u32> {
        Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    };
    let tables = u16::from_be_bytes(bytes.get(4..6)?.try_into().ok()?) as usize;
    (0..tables).find_map(|i| {
        let rec = 12 + 16 * i;
        if bytes.get(rec..rec + 4)? != b"head" {
            return None;
        }
        be32(be32(rec + 8)? as usize + 4)
    })
}

/// Whether a font file Ekushe installed earlier (`installed`) should be replaced by the bundled one:
/// it is an older version, or no longer a readable font. The same or a newer version is left alone.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn needs_update(installed: &[u8], bundled: &[u8]) -> bool {
    match (font_revision(installed), font_revision(bundled)) {
        (Some(have), Some(new)) => have < new,
        (None, Some(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod font_tests {
    use super::*;

    /// `s` as the UTF-16 big-endian bytes a TrueType name table stores Windows names in
    fn utf16be(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(|u| u.to_be_bytes()).collect()
    }

    #[test]
    fn shishir_bangla_is_bundled_in_four_weights_and_first() {
        let family = ekushe_core::config::DEFAULT_BENGALI_FONT;
        assert_eq!(family, "Shishir Bangla");
        let files: Vec<&str> = FONTS.iter().take(4).map(|f| f.file).collect();
        assert_eq!(
            files,
            [
                "ShishirBangla-Regular.ttf",
                "ShishirBangla-Medium.ttf",
                "ShishirBangla-SemiBold.ttf",
                "ShishirBangla-Bold.ttf"
            ]
        );
        let name = utf16be(family);
        for f in &FONTS[..4] {
            // A TrueType font (version 1.0), whose name table carries the family name
            assert_eq!(
                &f.bytes[..4],
                &[0, 1, 0, 0],
                "{} is not a TrueType font",
                f.file
            );
            assert!(
                f.bytes.windows(name.len()).any(|w| w == name.as_slice()),
                "{} does not name the family {family}",
                f.file
            );
            assert!(f.windows_name.starts_with("Shishir Bangla"));
            assert_eq!(font_file(f.file), Some(f.bytes));
        }
    }

    #[test]
    fn every_bundled_font_has_its_own_file_and_windows_name() {
        for (i, a) in FONTS.iter().enumerate() {
            assert!(a.file.ends_with(".ttf") && a.windows_name.ends_with(" (TrueType)"));
            for b in &FONTS[i + 1..] {
                assert!(a.file != b.file && a.windows_name != b.windows_name);
            }
        }
        for f in LEGACY_FILES {
            assert!(font_file(f).is_some(), "{f} is no longer bundled");
        }
        assert_eq!(font_file("../Cargo.toml"), None);
    }

    /// `bytes` with its `head` table's font revision set to `rev`
    fn with_revision(bytes: &[u8], rev: u32) -> Vec<u8> {
        let mut b = bytes.to_vec();
        let n = u16::from_be_bytes([b[4], b[5]]) as usize;
        for i in 0..n {
            let rec = 12 + 16 * i;
            if &b[rec..rec + 4] == b"head" {
                let off = u32::from_be_bytes(b[rec + 8..rec + 12].try_into().unwrap()) as usize;
                b[off + 4..off + 8].copy_from_slice(&rev.to_be_bytes());
            }
        }
        b
    }

    #[test]
    fn a_font_is_replaced_only_by_a_newer_version_of_itself() {
        let bundled = font_file("ShishirBangla-Regular.ttf").unwrap();
        // 0.003 is revision 0x00C5 in 16.16 fixed point (0.0030)
        assert_eq!(font_revision(bundled), Some(0xC5));
        assert_eq!(font_revision(b"not a font"), None);
        // An older Shishir Bangla Ekushe installed is replaced
        let older = with_revision(bundled, 0x83);
        assert_eq!(font_revision(&older), Some(0x83));
        assert!(needs_update(&older, bundled));
        // A broken copy too
        assert!(needs_update(b"half a fo", bundled));
        // The same file, or a newer one (from a newer Ekushe, or installed by hand), is left alone
        assert!(!needs_update(bundled, bundled));
        assert!(!needs_update(&with_revision(bundled, 0x200), bundled));
        // Same version number, other bytes: not replaced (a changed font gets a new version)
        assert!(!needs_update(&with_revision(bundled, 0xC5)[..], &{
            let mut b = bundled.to_vec();
            let last = b.len() - 1;
            b[last] ^= 1;
            b
        }));
    }
}
