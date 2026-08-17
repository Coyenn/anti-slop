use std::collections::HashSet;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprIndex, File, FnArg, ItemFn, Local, Pat};

use crate::find::{finding, Finding};
use crate::rules::{is_bare_value, is_value_soup, path_segments, peel_expr};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = JsonIndex {
        path,
        findings: Vec::new(),
        names: HashSet::new(),
    };
    v.visit_file(file);
    v.findings
}

struct JsonIndex<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
    names: HashSet<String>,
}

impl JsonIndex<'_> {
    fn root_is_value(&self, expr: &Expr) -> bool {
        match index_root(expr) {
            Expr::Path(p) => {
                path_is_value_root(&p.path)
                    || p.path
                        .get_ident()
                        .is_some_and(|id| self.names.contains(&id.to_string()))
            }
            Expr::Call(c) => match peel_index_wrap(&c.func) {
                Expr::Path(p) => path_is_value_root(&p.path),
                _ => false,
            },
            _ => false,
        }
    }
}

impl Visit<'_> for JsonIndex<'_> {
    fn visit_item_fn(&mut self, i: &ItemFn) {
        let saved = self.names.clone();
        syn::visit::visit_item_fn(self, i);
        self.names = saved;
    }

    fn visit_impl_item_fn(&mut self, i: &syn::ImplItemFn) {
        let saved = self.names.clone();
        syn::visit::visit_impl_item_fn(self, i);
        self.names = saved;
    }

    fn visit_fn_arg(&mut self, arg: &FnArg) {
        if let FnArg::Typed(t) = arg {
            if is_value_ty(&t.ty) {
                if let Pat::Ident(id) = &*t.pat {
                    self.names.insert(id.ident.to_string());
                }
            }
        }
        syn::visit::visit_fn_arg(self, arg);
    }

    fn visit_local(&mut self, local: &Local) {
        if let Pat::Type(pt) = &local.pat {
            if is_value_ty(&pt.ty) {
                if let Pat::Ident(id) = &*pt.pat {
                    self.names.insert(id.ident.to_string());
                }
            }
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_index(&mut self, node: &ExprIndex) {
        if self.root_is_value(&node.expr) {
            self.findings.push(finding(
                "no_json_index",
                "This `Value` index discards missing-key evidence as silent `Null`.",
                "Use `.get(...)` so a missing key stays `Option`, not silent `Null`.",
                self.path,
                node.span(),
            ));
        }
        syn::visit::visit_expr_index(self, node);
    }
}

fn is_value_ty(ty: &syn::Type) -> bool {
    is_value_soup(ty) || is_bare_value(ty)
}

fn path_is_value_root(path: &syn::Path) -> bool {
    let segs = path_segments(path);
    match segs.as_slice() {
        [.., v] if v == "Value" => true,
        [.., v, _] if v == "Value" => true,
        _ => false,
    }
}

fn index_root(expr: &Expr) -> &Expr {
    let expr = peel_index_wrap(expr);
    match expr {
        Expr::Index(i) => index_root(&i.expr),
        other => peel_expr(other),
    }
}

fn peel_index_wrap(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(p) => peel_index_wrap(&p.expr),
        Expr::Group(g) => peel_index_wrap(&g.expr),
        Expr::Try(t) => peel_index_wrap(&t.expr),
        other => other,
    }
}
