mod no_any_bag;
mod no_automock;
mod no_dead_guard;
mod no_discarded_error;
mod no_empty_safety;
mod no_erasure_cast;
mod no_error_swallow;
mod no_json_roundtrip;
mod no_untyped_map;
mod no_value_soup;

use std::path::Path;

use syn::visit::Visit;
use syn::{
    FnArg, GenericArgument, ImplItemFn, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait,
    ItemType, ItemUnion, PathArguments, ReturnType, TraitItemFn, Type, TypeParamBound,
};

use crate::find::{Comment, Finding};

pub(crate) fn lint_rule(
    rule: &str,
    file: &syn::File,
    src: &str,
    path: &Path,
    comments: &[Comment],
) -> Vec<Finding> {
    match rule {
        "no_erasure_cast" => no_erasure_cast::lint_file(file, src, path),
        "no_any_bag" => no_any_bag::lint_file(file, src, path),
        "no_value_soup" => no_value_soup::lint_file(file, src, path),
        "no_untyped_map" => no_untyped_map::lint_file(file, src, path),
        "no_discarded_error" => no_discarded_error::lint_file(file, src, path),
        "no_automock" => no_automock::lint_file(file, src, path),
        "no_dead_guard" => no_dead_guard::lint_file(file, src, path),
        "no_json_roundtrip" => no_json_roundtrip::lint_file(file, src, path),
        "no_error_swallow" => no_error_swallow::lint_file(file, src, path),
        "no_empty_safety" => no_empty_safety::lint_file(file, src, path, comments),
        _ => Vec::new(),
    }
}

pub(crate) fn path_segments(path: &syn::Path) -> Vec<String> {
    path.segments.iter().map(|s| s.ident.to_string()).collect()
}

pub(crate) fn last_seg(path: &syn::Path) -> String {
    match path.segments.last() {
        Some(s) => s.ident.to_string(),
        None => String::new(),
    }
}

pub(crate) fn strip_ref(ty: &Type) -> &Type {
    match ty {
        Type::Reference(r) => strip_ref(&r.elem),
        Type::Paren(p) => strip_ref(&p.elem),
        Type::Group(g) => strip_ref(&g.elem),
        other => other,
    }
}

pub(crate) fn is_any_trait(ty: &Type) -> bool {
    match strip_ref(ty) {
        Type::TraitObject(obj) => obj.bounds.iter().any(|b| match b {
            TypeParamBound::Trait(t) => last_seg(&t.path) == "Any",
            _ => false,
        }),
        Type::Paren(p) => is_any_trait(&p.elem),
        _ => false,
    }
}

pub(crate) fn is_smart_dyn_any(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    if !matches!(last.ident.to_string().as_str(), "Box" | "Arc" | "Rc") {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    args.args.iter().any(|arg| match arg {
        GenericArgument::Type(inner) => is_any_trait(inner),
        _ => false,
    })
}

pub(crate) fn is_any_bag(ty: &Type) -> bool {
    is_any_trait(ty) || is_smart_dyn_any(ty)
}

pub(crate) fn is_value_soup(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    let segs = path_segments(&p.path);
    matches!(
        segs.as_slice(),
        [.., crate_name, val]
            if val == "Value"
                && matches!(crate_name.as_str(), "serde_json" | "toml" | "serde_yaml")
    )
}

pub(crate) fn is_bare_value(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    last_seg(&p.path) == "Value"
}

pub(crate) fn is_stringish(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    matches!(last_seg(&p.path).as_str(), "String" | "str")
}

pub(crate) fn is_untyped_map(ty: &Type) -> bool {
    let Type::Path(p) = strip_ref(ty) else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    if !matches!(
        last.ident.to_string().as_str(),
        "HashMap" | "BTreeMap" | "IndexMap"
    ) {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
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
        return false;
    }
    let value = types[1];
    is_stringish(types[0]) && (is_any_bag(value) || is_value_soup(value) || is_bare_value(value))
}

/// Walk param / return / field / alias types only — the domain-type positions.
pub(crate) fn visit_domain_types(file: &syn::File, on_type: impl FnMut(&Type)) {
    let mut v = DomainTypes { on_type };
    v.visit_file(file);
}

struct DomainTypes<F> {
    on_type: F,
}

impl<F: FnMut(&Type)> DomainTypes<F> {
    fn sig(&mut self, sig: &syn::Signature) {
        for arg in &sig.inputs {
            if let FnArg::Typed(t) = arg {
                (self.on_type)(&t.ty);
            }
        }
        if let ReturnType::Type(_, ty) = &sig.output {
            (self.on_type)(ty);
        }
    }

    fn walk_block_items(&mut self, block: &syn::Block) {
        for stmt in &block.stmts {
            if let syn::Stmt::Item(item) = stmt {
                self.visit_item(item);
            }
        }
    }
}

impl<F: FnMut(&Type)> Visit<'_> for DomainTypes<F> {
    fn visit_item_fn(&mut self, i: &ItemFn) {
        self.sig(&i.sig);
        self.walk_block_items(&i.block);
    }

    fn visit_impl_item_fn(&mut self, i: &ImplItemFn) {
        self.sig(&i.sig);
        self.walk_block_items(&i.block);
    }

    fn visit_trait_item_fn(&mut self, i: &TraitItemFn) {
        self.sig(&i.sig);
        if let Some(block) = &i.default {
            self.walk_block_items(block);
        }
    }

    fn visit_item_struct(&mut self, i: &ItemStruct) {
        for field in &i.fields {
            (self.on_type)(&field.ty);
        }
    }

    fn visit_item_enum(&mut self, i: &ItemEnum) {
        for variant in &i.variants {
            for field in &variant.fields {
                (self.on_type)(&field.ty);
            }
        }
    }

    fn visit_item_union(&mut self, i: &ItemUnion) {
        for field in &i.fields.named {
            (self.on_type)(&field.ty);
        }
    }

    fn visit_item_type(&mut self, i: &ItemType) {
        (self.on_type)(&i.ty);
    }

    fn visit_item_impl(&mut self, i: &ItemImpl) {
        for item in &i.items {
            self.visit_impl_item(item);
        }
    }

    fn visit_item_trait(&mut self, i: &ItemTrait) {
        for item in &i.items {
            self.visit_trait_item(item);
        }
    }

    fn visit_item_mod(&mut self, i: &ItemMod) {
        if let Some((_, items)) = &i.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }
}

pub(crate) fn peel_expr(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Paren(p) => peel_expr(&p.expr),
        syn::Expr::Group(g) => peel_expr(&g.expr),
        syn::Expr::Try(t) => peel_expr(&t.expr),
        syn::Expr::MethodCall(m) => peel_expr(&m.receiver),
        other => other,
    }
}

pub(crate) fn path_is_serde_json(func: &syn::Expr, name: &str) -> bool {
    let syn::Expr::Path(p) = peel_expr(func) else {
        return false;
    };
    let segs = path_segments(&p.path);
    segs.last().is_some_and(|n| n == name) && segs.iter().any(|s| s == "serde_json")
}

pub(crate) fn receiver_key(expr: &syn::Expr) -> Option<String> {
    match peel_expr(expr) {
        syn::Expr::Path(p) => Some(
            p.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        syn::Expr::Field(f) => {
            let base = receiver_key(&f.base)?;
            let mem = match &f.member {
                syn::Member::Named(id) => id.to_string(),
                syn::Member::Unnamed(idx) => idx.index.to_string(),
            };
            Some(format!("{base}.{mem}"))
        }
        _ => None,
    }
}
