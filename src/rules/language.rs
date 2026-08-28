use std::path::Path;

use syn::File;

use crate::find::{language_escape, pos_at_offset, Comment, Finding, TextSurface};

const CHAT_PREAMBLES: &[&str] = &[
    "certainly",
    "of course",
    "great question",
    "i'd be happy to",
    "i would be happy to",
    "here's the updated",
    "here is the updated",
    "let me know if",
];

const ELISION_PHRASES: &[&str] = &[
    "rest of the code unchanged",
    "rest of code unchanged",
    "... existing code ...",
    "insert code here",
];

const SMART_QUOTES: &[char] = &['\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}'];

pub fn lint_emdash(
    src: &str,
    path: &Path,
    comments: &[Comment],
    strings: &[TextSurface],
) -> Vec<Finding> {
    lint_surfaces(
        "no_emdash",
        "This text uses an em dash or mid-prose dash.",
        "Rewrite as two sentences, a comma, or parentheses.",
        src,
        path,
        comments,
        strings,
        emdash_match,
    )
}

pub fn lint_smart_quotes(
    src: &str,
    path: &Path,
    comments: &[Comment],
    strings: &[TextSurface],
) -> Vec<Finding> {
    lint_surfaces(
        "no_smart_quotes",
        "This text uses curly quotes or apostrophes.",
        "Use ASCII `'` / `\"`.",
        src,
        path,
        comments,
        strings,
        smart_quote_match,
    )
}

pub fn lint_chat_preamble(
    src: &str,
    path: &Path,
    comments: &[Comment],
    strings: &[TextSurface],
) -> Vec<Finding> {
    lint_surfaces(
        "no_chat_preamble",
        "This text starts with assistant-style preamble.",
        "Delete the preamble; keep any actual technical note.",
        src,
        path,
        comments,
        strings,
        chat_preamble_match,
    )
}

pub fn lint_elision_comment(
    src: &str,
    path: &Path,
    comments: &[Comment],
    strings: &[TextSurface],
) -> Vec<Finding> {
    lint_surfaces(
        "no_elision_comment",
        "This comment elides code instead of showing the full edit.",
        "Delete; the edit should be complete.",
        src,
        path,
        comments,
        strings,
        elision_match,
    )
}

pub fn lint_fence_comment(src: &str, path: &Path, comments: &[Comment]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for comment in comments {
        if language_escape(&comment.body) || !fence_match(&comment.body) {
            continue;
        }
        findings.push(finding_for_comment(
            "no_fence_comment",
            "This comment is only a markdown code fence.",
            "Delete.",
            path,
            src,
            comment,
            0,
            comment.body.len(),
        ));
    }
    findings
}

pub fn lint_file(
    _file: &File,
    src: &str,
    path: &Path,
    comments: &[Comment],
    strings: &[TextSurface],
    rule: &str,
) -> Vec<Finding> {
    match rule {
        "no_emdash" => lint_emdash(src, path, comments, strings),
        "no_smart_quotes" => lint_smart_quotes(src, path, comments, strings),
        "no_chat_preamble" => lint_chat_preamble(src, path, comments, strings),
        "no_elision_comment" => lint_elision_comment(src, path, comments, strings),
        "no_fence_comment" => lint_fence_comment(src, path, comments),
        _ => Vec::new(),
    }
}

type SurfaceMatch = fn(&str) -> Option<(usize, usize)>;

fn lint_surfaces(
    rule: &str,
    message: &str,
    help: &str,
    src: &str,
    path: &Path,
    comments: &[Comment],
    strings: &[TextSurface],
    matcher: SurfaceMatch,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for comment in comments {
        if language_escape(&comment.body) {
            continue;
        }
        if let Some((start, end)) = matcher(&comment.body) {
            findings.push(finding_for_comment(
                rule, message, help, path, src, comment, start, end,
            ));
        }
    }
    for surface in strings {
        if language_escape(&surface.body) {
            continue;
        }
        if let Some((start, end)) = matcher(&surface.body) {
            findings.push(finding_for_surface(
                rule, message, help, path, src, surface, start, end,
            ));
        }
    }
    findings
}

fn finding_for_comment(
    rule: &str,
    message: &str,
    help: &str,
    path: &Path,
    src: &str,
    comment: &Comment,
    body_start: usize,
    body_end: usize,
) -> Finding {
    let abs_start = comment.body_start_byte + body_start;
    let abs_end = comment.body_start_byte + body_end;
    finding_for_range(rule, message, help, path, src, abs_start, abs_end)
}

