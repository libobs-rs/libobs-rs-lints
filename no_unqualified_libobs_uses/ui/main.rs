// This test should trigger the lint warning
use libobs::obs_get_audio;

fn test_unqualified() {
    unsafe { obs_get_audio() };
}

fn main() {}

// This test should NOT trigger the lint warning
fn test_qualified() {
    unsafe { libobs::obs_get_audio() };
}
