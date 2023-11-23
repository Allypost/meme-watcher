use std::{path::PathBuf, process::Command};

pub fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");

    let run = Command::new("sea-orm-cli")
        .args([
            "generate",
            "entity",
            "--with-serde",
            "both",
            "--with-copy-enums",
            "--model-extra-attributes",
            "serde(rename_all = \"camelCase\"),typeshare::typeshare",
            "--output-dir",
            "./crates/entity/src/entities",
        ])
        .current_dir(PathBuf::from(dir))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(run.success(), "sea-orm-cli failed: {run:?}");
}
