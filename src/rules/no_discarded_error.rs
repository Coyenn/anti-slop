use std::path::Path;

use syn::visit::Visit;
use syn::{ExprMethodCall, File};

use crate::find::{finding, Finding};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = Discard {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct Discard<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Visit<'_> for Discard<'_> {
    fn visit_expr_method_call(&mut self, node: &ExprMethodCall) {
        let name = node.method.to_string();
        if matches!(
            name.as_str(),
            "ok" | "unwrap_or_else"
                | "unwrap_or_default"
                | "map_or"
                | "map_or_else"
                | "get_or_insert"
                | "get_or_insert_with"
        ) {
            self.findings.push(finding(
                "no_discarded_error",
                "This call discards Result/Option evidence.",
                "Propagate the error or name the fallback and why it is sound. This rule also flags Option, including map_or and get_or_insert.",
                self.path,
                node.method.span(),
            ));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
