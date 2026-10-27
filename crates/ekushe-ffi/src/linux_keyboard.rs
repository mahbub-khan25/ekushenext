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
//! - **COSMIC** (experimental): COSMIC's own apps can't use any input method yet, and its
//!   on-screen keyboard takes the compositor's input-method slot. GTK and Qt apps (Firefox,
//!   LibreOffice, ...) can still talk to IBus directly, so Studio adds Ekushe to IBus, starts IBus
//!   with the session (an autostart entry shown only on COSMIC). Nothing it changes reaches the
//!   user's GNOME or KDE sessions.

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

/// A gsettings list with every Ekushe entry removed (`[('ibus', 'ekushe'), ('xkb', 'us')]` →
/// `[('xkb', 'us')]`; `['ekushe', 'xkb:us::eng']` → `['xkb:us::eng']`). Works for GNOME's
/// `(ss)` tuples and IBus's plain strings alike.
pub fn without_ekushe(list: &str) -> String {
    let inner = list_inner(list);
    if inner.is_empty() {
        return "[]".to_string();
    }
    // Split at the commas between entries: GNOME entries are parenthesised tuples, IBus entries are
    // quoted strings; neither contains a comma outside its own parentheses/quotes
    let mut items: Vec<String> = Vec::new();
    let (mut depth, mut quoted, mut cur) = (0usize, false, String::new());
    for c in inner.chars() {
        match c {
            '\'' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth = depth.saturating_sub(1),
            ',' if !quoted && depth == 0 => {
                items.push(cur.trim().to_string());
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() {
        items.push(cur.trim().to_string());
    }
    let kept: Vec<String> = items
        .into_iter()
        .filter(|i| !i.contains("'ekushe'"))
        .collect();
    format!("[{}]", kept.join(", "))
}

/// GNOME's most-recently-used list with Ekushe first and the user's history kept.
pub fn gnome_mru_with_ekushe(mru: &str) -> String {
    gnome_with_ekushe(&without_ekushe(mru)).replace(", ('xkb', 'us')]", "]")
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
        Desktop::Cosmic => "COSMIC-এ (পরীক্ষামূলক): টার্মিনালে ibus-daemon --daemonize --xim চালান, তারপর ibus-setup → Input Method → Add → Bangla → Ekushe Bangla। Firefox, LibreOffice ও অন্য GTK/Qt অ্যাপে কাজ করবে; COSMIC-এর নিজের অ্যাপে (Text Editor, Terminal, Files) এখনো কোনো ইনপুট মেথড চলে না।".into(),
        Desktop::Other => "টার্মিনালে ibus-setup চালান → Input Method → Add → Bangla → Ekushe Bangla। IBus চালু না থাকলে ডেস্কটপের ইনপুট-মেথড সেটিংসে (Ubuntu: im-config) IBus বেছে নিয়ে লগ আউট করে আবার ঢুকুন।".into(),
    }
}

/// `None` when this isn't a Linux desktop session at all (no settings to read).
pub fn status() -> Option<Status> {
    std::env::var_os("XDG_CURRENT_DESKTOP")?;
    let d = desktop();
    let added = match d {
        Desktop::Gnome => gsettings_list(GNOME_KEY[0], GNOME_KEY[1])?.contains("'ekushe'"),
        Desktop::Cosmic => {
            cosmic_autostart().is_some_and(|p| p.exists())
                && gsettings_list(IBUS_SCHEMA, "preload-engines")
                    .is_some_and(|l| l.contains("'ekushe'"))
        }
        Desktop::Kde | Desktop::Other => gsettings_list(IBUS_SCHEMA, "preload-engines")
            .map(|l| l.contains("'ekushe'"))
            .unwrap_or(false),
    };
    let can_add = match d {
        Desktop::Gnome => true,
        Desktop::Cosmic => {
            have("ibus-daemon") && gsettings_list(IBUS_SCHEMA, "preload-engines").is_some()
        }
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
                setup_note("kwinrc_input_method", &current);
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
        // Remember what ~/.xinputrc said before (or that there was none), to put it back on remove
        let prev = xinputrc_run_im();
        setup_note("xinputrc", prev.as_deref().unwrap_or("absent"));
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

fn config_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
}

/// COSMIC: the autostart entry that starts IBus with the session.
fn cosmic_autostart() -> Option<std::path::PathBuf> {
    config_dir().map(|c| c.join("autostart/ekushe-ibus.desktop"))
}

const COSMIC_AUTOSTART: &str = "[Desktop Entry]
Type=Application
Name=IBus for Ekushe
Comment=Starts IBus so Ekushe can type Bangla in this COSMIC session (added by Ekushe Studio)
Exec=ibus-daemon --daemonize --xim
OnlyShowIn=COSMIC;
NoDisplay=true
";

/// The GTK/Qt variables that make apps talk to IBus directly (COSMIC has no other route yet).
const IM_VARS: [(&str, &str); 3] = [
    ("GTK_IM_MODULE", "ibus"),
    ("QT_IM_MODULE", "ibus"),
    ("XMODIFIERS", "@im=ibus"),
];

/// COSMIC (experimental): Ekushe in IBus's list, IBus started with the session, and the GTK/Qt
/// input-method variables where the system doesn't set them. Ekushe then types in GTK and Qt apps
/// (Firefox, LibreOffice, ...), not yet in COSMIC's own apps.
fn add_cosmic() -> Added {
    let d = Desktop::Cosmic;
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
    let Some(autostart) = cosmic_autostart() else {
        return fail(d);
    };
    if let Some(dir) = autostart.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::write(&autostart, COSMIC_AUTOSTART).is_err() {
        return fail(d);
    }
    // Apps reach IBus on COSMIC only when the system sets the GTK/Qt input-method variables (Pop!_OS
    // does). Ekushe doesn't set them itself: they would apply to the user's GNOME and KDE sessions too.
    let vars_missing = IM_VARS
        .iter()
        .any(|(k, v)| std::env::var(k).as_deref() != Ok(*v));
    // Start IBus now if it isn't running, and select Ekushe
    if out(Command::new("pgrep").args(["-u", &whoami(), "-x", "ibus-daemon"])).is_none() {
        let _ = Command::new("ibus-daemon")
            .args(["--daemonize", "--xim"])
            .status();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    let _ = Command::new("ibus").args(["engine", "ekushe"]).output();
    let limits = "এটা পরীক্ষামূলক: Firefox, LibreOffice ও অন্য GTK/Qt অ্যাপে কাজ করবে; COSMIC-এর নিজের অ্যাপে (Text Editor, Terminal, Files) এখনো না, কারণ COSMIC সেগুলোতে এখনো কোনো ইনপুট মেথড চালায় না।";
    let message = if vars_missing {
        format!("একুশে যোগ হয়েছে, তবে এই সিস্টেমে অ্যাপগুলো এখনো IBus ব্যবহার করতে বলা নেই (Pop!_OS নিজেই এটা করে দেয়; অন্য সিস্টেমে ডিস্ট্রিবিউশনের ইনপুট-মেথড সেটিংসে IBus বেছে নিন)। তারপর লগ আউট করে আবার ঢুকুন। {limits}")
    } else {
        format!(
            "একুশে যোগ হয়েছে। একবার লগ আউট করে আবার ঢুকুন; তারপর দুবার Shift চেপে বাংলা ⇄ English। {limits}"
        )
    };
    Added {
        ok: true,
        relogin: true,
        message,
    }
}

/// beta.22 wrote GTK_IM_MODULE / QT_IM_MODULE / XMODIFIERS to `environment.d` for COSMIC; those
/// would also apply to GNOME and KDE sessions. Remove that file (only Ekushe's own).
pub fn remove_old_cosmic_env() {
    let Some(file) = config_dir().map(|c| c.join("environment.d/90-ekushe-ibus.conf")) else {
        return;
    };
    if std::fs::read_to_string(&file).is_ok_and(|t| t.starts_with("# Added by Ekushe Studio")) {
        let _ = std::fs::remove_file(file);
    }
}

/// Add Ekushe as the first keyboard on this desktop.
pub fn add() -> Added {
    let d = desktop();
    match d {
        Desktop::Cosmic => add_cosmic(),
        Desktop::Gnome => {
            let Some(list) = gsettings_list(GNOME_KEY[0], GNOME_KEY[1]) else {
                return fail(d);
            };
            if !list.contains("'ekushe'") {
                // Make Ekushe the most recently used keyboard too, so GNOME switches to it at once
                // (the user's own history stays behind it)
                let mru =
                    gsettings_list(GNOME_KEY[0], "mru-sources").unwrap_or_else(|| "[]".into());
                let _ = Command::new("gsettings")
                    .args([
                        "set",
                        GNOME_KEY[0],
                        "mru-sources",
                        &gnome_mru_with_ekushe(&mru),
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

/// `~/.config/Ekushe/keyboard-setup.txt`: what "add keyboard" changed outside the keyboard lists
/// (one `key=value` per line), so "remove keyboard" can put exactly that back.
fn setup_notes_path() -> std::path::PathBuf {
    ekushe_core::config::Config::default_dir().join("keyboard-setup.txt")
}

fn setup_notes() -> Vec<(String, String)> {
    std::fs::read_to_string(setup_notes_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            l.split_once('=')
                .map(|(k, v)| (k.trim().into(), v.trim().into()))
        })
        .collect()
}

/// Record `key=value` (the first value wins: the state before Ekushe changed anything).
fn setup_note(key: &str, value: &str) {
    let mut notes = setup_notes();
    if notes.iter().any(|(k, _)| k == key) {
        return;
    }
    notes.push((key.into(), value.into()));
    let path = setup_notes_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let text: String = notes.iter().map(|(k, v)| format!("{k}={v}\n")).collect();
    let _ = std::fs::write(path, text);
}

/// The input method `~/.xinputrc` selects (`run_im ibus` → `ibus`), or `None` when there is no file.
fn xinputrc_run_im() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    let text = std::fs::read_to_string(std::path::Path::new(&home).join(".xinputrc")).ok()?;
    Some(
        text.lines()
            .map(str::trim)
            .find_map(|l| l.strip_prefix("run_im ").map(|v| v.trim().to_string()))
            .unwrap_or_default(),
    )
}

/// Take Ekushe out of this desktop's keyboards again, and put back the input-method setting
/// "add keyboard" changed (only what it changed: a setting the user made is left alone).
/// Safe to run before uninstalling the package, and after.
pub fn remove() -> Added {
    let d = desktop();
    let mut ok_all = true;
    let mut relogin = false;
    match d {
        Desktop::Gnome => {
            if let Some(list) = gsettings_list(GNOME_KEY[0], GNOME_KEY[1]) {
                if list.contains("'ekushe'") {
                    ok_all &= ok(Command::new("gsettings").args([
                        "set",
                        GNOME_KEY[0],
                        GNOME_KEY[1],
                        &without_ekushe(&list),
                    ]));
                }
            }
            if let Some(mru) = gsettings_list(GNOME_KEY[0], "mru-sources") {
                if mru.contains("'ekushe'") {
                    let _ = Command::new("gsettings")
                        .args(["set", GNOME_KEY[0], "mru-sources", &without_ekushe(&mru)])
                        .output();
                }
            }
        }
        Desktop::Kde | Desktop::Other | Desktop::Cosmic => {
            for key in ["preload-engines", "engines-order"] {
                if let Some(list) = gsettings_list(IBUS_SCHEMA, key) {
                    if list.contains("'ekushe'") {
                        ok_all &= ok(Command::new("gsettings").args([
                            "set",
                            IBUS_SCHEMA,
                            key,
                            &without_ekushe(&list),
                        ]));
                    }
                }
            }
            // IBus may still have Ekushe selected: switch to the first remaining engine
            if let Some(list) = gsettings_list(IBUS_SCHEMA, "preload-engines") {
                let first = list_inner(&list)
                    .split(',')
                    .next()
                    .map(|e| e.trim().trim_matches('\'').to_string())
                    .unwrap_or_default();
                if !first.is_empty() {
                    let _ = Command::new("ibus").args(["engine", &first]).output();
                }
            }
        }
    }
    // What "add" changed beyond the keyboard lists
    for (key, value) in setup_notes() {
        match key.as_str() {
            "xinputrc" if have("im-config") => {
                // Only if it still says ibus (the user hasn't chosen something else since)
                if xinputrc_run_im().as_deref() == Some("ibus") {
                    if value == "absent" {
                        if let Some(home) = std::env::var_os("HOME") {
                            let _ =
                                std::fs::remove_file(std::path::Path::new(&home).join(".xinputrc"));
                        }
                    } else if !value.is_empty() {
                        let _ = Command::new("im-config").args(["-n", &value]).output();
                    }
                    relogin = true;
                }
            }
            "kwinrc_input_method" => {
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
                if let Some(tool) = tool {
                    if current.contains("IBus") {
                        let mut args = vec![
                            "--file",
                            "kwinrc",
                            "--group",
                            "Wayland",
                            "--key",
                            "InputMethod",
                        ];
                        if value.is_empty() {
                            args.push("--delete");
                        } else {
                            args.push(&value);
                        }
                        let _ = Command::new(tool).args(&args).output();
                        let _ = Command::new("qdbus")
                            .args(["org.kde.KWin", "/KWin", "reconfigure"])
                            .output();
                        relogin = true;
                    }
                }
            }
            _ => {}
        }
    }
    let _ = std::fs::remove_file(setup_notes_path());
    // COSMIC: the autostart entry and beta.22's environment file, if they are Ekushe's own
    if let Some(autostart) = cosmic_autostart() {
        if std::fs::read_to_string(&autostart).is_ok_and(|t| t.contains("Ekushe")) {
            let _ = std::fs::remove_file(autostart);
        }
    }
    remove_old_cosmic_env();
    Added {
        ok: ok_all,
        relogin,
        message: if !ok_all {
            format!("কিবোর্ডের তালিকা থেকে একুশে বাদ দেওয়া গেল না। {}", steps(d))
        } else if relogin {
            "একুশে আপনার কিবোর্ডের তালিকা থেকে বাদ গেছে; ইনপুট-মেথড সেটিং আগের মতো করা হয়েছে। একবার লগ আউট করে আবার ঢুকুন।".into()
        } else {
            "একুশে আপনার কিবোর্ডের তালিকা থেকে বাদ গেছে।".into()
        },
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

    #[test]
    fn ekushe_is_removed_and_the_rest_kept() {
        assert_eq!(
            without_ekushe("[('ibus', 'ekushe'), ('xkb', 'us')]"),
            "[('xkb', 'us')]"
        );
        assert_eq!(
            without_ekushe("[('xkb', 'de'), ('ibus', 'ekushe'), ('ibus', 'mozc-jp')]"),
            "[('xkb', 'de'), ('ibus', 'mozc-jp')]"
        );
        assert_eq!(without_ekushe("[('ibus', 'ekushe')]"), "[]");
        assert_eq!(without_ekushe("[]"), "[]");
        assert_eq!(without_ekushe("[('xkb', 'us')]"), "[('xkb', 'us')]");
        assert_eq!(
            without_ekushe("['ekushe', 'xkb:us::eng']"),
            "['xkb:us::eng']"
        );
        assert_eq!(
            without_ekushe("['xkb:us::eng', 'ekushe']"),
            "['xkb:us::eng']"
        );
        assert_eq!(without_ekushe("['ekushe']"), "[]");
        // adding after removing gives the original back
        let l = "[('ibus', 'ekushe'), ('xkb', 'us')]";
        assert_eq!(gnome_with_ekushe(&without_ekushe(l)), l);
    }

    #[test]
    fn mru_keeps_the_users_history() {
        assert_eq!(
            gnome_mru_with_ekushe("[('xkb', 'us'), ('xkb', 'de')]"),
            "[('ibus', 'ekushe'), ('xkb', 'us'), ('xkb', 'de')]"
        );
        assert_eq!(gnome_mru_with_ekushe("[]"), "[('ibus', 'ekushe')]");
        assert_eq!(
            gnome_mru_with_ekushe("[('xkb', 'de'), ('ibus', 'ekushe')]"),
            "[('ibus', 'ekushe'), ('xkb', 'de')]"
        );
    }
}
