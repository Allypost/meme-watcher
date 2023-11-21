use std::{path::PathBuf, process::Command};

pub fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let migrations_dir = dir.join("./crates/migration");

    let migrations = Command::new("cargo")
        .args(["run", "--", "up"])
        .current_dir(migrations_dir)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if !migrations.status.success() {
        panic!("Migration failed");
    }

    Command::new("cargo")
        .args(["generate-entities"])
        .current_dir(dir)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}
