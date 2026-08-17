use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, ItemType};

use crate::find::{finding, Finding};
use crate::rules::{is_any_bag, is_untyped_map, is_value_soup};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = AliasBag {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct AliasBag<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Visit<'_> for AliasBag<'_> {
    fn visit_item_type(&mut self, i: &ItemType) {
        if is_value_soup(&i.ty) || is_any_bag(&i.ty) || is_untyped_map(&i.ty) {
            self.findings.push(finding(
                "no_type_alias_bag",
                "This type alias is a bag that hides a weak contract.",
                "Give the alias a named owner type, or parse at the I/O boundary.",
                self.path,
                i.ty.span(),
            ));
        }
        syn::visit::visit_item_type(self, i);
    }
}