fn finding_for_surface(
    rule: &str,
    message: &str,
    help: &str,
    path: &Path,
    src: &str,
    surface: &TextSurface,
    body_start: usize,
    body_end: usize,
) -> Finding {
    let abs_start = surface.body_start_byte + body_start;
    let abs_end = surface.body_start_byte + body_end;
    finding_for_range(rule, message, help, path, src, abs_start, abs_end)
}

fn finding_for_range(
    rule: &str,
    message: &str,
    help: &str,
    path: &Path,
    src: &str,
    start_byte: usize,
    end_byte: usize,
) -> Finding {
    let (line, col) = pos_at_offset(src, start_byte);
    let (end_line, end_col) = pos_at_offset(src, end_byte);
    Finding {
        rule: rule.to_string(),
        message: message.to_string(),
        help: help.to_string(),
        file: path.display().to_string(),
        line,
        column: col,
        end_line,
        end_column: end_col,
    }
}

fn emdash_match(body: &str) -> Option<(usize, usize)> {
    if is_section_divider(body) || is_attribution_only(body) {
        return None;
    }
    for (i, ch) in body.char_indices() {
        let ch_len = ch.len_utf8();
        if ch == '\u{2014}' && !numeric_dash_context(body, i, ch) {
            return Some((i, i + ch_len));
        }
        if ch == '\u{2013}' && !numeric_dash_context(body, i, ch) {
            return Some((i, i + ch_len));
        }
        if ch == '-' && spaced_ascii_dash(body, i) && !numeric_dash_context(body, i, ch) {
            return Some((i, i + 1));
        }
    }
    None
}

fn smart_quote_match(body: &str) -> Option<(usize, usize)> {
    for (i, ch) in body.char_indices() {
        if SMART_QUOTES.contains(&ch) {
            return Some((i, i + ch.len_utf8()));
        }
    }
    None
}

fn chat_preamble_match(body: &str) -> Option<(usize, usize)> {
    let trimmed = body.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    for preamble in CHAT_PREAMBLES {
        if !lower.starts_with(preamble) {
            continue;
        }
        let tail = lower.as_bytes().get(preamble.len()).copied();
        if matches!(tail, None | Some(b'!' | b',' | b'.' | b':' | b' ')) {
            let skip = body.len() - trimmed.len();
            return Some((skip, body.len()));
        }
    }
    None
}

fn elision_match(body: &str) -> Option<(usize, usize)> {
    let lower = body.to_ascii_lowercase();
    for phrase in ELISION_PHRASES {
        if let Some(at) = lower.find(phrase) {
            return Some((at, at + phrase.len()));
        }
    }
    None
}

fn fence_match(body: &str) -> bool {
    let trimmed = body.trim();
    trimmed.starts_with("```") && trimmed[3..].chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_section_divider(body: &str) -> bool {
    let t = body.trim();
    t.len() >= 3 && t.chars().all(|c| c == '-' || c.is_whitespace())
}

fn is_attribution_only(body: &str) -> bool {
    let t = body.trim();
    matches!(t.chars().next(), Some('\u{2014}' | '\u{2013}' | '-')) && t.len() <= 120
}

fn numeric_dash_context(body: &str, dash_idx: usize, dash: char) -> bool {
    let before = body[..dash_idx].trim_end();
    let after_start = dash_idx + dash.len_utf8();
    let after = body.get(after_start..).unwrap_or("").trim_start();
    let prev = before.chars().last();
    let next = after.chars().next();
    match (prev, next) {
        (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit() => true,
        (Some(a), Some(b)) if a.is_ascii_digit() && b == '.' || a == '.' && b.is_ascii_digit() => {
            true
        }
        _ => false,
    }
}

fn spaced_ascii_dash(body: &str, dash_idx: usize) -> bool {
    if body.as_bytes().get(dash_idx) != Some(&b'-') {
        return false;
    }
    if body.as_bytes().get(dash_idx + 1) == Some(&b'-') {
        return false;
    }
    body[..dash_idx]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_whitespace())
        && body[dash_idx + 1..]
            .chars()
            .next()
            .is_some_and(|c| c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_en_dash_is_exempt() {
        assert!(emdash_match("covers 2020–2024").is_none());
        assert!(emdash_match("semver 1.0–2.0 ok").is_none());
    }

    #[test]
    fn prose_em_dash_fires() {
        assert!(emdash_match("parse once — keep evidence").is_some());
    }

    #[test]
    fn chat_preamble_detected() {
        assert!(chat_preamble_match("Certainly! Here is the handler.").is_some());
    }

    #[test]
    fn fence_only() {
        assert!(fence_match("```rust"));
        assert!(!fence_match("```rust\nfn main() {}"));
    }
}
