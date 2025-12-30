// This test should trigger the lint warning
use libobs::obs_get_audio;

use libobs_window_helper::{WindowSearchMode, get_all_windows};

fn test_unqualified() {
    unsafe { obs_get_audio() };
}

fn main() {}

// This test should NOT trigger the lint warning
fn test_qualified() {
    // This is from the libobs_sys crate, so it should work fine
    let _ = get_all_windows(WindowSearchMode::ExcludeMinimized);
    unsafe { libobs::obs_get_audio() };
}
