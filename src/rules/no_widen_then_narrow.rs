use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprCast, ExprMethodCall, File, Type};

use crate::find::{finding, Finding};
use crate::rules::{last_seg, strip_ref};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = Widen {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct Widen<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Widen<'_> {
    fn push(&mut self, span: proc_macro2::Span) {
        self.findings.push(finding(
            "no_widen_then_narrow",
            "This widen-then-narrow conversion discards range evidence.",
            "Keep the owner width, or convert once through a named type.",
            self.path,
            span,
        ));
    }
}

impl Visit<'_> for Widen<'_> {
    fn visit_expr_cast(&mut self, node: &ExprCast) {
        if let Expr::Cast(inner) = peel_wrap(&node.expr) {
            if let (Some(outer), Some(inner_k)) = (numeric_kind(&node.ty), numeric_kind(&inner.ty))
            {
                if is_narrower(outer, inner_k) {
                    self.push(node.span());
                }
            }
        }
        syn::visit::visit_expr_cast(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &ExprMethodCall) {
        let name = node.method.to_string();
        if matches!(name.as_str(), "unwrap" | "expect") {
            if let Expr::MethodCall(inner) = peel_wrap(&node.receiver) {
                if inner.method == "try_into" && matches!(peel_wrap(&inner.receiver), Expr::Cast(_))
                {
                    self.push(node.method.span());
                }
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn peel_wrap(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(p) => peel_wrap(&p.expr),
        Expr::Group(g) => peel_wrap(&g.expr),
        Expr::Try(t) => peel_wrap(&t.expr),
        other => other,
    }
}

#[derive(Clone, Copy)]
enum NumKind {
    Int(u8),
    Float(u8),
    Usize,
    Isize,
}

fn numeric_kind(ty: &Type) -> Option<NumKind> {
    let Type::Path(p) = strip_ref(ty) else {
        return None;
    };
    match last_seg(&p.path).as_str() {
        "u8" | "i8" => Some(NumKind::Int(8)),
        "u16" | "i16" => Some(NumKind::Int(16)),
        "u32" | "i32" => Some(NumKind::Int(32)),
        "u64" | "i64" => Some(NumKind::Int(64)),
        "u128" | "i128" => Some(NumKind::Int(128)),
        "f32" => Some(NumKind::Float(32)),
        "f64" => Some(NumKind::Float(64)),
        "usize" => Some(NumKind::Usize),
        "isize" => Some(NumKind::Isize),
        _ => None,
    }
}

fn width(kind: NumKind) -> Option<u8> {
    match kind {
        NumKind::Int(w) | NumKind::Float(w) => Some(w),
        NumKind::Usize | NumKind::Isize => None,
    }
}

fn is_ptr_unknown_pair(a: NumKind, b: NumKind) -> bool {
    let other = match (a, b) {
        (NumKind::Usize, other) | (other, NumKind::Usize) => other,
        (NumKind::Isize, other) | (other, NumKind::Isize) => other,
        _ => return false,
    };
    matches!(other, NumKind::Int(8 | 16 | 32))
}

fn is_narrower(outer: NumKind, inner: NumKind) -> bool {
    match (outer, inner) {
        (NumKind::Usize, NumKind::Usize)
        | (NumKind::Isize, NumKind::Isize)
        | (NumKind::Usize, NumKind::Isize)
        | (NumKind::Isize, NumKind::Usize) => false,
        _ if is_ptr_unknown_pair(outer, inner) => true,
        _ => match (width(outer), width(inner)) {
            (Some(o), Some(i)) => o < i,
            _ => false,
        },
    }
}
