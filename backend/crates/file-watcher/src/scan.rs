use std::{collections::HashSet, path::PathBuf};

use config::CONFIG;
use tokio::task;
use walkdir::WalkDir;

use crate::FileWatcher;

impl FileWatcher {
    pub async fn scan_directory(&self) -> HashSet<PathBuf> {
        let dir = &CONFIG.app.directory;
        let depth = if self.recursive { 10 } else { 1 };
        logger::debug!(dir = ?&dir, depth, "Scanning directory");

        task::spawn_blocking(move || {
            let mut files = HashSet::new();

            for entry in WalkDir::new(dir)
                .max_depth(depth)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                match entry.metadata() {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            files.insert(entry.path().to_path_buf());
                        }
                    }
                    Err(e) => {
                        logger::trace!("Skipping entry: {:?}", e);
                    }
                }
            }

            files
        })
        .await
        .unwrap_or_default()
    }
}
