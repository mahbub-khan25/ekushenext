//! The Bengali fonts that ship with Ekushe (from `assets/fonts/`): served to Studio at `/fonts/<file>`,
//! installed for the user on Windows and macOS, and the rules for replacing an older Shishir Bangla.
//! Shared by the Windows tray app and the Linux/macOS Studio (`ekushe-ffi`), so both carry one list.
//!
//! Shishir Bangla (four weights) is the default Bengali font, [`ekushe_core::config::DEFAULT_BENGALI_FONT`].
//! The Windows app carries every font inside it (it is its own installer). On Linux and macOS the
//! packages install the font files, and Studio reads them from there; only Shishir Bangla is also
//! built in, so Studio's default font works even without them.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// One bundled font file.
pub struct BundledFont {
    /// File name in `assets/fonts/` and in Studio's `/fonts/`
    pub file: &'static str,
    /// The name Windows lists it under (`HKCU\...\Fonts` value name)
    #[cfg_attr(not(windows), allow(dead_code))]
    pub windows_name: &'static str,
    /// The file itself, built in (empty when this build reads it from disk instead)
    pub bytes: &'static [u8],
    /// Installed under a name with its version (`ShishirBangla-Regular-0.003.ttf`), so a newer
    /// version is a new file that Windows can take while the old one is still in use
    pub versioned: bool,
}

macro_rules! default_font {
    ($file:literal, $name:literal) => {
        BundledFont {
            file: $file,
            windows_name: $name,
            bytes: include_bytes!(concat!("../../../assets/fonts/", $file)),
            versioned: true,
        }
    };
}

#[cfg(windows)]
macro_rules! other_font_bytes {
    ($file:literal) => {
        include_bytes!(concat!("../../../assets/fonts/", $file))
    };
}
#[cfg(not(windows))]
macro_rules! other_font_bytes {
    ($file:literal) => {
        &[]
    };
}

macro_rules! other_font {
    ($file:literal, $name:literal) => {
        BundledFont {
            file: $file,
            windows_name: $name,
            bytes: other_font_bytes!($file),
            versioned: false,
        }
    };
}

/// Every bundled font, the default family first.
pub static FONTS: &[BundledFont] = &[
    default_font!("ShishirBangla-Regular.ttf", "Shishir Bangla (TrueType)"),
    default_font!(
        "ShishirBangla-Medium.ttf",
        "Shishir Bangla Medium (TrueType)"
    ),
    default_font!(
        "ShishirBangla-SemiBold.ttf",
        "Shishir Bangla SemiBold (TrueType)"
    ),
    default_font!("ShishirBangla-Bold.ttf", "Shishir Bangla Bold (TrueType)"),
    other_font!("Kalpurush.ttf", "Kalpurush (TrueType)"),
    other_font!("SolaimanLipi.ttf", "SolaimanLipi (TrueType)"),
    other_font!("SiyamRupali.ttf", "Siyam Rupali (TrueType)"),
    other_font!("HindSiliguri-Regular.ttf", "Hind Siliguri (TrueType)"),
    other_font!("HindSiliguri-Bold.ttf", "Hind Siliguri Bold (TrueType)"),
];

/// The fonts Ekushe installed before it kept a list of them (Windows): an install from then is taken
/// to have installed these.
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

/// Where this build looks for font files on disk (Linux and macOS): next to the program (the
/// packages, the AppImage, `Ekushe.app/Contents/Resources/Fonts`), the system's folders, and the
/// source tree for a build that runs from it.
#[cfg_attr(windows, allow(dead_code))]
fn font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        for rel in [
            "../Resources/Fonts",
            "../share/ekushe/fonts",
            "../share/fonts/shishir-bangla",
            "../share/fonts/truetype/shishir-bangla",
            "../../share/ekushe/fonts",
            "../../share/fonts/shishir-bangla",
            "../../share/fonts/truetype/shishir-bangla",
        ] {
            dirs.push(exe_dir.join(rel));
        }
    }
    for d in [
        "/usr/share/ekushe/fonts",
        "/usr/share/fonts/shishir-bangla",
        "/usr/share/fonts/truetype/shishir-bangla",
        "/usr/local/share/ekushe/fonts",
        "/usr/local/share/fonts/shishir-bangla",
    ] {
        dirs.push(PathBuf::from(d));
    }
    dirs.push(PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/fonts"
    )));
    dirs
}

/// The bundled font file `name` (for Studio's `/fonts/<name>`): only names in [`FONTS`]. On Linux and
/// macOS read from the installed files, the built-in copy only when they are missing.
pub fn font_file(name: &str) -> Option<Cow<'static, [u8]>> {
    let font = FONTS.iter().find(|f| f.file == name)?;
    #[cfg(not(windows))]
    for dir in font_dirs() {
        if let Ok(bytes) = std::fs::read(dir.join(font.file)) {
            return Some(Cow::Owned(bytes));
        }
    }
    (!font.bytes.is_empty()).then_some(Cow::Borrowed(font.bytes))
}

