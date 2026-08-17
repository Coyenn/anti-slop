//! rustc / Clippy-shaped human diagnostics.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::Finding;

/// Whether to paint ANSI codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

pub fn color_enabled(mode: ColorMode) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            use std::io::IsTerminal;
            std::io::stdout().is_terminal() && !crate::config::ColorSettings::from_env().no_color
        }
    }
}

/// Render every finding, then a rustc-like abort footer.
pub fn render_text(findings: &[Finding], color: bool) -> String {
    let mut out = String::new();
    for (i, f) in findings.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&render_one(f, color));
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    if !findings.is_empty() {
        out.push('\n');
        out.push_str(&footer(findings.len(), color));
        out.push('\n');
    }
    out
}

pub fn render_one(f: &Finding, color: bool) -> String {
    let mut out = String::new();
    let id = format!("anti_slop::{}", f.rule);
    let head = format!("error[{id}]: {}", f.message);
    let _ = writeln!(out, "{}", paint(color, "1;31", &head));

    let loc = format!("  --> {}:{}:{}", f.file, f.line, f.column);
    let _ = writeln!(out, "{}", paint(color, "1;36", &loc));

    let gutter = f.line.to_string().len().max(1);
    let bar = paint(color, "1;34", "|");
    let blank = format!("{:>gutter$} {bar}", "");
    let _ = writeln!(out, "{blank}");

    if let Some(src_line) = source_line(&f.file, f.line) {
        let shown = src_line.replace('\t', "    ");
        let _ = writeln!(
            out,
            "{:>gutter$} {bar} {shown}",
            paint(color, "1;34", &f.line.to_string())
        );
        let (pad, marks) = caret(&shown, f);
        let caret = paint(color, "1;31", &marks);
        let _ = writeln!(out, "{blank} {pad}{caret}");
    }

    let _ = writeln!(out, "{blank}");
    let eq = format!("{:>gutter$} {}", "", paint(color, "1;32", "="));
    if !f.help.is_empty() {
        let help = paint(color, "1", "help:");
        let _ = writeln!(out, "{eq} {help} {}", f.help);
    }
    let note = paint(color, "1", "note:");
    let _ = writeln!(out, "{eq} {note} `#[deny({id})]` on by default");
    out
}

fn footer(n: usize, color: bool) -> String {
    let word = if n == 1 { "error" } else { "errors" };
    paint(
        color,
        "1;31",
        &format!("error: aborting due to {n} previous {word}"),
    )
}

fn source_line(file: &str, line: usize) -> Option<String> {
    if line == 0 {
        return None;
    }
    let path = Path::new(file);
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return None,
    };
    src.lines().nth(line - 1).map(|s| s.to_string())
}

fn caret(shown: &str, f: &Finding) -> (String, String) {
    let start = f.column.max(1);
    let end = if f.end_line == f.line {
        f.end_column.max(start + 1)
    } else {
        shown.chars().count() + 1
    };
    let pad_cols = start.saturating_sub(1);
    let mark_cols = end.saturating_sub(start).max(1);
    let prefix: String = shown.chars().take(pad_cols).collect();
    let mut pad: String = prefix
        .chars()
        .map(|c| if c.is_whitespace() { c } else { ' ' })
        .collect();
    if pad.chars().count() < pad_cols {
        pad.push_str(&" ".repeat(pad_cols - pad.chars().count()));
    }
    (pad, "^".repeat(mark_cols))
}

fn paint(color: bool, code: &str, text: &str) -> String {
    if color {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Finding;

    fn sample() -> Finding {
        Finding {
            rule: "no_erasure_cast".into(),
            message: "This `as _` cast discards type evidence.".into(),
            help: "Name the target type so the conversion stays visible.".into(),
            file: "tests/fixtures/bad/no_erasure_cast.rs".into(),
            line: 2,
            column: 7,
            end_line: 2,
            end_column: 13,
        }
    }

    #[test]
    fn rustc_shape_without_color() {
        let text = render_one(&sample(), false);
        assert!(
            text.contains("error[anti_slop::no_erasure_cast]:"),
            "{text}"
        );
        assert!(
            text.contains("  --> tests/fixtures/bad/no_erasure_cast.rs:2:7"),
            "{text}"
        );
        assert!(text.contains(" x as _"), "{text}");
        assert!(text.contains('^'), "{text}");
        assert!(text.contains("= help:"), "{text}");
        assert!(
            text.contains("= note: `#[deny(anti_slop::no_erasure_cast)]`"),
            "{text}"
        );
        assert!(!text.contains('\u{1b}'), "{text}");
    }
}
