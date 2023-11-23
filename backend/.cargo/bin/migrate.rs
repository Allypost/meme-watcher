use std::{path::PathBuf, process::Command};

pub fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let migrations_dir = dir.join("./crates/migration");

    let run = Command::new("cargo")
        .args(["run", "--", "up"])
        .current_dir(migrations_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(run.success(), "Failed to generate entities: {run:?}");

    let run = Command::new("cargo")
        .args(["generate-entities"])
        .current_dir(dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(run.success(), "Failed to generate entities: {run:?}");
}
