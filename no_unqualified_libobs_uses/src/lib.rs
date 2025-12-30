#![feature(rustc_private)]
#![allow(unused_extern_crates)]

extern crate rustc_arena;
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_index;
extern crate rustc_infer;
extern crate rustc_lexer;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_parse;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;

use rustc_hir::{Expr, ExprKind, QPath};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects unqualified uses of libobs functions that were imported with `use` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Using fully qualified paths (e.g., `libobs::obs_get_error()`) makes it clearer where
    /// functions come from, especially for FFI bindings. This improves code readability and
    /// makes it explicit that you're calling into the libobs C library.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use libobs::obs_get_audio;
    ///
    /// fn test() {
    ///     unsafe { obs_get_audio(); }
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// fn test() {
    ///     unsafe { libobs::obs_get_audio(); }
    /// }
    /// ```
    pub NO_UNQUALIFIED_LIBOBS_USES,
    Warn,
    "use of unqualified libobs functions; use fully qualified paths like `libobs::function_name()` instead"
}

impl<'tcx> LateLintPass<'tcx> for NoUnqualifiedLibobsUses {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Check if this is a function call or method call
        if let ExprKind::Call(func, _) = expr.kind {
            // Check if the function is a path expression
            if let ExprKind::Path(qpath) = &func.kind {
                // Check if this is an unqualified path (no module prefix)
                if let QPath::Resolved(None, path) = qpath {
                    // Get the resolution of this path
                    if let Some(def_id) = path.res.opt_def_id() {
                        // Check if this function is from the libobs crate
                        if is_from_libobs_crate(cx.tcx, def_id) {
                            // Check if the path has no qualifier (just the function name)
                            if path.segments.len() == 1 {
                                let fn_name = path.segments[0].ident.as_str();
                                cx.span_lint(
                                    NO_UNQUALIFIED_LIBOBS_USES,
                                    expr.span,
                                    |diag| {
                                        diag.note(format!(
                                            "use of unqualified libobs function; use `libobs::{}()` instead",
                                            fn_name
                                        ));
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_from_libobs_crate(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    // Get the crate name for this def_id
    if let Some(crate_name) = tcx.crate_name(def_id.krate).as_str().strip_prefix("libobs") {
        // Check if it's exactly "libobs" or starts with "libobs-" or "libobs_"
        crate_name.is_empty() || crate_name.starts_with('-') || crate_name.starts_with('_')
    } else {
        false
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "ui");
}
