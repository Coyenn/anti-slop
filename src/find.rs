use std::path::Path;

use proc_macro2::Span;
use serde::Serialize;
use syn::File;

use crate::rules;

/// All rule names, in the order they run.
pub const RULES: &[&str] = &[
    "no_erasure_cast",
    "no_any_bag",
    "no_value_soup",
    "no_untyped_map",
    "no_discarded_error",
    "no_automock",
    "no_dead_guard",
    "no_json_roundtrip",
    "no_error_swallow",
    "no_empty_safety",
    "no_json_index",
    "no_type_alias_bag",
    "no_boxed_error",
    "no_stringly_error",
    "no_typeid_branch",
    "no_type_only_assert",
    "no_widen_then_narrow",
    "no_ad_hoc_env",
    "no_over_1k",
    "no_emdash",
    "no_smart_quotes",
    "no_chat_preamble",
    "no_elision_comment",
    "no_fence_comment",
];

/// Lint configuration. Every rule is denied (enabled) by default.
#[derive(Debug, Clone)]
pub struct Config {
    /// Rule names to disable.
    pub allow: Vec<String>,
    /// Rule names to enable. Empty means every known rule.
    pub deny: Vec<String>,
    /// Path globs or directory names to skip. `target` and `.git` are always skipped.
    pub exclude: Vec<String>,
    /// When true, language rules also scan string literals. Default: comments and docs only.
    pub lint_strings: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            allow: Vec::new(),
            deny: Vec::new(),
            exclude: vec!["target".into(), ".git".into()],
            lint_strings: false,
        }
    }
}

impl Config {
    pub fn is_enabled(&self, rule: &str) -> bool {
        if self.allow.iter().any(|r| r == rule) {
            return false;
        }
        if self.deny.is_empty() {
            return RULES.contains(&rule);
        }
        self.deny.iter().any(|r| r == rule)
    }
}

/// One diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: String,
    pub message: String,
    pub help: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    /// 1-based, exclusive when on the same line as `line`.
    pub end_line: usize,
    pub end_column: usize,
}

pub(crate) fn finding(rule: &str, message: &str, help: &str, path: &Path, span: Span) -> Finding {
    let start = span.start();
    let end = span.end();
    let end_line = if end.line == 0 { start.line } else { end.line };
    let mut end_column = end.column + 1;
    if end_line == start.line && end_column <= start.column + 1 {
        end_column = start.column + 2;
    }
    Finding {
        rule: rule.to_string(),
        message: message.to_string(),
        help: help.to_string(),
        file: path.display().to_string(),
        line: start.line,
        column: start.column + 1,
        end_line,
        end_column,
    }
}

/// A source comment captured by the scanner.
#[derive(Debug, Clone)]
pub(crate) struct Comment {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub body: String,
    pub body_start_byte: usize,
}

/// A string literal surface for optional language linting.
#[derive(Debug, Clone)]
pub(crate) struct TextSurface {
    pub body: String,
    pub body_start_byte: usize,
}

pub(crate) fn lint_source(src: &str, path: &Path, config: &Config) -> anyhow::Result<Vec<Finding>> {
    let file: File = syn::parse_file(src).map_err(|e| {
        anyhow::anyhow!(
            "{}:{}: parse error: {e}",
            path.display(),
            e.span().start().line
        )
    })?;
    let comments = scan_comments(src);
    let strings = if config.lint_strings {
        scan_string_literals(src)
    } else {
        Vec::new()
    };
    let mut findings = Vec::new();
    for rule in RULES {
        if !config.is_enabled(rule) {
            continue;
        }
        findings.extend(rules::lint_rule(
            rule, &file, src, path, &comments, &strings,
        ));
    }
    findings.retain(|f| {
        f.rule == "no_empty_safety"
            || f.rule == "no_over_1k"
            || !has_safety_escape(&comments, src, f.line, f.column)
    });
    Ok(findings)
}

pub(crate) fn lint_one(path: &Path, config: &Config) -> anyhow::Result<Vec<Finding>> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
    lint_source(&src, path, config)
}

/// `SAFETY:` (word-boundary, optional space, colon). Case-sensitive.
pub(crate) fn safety_body(text: &str) -> Option<&str> {
    for (i, _) in text.char_indices() {
        if !text[i..].starts_with("SAFETY") {
            continue;
        }
        let before_ok = match text[..i].chars().next_back() {
            None => true,
            Some(c) => !c.is_ascii_alphanumeric() && c != '_',
        };
        if !before_ok {
            continue;
        }
        let after = i + "SAFETY".len();
        if after > text.len() {
            continue;
        }
        let mut rest = &text[after..];
        if rest.starts_with(' ') {
            rest = &rest[1..];
        }
        if let Some(stripped) = rest.strip_prefix(':') {
            return Some(stripped);
        }
    }
    None
}

