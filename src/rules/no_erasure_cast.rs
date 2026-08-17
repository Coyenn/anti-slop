use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ExprCast, File, Type};

use crate::find::{finding, Finding};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = Erasure {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct Erasure<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Visit<'_> for Erasure<'_> {
    fn visit_expr_cast(&mut self, node: &ExprCast) {
        if matches!(&*node.ty, Type::Infer(_)) {
            self.findings.push(finding(
                "no_erasure_cast",
                "This `as _` cast discards type evidence.",
                "Name the target type so the conversion stays visible.",
                self.path,
                node.span(),
            ));
        }
        if let syn::Expr::Cast(inner) = &*node.expr {
            if is_ptr(&inner.ty) && is_ptr(&node.ty) {
                self.findings.push(finding(
                    "no_erasure_cast",
                    "This cast chain discards type evidence.",
                    "Keep a named owner type, or convert through a single documented pointer type.",
                    self.path,
                    node.span(),
                ));
            }
        }
        syn::visit::visit_expr_cast(self, node);
    }
}

fn is_ptr(ty: &Type) -> bool {
    match ty {
        Type::Ptr(_) => true,
        Type::Infer(_) => false,
        Type::Paren(p) => is_ptr(&p.elem),
        Type::Group(g) => is_ptr(&g.elem),
        _ => false,
    }
}