/// A TrueType font's own version: its `head` table's font revision (16.16 fixed point), or `None`
/// when `bytes` is not a readable font.
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
pub fn needs_update(installed: &[u8], bundled: &[u8]) -> bool {
    match (font_revision(installed), font_revision(bundled)) {
        (Some(have), Some(new)) => have < new,
        (None, Some(_)) => true,
        _ => false,
    }
}

/// The version in a revision, as the font's makers write it: 0x00C5 is "0.003"
fn version_text(rev: u32) -> String {
    format!("{:.3}", rev as f64 / 65536.0)
}

impl BundledFont {
    /// The file name it is installed under in the user's font folder (Windows)
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn install_name(&self) -> String {
        match (self.versioned, font_revision(self.bytes)) {
            (true, Some(rev)) => {
                let stem = self.file.trim_end_matches(".ttf");
                format!("{stem}-{}.ttf", version_text(rev))
            }
            _ => self.file.to_string(),
        }
    }

    /// Whether `name` (a file in the font folder) is this font, under any version
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn is_copy(&self, name: &str) -> bool {
        if name == self.file {
            return true;
        }
        let stem = self.file.trim_end_matches(".ttf");
        self.versioned
            && name
                .strip_prefix(stem)
                .and_then(|r| r.strip_prefix('-'))
                .and_then(|r| r.strip_suffix(".ttf"))
                .is_some_and(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'))
    }
}

/// What setup does with one bundled font in the user's font folder (Windows).
#[derive(Debug, PartialEq, Eq)]
pub struct FontPlan {
    /// The file Windows' font entry points at, or `None`: this font is the user's own, leave it
    pub use_file: Option<String>,
    /// Write the bundled font to `use_file` (it is not there yet)
    pub write: bool,
    /// Older copies Ekushe installed: take them out of use and remove them
    pub retire: Vec<String>,
}

/// Decide what to do with `font`. `ours`: the files Ekushe installed (its list); `present`: the
/// files in the user's font folder, each with its bytes; `registered`: the file the font's Windows
/// entry points at now, if any. A font the user installed (a file or an entry that isn't Ekushe's)
/// is never touched; an older Ekushe copy is replaced by a new file under this version's name; a
/// newer one, from a newer Ekushe, is kept.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn plan_font(
    font: &BundledFont,
    ours: &[String],
    present: &[(String, Vec<u8>)],
    registered: Option<&str>,
) -> FontPlan {
    let is_ours = |n: &str| ours.iter().any(|o| o.eq_ignore_ascii_case(n));
    let leave = FontPlan {
        use_file: None,
        write: false,
        retire: Vec::new(),
    };
    if registered.is_some_and(|r| !is_ours(r)) {
        return leave;
    }
    let target = font.install_name();
    if present.iter().any(|(n, _)| *n == target) && !is_ours(&target) {
        return leave;
    }
    let copies: Vec<&(String, Vec<u8>)> = present
        .iter()
        .filter(|(n, _)| font.is_copy(n) && is_ours(n))
        .collect();
    let bundled_rev = font_revision(font.bytes);
    // A newer version from a newer Ekushe stays in use
    let newest = copies
        .iter()
        .filter(|(n, _)| *n != target)
        .filter_map(|(n, b)| Some((n, font_revision(b)?)))
        .max_by_key(|(_, r)| *r);
    let (use_file, write) = match newest {
        Some((n, rev)) if bundled_rev.is_some_and(|b| rev > b) => (n.clone(), false),
        _ => {
            let here = copies.iter().find(|(n, _)| *n == target);
            let write = match here {
                Some((_, b)) => needs_update(b, font.bytes),
                None => true,
            };
            (target, write)
        }
    };
    let retire = copies
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| *n != use_file)
        .collect();
    FontPlan {
        use_file: Some(use_file),
        write,
        retire,
    }
}

/// Install the bundled Shishir Bangla files in `dst` (macOS: `/Library/Fonts`), keeping the list of
/// the files Ekushe put there in `record`. A file that is already there and not on the list is the
/// user's own and stays; one on the list is replaced only by a newer version. Returns the files
/// written.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn install_into(dst: &Path, record: &Path) -> std::io::Result<Vec<String>> {
    let mut listed: Vec<String> = std::fs::read_to_string(record)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let mut written = Vec::new();
    std::fs::create_dir_all(dst)?;
    for font in FONTS.iter().filter(|f| f.versioned) {
        let Some(bytes) = font_file(font.file) else {
            continue;
        };
        let path = dst.join(font.file);
        let ours = listed.iter().any(|l| l == font.file);
        let write = match std::fs::read(&path) {
            Ok(have) => ours && needs_update(&have, &bytes),
            Err(_) => !path.exists(),
        };
        if write {
            std::fs::write(&path, &bytes)?;
            written.push(font.file.to_string());
            if !ours {
                listed.push(font.file.to_string());
            }
        }
    }
    if let Some(dir) = record.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(record, listed.join("\n") + "\n")?;
    Ok(written)
}

