//! Nothing typed in a private field, and nothing that looks like a password, is learned.
use ekushe_core::{config::Config, looks_secret, Ekushe};

fn type_and_pick(ek: &mut Ekushe, roman: &str) {
    for c in roman.chars() {
        ek.push_char(c, false);
    }
    ek.select_candidate(0, Some(' '));
}

#[test]
fn password_like_text_is_not_learned() {
    let mut ek = Ekushe::new(Config::default());
    for secret in [
        "AuditSecret739",
        "hunter2",
        "me@mail",
        "pa$$word",
        "abc_def",
    ] {
        type_and_pick(&mut ek, secret);
        assert!(
            !ek.suggestions_engine()
                .user_dict_tsv()
                .to_lowercase()
                .contains(&secret.to_lowercase()),
            "{secret} was learned"
        );
    }
    assert_eq!(ek.suggestions_engine().learned_version(), 0);
    // Ordinary words still are, and Ekushe's own marks are not "symbols"
    type_and_pick(&mut ek, "amar");
    type_and_pick(&mut ek, "allah,,");
    type_and_pick(&mut ek, "he^se");
    assert!(ek.suggestions_engine().learned_version() >= 3);
    for ok in [
        "amar", "allah,,", "he^se", "du:kho", "k+t", "bol`le", "hothat``",
    ] {
        assert!(!looks_secret(ok), "{ok}");
    }
}

#[test]
fn a_private_field_learns_nothing_and_keeps_no_context() {
    let mut ek = Ekushe::new(Config::default());
    ek.set_private_field(true);
    type_and_pick(&mut ek, "amar");
    type_and_pick(&mut ek, "sonar");
    assert_eq!(ek.suggestions_engine().learned_version(), 0);
    assert!(!ek.has_undo());
    ek.set_private_field(false);
    type_and_pick(&mut ek, "amar");
    assert_eq!(ek.suggestions_engine().learned_version(), 1);
}

#[cfg(unix)]
#[test]
fn saved_files_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let base = std::env::temp_dir().join(format!("ekushe-perm-test-{}", std::process::id()));
    let dir = base.join("Ekushe");
    let file = dir.join("learned.tsv");
    ekushe_core::suggest::write_atomic(&file, "amar\tআমার\t1\n").unwrap();
    ekushe_core::suggest::write_atomic(&file, "amar\tআমার\t2\n").unwrap();
    let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&file), 0o600);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "amar\tআমার\t2\n");
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        1,
        "no temporary files left"
    );
    let cfg = dir.join("config.toml");
    ekushe_core::config::Config::default()
        .save_to(&cfg)
        .unwrap();
    assert_eq!(mode(&cfg), 0o600);
    // Files left readable by an older version are tightened
    std::fs::set_permissions(&cfg, std::fs::Permissions::from_mode(0o644)).unwrap();
    ekushe_core::suggest::tighten_permissions(&dir);
    assert_eq!(mode(&cfg), 0o600);
    let _ = std::fs::remove_dir_all(&base);
}
