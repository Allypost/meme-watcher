use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use config::CONFIG;
use entity::{file_metadata, files};
use futures::StreamExt;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{thumb::ThumbSize, FileWatcher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    #[serde(rename = "type")]
    file_type: Option<String>,
    size: Option<i64>,
    created: Option<DateTime<Utc>>,
    modified: Option<DateTime<Utc>>,
}

impl FileWatcher {
    pub async fn get_indexed(&self) -> Result<Vec<files::Model>> {
        let files = files::Entity::find()
            .filter(file_metadata::Column::Id.is_not_null())
            .left_join(file_metadata::Entity)
            .all(self.db())
            .await?;

        Ok(files)
    }

    pub async fn get_indexed_paths(&self) -> Result<Vec<PathBuf>> {
        let files = self.get_indexed().await?;

        Ok(files
            .into_iter()
            .map(|x| CONFIG.app.directory_absolute(&x.path))
            .collect())
    }

    pub async fn get_unindexed<'a, T>(&self, files_in_directory: &T) -> Result<Vec<PathBuf>>
    where
        T: IntoIterator<Item = PathBuf> + Clone,
    {
        let indexed_files = self
            .get_indexed()
            .await?
            .into_iter()
            .map(|x| x.path)
            .collect::<HashSet<_>>();

        logger::trace!(
            num_files = indexed_files.len(),
            "found indexed files in database",
        );

        let new_files = files_in_directory
            .clone()
            .into_iter()
            .map(|x| CONFIG.app.directory_relative(x))
            .collect::<Result<HashSet<_>>>()?;

        logger::trace!(num_files = new_files.len(), "found files in directory");

        let unindexed_files = new_files
            .difference(&indexed_files)
            .map(|x| CONFIG.app.directory_absolute(x))
            .collect::<Vec<_>>();

        Ok(unindexed_files)
    }

    pub async fn prune_indexed<'a, T>(&self, files_in_directory: &T) -> Result<Arc<Vec<String>>>
    where
        T: IntoIterator<Item = PathBuf> + Clone,
    {
        logger::trace!("pruning moved/deleted inspected files");
        let db_files = self.get_indexed_paths().await?;
        logger::trace!(
            num_files = db_files.len(),
            "found indexed files in database"
        );
        let db_files = db_files.into_iter().collect::<HashSet<_>>();
        logger::trace!(num_files = db_files.len(), "of those are unique");

        let files = files_in_directory
            .clone()
            .into_iter()
            .collect::<HashSet<_>>();

        let removed_files = db_files
            .difference(&files)
            .map(|x| CONFIG.app.directory_relative(x))
            .collect::<Result<Vec<_>>>()?;
        logger::trace!(num_files = removed_files.len(), "found files to remove");
        let removed_files = Arc::new(removed_files);

        if removed_files.is_empty() {
            return Ok(removed_files);
        }

        let res = files::Entity::delete_many()
            .filter(files::Column::Path.is_in(removed_files.iter()))
            .exec(self.db())
            .await?;

        logger::trace!(
            count = res.rows_affected,
            files = ?removed_files,
            "Removed files",
        );

        Ok(removed_files)
    }

    #[instrument(skip(self))]
    pub async fn index_file(&self, file_path: &Path) -> Result<PathBuf> {
        logger::debug!("Inspecting file: {:?}", file_path);

        let file = self.get_or_create_file(file_path).await?;

        self.get_or_create_file_metadata(&file.ulid).await?;

        self.get_or_generate_media_dimensions(&file.ulid).await?;

        if self
            .generate_blurhash(file_path.to_path_buf(), file.id)
            .await
            .is_err()
        {
            logger::debug!("Failed to generate blurhash from raw file. Generating from thumb");

            let thumb = self
                .get_or_generate_thumb(&file.ulid, ThumbSize::Poster)
                .await;

            match thumb {
                Ok(thumb) => {
                    let thumb_path = CONFIG
                        .app
                        .metadata_directory_absolute(&thumb.path.to_string_lossy());
                    if let Err(e) = self.generate_blurhash(thumb_path, file.id).await {
                        logger::warn!(err = ?e, ?thumb, "failed to generate blurhash");
                    }
                }
                Err(e) => {
                    logger::warn!(err = ?e, ?file, "failed to generate thumb");
                }
            }
        };

        Ok(file_path.to_path_buf())
    }

    pub async fn index_files(&self) -> Result<Vec<PathBuf>> {
        let files = self.scan_directory().await;
        logger::trace!(num_files = files.len(), "scanned directory");

        let old_files = self.prune_indexed(&files).await?;
        logger::trace!(num_old_files = old_files.len(), "pruned old files");

        let new_files = self.get_unindexed(&files).await?;
        logger::trace!(num_new_files = new_files.len(), "found new files");

        logger::trace!("starting file inspection");
        let res = new_files
            .iter()
            .map(|x| async move { (x.clone(), self.index_file(x).await) });
        let mut buff = tokio_stream::iter(res)
            .buffer_unordered(num_cpus::get())
            .boxed();
        let mut inspected = Vec::new();
        while let Some((path, res)) = buff.next().await {
            match res {
                Ok(x) => inspected.push(x),
                Err(e) => {
                    logger::error!(path = ?path, "failed to index file: {}", e);
                    continue;
                }
            }
        }
        logger::trace!(num_inspected = inspected.len(), "finished inspecting files");

        Ok(inspected)
    }
}
