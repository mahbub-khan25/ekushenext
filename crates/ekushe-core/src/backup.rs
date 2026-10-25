//! Backup and restore of the user's own Ekushe files: settings and snippets (`config.toml`), learned
//! words (`learned.tsv`) and reported wrong words (`wrong-words.tsv`), as one JSON file that can be
//! moved to another computer, on any system.
//!
//! Restoring never throws anything away: settings are taken from the backup, but snippets, learned
//! words and reported words are added to what is already there (the backup wins where both have one).

use crate::config::Config;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// The files a backup holds (in the settings folder).
pub const FILES: &[&str] = &["config.toml", "learned.tsv", "wrong-words.tsv"];
/// Largest backup accepted (the Studio server takes at most 1 MB anyway).
pub const MAX_BACKUP: usize = 900 * 1024;
const WRONG_WORDS_KEPT: usize = 500;

/// What a restore changed, for the message shown to the user.
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize)]
pub struct Restored {
    pub settings: bool,
    pub snippets: usize,
    pub learned_words: usize,
    pub wrong_words: usize,
}

/// The backup of the files in `dir`, as JSON. `created` is shown in the file (e.g. "2026-10-24").
pub fn create(dir: &Path, created: &str) -> String {
    let mut files = BTreeMap::new();
    for name in FILES {
        if let Ok(text) = std::fs::read_to_string(dir.join(name)) {
            files.insert(name.to_string(), text);
        }
    }
    serde_json::to_string_pretty(&serde_json::json!({
        "ekushe_backup": 1,
        "app_version": include_str!("../../../.github/release/VERSION").trim(),
        "created": created,
        "files": files,
    }))
    .unwrap_or_default()
}

/// `roman \t bangla \t count` lines into a map (counts default to 1).
fn parse_learned(text: &str) -> BTreeMap<(String, String), u32> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 2 {
            continue;
        }
        let (roman, bangla) = (f[0].trim().to_lowercase(), f[1].trim().to_string());
        if roman.is_empty() || bangla.is_empty() || roman.len() > 64 || bangla.len() > 256 {
            continue;
        }
        let count = f.get(2).and_then(|c| c.trim().parse().ok()).unwrap_or(1u32);
        let e = out.entry((roman, bangla)).or_insert(0);
        *e = (*e).max(count);
    }
    out
}

