use std::time::UNIX_EPOCH;

fn main() {
    let time = UNIX_EPOCH.elapsed().unwrap().as_secs();
    println!("cargo::rustc-env=COMPILE_CURRENT_TIME={time}");
}
