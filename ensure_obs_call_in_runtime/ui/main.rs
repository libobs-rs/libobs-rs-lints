// Lint should warn on libobs calls outside runtime helpers
use libobs::obs_get_audio;

struct Runtime;

impl Runtime {
    fn run_with_obs<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        f();
    }

    fn run_with_obs_result<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        f()
    }
}

fn main() {
    let runtime = Runtime;

    runtime.run_with_obs(move || unsafe {
        obs_get_audio();
    });

    let _ = runtime.run_with_obs_result(move || unsafe {
        obs_get_audio();
        42
    });

    unsafe {
        obs_get_audio(); // expect warning
    }
}

fn test() {
    unsafe {
        obs_get_audio(); // expect warning
    }
}