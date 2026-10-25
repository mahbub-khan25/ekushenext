//! Linux: is Ekushe one of the user's keyboards, and add it when it isn't. Installing the package
//! makes the keyboard available; each desktop keeps its own list of keyboards, so Studio adds it
//! there for the user:
//!
//! - **GNOME** (Fedora, Ubuntu, Debian, Pop!_OS's GNOME, Budgie): GNOME's keyboard list
//!   (`org.gnome.desktop.input-sources`). Takes effect at once.
//! - **KDE Plasma**: IBus's keyboard list, plus IBus as Plasma's input method (on Wayland:
//!   System Settings → Keyboard → Virtual Keyboard; on X11: the distribution's input-method
//!   setting). Takes effect after logging out and in, when IBus wasn't the input method yet.
//! - **Xfce, Cinnamon, MATE, LXQt, LXDE** and other desktops that run IBus: IBus's keyboard list,
//!   and IBus as the input method where it wasn't.
//! - **COSMIC**: not yet. IBus doesn't work reliably on COSMIC's current release, so Studio shows
//!   the steps instead of changing anything.

use std::process::Command;

const EKUSHE_GNOME: &str = "('ibus', 'ekushe')";
const GNOME_KEY: [&str; 2] = ["org.gnome.desktop.input-sources", "sources"];
const IBUS_SCHEMA: &str = "org.freedesktop.ibus.general";
/// The IBus panel KWin starts as Plasma's input method on Wayland
const IBUS_WAYLAND_PANEL: &str =
    "/usr/share/applications/org.freedesktop.IBus.Panel.Wayland.Gtk3.desktop";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desktop {
    Gnome,
    Kde,
    Cosmic,
    /// Xfce, Cinnamon, MATE, LXQt and the rest: IBus's own keyboard list
    Other,
}

impl Desktop {
    pub fn name(self) -> &'static str {
        match self {
            Desktop::Gnome => "GNOME",
            Desktop::Kde => "KDE Plasma",
            Desktop::Cosmic => "COSMIC",
            Desktop::Other => "other",
        }
    }
}

/// Which desktop this is, from `XDG_CURRENT_DESKTOP` (e.g. "ubuntu:GNOME", "KDE", "COSMIC",
/// "XFCE", "X-Cinnamon", "Budgie:GNOME").
pub fn desktop_from(current_desktop: &str) -> Desktop {
    let parts: Vec<String> = current_desktop
        .split(':')
        .map(|p| p.trim().to_ascii_lowercase())
        .collect();
    let has = |n: &str| parts.iter().any(|p| p == n);
    if has("cosmic") {
        Desktop::Cosmic
    } else if has("kde") {
        Desktop::Kde
    } else if has("gnome") || has("unity") {
        Desktop::Gnome
    } else {
        Desktop::Other
    }
}

pub fn desktop() -> Desktop {
    desktop_from(&std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default())
}

fn wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|t| t == "wayland")
        .unwrap_or(false)
        || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn ok(cmd: &mut Command) -> bool {
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

fn out(cmd: &mut Command) -> Option<String> {
    let o = cmd.output().ok()?;
    o.status
        .success()
        .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn have(tool: &str) -> bool {
    ok(Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {tool}")))
}

/// A gsettings list value without its type prefix (`@a(ss) []` → `[]`), or `None` if unreadable.
fn gsettings_list(schema: &str, key: &str) -> Option<String> {
    let v = out(Command::new("gsettings").args(["get", schema, key]))?;
    let v = v
        .strip_prefix("@a(ss) ")
        .or_else(|| v.strip_prefix("@as "))
        .unwrap_or(&v)
        .to_string();
    (v.starts_with('[') && v.ends_with(']')).then_some(v)
}

fn list_inner(list: &str) -> &str {
    list.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim()
}

/// GNOME's keyboard list with Ekushe first (the one in use), then the user's own keyboards; an
/// English layout if there was none, so Super+Space always has a plain keyboard to go to.
pub fn gnome_with_ekushe(list: &str) -> String {
    if list.contains("'ekushe'") {
        return list.to_string();
    }
    match list_inner(list) {
        "" => format!("[{EKUSHE_GNOME}, ('xkb', 'us')]"),
        inner => format!("[{EKUSHE_GNOME}, {inner}]"),
    }
}

/// IBus's keyboard list (`preload-engines`) with Ekushe first, keeping the user's own.
pub fn ibus_with_ekushe(list: &str) -> String {
    if list.contains("'ekushe'") {
        return list.to_string();
    }
    match list_inner(list) {
        "" => "['ekushe', 'xkb:us::eng']".to_string(),
        inner => format!("['ekushe', {inner}]"),
    }
}

/// What Studio shows: is Ekushe added, can Studio add it here, and the steps to do it by hand.
#[derive(Debug, Clone)]
pub struct Status {
    pub desktop: &'static str,
    pub added: bool,
    pub can_add: bool,
    /// The steps in this desktop's own settings (Bangla), for when Studio can't do it
    pub steps: String,
}

impl Status {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "desktop": self.desktop, "added": self.added, "can_add": self.can_add, "steps": self.steps })
    }
}

