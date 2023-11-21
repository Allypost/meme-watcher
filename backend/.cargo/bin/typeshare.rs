use std::{collections::HashMap, path::PathBuf, process::Command};

pub fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");

    let mapping: HashMap<&str, Vec<&str>> = HashMap::from_iter([
        // (
        //     "../frontend/src/types/rust.entity.d.ts",
        //     vec!["./crates/entity"],
        // ),
        ("../frontend/src/types/rust.api.d.ts", vec!["./crates/api"]),
    ]);

    for (output, inputs) in mapping {
        let output = PathBuf::from_iter([dir, output]);
        let inputs = inputs
            .iter()
            .map(|input| PathBuf::from_iter([dir, input]))
            .collect::<Vec<_>>();

        Command::new("typeshare")
            .args(inputs)
            .args(["--lang", "typescript", "--output-file"])
            .arg(output)
            .current_dir(PathBuf::from(dir))
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
    }
}
