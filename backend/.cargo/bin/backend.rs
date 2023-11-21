use std::{path::PathBuf, process::Command};

pub fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");

    Command::new("cargo")
        .args([
            "watch", "--clear", "--quiet", "--watch", "backend", "--exec", "run",
        ])
        .current_dir(PathBuf::from(dir))
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}
