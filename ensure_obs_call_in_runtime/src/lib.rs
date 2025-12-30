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

use std::cell::RefCell;

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::{Expr, ExprKind, HirId};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Ensures every libobs call happens inside a closure passed to
    /// `runtime.run_with_obs` or `runtime.run_with_obs_result`.
    ///
    /// ### Why is this bad?
    ///
    /// libobs must only be accessed when the runtime is active. Wrapping calls in
    /// the runtime helpers guarantees the library is initialized and used on the
    /// correct thread.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn main() {
    ///     unsafe { libobs::obs_get_audio() }; // warns
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```no_check
    /// fn main(runtime: &ObsRuntime) {
    ///     runtime.run_with_obs(move || unsafe { libobs::obs_get_audio() });
    /// }
    /// ```
    pub ENSURE_OBS_CALL_IN_RUNTIME,
    Warn,
    "libobs calls must be wrapped in runtime.run_with_obs or runtime.run_with_obs_result"
}

impl<'tcx> LateLintPass<'tcx> for EnsureObsCallInRuntime {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::MethodCall(segment, _receiver, args, _) = &expr.kind {
            let name = segment.ident.name.as_str();
            if name == "run_with_obs" || name == "run_with_obs_result" {
                mark_allowed_closure_args(args);
            }
        }

        if let ExprKind::Closure(_) = expr.kind {
            push_closure_allowed(expr.hir_id);
        }

        if let ExprKind::Call(func, _) = expr.kind {
            if let ExprKind::Path(qpath) = &func.kind {
                if let Some(def_id) = cx.qpath_res(qpath, func.hir_id).opt_def_id() {
                    if is_from_libobs_crate(cx.tcx, def_id) && !currently_allowed() {
                        cx.span_lint(
                            ENSURE_OBS_CALL_IN_RUNTIME,
                            expr.span,
                            |diag| {
                                diag.help(
                                    "wrap libobs calls in runtime.run_with_obs or runtime.run_with_obs_result",
                                );
                            },
                        );
                    }
                }
            }
        }
    }

    fn check_expr_post(&mut self, _cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::Closure(_) = expr.kind {
            pop_closure_allowed();
        }
    }
}

fn is_from_libobs_crate(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.crate_name(def_id.krate).as_str() == "libobs"
}

thread_local! {
    static ALLOWED_CLOSURES: RefCell<FxHashSet<HirId>> = RefCell::new(FxHashSet::default());
    static CLOSURE_STACK: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

fn mark_allowed_closure_args(args: &[Expr<'_>]) {
    ALLOWED_CLOSURES.with(|allowed| {
        let mut allowed = allowed.borrow_mut();
        for arg in args {
            if matches!(arg.kind, ExprKind::Closure(_)) {
                allowed.insert(arg.hir_id);
            }
        }
    });
}

fn push_closure_allowed(hir_id: HirId) {
    let is_allowed = ALLOWED_CLOSURES.with(|allowed| allowed.borrow().contains(&hir_id));

    CLOSURE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let inherited = *stack.last().unwrap_or(&false);
        stack.push(is_allowed || inherited);
    });
}

fn pop_closure_allowed() {
    CLOSURE_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
}

fn currently_allowed() -> bool {
    CLOSURE_STACK.with(|stack| *stack.borrow().last().unwrap_or(&false))
}

#[test]
fn ui() {
    dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "ui");
}
