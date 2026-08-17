use std::path::Path;

use syn::visit::Visit;
use syn::{Expr, ExprIf, File};

use crate::find::{finding, Finding};
use crate::rules::receiver_key;

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = Dead {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct Dead<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Visit<'_> for Dead<'_> {
    fn visit_expr_if(&mut self, node: &ExprIf) {
        if let Some(key) = guard_key(&node.cond) {
            let mut hunt = Hunt {
                key,
                path: self.path,
                findings: &mut self.findings,
            };
            hunt.visit_block(&node.then_branch);
        }
        syn::visit::visit_expr_if(self, node);
    }
}

fn guard_key(cond: &Expr) -> Option<String> {
    let cond = match cond {
        Expr::Paren(p) => &*p.expr,
        other => other,
    };
    let Expr::MethodCall(m) = cond else {
        return None;
    };
    let method = m.method.to_string();
    if !matches!(method.as_str(), "is_ok" | "is_some") {
        return None;
    }
    receiver_key(&m.receiver)
}

struct Hunt<'a> {
    key: String,
    path: &'a Path,
    findings: &'a mut Vec<Finding>,
}

impl Visit<'_> for Hunt<'_> {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        let method = node.method.to_string();
        if matches!(method.as_str(), "unwrap" | "expect" | "unwrap_unchecked")
            && receiver_key(&node.receiver).as_deref() == Some(self.key.as_str())
        {
            self.findings.push(finding(
                "no_dead_guard",
                "This `is_ok`/`is_some` guard then unwrap discards evidence the branch already proved.",
                "Use `if let` or `match` so the proven value stays in a binding.",
                self.path,
                node.method.span(),
            ));
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item_fn(&mut self, _i: &syn::ItemFn) {
        // nested function is a different body
    }
}
