use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ExprPath, File};

use crate::find::{finding, Finding};
use crate::rules::{last_seg, path_segments};

pub fn lint_file(file: &File, _src: &str, path: &Path) -> Vec<Finding> {
    if allowed_env_file(path) {
        return Vec::new();
    }
    let mut v = AdHoc {
        path,
        findings: Vec::new(),
    };
    v.visit_file(file);
    v.findings
}

struct AdHoc<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl Visit<'_> for AdHoc<'_> {
    fn visit_expr_path(&mut self, node: &ExprPath) {
        if is_env_var_path(&node.path) {
            self.findings.push(finding(
                "no_ad_hoc_env",
                "This ad-hoc env read skips a named config type.",
                "Read env once in a config module and parse into a named type.",
                self.path,
                node.path.span(),
            ));
        }
        syn::visit::visit_expr_path(self, node);
    }
}

fn is_env_var_path(path: &syn::Path) -> bool {
    let segs = path_segments(path);
    let last = last_seg(path);
    if !matches!(last.as_str(), "var" | "var_os" | "vars" | "vars_os") {
        return false;
    }
    segs.iter().any(|s| s == "env") || segs.len() == 1
}

fn allowed_env_file(path: &Path) -> bool {
    if path
        .file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|stem| stem == "config" || stem == "settings")
    {
        return true;
    }
    path.components()
        .any(|c| matches!(c.as_os_str().to_str(), Some("config") | Some("settings")))
}
