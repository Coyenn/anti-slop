use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, File, Macro};

use crate::find::{finding, Finding};
use crate::rules::last_seg;

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    let mut v = Auto {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct Auto<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Auto<'_> {
    fn check_macro(&mut self, mac: &Macro) {
        if last_seg(&mac.path) != "mock" {
            return;
        }
        self.findings.push(finding(
            "no_automock",
            "This mock fabricates a stand-in instead of a real dependency seam.",
            "Inject a named trait or type the test can provide.",
            self.path,
            match mac.path.segments.last() {
                Some(s) => s.ident.span(),
                None => mac.span(),
            },
        ));
    }
}

impl Visit<'_> for Auto<'_> {
    fn visit_attribute(&mut self, attr: &Attribute) {
        if is_automock(attr) {
            self.findings.push(finding(
                "no_automock",
                "This mock fabricates a stand-in instead of a real dependency seam.",
                "Inject a named trait or type the test can provide.",
                self.path,
                match attr.path().segments.last() {
                    Some(s) => s.ident.span(),
                    None => attr.span(),
                },
            ));
        }
    }

    fn visit_item_macro(&mut self, i: &syn::ItemMacro) {
        self.check_macro(&i.mac);
        syn::visit::visit_item_macro(self, i);
    }

    fn visit_stmt_macro(&mut self, i: &syn::StmtMacro) {
        self.check_macro(&i.mac);
        syn::visit::visit_stmt_macro(self, i);
    }

    fn visit_expr_macro(&mut self, i: &syn::ExprMacro) {
        self.check_macro(&i.mac);
        syn::visit::visit_expr_macro(self, i);
    }
}

fn is_automock(attr: &Attribute) -> bool {
    if last_seg(attr.path()) == "automock" {
        return true;
    }
    let syn::Meta::List(list) = &attr.meta else {
        return false;
    };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    list.tokens.to_string().split(',').any(|part| {
        let p = part.trim();
        p == "automock" || p.ends_with("automock")
    })
}
