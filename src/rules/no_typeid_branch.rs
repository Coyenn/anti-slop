use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ExprMethodCall, ExprPath, File};

use crate::find::{finding, Finding};
use crate::rules::{last_seg, path_segments};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = TypeIdBranch {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct TypeIdBranch<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl TypeIdBranch<'_> {
    fn push(&mut self, span: proc_macro2::Span) {
        self.findings.push(finding(
            "no_typeid_branch",
            "This TypeId/downcast branch is a runtime typeof.",
            "Use a named enum or trait. TypeId/downcast is runtime typeof.",
            self.path,
            span,
        ));
    }
}

impl Visit<'_> for TypeIdBranch<'_> {
    fn visit_expr_path(&mut self, node: &ExprPath) {
        if is_typeid_path(&node.path) {
            self.push(node.path.span());
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &ExprMethodCall) {
        let name = node.method.to_string();
        if matches!(name.as_str(), "downcast_ref" | "downcast_mut" | "downcast") {
            self.push(node.method.span());
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn is_typeid_path(path: &syn::Path) -> bool {
    let segs = path_segments(path);
    match last_seg(path).as_str() {
        "of" => segs.iter().any(|s| s == "TypeId"),
        "type_name" | "type_name_of_val" => true,
        _ => false,
    }
}
