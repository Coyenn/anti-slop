use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Collect `.rs` files under `path`. A file path is returned as-is.
pub fn rust_files(path: &Path, exclude: &[String]) -> anyhow::Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.exists() {
        anyhow::bail!("path does not exist: {}", path.display());
    }
    let mut out = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            !dir_excluded(e.path(), exclude)
        } else {
            true
        }
    }) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if excluded(entry.path(), exclude) {
            continue;
        }
        out.push(entry.path().to_path_buf());
    }
    out.sort();
    Ok(out)
}

fn dir_excluded(path: &Path, exclude: &[String]) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| {
            name == "target"
                || name == ".git"
                || exclude.iter().any(|ex| ex == name || glob_match(ex, name))
        })
        || excluded(path, exclude)
}

fn excluded(path: &Path, exclude: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    for ex in exclude {
        if path.components().any(|c| c.as_os_str() == ex.as_str()) {
            return true;
        }
        if glob_match(ex, &path_str) {
            return true;
        }
    }
    false
}

/// Tiny glob: `*` (no slash), `**` (any), `?` (one non-slash).
fn glob_match(pat: &str, text: &str) -> bool {
    glob_rec(pat.as_bytes(), text.as_bytes())
}

fn glob_rec(pat: &[u8], text: &[u8]) -> bool {
    if pat.is_empty() {
        return text.is_empty();
    }
    if pat.starts_with(b"**") {
        let rest = if pat.len() > 2 && pat[2] == b'/' {
            &pat[3..]
        } else {
            &pat[2..]
        };
        if rest.is_empty() {
            return true;
        }
        if glob_rec(rest, text) {
            return true;
        }
        for i in 0..text.len() {
            if glob_rec(rest, &text[i + 1..]) {
                return true;
            }
        }
        return false;
    }
    if pat[0] == b'*' {
        if glob_rec(&pat[1..], text) {
            return true;
        }
        if !text.is_empty() && text[0] != b'/' && glob_rec(pat, &text[1..]) {
            return true;
        }
        return false;
    }
    if pat[0] == b'?' {
        return !text.is_empty() && text[0] != b'/' && glob_rec(&pat[1..], &text[1..]);
    }
    !text.is_empty() && pat[0] == text[0] && glob_rec(&pat[1..], &text[1..])
}

#[cfg(test)]
mod tests {
    use super::glob_match;

    #[test]
    fn glob_starstar() {
        assert!(glob_match("**/foo.rs", "a/b/foo.rs"));
        assert!(glob_match("target", "target"));
        assert!(!glob_match("*.rs", "a/b.rs"));
        assert!(glob_match("*.rs", "b.rs"));
    }
}
