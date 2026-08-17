use std::path::Path;

use syn::spanned::Spanned;
use syn::File;

use crate::find::{finding, Finding};
use crate::rules::{is_value_soup, visit_domain_types};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    visit_domain_types(file, |ty| {
        if is_value_soup(ty) {
            findings.push(finding(
                "no_value_soup",
                "This `Value` soup discards parsed domain evidence.",
                "Parse once at the I/O boundary into a named domain type.",
                path,
                ty.span(),
            ));
        }
    });
    findings
}