pub(crate) fn is_generic_safety(body: &str) -> bool {
    let b = body
        .trim()
        .trim_end_matches(['.', '!', ','])
        .trim()
        .to_ascii_lowercase();
    matches!(
        b.as_str(),
        "" | "safe" | "trust me" | "it's fine" | "this is ok" | "okay" | "lints"
    )
}

pub(crate) fn is_real_safety(comment: &Comment) -> bool {
    match safety_body(&comment.body) {
        Some(body) => !is_generic_safety(body),
        None => false,
    }
}

/// `LANGUAGE:` (word-boundary, optional space, colon). Case-sensitive.
pub(crate) fn language_escape(text: &str) -> bool {
    language_body(text).is_some()
}

pub(crate) fn language_body(text: &str) -> Option<&str> {
    for (i, _) in text.char_indices() {
        if !text[i..].starts_with("LANGUAGE") {
            continue;
        }
        let before_ok = match text[..i].chars().next_back() {
            None => true,
            Some(c) => !c.is_ascii_alphanumeric() && c != '_',
        };
        if !before_ok {
            continue;
        }
        let after = i + "LANGUAGE".len();
        let mut rest = &text[after..];
        if rest.starts_with(' ') {
            rest = &rest[1..];
        }
        if let Some(stripped) = rest.strip_prefix(':') {
            return Some(stripped);
        }
    }
    None
}

/// Map a byte offset in `src` to a 1-based `(line, column)`.
pub(crate) fn pos_at_offset(src: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col.max(1))
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// A real SAFETY comment on the previous non-empty line, or a block comment
/// sitting immediately before the span, suppresses a finding.
pub(crate) fn has_safety_escape(
    comments: &[Comment],
    src: &str,
    line: usize,
    column: usize,
) -> bool {
    if line == 0 {
        return false;
    }
    let lines: Vec<&str> = src.lines().collect();
    let mut prev = line;
    while prev > 1 {
        prev -= 1;
        let text = lines.get(prev - 1).map(|s| s.trim()).unwrap_or("");
        if text.is_empty() {
            continue;
        }
        for c in comments {
            if c.start_line == prev && is_real_safety(c) {
                return true;
            }
        }
        break;
    }

    for c in comments {
        if is_real_safety(c) && only_ws_between(src, c.end_line, c.end_col, line, column) {
            return true;
        }
    }
    false
}

fn only_ws_between(
    src: &str,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
) -> bool {
    if start_line > end_line || (start_line == end_line && start_col > end_col) {
        return false;
    }
    let Some(from) = offset_at(src, start_line, start_col) else {
        return false;
    };
    let Some(to) = offset_at(src, end_line, end_col) else {
        return false;
    };
    if from > to {
        return false;
    }
    src.get(from..to)
        .is_some_and(|s| s.chars().all(char::is_whitespace))
}

fn offset_at(src: &str, line: usize, column: usize) -> Option<usize> {
    let mut cur_line = 1usize;
    let mut cur_col = 1usize;
    for (i, ch) in src.char_indices() {
        if cur_line == line && cur_col == column {
            return Some(i);
        }
        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
    }
    if cur_line == line && cur_col == column {
        return Some(src.len());
    }
    None
}