/// Restore a backup made by [`create`] into `dir`. Nothing is written unless the whole backup is valid.
pub fn restore(dir: &Path, backup: &str) -> Result<Restored, String> {
    if backup.len() > MAX_BACKUP {
        return Err("the backup file is too large".into());
    }
    let v: serde_json::Value = serde_json::from_str(backup)
        .map_err(|_| "this is not an Ekushe Next backup file".to_string())?;
    if v.get("ekushe_backup").and_then(|x| x.as_u64()) != Some(1) {
        return Err("this is not an Ekushe Next backup file".into());
    }
    let files = v
        .get("files")
        .and_then(|f| f.as_object())
        .ok_or("the backup has no files")?;
    let get = |name: &str| files.get(name).and_then(|t| t.as_str());

    let mut done = Restored::default();
    // Check everything first
    let new_cfg = match get("config.toml") {
        Some(t) => Some(
            toml::from_str::<Config>(t)
                .map_err(|_| "the settings in the backup are damaged".to_string())?,
        ),
        None => None,
    };

    let cfg_path = dir.join("config.toml");
    if let Some(mut cfg) = new_cfg {
        let current = Config::load_from(&cfg_path).ok();
        let backup_snips = cfg.custom_snippets.len();
        if let Some(cur) = current {
            let mut snips: HashMap<String, String> = cur.custom_snippets;
            snips.extend(cfg.custom_snippets);
            cfg.custom_snippets = snips;
        }
        cfg.save_to(&cfg_path)
            .map_err(|e| format!("settings could not be saved: {e}"))?;
        done.settings = true;
        done.snippets = backup_snips;
    }

    if let Some(t) = get("learned.tsv") {
        let path = dir.join("learned.tsv");
        let incoming = parse_learned(t);
        let mut all = parse_learned(&std::fs::read_to_string(&path).unwrap_or_default());
        for (k, c) in &incoming {
            let e = all.entry(k.clone()).or_insert(0);
            *e = (*e).max(*c);
        }
        let mut out = String::new();
        for ((roman, bangla), c) in &all {
            out.push_str(&format!("{roman}\t{bangla}\t{c}\n"));
        }
        crate::suggest::write_atomic(&path, &out)
            .map_err(|e| format!("learned words could not be saved: {e}"))?;
        done.learned_words = incoming.len();
    }

    if let Some(t) = get("wrong-words.tsv") {
        let path = dir.join("wrong-words.tsv");
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut lines: Vec<String> = old
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect();
        for l in t.lines().filter(|l| !l.trim().is_empty() && l.len() < 1024) {
            if !lines.iter().any(|x| x == l) {
                lines.push(l.to_string());
                done.wrong_words += 1;
            }
        }
        let skip = lines.len().saturating_sub(WRONG_WORDS_KEPT);
        let mut out = lines[skip..].join("\n");
        out.push('\n');
        crate::suggest::write_atomic(&path, &out)
            .map_err(|e| format!("reported words could not be saved: {e}"))?;
    }
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backup_restores_on_another_computer_without_losing_anything() {
        let base = std::env::temp_dir().join(format!("ekushe-backup-test-{}", std::process::id()));
        let (a, b) = (base.join("a"), base.join("b"));
        // Computer A: settings, a snippet, learned words
        let mut cfg = Config {
            compact_popup: true,
            ..Default::default()
        };
        cfg.custom_snippets.insert(";addr".into(), "ঢাকা".into());
        std::fs::create_dir_all(&a).unwrap();
        cfg.save_to(&a.join("config.toml")).unwrap();
        crate::suggest::write_atomic(&a.join("learned.tsv"), "ase\tআছে\t4\nvalo\tভালো\t2\n")
            .unwrap();
        crate::suggest::write_atomic(&a.join("wrong-words.tsv"), "ase\tআসে\tআছে\tকেমন\n").unwrap();
        let backup = create(&a, "2026-10-24");
        assert!(backup.contains("\"ekushe_backup\": 1"));

        // Computer B already has its own snippet and words
        let mut cfg_b = Config::default();
        cfg_b.custom_snippets.insert(";mine".into(), "আমার".into());
        std::fs::create_dir_all(&b).unwrap();
        cfg_b.save_to(&b.join("config.toml")).unwrap();
        crate::suggest::write_atomic(&b.join("learned.tsv"), "ase\tআছে\t9\nkhub\tখুব\t1\n").unwrap();

        let r = restore(&b, &backup).unwrap();
        assert_eq!(
            r,
            Restored {
                settings: true,
                snippets: 1,
                learned_words: 2,
                wrong_words: 1
            }
        );
        let got = Config::load_from(&b.join("config.toml")).unwrap();
        assert!(got.compact_popup);
        assert_eq!(got.custom_snippets.len(), 2, "both snippets kept");
        assert_eq!(
            std::fs::read_to_string(b.join("learned.tsv")).unwrap(),
            "ase\tআছে\t9\nkhub\tখুব\t1\nvalo\tভালো\t2\n"
        );
        // Restoring twice changes nothing more
        assert_eq!(restore(&b, &backup).unwrap().wrong_words, 0);

        // Anything else is refused and nothing is written
        assert!(restore(&b, "{}").is_err());
        assert!(restore(&b, "not json").is_err());
        assert!(restore(
            &b,
            r#"{"ekushe_backup":1,"files":{"config.toml":"max_candidates = \"x\""}}"#
        )
        .is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
