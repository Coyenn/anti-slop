use std::path::Path;

use syn::spanned::Spanned;
use syn::{GenericArgument, PathArguments, Type, TypeParamBound};

use crate::find::{finding, Finding};
use crate::rules::{last_seg, path_segments, strip_ref, visit_domain_types};

pub fn lint_file(file: &syn::File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    visit_domain_types(file, |ty| {
        if is_erased_error(ty) {
            findings.push(finding(
                "no_boxed_error",
                "This erased error type discards the domain error contract.",
                "Name a domain error type. `Box<dyn Error>` / anyhow erase the contract.",
                path,
                ty.span(),
            ));
        }
    });
    findings
}

fn is_erased_error(ty: &Type) -> bool {
    if is_box_dyn_error(ty) || is_anyhow_error(ty) || is_eyre_report(ty) {
        return true;
    }
    if let Some(err) = result_err_ty(ty) {
        return is_box_dyn_error(err) || is_anyhow_error(err) || is_eyre_report(err);
    }
    false
}

fn is_box_dyn_error(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    if last.ident != "Box" {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    args.args.iter().any(|arg| match arg {
        GenericArgument::Type(inner) => is_dyn_error(inner),
        _ => false,
    })
}

fn is_dyn_error(ty: &Type) -> bool {
    match strip_ref(ty) {
        Type::TraitObject(obj) => obj.bounds.iter().any(|b| match b {
            TypeParamBound::Trait(t) => last_seg(&t.path) == "Error",
            _ => false,
        }),
        Type::Paren(p) => is_dyn_error(&p.elem),
        Type::Group(g) => is_dyn_error(&g.elem),
        _ => false,
    }
}

fn is_anyhow_error(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    last_seg(&p.path) == "Error" && path_segments(&p.path).iter().any(|s| s == "anyhow")
}

fn is_eyre_report(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    last_seg(&p.path) == "Report" && path_segments(&p.path).iter().any(|s| s == "eyre")
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
