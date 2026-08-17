use std::path::Path;

use syn::spanned::Spanned;
use syn::{GenericArgument, PathArguments, Type};

use crate::find::{finding, Finding};
use crate::rules::{is_stringish, last_seg, strip_ref, visit_domain_types};

pub fn lint_file(file: &syn::File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    visit_domain_types(file, |ty| {
        if is_stringly_result(ty) {
            findings.push(finding(
                "no_stringly_error",
                "This `Result<T, String>` (or `str` / `Cow<str>`) is a weak error contract.",
                "Use a named error type. `Result<T, String>` is a weak contract.",
                path,
                ty.span(),
            ));
        }
    });
    findings
}

fn is_stringly_result(ty: &Type) -> bool {
    let Some(err) = result_err_ty(ty) else {
        return false;
    };
    is_stringly_err(err)
}

fn is_stringly_err(ty: &Type) -> bool {
    let ty = strip_ref(ty);
    if is_stringish(ty) {
        return true;
    }
    is_cow_str(ty)
}

fn is_cow_str(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    if last_seg(&p.path) != "Cow" {
        return false;
    }
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    args.args.iter().any(|arg| match arg {
        GenericArgument::Type(inner) => {
            let Type::Path(p) = strip_ref(inner) else {
                return false;
            };
            last_seg(&p.path) == "str"
        }
        _ => false,
    })
}

fn result_err_ty(ty: &Type) -> Option<&Type> {
    let Type::Path(p) = strip_ref(ty) else {
        return None;
    };
    let last = p.path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let types: Vec<&Type> = args
        .args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::Type(t) => Some(t),
            _ => None,
        })
        .collect();
    if types.len() < 2 {
        return None;
    }
    Some(types[1])
}
