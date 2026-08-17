use std::path::Path;

use syn::spanned::Spanned;
use syn::File;

use crate::find::{finding, Finding};
use crate::rules::{is_any_bag, visit_domain_types};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    visit_domain_types(file, |ty| {
        if is_any_bag(ty) {
            findings.push(finding(
                "no_any_bag",
                "This `dyn Any` bag discards domain type evidence.",
                "Use a named owner or enum instead of type-erasing the value.",
                path,
                ty.span(),
            ));
        }
    });
    findings
}
