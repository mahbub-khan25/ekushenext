//! Other Bangla keyboard programs running next to Ekushe. Two programs that both watch the keyboard
//! turn each key into Bangla twice (doubled or wrong letters), and people blame whichever one they
//! were trying to use. Ekushe looks for them now and then and says plainly which one to close.

/// The name to show for a running program's file name, when it is another Bangla keyboard.
pub fn conflicting_name(exe: &str) -> Option<&'static str> {
    let exe = exe.to_ascii_lowercase();
    let stem = exe.strip_suffix(".exe").unwrap_or(&exe);
    if stem.contains("bijoy") {
        Some("বিজয় (Bijoy)")
    } else if stem.starts_with("avro") {
        Some("অভ্র (Avro Keyboard)")
    } else if stem.contains("lekhoni") {
        Some("লেখনী (Lekhoni)")
    } else if stem.contains("ridmik") {
        Some("রিদমিক (Ridmik)")
    } else {
        None
    }
}

/// The other Bangla keyboards running now, each named once.
#[cfg(windows)]
pub fn running() -> Vec<&'static str> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    let mut found = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return found;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snap, &mut entry) != 0;
        while ok {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let exe = String::from_utf16_lossy(&entry.szExeFile[..len]);
            if let Some(name) = conflicting_name(&exe) {
                if !found.contains(&name) {
                    found.push(name);
                }
            }
            ok = Process32NextW(snap, &mut entry) != 0;
        }
        CloseHandle(snap);
    }
    found
}

#[cfg(not(windows))]
pub fn running() -> Vec<&'static str> {
    Vec::new()
}

/// The note shown for the programs found (empty when none).
pub fn message(found: &[&str]) -> String {
    if found.is_empty() {
        return String::new();
    }
    format!(
        "{} চালু আছে। দুটি বাংলা কীবোর্ড একসাথে চললে অক্ষর দুবার বা ভুল আসতে পারে: একুশে ব্যবহারের সময় ওটি বন্ধ রাখুন।",
        found.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_bangla_keyboards_are_recognised_by_their_program_names() {
        assert_eq!(conflicting_name("Bijoy Bayanno.exe"), Some("বিজয় (Bijoy)"));
        assert_eq!(conflicting_name("BIJOY52.EXE"), Some("বিজয় (Bijoy)"));
        assert_eq!(
            conflicting_name("Avro Keyboard.exe"),
            Some("অভ্র (Avro Keyboard)")
        );
        assert_eq!(conflicting_name("ekushe.exe"), None);
        assert_eq!(conflicting_name("explorer.exe"), None);
        assert_eq!(conflicting_name("WINWORD.EXE"), None);
        assert!(message(&[]).is_empty());
        assert!(message(&["বিজয় (Bijoy)"]).starts_with("বিজয় (Bijoy) চালু আছে"));
    }
}
