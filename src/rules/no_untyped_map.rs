use std::path::Path;

use syn::spanned::Spanned;
use syn::File;

use crate::find::{finding, Finding};
use crate::rules::{is_untyped_map, visit_domain_types};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    visit_domain_types(file, |ty| {
        if is_untyped_map(ty) {
            findings.push(finding(
                "no_untyped_map",
                "This string-keyed map of untyped values discards domain evidence.",
                "Use a named struct or a map with a typed key and a typed value.",
                path,
                ty.span(),
            ));
        }
    });
    findings
}