/// Scan `//` and `/* */` comments, skipping those that live inside strings.
pub(crate) fn scan_comments(src: &str) -> Vec<Comment> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;

    let bump = |i: &mut usize, line: &mut usize, col: &mut usize| {
        if *i < bytes.len() && bytes[*i] == b'\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        *i += 1;
    };

    while i < bytes.len() {
        if is_raw_string_start(bytes, i) {
            skip_raw_string(bytes, &mut i, &mut line, &mut col);
            continue;
        }
        if bytes[i] == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
            bump(&mut i, &mut line, &mut col);
            skip_char(bytes, &mut i, &mut line, &mut col);
            continue;
        }
        if bytes[i] == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            bump(&mut i, &mut line, &mut col);
            skip_string(bytes, &mut i, &mut line, &mut col);
            continue;
        }
        match bytes[i] {
            b'"' => skip_string(bytes, &mut i, &mut line, &mut col),
            b'\'' => skip_char_or_lifetime(bytes, &mut i, &mut line, &mut col),
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                let start_line = line;
                let start_col = col;
                bump(&mut i, &mut line, &mut col);
                bump(&mut i, &mut line, &mut col);
                let body_start = i;
                while i < bytes.len() && bytes[i] != b'\n' {
                    bump(&mut i, &mut line, &mut col);
                }
                let body = src[body_start..i].to_string();
                out.push(Comment {
                    start_line,
                    start_col,
                    end_line: line,
                    end_col: col,
                    body,
                    body_start_byte: body_start,
                });
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                let start_line = line;
                let start_col = col;
                bump(&mut i, &mut line, &mut col);
                bump(&mut i, &mut line, &mut col);
                let body_start = i;
                let mut depth = 1u32;
                while i + 1 < bytes.len() && depth > 0 {
                    if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                        depth += 1;
                        bump(&mut i, &mut line, &mut col);
                        bump(&mut i, &mut line, &mut col);
                    } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        depth -= 1;
                        if depth == 0 {
                            let body = src[body_start..i].to_string();
                            bump(&mut i, &mut line, &mut col);
                            bump(&mut i, &mut line, &mut col);
                            out.push(Comment {
                                start_line,
                                start_col,
                                end_line: line,
                                end_col: col,
                                body,
                                body_start_byte: body_start,
                            });
                            break;
                        }
                        bump(&mut i, &mut line, &mut col);
                        bump(&mut i, &mut line, &mut col);
                    } else {
                        bump(&mut i, &mut line, &mut col);
                    }
                }
            }
            _ => bump(&mut i, &mut line, &mut col),
        }
    }
    out
}

/// Scan `"..."`, `'...'`, and `r#"..."#` literal contents (not raw identifiers).
pub(crate) fn scan_string_literals(src: &str) -> Vec<TextSurface> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;

    let bump = |i: &mut usize, line: &mut usize, col: &mut usize| {
        if *i < bytes.len() && bytes[*i] == b'\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        *i += 1;
    };

    while i < bytes.len() {
        if is_raw_string_start(bytes, i) {
            let start_i = i;
            if bytes[i] == b'b' {
                bump(&mut i, &mut line, &mut col);
            }
            bump(&mut i, &mut line, &mut col); // r
            let mut hashes = 0usize;
            while i < bytes.len() && bytes[i] == b'#' {
                hashes += 1;
                bump(&mut i, &mut line, &mut col);
            }
            if i < bytes.len() && bytes[i] == b'"' {
                bump(&mut i, &mut line, &mut col);
            }
            let body_start = i;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    let mut ok = true;
                    for k in 0..hashes {
                        if i + 1 + k >= bytes.len() || bytes[i + 1 + k] != b'#' {
                            ok = false;
                            break;
                        }
                    }
                    if ok {
                        out.push(TextSurface {
                            body: src[body_start..i].to_string(),
                            body_start_byte: body_start,
                        });
                        bump(&mut i, &mut line, &mut col);
                        for _ in 0..hashes {
                            bump(&mut i, &mut line, &mut col);
                        }
                        break;
                    }
                }
                bump(&mut i, &mut line, &mut col);
            }
            let _ = start_i;
            continue;
        }
        if bytes[i] == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            bump(&mut i, &mut line, &mut col);
            push_string_body(src, bytes, &mut i, &mut line, &mut col, &mut out);
            continue;
        }
        if bytes[i] == b'"' {
            push_string_body(src, bytes, &mut i, &mut line, &mut col, &mut out);
            continue;
        }
        if bytes[i] == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
            bump(&mut i, &mut line, &mut col);
            skip_char(bytes, &mut i, &mut line, &mut col);
            continue;
        }
        if bytes[i] == b'\'' {
            let next = bytes.get(i + 1).copied();
            let next2 = bytes.get(i + 2).copied();
            if next == Some(b'\\') || (next.is_some() && next2 == Some(b'\'')) {
                skip_char(bytes, &mut i, &mut line, &mut col);
            } else {
                bump(&mut i, &mut line, &mut col);
            }
            continue;
        }
        if bytes[i] == b'/' && i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'*')
        {
            if bytes[i + 1] == b'/' {
                bump(&mut i, &mut line, &mut col);
                bump(&mut i, &mut line, &mut col);
                while i < bytes.len() && bytes[i] != b'\n' {
                    bump(&mut i, &mut line, &mut col);
                }
            } else {
                bump(&mut i, &mut line, &mut col);
                bump(&mut i, &mut line, &mut col);
                let mut depth = 1u32;
                while i + 1 < bytes.len() && depth > 0 {
                    if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                        depth += 1;
                        bump(&mut i, &mut line, &mut col);
                        bump(&mut i, &mut line, &mut col);
                    } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        depth -= 1;
                        bump(&mut i, &mut line, &mut col);
                        bump(&mut i, &mut line, &mut col);
                    } else {
                        bump(&mut i, &mut line, &mut col);
                    }
                }
            }
            continue;
        }
        bump(&mut i, &mut line, &mut col);
    }
    out
}

