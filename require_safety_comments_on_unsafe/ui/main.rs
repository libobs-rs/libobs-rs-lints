// Lint should warn on unsafe functions without safety comments

// expect warning: missing safety comment on function
unsafe fn dangerous_function() {
    // ...
}

// expect warning: missing safety comment on function
unsafe fn another_dangerous() {
    // ...
}

/// # Safety
///
/// This function is safe to call only if the preconditions are met.
unsafe fn safe_documented_function() {
    // ...
}

fn main() {
    // expect warning: missing safety comment before call
    unsafe {
        dangerous_function();
    }

    // No warning: SAFETY comment provided
    unsafe {
        // SAFETY: all preconditions are met
        another_dangerous();
    }

    // Still should cause errors because you have to document why this function is safe to call in this context
    unsafe {
        safe_documented_function();
    }

    // This should not emit any warnings because the function itself is documented and its call
    unsafe {
        // Safety: I can call this because I'm cool
        safe_documented_function();
    }
}

fn nested_test() {
    // expect warning: missing safety comment before call
    unsafe {
        // safety comment is missing here
        dangerous_function();
    }

    // No warning: SAFETY comment provided
    unsafe {
        // SAFETY: we have verified the invariants
        another_dangerous();
    }

    // Warning: missing safety comment before call
    unsafe {
        // Safety comment should be here
        safe_documented_function();

        // expect warning: missing safety comment before call
        safe_documented_function();
    }
}

fn external_call() {
    unsafe {
        // expect warning: missing SAFETY comment
        dangerous_function();
    }
}
