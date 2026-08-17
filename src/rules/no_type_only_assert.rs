use std::path::Path;

use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, File, Macro, UnOp};

use crate::find::{finding, Finding};
use crate::rules::last_seg;

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = TypeOnly {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct TypeOnly<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl TypeOnly<'_> {
    fn check_macro(&mut self, mac: &Macro) {
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
                "Assert the value (`assert_eq!(x, Some(v))`). `is_ok`/`is_some` discards the evidence.",
                self.path,
                mac.path.span(),
            ));
        }
    }
}

impl Visit<'_> for TypeOnly<'_> {
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
