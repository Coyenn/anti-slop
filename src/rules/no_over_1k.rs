use std::path::Path;

use syn::File;

use crate::find::Finding;

const LIMIT: usize = 1000;

pub fn lint_file(_file: &File, src: &str, path: &Path) -> Vec<Finding> {
    let lines = src.lines().count();
    if lines <= LIMIT {
        return Vec::new();
    }
    vec![Finding {
        rule: "no_over_1k".into(),
        message: "This file is over 1000 lines.".into(),
        help: "Split it. A file past 1k lines hides the owner boundary.".into(),
        file: path.display().to_string(),
        line: LIMIT + 1,
        column: 1,
        end_line: LIMIT + 1,
        end_column: 2,
    }]
}
