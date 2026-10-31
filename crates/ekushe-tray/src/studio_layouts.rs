//! Studio's keyboard-layout calls, shared by the Windows app (this crate) and by Studio on macOS and
//! Linux (`crates/ekushe-ffi`, which includes this file):
//!
//! - `GET  /api/layouts`             the layouts to choose from (built-in and imported)
//! - `POST /api/layouts/import`      body: a layout file's text; checked, then kept
//! - `POST /api/layouts/remove`      body: `{"id": "..."}`
//! - `GET  /api/layouts/export?id=`  a layout as a layout file (a starting point for your own)
//!
//! The layout setting itself travels as one value: a built-in layout's name (`UniJoy`) or
//! `Custom:<id>` for an imported one.

use ekushe_core::config::Config;
use ekushe_core::layout::{self, FixedLayout, LayoutId};

/// The built-in layouts, in the order Studio and the menus show them.
pub use ekushe_core::layout::BUILT_IN_LAYOUTS as BUILT_IN;
/// The setting's value for a configuration (`UniJoy`, `Custom:my-layout`), and applying one
pub use ekushe_core::layout::{apply_setting, setting_value};

/// The name of the layout in use, and its short label (for the tray, menus and panels).
pub fn active_names(cfg: &Config) -> (String, String) {
    match FixedLayout::for_config(cfg) {
        Some(l) => (l.name, l.short),
        None => (
            LayoutId::AvroPhonetic.display_name().to_string(),
            "বাং".into(),
        ),
    }
}

fn json_list(cfg: &Config) -> String {
    let mut items: Vec<serde_json::Value> = BUILT_IN
        .iter()
        .map(|l| {
            serde_json::json!({
                "value": format!("{l:?}"),
                "name": l.display_name(),
                "custom": false,
            })
        })
        .collect();
    for c in layout::list_custom_layouts() {
        items.push(serde_json::json!({
            "value": format!("Custom:{}", c.id),
            "id": c.id,
            "name": c.name,
            "short": c.short,
            "author": c.author,
            "description": c.description,
            "custom": true,
        }));
    }
    serde_json::json!({ "layouts": items, "active": setting_value(cfg) }).to_string()
}

/// Outcome of a layout call: the JSON (or file) to send, and whether the configuration changed so
/// the engine has to load it again.
pub struct Reply {
    pub status: &'static str,
    pub content_type: &'static str,
    pub body: String,
    pub config_changed: bool,
}

fn ok_json(body: String, config_changed: bool) -> Reply {
    Reply {
        status: "200 OK",
        content_type: "application/json; charset=utf-8",
        body,
        config_changed,
    }
}

fn error(msg: &str) -> Reply {
    Reply {
        status: "400 Bad Request",
        content_type: "application/json; charset=utf-8",
        body: serde_json::json!({ "error": msg }).to_string(),
        config_changed: false,
    }
}

/// Handle a layout call (`None` when the path isn't one). `cfg` is the live configuration; when a
/// call changes it (removing the layout in use), it is updated and saved here.
pub fn handle(method: &str, path: &str, body: &str, cfg: &mut Config) -> Option<Reply> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    match (method, path) {
        ("GET", "/api/layouts") => Some(ok_json(json_list(cfg), false)),
        ("POST", "/api/layouts/import") => Some(match layout::import_layout(body) {
            Ok(info) => {
                // Re-importing the layout in use takes effect at once
                let reload = cfg.custom_layout.as_deref() == Some(info.id.as_str())
                    && cfg.active_layout == LayoutId::Custom;
                ok_json(
                    serde_json::json!({
                        "imported": info.name,
                        "value": format!("Custom:{}", info.id),
                        "list": serde_json::from_str::<serde_json::Value>(&json_list(cfg)).unwrap_or_default(),
                    })
                    .to_string(),
                    reload,
                )
            }
            Err(e) => error(&e),
        }),
        ("POST", "/api/layouts/remove") => Some({
            let id = serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| v["id"].as_str().map(str::to_string))
                .unwrap_or_default();
            match layout::remove_custom_layout(&id) {
                Ok(()) => {
                    let in_use = cfg.active_layout == LayoutId::Custom
                        && cfg.custom_layout.as_deref() == Some(id.as_str());
                    if in_use {
                        cfg.active_layout = LayoutId::AvroPhonetic;
                        cfg.custom_layout = None;
                        let _ = cfg.save_to(&Config::default_config_path());
                    }
                    ok_json(json_list(cfg), in_use)
                }
                Err(e) => error(&e),
            }
        }),
        ("GET", "/api/layouts/export") => Some({
            let value = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("id="))
                .unwrap_or_default();
            let layout = if let Some(id) = value.strip_prefix("Custom:") {
                layout::load_custom_layout(id).ok()
            } else {
                BUILT_IN
                    .iter()
                    .find(|l| format!("{l:?}") == value)
                    .and_then(|l| FixedLayout::builtin(*l))
            };
            match layout {
                Some(l) => Reply {
                    status: "200 OK",
                    content_type: "application/json; charset=utf-8",
                    body: l.to_file_text("", ""),
                    config_changed: false,
                },
                None => {
                    error("Phonetic typing has no key layout to export; choose a fixed layout.")
                }
            }
        }),
        _ => None,
    }
}
