use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ekushe.rc");
    println!("cargo:rerun-if-changed=ekushe.manifest");
    println!("cargo:rerun-if-changed=../../assets/icons/ekushe.ico");

    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let res_path = out_dir.join("ekushe.res");

    if let Some(rc_exe) = find_rc_exe() {
        let status = Command::new(rc_exe)
            .args(["/nologo", "/fo"])
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
