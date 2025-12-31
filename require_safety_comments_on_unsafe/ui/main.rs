// Lint should warn on unsafe blocks without safety comments

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

    // Again no warning should be emitted here
    // SAFETY: all preconditions are met
    unsafe {
        another_dangerous();
    }

    // This should be valid
    // SAFETY: Some security conditions are satisfied
    unsafe {
        safe_documented_function();
    }

    // This should emit a warning because this unsafe block has no safety comment
    unsafe {
        safe_documented_function();
    }
}