#[cfg(test)]
mod font_tests {
    use super::*;

    /// `s` as the UTF-16 big-endian bytes a TrueType name table stores Windows names in
    fn utf16be(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(|u| u.to_be_bytes()).collect()
    }

    fn font(name: &str) -> &'static BundledFont {
        FONTS.iter().find(|f| f.file == name).unwrap()
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
            // Built in everywhere: a TrueType font (version 1.0) whose name table carries the family
            assert_eq!(&f.bytes[..4], &[0, 1, 0, 0], "{} is not TrueType", f.file);
            assert!(
                f.bytes.windows(name.len()).any(|w| w == name.as_slice()),
                "{} does not name the family {family}",
                f.file
            );
            assert!(f.windows_name.starts_with("Shishir Bangla") && f.versioned);
            // Studio serves the same font (from disk or built in)
            assert_eq!(font_file(f.file).as_deref(), Some(f.bytes));
        }
    }

    #[test]
    fn every_bundled_font_has_its_own_file_and_windows_name() {
        for (i, a) in FONTS.iter().enumerate() {
            assert!(a.file.ends_with(".ttf") && a.windows_name.ends_with(" (TrueType)"));
            for b in &FONTS[i + 1..] {
                assert!(a.file != b.file && a.windows_name != b.windows_name);
            }
            // Every font can be served (from the source tree in this test build)
            assert!(font_file(a.file).is_some(), "{} cannot be served", a.file);
        }
        for f in LEGACY_FILES {
            assert!(!font(f).versioned, "{f} is no longer bundled as before");
        }
        assert_eq!(font_file("../Cargo.toml"), None);
        assert_eq!(font_file("OFL-1.1-ShishirBangla.txt"), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn only_shishir_bangla_is_built_in_on_linux_and_macos() {
        for f in FONTS {
            assert_eq!(f.bytes.is_empty(), !f.versioned, "{}", f.file);
        }
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
        let bundled = font("ShishirBangla-Regular.ttf").bytes;
        // 0.003 is revision 0x00C5 in 16.16 fixed point (0.0030)
        assert_eq!(font_revision(bundled), Some(0xC5));
        assert_eq!(font_revision(b"not a font"), None);
        let older = with_revision(bundled, 0x83);
        assert_eq!(font_revision(&older), Some(0x83));
        assert!(needs_update(&older, bundled));
        assert!(needs_update(b"half a fo", bundled));
        assert!(!needs_update(bundled, bundled));
        assert!(!needs_update(&with_revision(bundled, 0x200), bundled));
    }

    #[test]
    fn shishir_bangla_is_installed_under_its_version() {
        assert_eq!(
            font("ShishirBangla-Regular.ttf").install_name(),
            "ShishirBangla-Regular-0.003.ttf"
        );
        assert_eq!(
            font("ShishirBangla-SemiBold.ttf").install_name(),
            "ShishirBangla-SemiBold-0.003.ttf"
        );
        assert_eq!(version_text(0x0001_0000), "1.000");
        let r = font("ShishirBangla-Regular.ttf");
        assert!(
            r.is_copy("ShishirBangla-Regular.ttf") && r.is_copy("ShishirBangla-Regular-0.002.ttf")
        );
        assert!(
            !r.is_copy("ShishirBangla-RegularX.ttf") && !r.is_copy("ShishirBangla-Bold-0.003.ttf")
        );
    }

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn setup_plans_each_font_safely() {
        let f = font("ShishirBangla-Regular.ttf");
        let new = "ShishirBangla-Regular-0.003.ttf".to_string();
        // A first install: write this version's file
        assert_eq!(
            plan_font(f, &[], &[], None),
            FontPlan {
                use_file: Some(new.clone()),
                write: true,
                retire: vec![]
            }
        );
        // An older one Ekushe installed (in use, under its old name): a new file, the old one retired
        let old = (
            "ShishirBangla-Regular.ttf".to_string(),
            with_revision(f.bytes, 0x83),
        );
        assert_eq!(
            plan_font(
                f,
                &names(&["ShishirBangla-Regular.ttf"]),
                std::slice::from_ref(&old),
                Some("ShishirBangla-Regular.ttf")
            ),
            FontPlan {
                use_file: Some(new.clone()),
                write: true,
                retire: names(&["ShishirBangla-Regular.ttf"])
            }
        );
        // This version already there: nothing to write
        let same = (new.clone(), f.bytes.to_vec());
        assert_eq!(
            plan_font(f, &names(&[&new]), std::slice::from_ref(&same), Some(&new)),
            FontPlan {
                use_file: Some(new.clone()),
                write: false,
                retire: vec![]
            }
        );
        // A newer one from a newer Ekushe stays (no going back to an older version)
        let newer = (
            "ShishirBangla-Regular-0.004.ttf".to_string(),
            with_revision(f.bytes, 0x106),
        );
        assert_eq!(
            plan_font(
                f,
                &names(&["ShishirBangla-Regular-0.004.ttf"]),
                &[newer],
                Some("ShishirBangla-Regular-0.004.ttf")
            ),
            FontPlan {
                use_file: Some("ShishirBangla-Regular-0.004.ttf".into()),
                write: false,
                retire: vec![]
            }
        );
        // The user installed Shishir Bangla themselves: an entry or a file that isn't Ekushe's
        let leave = FontPlan {
            use_file: None,
            write: false,
            retire: vec![],
        };
        assert_eq!(
            plan_font(
                f,
                &[],
                std::slice::from_ref(&old),
                Some("ShishirBangla-Regular.ttf")
            ),
            leave
        );
        assert_eq!(plan_font(f, &[], &[same], None), leave);
        // Not Ekushe's file under the old name, nothing registered: write ours, theirs untouched
        assert_eq!(
            plan_font(f, &[], &[old], None),
            FontPlan {
                use_file: Some(new),
                write: true,
                retire: vec![]
            }
        );
        // The other fonts keep their names; one the user had is left alone
        let k = font("Kalpurush.ttf");
        let theirs = ("Kalpurush.ttf".to_string(), b"x".to_vec());
        assert_eq!(
            plan_font(k, &[], std::slice::from_ref(&theirs), None),
            leave
        );
        assert_eq!(
            plan_font(
                k,
                &names(&["Kalpurush.ttf"]),
                &[theirs],
                Some("Kalpurush.ttf")
            )
            .use_file
            .as_deref(),
            Some("Kalpurush.ttf")
        );
    }

    /// A folder of its own, removed again when the test ends
    struct Folder(PathBuf);
    impl Folder {
        fn new(name: &str) -> Self {
            let d =
                std::env::temp_dir().join(format!("ekushe-fonts-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Folder(d)
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn macos_install_keeps_the_users_fonts_and_never_goes_back_a_version() {
        let dir = Folder::new("mac");
        let (dst, record) = (
            dir.0.join("Fonts"),
            dir.0.join("Ekushe/fonts-installed.txt"),
        );
        std::fs::create_dir_all(&dst).unwrap();
        let regular = font("ShishirBangla-Regular.ttf").bytes;
        // The user's own Bold, and an older Regular from an earlier Ekushe
        std::fs::write(dst.join("ShishirBangla-Bold.ttf"), b"mine").unwrap();
        std::fs::write(
            dst.join("ShishirBangla-Regular.ttf"),
            with_revision(regular, 0x83),
        )
        .unwrap();
        std::fs::create_dir_all(record.parent().unwrap()).unwrap();
        std::fs::write(&record, "ShishirBangla-Regular.ttf\n").unwrap();
        let written = install_into(&dst, &record).unwrap();
        assert_eq!(
            written,
            names(&[
                "ShishirBangla-Regular.ttf",
                "ShishirBangla-Medium.ttf",
                "ShishirBangla-SemiBold.ttf"
            ])
        );
        assert_eq!(
            std::fs::read(dst.join("ShishirBangla-Regular.ttf")).unwrap(),
            regular
        );
        assert_eq!(
            std::fs::read(dst.join("ShishirBangla-Bold.ttf")).unwrap(),
            b"mine"
        );
        let list = std::fs::read_to_string(&record).unwrap();
        assert!(list.lines().any(|l| l == "ShishirBangla-Medium.ttf"));
        assert!(
            !list.lines().any(|l| l == "ShishirBangla-Bold.ttf"),
            "the user's font listed"
        );
        // A newer Regular (from a newer Ekushe) is not replaced by this older one
        std::fs::write(
            dst.join("ShishirBangla-Regular.ttf"),
            with_revision(regular, 0x106),
        )
        .unwrap();
        assert!(install_into(&dst, &record).unwrap().is_empty());
        assert_eq!(
            font_revision(&std::fs::read(dst.join("ShishirBangla-Regular.ttf")).unwrap()),
            Some(0x106)
        );
    }
}