impl Added {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "ok": self.ok, "relogin": self.relogin, "message": self.message })
    }
}

fn steps(d: Desktop) -> String {
    match d {
        Desktop::Gnome => "Settings → Keyboard → Input Sources → + → Bangla (Bangladesh) → Ekushe Bangla → Add।".into(),
        Desktop::Kde => "System Settings → Keyboard → Virtual Keyboard → IBus Wayland বেছে নিন, লগ আউট করে আবার ঢুকুন; তারপর টার্মিনালে ibus-setup → Input Method → Add → Bangla → Ekushe Bangla।".into(),
        Desktop::Cosmic => "COSMIC-এর বর্তমান সংস্করণে IBus ঠিকমতো চলে না, তাই একুশে এখনো COSMIC-এ ব্যবহার করা যাচ্ছে না। COSMIC-এ IBus-এর সমস্যা মিটলে একুশে এখানেও কাজ করবে; ততদিন GNOME বা KDE সেশনে লগ ইন করে একুশে ব্যবহার করুন।".into(),
        Desktop::Other => "টার্মিনালে ibus-setup চালান → Input Method → Add → Bangla → Ekushe Bangla। IBus চালু না থাকলে ডেস্কটপের ইনপুট-মেথড সেটিংসে (Ubuntu: im-config) IBus বেছে নিয়ে লগ আউট করে আবার ঢুকুন।".into(),
    }
}

/// `None` when this isn't a Linux desktop session at all (no settings to read).
pub fn status() -> Option<Status> {
    std::env::var_os("XDG_CURRENT_DESKTOP")?;
    let d = desktop();
    let added = match d {
        Desktop::Gnome => gsettings_list(GNOME_KEY[0], GNOME_KEY[1])?.contains("'ekushe'"),
        Desktop::Cosmic => false,
        Desktop::Kde | Desktop::Other => gsettings_list(IBUS_SCHEMA, "preload-engines")
            .map(|l| l.contains("'ekushe'"))
            .unwrap_or(false),
    };
    let can_add = match d {
        Desktop::Gnome => true,
        Desktop::Cosmic => false,
        Desktop::Kde | Desktop::Other => gsettings_list(IBUS_SCHEMA, "preload-engines").is_some(),
    };
    Some(Status {
        desktop: d.name(),
        added,
        can_add,
        steps: steps(d),
    })
}

/// The result of adding, for the message Studio shows.
#[derive(Debug, Clone)]
pub struct Added {
    pub ok: bool,
    /// Log out and in to finish (IBus wasn't this desktop's input method yet)
    pub relogin: bool,
    pub message: String,
}

fn fail(d: Desktop) -> Added {
    Added {
        ok: false,
        relogin: false,
        message: format!("নিজে থেকে যোগ করা গেল না। {}", steps(d)),
    }
}

/// Make IBus this session's input method where it isn't yet. Returns whether a re-login is
/// needed for it to take effect.
fn make_ibus_the_input_method(d: Desktop) -> bool {
    if d == Desktop::Kde && wayland() {
        // Plasma on Wayland: KWin starts the input method chosen in Virtual Keyboard
        let tool = ["kwriteconfig6", "kwriteconfig5"]
            .into_iter()
            .find(|t| have(t));
        let current = ["kreadconfig6", "kreadconfig5"]
            .into_iter()
            .find(|t| have(t))
            .and_then(|t| {
                out(Command::new(t).args([
                    "--file",
                    "kwinrc",
                    "--group",
                    "Wayland",
                    "--key",
                    "InputMethod",
                ]))
            })
            .unwrap_or_default();
        if current.contains("IBus") {
            return false;
        }
        if let Some(tool) = tool {
            if std::path::Path::new(IBUS_WAYLAND_PANEL).exists() {
                let set = ok(Command::new(tool).args([
                    "--file",
                    "kwinrc",
                    "--group",
                    "Wayland",
                    "--key",
                    "InputMethod",
                    IBUS_WAYLAND_PANEL,
                ]));
                // Ask KWin to read it again; a new login always does
                let _ = Command::new("qdbus")
                    .args(["org.kde.KWin", "/KWin", "reconfigure"])
                    .output();
                return set;
            }
        }
        return false;
    }
    // X11 desktops, and Wayland ones other than Plasma: the distribution's input-method switch
    if out(Command::new("pgrep").args(["-u", &whoami(), "-x", "ibus-daemon"])).is_some() {
        return false; // IBus is already running for this user
    }
    if have("im-config") {
        return ok(Command::new("im-config").args(["-n", "ibus"]));
    }
    if have("imsettings-switch") {
        return ok(Command::new("imsettings-switch").arg("ibus"));
    }
    false
}

fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| out(&mut Command::new("whoami")).unwrap_or_default())
}

/// Add Ekushe as the first keyboard on this desktop.
pub fn add() -> Added {
    let d = desktop();
    match d {
        Desktop::Cosmic => fail(d),
        Desktop::Gnome => {
            let Some(list) = gsettings_list(GNOME_KEY[0], GNOME_KEY[1]) else {
                return fail(d);
            };
            if !list.contains("'ekushe'") {
                // Make Ekushe the most recently used keyboard too, so GNOME switches to it at once
                let _ = Command::new("gsettings")
                    .args([
                        "set",
                        GNOME_KEY[0],
                        "mru-sources",
                        &format!("[{EKUSHE_GNOME}]"),
                    ])
                    .output();
                if !ok(Command::new("gsettings").args([
                    "set",
                    GNOME_KEY[0],
                    GNOME_KEY[1],
                    &gnome_with_ekushe(&list),
                ])) {
                    return fail(d);
                }
            }
            Added {
                ok: true,
                relogin: false,
                message: "একুশে যোগ হয়েছে ও চালু আছে। দুবার Shift চেপে বাংলা ⇄ English।".into(),
            }
        }
        Desktop::Kde | Desktop::Other => {
            let Some(list) = gsettings_list(IBUS_SCHEMA, "preload-engines") else {
                return fail(d);
            };
            let new = ibus_with_ekushe(&list);
            if !ok(Command::new("gsettings").args(["set", IBUS_SCHEMA, "preload-engines", &new])) {
                return fail(d);
            }
            let _ = Command::new("gsettings")
                .args(["set", IBUS_SCHEMA, "engines-order", &new])
                .output();
            let relogin = make_ibus_the_input_method(d);
            // Use it now where IBus is already running
            let now = !relogin && ok(Command::new("ibus").args(["engine", "ekushe"]));
            Added {
                ok: true,
                relogin,
                message: if relogin {
                    "একুশে যোগ হয়েছে। চালু করতে একবার লগ আউট করে আবার ঢুকুন; তারপর দুবার Shift চেপে বাংলা ⇄ English।".into()
                } else if now {
                    "একুশে যোগ হয়েছে ও চালু আছে। দুবার Shift চেপে বাংলা ⇄ English।".into()
                } else {
                    "একুশে যোগ হয়েছে। কাজ না করলে একবার লগ আউট করে আবার ঢুকুন।".into()
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktops_are_recognised() {
        assert_eq!(desktop_from("ubuntu:GNOME"), Desktop::Gnome);
        assert_eq!(desktop_from("GNOME"), Desktop::Gnome);
        assert_eq!(desktop_from("Budgie:GNOME"), Desktop::Gnome);
        assert_eq!(desktop_from("pop:GNOME"), Desktop::Gnome);
        assert_eq!(desktop_from("KDE"), Desktop::Kde);
        assert_eq!(desktop_from("COSMIC"), Desktop::Cosmic);
        assert_eq!(desktop_from("pop:COSMIC"), Desktop::Cosmic);
        assert_eq!(desktop_from("XFCE"), Desktop::Other);
        assert_eq!(desktop_from("X-Cinnamon"), Desktop::Other);
        assert_eq!(desktop_from("MATE"), Desktop::Other);
        assert_eq!(desktop_from("LXQt"), Desktop::Other);
    }

    #[test]
    fn ekushe_is_added_first() {
        assert_eq!(
            gnome_with_ekushe("[('xkb', 'us')]"),
            "[('ibus', 'ekushe'), ('xkb', 'us')]"
        );
        assert_eq!(
            gnome_with_ekushe("[]"),
            "[('ibus', 'ekushe'), ('xkb', 'us')]"
        );
        assert_eq!(
            gnome_with_ekushe("[('xkb', 'de'), ('ibus', 'mozc-jp')]"),
            "[('ibus', 'ekushe'), ('xkb', 'de'), ('ibus', 'mozc-jp')]"
        );
        let done = "[('ibus', 'ekushe'), ('xkb', 'us')]";
        assert_eq!(gnome_with_ekushe(done), done);
        assert_eq!(ibus_with_ekushe("[]"), "['ekushe', 'xkb:us::eng']");
        assert_eq!(
            ibus_with_ekushe("['xkb:de::ger']"),
            "['ekushe', 'xkb:de::ger']"
        );
        assert_eq!(ibus_with_ekushe("['ekushe']"), "['ekushe']");
    }
}
