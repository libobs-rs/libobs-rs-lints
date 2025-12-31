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

use rustc_hir::{Block, BlockCheckMode, Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::BytePos;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Ensures that every unsafe function has a safety comment explaining why it is unsafe.
    ///
    /// ### Why is this bad?
    ///
    /// Unsafe functions should be accompanied by documentation explaining the safety
    /// requirements and invariants that must be upheld by callers.
    ///
    /// ### Example
    ///
    /// ```rust
    /// unsafe fn dangerous_function() {
    ///     // ...
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// /// # Safety
    /// ///
    /// /// This function is unsafe because it dereferences a raw pointer.
    /// /// The caller must ensure the pointer is valid and properly aligned.
    /// unsafe fn dangerous_function() {
    ///     // ...
    /// }
    /// ```
    pub REQUIRE_SAFETY_COMMENTS_ON_UNSAFE,
    Warn,
    "unsafe functions must have a safety comment"
}


impl<'tcx> LateLintPass<'tcx> for RequireSafetyCommentsOnUnsafe {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        if let ItemKind::Fn { sig: fn_sig, .. } = &item.kind {
            // Check if the function is unsafe
            if fn_sig.header.is_unsafe() {
                if !has_safety_doc_comment(cx, item) {
                    cx.span_lint(
                        REQUIRE_SAFETY_COMMENTS_ON_UNSAFE,
                        item.span,
                        |diag| {
                            diag.help("add a safety comment explaining when it is safe to call this function");
                        },
                    );
                }
            }
        }
    }

    fn check_block(&mut self, cx: &LateContext<'tcx>, block: &'tcx Block<'tcx>) {
        // Check for unsafe blocks
        if matches!(block.rules, BlockCheckMode::UnsafeBlock(_)) {
            if !has_safety_comment_before_block(cx, block) {
                cx.span_lint(
                    REQUIRE_SAFETY_COMMENTS_ON_UNSAFE,
                    block.span,
                    |diag| {
                        diag.help("add a SAFETY comment explaining why this unsafe block is safe");
                    },
                );
            }
        }
    }
}

fn has_safety_doc_comment(cx: &LateContext<'_>, item: &Item<'_>) -> bool {
    // Look for "# Safety" in documentation comments above the function
    let source_map = cx.tcx.sess.source_map();
    let item_span = item.span;
    
    // Get the source file and calculate search range
    let file = source_map.lookup_source_file(item_span.lo());
    let file_start = file.start_pos;
    let search_start = if item_span.lo().0 >= file_start.0 + 1000 {
        BytePos(item_span.lo().0 - 1000)
    } else {
        file_start
    };
    
    // Search for "# Safety" in the text before the function
    let search_span = item_span.with_lo(search_start).with_hi(item_span.lo());
    if let Ok(preceding_text) = source_map.span_to_snippet(search_span) {
        if preceding_text.contains("# Safety") {
            return true;
        }
    }
    
    false
}

fn has_safety_comment_before_block(cx: &LateContext<'_>, block: &Block<'_>) -> bool {
    // Look for "// SAFETY:" comment before or inside an unsafe block
    let source_map = cx.tcx.sess.source_map();
    
    // Empty blocks don't need comments
    if block.stmts.is_empty() && block.expr.is_none() {
        return false;
    }
    
    // Get the start of the block
    let block_start = block.span.lo();
    let file = source_map.lookup_source_file(block_start);
    let file_start = file.start_pos;
    
    // Search back up to 300 characters for a SAFETY comment  
    let search_start = if block_start.0 >= file_start.0 + 300 {
        BytePos(block_start.0 - 300)
    } else {
        file_start
    };
    
    // Get text from 300 chars before the unsafe block to the start of it
    let search_span = block.span.with_lo(search_start).with_hi(block_start);
    
    if let Ok(preceding_text) = source_map.span_to_snippet(search_span) {
        // Check if "// SAFETY:" appears in the text before the block
        if preceding_text.contains("// SAFETY:") {
            return true;
        }
        // Also check for "// Safety:" variant
        if preceding_text.contains("// Safety:") {
            return true;
        }
    }
    
    false
}

#[test]
fn ui() {
    dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "ui");
}
