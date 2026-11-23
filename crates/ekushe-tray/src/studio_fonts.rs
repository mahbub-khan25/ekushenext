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
}