fn push_string_body(
    src: &str,
    bytes: &[u8],
    i: &mut usize,
    line: &mut usize,
    col: &mut usize,
    out: &mut Vec<TextSurface>,
) {
    advance(bytes, i, line, col); // opening "
    let body_start = *i;
    while *i < bytes.len() {
        match bytes[*i] {
            b'\\' => {
                advance(bytes, i, line, col);
                if *i < bytes.len() {
                    advance(bytes, i, line, col);
                }
            }
            b'"' => {
                out.push(TextSurface {
                    body: src[body_start..*i].to_string(),
                    body_start_byte: body_start,
                });
                advance(bytes, i, line, col);
                return;
            }
            _ => advance(bytes, i, line, col),
        }
    }
}

fn is_raw_string_start(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    if j < bytes.len() && bytes[j] == b'b' {
        j += 1;
    }
    if j >= bytes.len() || bytes[j] != b'r' {
        return false;
    }
    j += 1;
    while j < bytes.len() && bytes[j] == b'#' {
        j += 1;
    }
    j < bytes.len() && bytes[j] == b'"'
}

fn skip_raw_string(bytes: &[u8], i: &mut usize, line: &mut usize, col: &mut usize) {
    if *i < bytes.len() && bytes[*i] == b'b' {
        advance(bytes, i, line, col);
    }
    advance(bytes, i, line, col); // r
    let mut hashes = 0usize;
    while *i < bytes.len() && bytes[*i] == b'#' {
        hashes += 1;
        advance(bytes, i, line, col);
    }
    if *i < bytes.len() && bytes[*i] == b'"' {
        advance(bytes, i, line, col);
    }
    while *i < bytes.len() {
        if bytes[*i] == b'"' {
            let mut ok = true;
            for k in 0..hashes {
                if *i + 1 + k >= bytes.len() || bytes[*i + 1 + k] != b'#' {
                    ok = false;
                    break;
                }
            }
            if ok {
                advance(bytes, i, line, col);
                for _ in 0..hashes {
                    advance(bytes, i, line, col);
                }
                return;
            }
        }
        advance(bytes, i, line, col);
    }
}

fn skip_string(bytes: &[u8], i: &mut usize, line: &mut usize, col: &mut usize) {
    advance(bytes, i, line, col); // opening "
    while *i < bytes.len() {
        match bytes[*i] {
            b'\\' => {
                advance(bytes, i, line, col);
                if *i < bytes.len() {
                    advance(bytes, i, line, col);
                }
            }
            b'"' => {
                advance(bytes, i, line, col);
                return;
            }
            _ => advance(bytes, i, line, col),
        }
    }
}

fn skip_char(bytes: &[u8], i: &mut usize, line: &mut usize, col: &mut usize) {
    advance(bytes, i, line, col); // opening '
    if *i < bytes.len() && bytes[*i] == b'\\' {
        advance(bytes, i, line, col);
        if *i < bytes.len() {
            advance(bytes, i, line, col);
        }
    } else if *i < bytes.len() {
        advance(bytes, i, line, col);
    }
    if *i < bytes.len() && bytes[*i] == b'\'' {
        advance(bytes, i, line, col);
    }
}

fn skip_char_or_lifetime(bytes: &[u8], i: &mut usize, line: &mut usize, col: &mut usize) {
    let next = bytes.get(*i + 1).copied();
    let next2 = bytes.get(*i + 2).copied();
    if next == Some(b'\\') || (next.is_some() && next2 == Some(b'\'')) {
        skip_char(bytes, i, line, col);
        return;
    }
    advance(bytes, i, line, col); // '
    while *i < bytes.len() && is_ident_byte(bytes[*i]) {
        advance(bytes, i, line, col);
    }
}

fn advance(bytes: &[u8], i: &mut usize, line: &mut usize, col: &mut usize) {
    if *i >= bytes.len() {
        return;
    }
    if bytes[*i] == b'\n' {
        *line += 1;
        *col = 1;
    } else {
        *col += 1;
    }
    *i += 1;
}
