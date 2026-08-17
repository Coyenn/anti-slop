use std::collections::HashSet;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprCall, File, ItemFn, Local, Pat};

use crate::find::{finding, Finding};
use crate::rules::{path_is_serde_json, peel_expr};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = FileScan {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct FileScan<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl FileScan<'_> {
    fn scan_block(&mut self, block: &syn::Block) {
        let mut scan = Scan {
            path: self.path,
            findings: &mut self.findings,
            names: HashSet::new(),
        };
        scan.visit_block(block);
    }
}

impl Visit<'_> for FileScan<'_> {
    fn visit_item_fn(&mut self, i: &ItemFn) {
        self.scan_block(&i.block);
    }

    fn visit_impl_item_fn(&mut self, i: &syn::ImplItemFn) {
        self.scan_block(&i.block);
    }

    fn visit_trait_item_fn(&mut self, i: &syn::TraitItemFn) {
        if let Some(block) = &i.default {
            self.scan_block(block);
        }
    }
}

struct Scan<'a> {
    path: &'a Path,
    findings: &'a mut Vec<Finding>,
    names: HashSet<String>,
}

impl Visit<'_> for Scan<'_> {
    fn visit_local(&mut self, local: &Local) {
        if let Some(init) = &local.init {
            if is_to_value_call(&init.expr) {
                if let Pat::Ident(id) = &local.pat {
                    self.names.insert(id.ident.to_string());
                }
            }
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_call(&mut self, call: &ExprCall) {
        if path_is_serde_json(&call.func, "from_value") {
            if let Some(arg) = call.args.first() {
                if is_to_value_call(arg) || is_named_binding(arg, &self.names) {
                    self.findings.push(finding(
                        "no_json_roundtrip",
                        "This JSON round-trip launders a type and fabricates evidence.",
                        "Convert through a named domain type, or parse once at the I/O boundary.",
                        self.path,
                        call.func.span(),
                    ));
                }
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_item_fn(&mut self, i: &ItemFn) {
        let mut inner = Scan {
            path: self.path,
            findings: self.findings,
            names: HashSet::new(),
        };
        inner.visit_block(&i.block);
    }
}

fn is_to_value_call(expr: &Expr) -> bool {
    match peel_expr(expr) {
        Expr::Call(c) => path_is_serde_json(&c.func, "to_value"),
        _ => false,
    }
}

fn is_named_binding(expr: &Expr, names: &HashSet<String>) -> bool {
    match peel_expr(expr) {
        Expr::Path(p) => p
            .path
            .get_ident()
            .is_some_and(|id| names.contains(&id.to_string())),
        _ => false,
    }
}
