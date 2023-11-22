use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc};
use config::CONFIG;
use entity::files;
use file_format::FileFormat;
use infer::get_from_path as infer_from_path;
use sea_orm::{prelude::*, Condition, Set, TransactionTrait};
use tokio::fs;
use tracing::instrument;
use tree_magic_mini::from_filepath as magic_infer_from_filepath;
use ulid::Ulid;

use crate::{helpers::file::file_hash, FileWatcher};

impl FileWatcher {
    #[instrument(skip(self))]
    pub(crate) async fn get_or_create_file(&self, file_path: &Path) -> Result<files::Model> {
        let meta = fs::metadata(file_path)
            .await
            .map_err(|e| anyhow!("Failed to get metadata of file {:?}: {}", file_path, e))?;

        if !meta.is_file() {
            bail!("Not a file");
        }

        let file_path_rel = CONFIG.app.directory_relative(file_path)?;

        let file_hash = {
            let now = Instant::now();
            let res = file_hash(file_path).await;
            logger::trace!(hash = ?res, took = ?now.elapsed(), "Calculated file hash");
            res
        }?;

        let txn = self.db().begin().await?;

        let res = files::Entity::find()
            .filter(
                Condition::all()
                    .add(files::Column::Path.eq(&file_path_rel))
                    .add(files::Column::Hash.eq(&file_hash)),
            )
            .one(&txn)
            .await?;

        let db_file = match res {
            Some(db_file) => db_file,
            None => {
                let meta = fs::metadata(file_path)
                    .await
                    .map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;

                let file_type = match infer_file_type(file_path.to_path_buf()).await {
                    Ok(x) => Some(x),
                    Err(e) => {
                        logger::warn!(err = ?e, "Failed to infer file type");
                        None
                    }
                };
                let file_size: Option<i64> = meta.len().try_into().ok();
                let file_ctime = meta
                    .created()
                    .ok()
                    .map(DateTime::<Utc>::from)
                    .map(|x| x.to_rfc3339());
                let file_mtime = meta
                    .modified()
                    .ok()
                    .map(DateTime::<Utc>::from)
                    .map(|x| x.to_rfc3339());

                let db_file = files::ActiveModel {
                    path: Set(file_path_rel),
                    hash: Set(file_hash),
                    ulid: Set(Ulid::new().to_string()),
                    file_type: Set(file_type),
                    file_size: Set(file_size),
                    file_ctime: Set(file_ctime),
                    file_mtime: Set(file_mtime),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;

                logger::trace!(file = ?db_file, "File inserted into database");

                db_file
            }
        };

        txn.commit().await?;

        Ok(db_file)
    }
}

async fn infer_file_type(file: PathBuf) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        infer_from_path(&file)?
            .map(|x| x.mime_type().to_string())
            .or_else(|| {
                FileFormat::from_file(&file)
                    .map(|x| x.media_type().to_string())
                    .ok()
            })
            .or_else(|| magic_infer_from_filepath(&file).map(ToString::to_string))
            .ok_or_else(|| anyhow!("Could not infer file type for file: {:?}", &file))
    })
    .await?
}
