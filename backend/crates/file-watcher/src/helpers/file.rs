use std::{fs, io, path::Path};

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha512};
use tokio::task;

pub async fn file_hash(file: &Path) -> Result<String> {
    let f = file.to_path_buf();

    task::spawn_blocking(|| -> Result<String> {
        let input = fs::File::open(f)?;
        let mut reader = io::BufReader::new(input);
        let mut hasher = Sha512::new();

        io::copy(&mut reader, &mut hasher).map_err(|e| anyhow!("Failed to hash file: {}", e))?;

        Ok(format!("{digest:x}", digest = hasher.finalize()))
    })
    .await?
}
