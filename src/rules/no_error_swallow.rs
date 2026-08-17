use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprMatch, File, Pat};

use crate::find::{finding, Finding};
use crate::rules::last_seg;

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = Swallow {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct Swallow<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Visit<'_> for Swallow<'_> {
    fn visit_expr_match(&mut self, node: &ExprMatch) {
        for arm in &node.arms {
            if is_err_wild(&arm.pat) && is_empty_or_discard(&arm.body) {
                self.findings.push(finding(
                    "no_error_swallow",
                    "This `Err(_)` arm discards error evidence.",
                    "Name the error and handle or propagate it.",
                    self.path,
                    arm.pat.span(),
                ));
            }
        }
        syn::visit::visit_expr_match(self, node);
    }
}

fn is_err_wild(pat: &Pat) -> bool {
    let pat = match pat {
        Pat::Paren(p) => &*p.pat,
        other => other,
    };
    match pat {
        Pat::TupleStruct(ts) => {
            last_seg(&ts.path) == "Err"
                && ts.elems.len() == 1
                && matches!(ts.elems.first(), Some(Pat::Wild(_)))
        }
        _ => false,
    }
}

fn is_empty_or_discard(expr: &Expr) -> bool {
    match expr {
        Expr::Tuple(t) if t.elems.is_empty() => true,
        Expr::Block(b) => block_is_empty_or_discard(&b.block),
        _ => false,
    }
}

fn block_is_empty_or_discard(block: &syn::Block) -> bool {
    if block.stmts.is_empty() {
        return true;
    }
    if block.stmts.len() != 1 {
        return false;
    }
    match &block.stmts[0] {
        syn::Stmt::Local(local) => matches!(local.pat, Pat::Wild(_)),
        syn::Stmt::Expr(Expr::Tuple(t), _) if t.elems.is_empty() => true,
        syn::Stmt::Expr(Expr::Block(inner), _) => block_is_empty_or_discard(&inner.block),
        _ => false,
    }
}
