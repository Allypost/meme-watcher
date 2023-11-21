use std::{path::Path, time::Instant};

use anyhow::{anyhow, bail, Result};
use entity::files;
use sea_orm::{prelude::*, Condition, Set, TransactionTrait};
use tokio::fs;
use tracing::instrument;
use ulid::Ulid;

use crate::{
    helpers::file::{file_hash, relative_to_app_dir},
    FileWatcher,
};

impl FileWatcher {
    #[instrument(skip(self))]
    pub(crate) async fn get_or_create_file(&self, file_path: &Path) -> Result<files::Model> {
        let meta = fs::metadata(file_path)
            .await
            .map_err(|e| anyhow!("Failed to get metadata of file {:?}: {}", file_path, e))?;

        if !meta.is_file() {
            bail!("Not a file");
        }

        let file_path_rel = relative_to_app_dir(file_path)?;

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
                let db_file = files::ActiveModel {
                    path: Set(file_path_rel),
                    hash: Set(file_hash),
                    ulid: Set(Ulid::new().to_string()),
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
