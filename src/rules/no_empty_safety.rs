use std::path::Path;

use syn::File;

use crate::find::{is_generic_safety, safety_body, Comment, Finding};

pub fn lint_file(_file: &File, _src: &str, path: &Path, comments: &[Comment]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for c in comments {
        let Some(body) = safety_body(&c.body) else {
            continue;
        };
        if is_generic_safety(body) {
            findings.push(Finding {
                rule: "no_empty_safety".into(),
                message: "This SAFETY comment fabricates an invariant without stating one.".into(),
                help: "Write the checked invariant, or remove the comment.".into(),
                file: path.display().to_string(),
                line: c.start_line,
                column: c.start_col,
                end_line: c.end_line,
                end_column: c.end_col,
            });
        }
    }
    findings
}
