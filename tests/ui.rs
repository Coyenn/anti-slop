use std::path::PathBuf;

use anti_slop::{lint_path, Config};

fn fixture_dir(kind: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(kind)
}

#[test]
fn bad_fixtures_fire_their_rule() {
    let dir = fixture_dir("bad");
    let mut seen = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rule = path.file_stem().unwrap().to_string_lossy().to_string();
        let findings = lint_path(&path, &Config::default()).unwrap();
        if std::fs::read_to_string(&path).unwrap().contains("marker") {
            seen.push(rule);
            continue;
        }
        assert!(
            findings.iter().any(|f| f.rule == rule),
            "expected rule `{rule}` in {}, got {:#?}",
            path.display(),
            findings
        );
        seen.push(rule);
    }
    seen.sort();
    let mut expected: Vec<String> = anti_slop::RULES.iter().map(|s| (*s).to_string()).collect();
    expected.sort();
    assert_eq!(seen, expected, "every shipped rule needs a bad fixture");
}

#[test]
fn good_fixtures_are_clean() {
    let dir = fixture_dir("good");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let findings = lint_path(&path, &Config::default()).unwrap();
        assert!(
            findings.is_empty(),
            "expected {} to be clean, got {:#?}",
            path.display(),
            findings
        );
    }
}

#[test]
fn safety_escape_suppresses_erasure_cast() {
    let src = r#"
fn f(x: u64) -> usize {
    // SAFETY: the caller parsed x as a page offset that fits usize on this target.
    x as _
}
"#;
    let findings =
        anti_slop::lint_source(src, std::path::Path::new("t.rs"), &Config::default()).unwrap();
    assert!(
        findings.iter().all(|f| f.rule != "no_erasure_cast"),
        "{findings:#?}"
    );
}

#[test]
fn generic_safety_does_not_suppress() {
    let src = r#"
fn f(x: u64) -> usize {
    // SAFETY: trust me
    x as _
}
"#;
    let findings =
        anti_slop::lint_source(src, std::path::Path::new("t.rs"), &Config::default()).unwrap();
    assert!(
        findings.iter().any(|f| f.rule == "no_erasure_cast"),
        "{findings:#?}"
    );
    assert!(
        findings.iter().any(|f| f.rule == "no_empty_safety"),
        "{findings:#?}"
    );
}

#[test]
fn allow_disables_a_rule() {
    let path = fixture_dir("bad").join("no_erasure_cast.rs");
    let mut config = Config::default();
    config.allow.push("no_erasure_cast".into());
    let findings = lint_path(&path, &config).unwrap();
    assert!(findings.iter().all(|f| f.rule != "no_erasure_cast"));
}

#[test]
fn unicode_comments_do_not_panic() {
    let src = "// keep evidence — parse once at the I/O boundary.
fn f() {}
";
    let findings =
        anti_slop::lint_source(src, std::path::Path::new("t.rs"), &Config::default()).unwrap();
    assert!(
        findings.iter().any(|f| f.rule == "no_emdash"),
        "{findings:#?}"
    );
}

#[test]
fn language_escape_suppresses_emdash() {
    let src = "// LANGUAGE: keep — stylistic dash in this note.
fn f() {}
";
    let findings =
        anti_slop::lint_source(src, std::path::Path::new("t.rs"), &Config::default()).unwrap();
    assert!(
        findings.iter().all(|f| f.rule != "no_emdash"),
        "{findings:#?}"
    );
}

#[test]
fn strings_default_off() {
    let src = r#"
fn f() {
    let _ = "Certainly! Here's the thing.";
}
"#;
    let findings =
        anti_slop::lint_source(src, std::path::Path::new("t.rs"), &Config::default()).unwrap();
    assert!(
        findings.iter().all(|f| f.rule != "no_chat_preamble"),
        "{findings:#?}"
    );
}

#[test]
fn strings_opt_in() {
    let src = r#"
fn f() {
    let _ = "Certainly! Here's the thing.";
}
"#;
    let config = Config {
        lint_strings: true,
        ..Config::default()
    };
    let findings =
        anti_slop::lint_source(src, std::path::Path::new("t.rs"), &config).unwrap();
    assert!(
        findings.iter().any(|f| f.rule == "no_chat_preamble"),
        "{findings:#?}"
    );
}

#[test]
fn file_line_cap() {
    let mut src = String::from("fn f() {}\n");
    for _ in 0..10 {
        src.push_str(&"//\n".repeat(100));
    }
    let findings =
        anti_slop::lint_source(&src, std::path::Path::new("t.rs"), &Config::default()).unwrap();
    assert!(
        findings.iter().any(|f| f.rule.ends_with("1k")),
        "{findings:#?}"
    );
}
