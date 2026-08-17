//! Opinionated lints that reject low-evidence Rust patterns.
//!
//! Prefer a named owner / domain type, parse once at the I/O boundary, and
//! keep evidence. This is not an AI-comment scanner.

mod find;
mod rules;
mod walk;

use std::path::Path;

pub use find::{Config, Finding, RULES};

/// Lint a file or a directory of `.rs` files.
pub fn lint_path(path: &Path, config: &Config) -> anyhow::Result<Vec<Finding>> {
    let mut exclude = config.exclude.clone();
    for extra in ["target", ".git"] {
        if !exclude.iter().any(|e| e == extra) {
            exclude.push(extra.to_string());
        }
    }
    let files = walk::rust_files(path, &exclude)?;
    let mut all = Vec::new();
    for file in files {
        all.extend(find::lint_one(&file, config)?);
    }
    all.sort_by(|a, b| {
        (&a.file, a.line, a.column, &a.rule).cmp(&(&b.file, b.line, b.column, &b.rule))
    });
    Ok(all)
}

/// Lint a source string as if it lived at `path`.
pub fn lint_source(src: &str, path: &Path, config: &Config) -> anyhow::Result<Vec<Finding>> {
    find::lint_source(src, path, config)
}
