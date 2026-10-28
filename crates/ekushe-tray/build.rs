use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ekushe.rc");
    println!("cargo:rerun-if-changed=../../.github/release/VERSION");
    println!("cargo:rerun-if-changed=ekushe.manifest");
    println!("cargo:rerun-if-changed=../../assets/icons/ekushe.ico");

    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let res_path = out_dir.join("ekushe.res");
    write_version_header(&out_dir.join("version.h"));

    if let Some(rc_exe) = find_rc_exe() {
        let status = Command::new(rc_exe)
            .args(["/nologo", "/i"])
            .arg(&out_dir)
            .arg("/fo")
            .arg(&res_path)
            .arg("ekushe.rc")
            .status();
        match status {
            Ok(st) if st.success() && res_path.exists() => {
                println!("cargo:rustc-link-arg-bins={}", res_path.display());
            }
            _ => println!(
                "cargo:warning=rc.exe failed: icon, version details and manifest not embedded"
            ),
        }
    } else {
        println!("cargo:warning=rc.exe not found: icon, version details and manifest not embedded");
    }
}

/// The program's version details (Properties → Details, Installed Apps) come from the release file,
/// so they always match the release: `v0.2.0-beta.1` → "0.2.0-beta.1" and 0,2,0,1 (a final release
/// gets 999 as its fourth number, so it sorts after its betas, as on Android).
fn write_version_header(path: &Path) {
    let release = std::fs::read_to_string("../../.github/release/VERSION").unwrap_or_default();
    let release = release.trim().trim_start_matches('v').to_string();
    let (main, pre) = match release.split_once('-') {
        Some((m, p)) => (m, Some(p)),
        None => (release.as_str(), None),
    };
    let mut n = main.split('.').map(|x| x.parse::<u32>().unwrap_or(0));
    let (a, b, c) = (
        n.next().unwrap_or(0),
        n.next().unwrap_or(0),
        n.next().unwrap_or(0),
    );
    let d = match pre {
        Some(p) => p
            .rsplit(|ch: char| !ch.is_ascii_digit())
            .next()
            .and_then(|x| x.parse::<u32>().ok())
            .unwrap_or(0),
        None => 999,
    };
    let flags = if pre.is_some() { "0x2L" } else { "0x0L" }; // VS_FF_PRERELEASE
    let header = format!(
        "#define EKUSHE_VERSION_NUM {a},{b},{c},{d}\n#define EKUSHE_VERSION_STR \"{release}\"\n#define EKUSHE_VERSION_FLAGS {flags}\n"
    );
    let _ = std::fs::write(path, header);
}

fn find_rc_exe() -> Option<PathBuf> {
    if let Ok(output) = Command::new("where.exe").arg("rc.exe").output() {
        if output.status.success() {
            if let Some(line) = String::from_utf8_lossy(&output.stdout).lines().next() {
                let p = PathBuf::from(line.trim());
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    let kits_root = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if let Ok(entries) = std::fs::read_dir(kits_root) {
        let mut candidates: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path().join("x64").join("rc.exe"))
            .filter(|p| p.exists())
            .collect();
        candidates.sort();
        if let Some(last) = candidates.pop() {
            return Some(last);
        }
    }
    None
}
