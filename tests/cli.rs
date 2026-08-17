use std::path::PathBuf;
use std::process::Command;

fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-anti-slop"))
}

fn bad(rule: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bad")
        .join(format!("{rule}.rs"))
}

#[test]
fn rustc_like_text_and_exit_one() {
    let out = Command::new(exe())
        .arg(bad("no_erasure_cast"))
        .output()
        .expect("run cargo-anti-slop");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("error: anti-slop/no_erasure_cast:"),
        "{stdout}"
    );
    assert!(stdout.contains("--> "), "{stdout}");
    assert!(stdout.contains("help:"), "{stdout}");
}

#[test]
fn strips_cargo_plugin_arg() {
    let out = Command::new(exe())
        .arg("anti-slop")
        .arg(bad("no_erasure_cast"))
        .output()
        .expect("run cargo-anti-slop");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("no_erasure_cast"), "{stdout}");
}

#[test]
fn json_format() {
    let out = Command::new(exe())
        .arg("--format")
        .arg("json")
        .arg(bad("no_erasure_cast"))
        .output()
        .expect("run cargo-anti-slop");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect(&stdout);
    assert!(parsed.as_array().is_some_and(|a| !a.is_empty()));
}

#[test]
fn allow_clears_findings() {
    let out = Command::new(exe())
        .arg("--allow")
        .arg("no_erasure_cast")
        .arg(bad("no_erasure_cast"))
        .output()
        .expect("run cargo-anti-slop");
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
