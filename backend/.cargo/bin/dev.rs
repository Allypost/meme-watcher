use std::{
    path::PathBuf,
    process::{Child, Command},
};

struct ExecCommand {
    cmd: Command,
    process: Option<Child>,
}

impl Drop for ExecCommand {
    fn drop(&mut self) {
        if let Some(process) = self.process.as_mut() {
            process.kill().unwrap();
        }
    }
}

impl ExecCommand {
    fn new(cmd: Command) -> Self {
        Self { cmd, process: None }
    }

    fn exec(mut self) {
        if cfg!(windows) {
            let child = self.cmd.spawn().unwrap();

            self.process = Some(child);

            let child = self.process.as_mut().unwrap();

            child.wait().unwrap();
            child.kill().unwrap();
        } else {
            use std::os::unix::process::CommandExt;

            self.cmd.exec();
        }
    }
}
pub fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");

    let mut cmd = Command::new("cargo");
    cmd.args([
        "watch",
        "--clear",
        "--quiet",
        "--watch",
        "./src",
        "--watch",
        "./crates",
        "--ignore",
        "./crates/migration/**/*",
        "--exec",
        "typeshare", // Build types for the frontend
        "--exec",
        "run", // And then run the server
    ])
    .current_dir(PathBuf::from(dir));

    ExecCommand::new(cmd).exec();
}
