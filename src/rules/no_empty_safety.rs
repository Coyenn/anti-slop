use std::path::Path;

use syn::File;

use crate::find::{finding_at, is_generic_safety, safety_body, Comment, Finding};

pub fn lint_file(_file: &File, _src: &str, path: &Path, comments: &[Comment]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for c in comments {
        let Some(body) = safety_body(&c.body) else {
            continue;
        };
        if is_generic_safety(body) {
            findings.push(finding_at(
                "no_empty_safety",
                "This SAFETY comment fabricates an invariant without stating one.",
                "Write the checked invariant, or remove the comment.",
                path,
                c.start_line,
                c.start_col,
            ));
        }
    }
    findings
}
