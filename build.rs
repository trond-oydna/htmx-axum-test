use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rustc-env=COMPILE_CURRENT_TIME={time}");
}
