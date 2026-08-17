use std::path::Path;

use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Expr, File, ImplItemFn, ItemFn, ItemMod, Macro, TraitItemFn, UnOp};

use crate::find::{finding, Finding};
use crate::rules::last_seg;

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    if is_integration_test_path(path) {
        return Vec::new();
    }
    let mut v = TypeOnly {
        path,
        findings: Vec::new(),
        in_test: file.attrs.iter().any(is_test_attr),
    };
    v.visit_file(file);
    v.findings
}

struct TypeOnly<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
    in_test: bool,
}

impl TypeOnly<'_> {
    fn enter_test(&mut self, attrs: &[Attribute]) -> bool {
        let was = self.in_test;
        if attrs.iter().any(is_test_attr) {
            self.in_test = true;
        }
        was
    }

    fn check_macro(&mut self, mac: &Macro) {
        if self.in_test {
            return;
        }
        if last_seg(&mac.path) != "assert" {
            return;
        }
        let Some(cond) = parse_assert_cond(mac.tokens.clone()) else {
            return;
        };
        if is_type_only_pred(&cond) {
            self.findings.push(finding(
                "no_type_only_assert",
                "This assert checks a type tag and discards the value.",
                "Assert the value (`assert_eq!(x, Some(v))`). `is_ok`/`is_some` discards the evidence. Tests are exempt.",
                self.path,
                mac.path.span(),
            ));
        }
    }
}

impl Visit<'_> for TypeOnly<'_> {
    fn visit_item_fn(&mut self, i: &ItemFn) {
        let was = self.enter_test(&i.attrs);
        syn::visit::visit_item_fn(self, i);
        self.in_test = was;
    }

    fn visit_impl_item_fn(&mut self, i: &ImplItemFn) {
        let was = self.enter_test(&i.attrs);
        syn::visit::visit_impl_item_fn(self, i);
        self.in_test = was;
    }

    fn visit_trait_item_fn(&mut self, i: &TraitItemFn) {
        let was = self.enter_test(&i.attrs);
        syn::visit::visit_trait_item_fn(self, i);
        self.in_test = was;
    }

    fn visit_item_mod(&mut self, i: &ItemMod) {
        let was = self.enter_test(&i.attrs);
        syn::visit::visit_item_mod(self, i);
        self.in_test = was;
    }

    fn visit_expr_macro(&mut self, i: &syn::ExprMacro) {
        self.check_macro(&i.mac);
        syn::visit::visit_expr_macro(self, i);
    }

    fn visit_stmt_macro(&mut self, i: &syn::StmtMacro) {
        self.check_macro(&i.mac);
        syn::visit::visit_stmt_macro(self, i);
    }

    fn visit_item_macro(&mut self, i: &syn::ItemMacro) {
        self.check_macro(&i.mac);
        syn::visit::visit_item_macro(self, i);
    }
}

fn is_integration_test_path(path: &Path) -> bool {
    let comps: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    match comps.iter().position(|c| *c == "tests") {
        Some(i) => !comps[i + 1..].contains(&"fixtures"),
        None => false,
    }
}

fn is_test_attr(attr: &Attribute) -> bool {
    if last_seg(attr.path()) == "test" {
        return true;
    }
    let syn::Meta::List(list) = &attr.meta else {
        return false;
    };
    if !list.path.is_ident("cfg") {
        return false;
    }
    let words: Vec<String> = list
        .tokens
        .to_string()
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    words.iter().any(|w| w == "test") && words.iter().all(|w| w != "not")
}

fn parse_assert_cond(tokens: proc_macro2::TokenStream) -> Option<Expr> {
    fn leading_expr(input: ParseStream<'_>) -> syn::Result<Expr> {
        let expr: Expr = input.parse()?;
        if !input.is_empty() {
            let _: proc_macro2::TokenStream = input.parse()?;
        }
        Ok(expr)
    }
    match leading_expr.parse2(tokens) {
        Ok(expr) => Some(expr),
        Err(err) => {
            let _parse_err = err;
            None
        }
    }
}

fn is_type_only_pred(expr: &Expr) -> bool {
    match peel_assert_cond(expr) {
        Expr::MethodCall(m) => matches!(
            m.method.to_string().as_str(),
            "is_ok" | "is_err" | "is_some" | "is_none" | "is_empty"
        ),
        _ => false,
    }
}

fn peel_assert_cond(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(p) => peel_assert_cond(&p.expr),
        Expr::Group(g) => peel_assert_cond(&g.expr),
        Expr::Try(t) => peel_assert_cond(&t.expr),
        Expr::Unary(u) if matches!(u.op, UnOp::Not(_)) => peel_assert_cond(&u.expr),
        other => other,
    }
}
