use std::path::PathBuf;

use anyhow::{anyhow, Result};
use blurhash::encode as blurhash_encode;
use config::CONFIG;
use entity::{file_data, files};
use image::{EncodableLayout, GenericImageView};
use sea_orm::{prelude::*, Set};
use tokio::task;
use tracing::instrument;

use crate::FileWatcher;

pub const BLURHASH_COMPONENTS: (u32, u32) = (3, 3);
pub const FILE_DATA_BLURHASH_KEY: &str = "blurhash";

impl FileWatcher {
    #[instrument(skip(self))]
    pub async fn get_or_generate_blurhash(&self, ulid: &str) -> Result<String> {
        let db_file = files::Entity::find()
            .filter(files::Column::Ulid.eq(ulid.to_uppercase()))
            .one(self.db())
            .await?
            .ok_or_else(|| anyhow!("File not found: {:?}", ulid))?;

        let file_data_blurhash = file_data::Entity::find()
            .filter(file_data::Column::FileId.eq(db_file.id))
            .filter(file_data::Column::Key.eq(FILE_DATA_BLURHASH_KEY))
            .one(self.db())
            .await?;

        if let Some(file_data_blurhash) = file_data_blurhash {
            logger::trace!(data = ?file_data_blurhash, "File has blurhash data");

            return Ok(file_data_blurhash.value);
        }

        let file_path = CONFIG.app.directory.join(&db_file.path);

        logger::trace!(path = ?file_path, "Generating blurhash");

        let hash = self.generate_blurhash(file_path, db_file.id).await?;

        Ok(hash)
    }

    #[instrument(skip(self))]
    pub(crate) async fn generate_blurhash(
        &self,
        image_path: PathBuf,
        file_id: i32,
    ) -> Result<String> {
        logger::trace!(path = ?image_path, "Generating blurhash");

        let hash = task::spawn_blocking(move || {
            let img = image::open(&image_path)
                .map_err(|e| anyhow!("Failed to open image {:?}: {}", &image_path, e))?;
            let (width, height) = img.dimensions();

            blurhash_encode(
                BLURHASH_COMPONENTS.0,
                BLURHASH_COMPONENTS.1,
                width,
                height,
                img.to_rgba8().as_bytes(),
            )
            .map_err(|e| anyhow!("Failed to generate blurhash: {}", e.to_string()))
        })
        .await??;

        logger::trace!(hash = ?&hash, "Generated blurhash");

        let file_data_model = file_data::ActiveModel {
            file_id: Set(file_id),
            key: Set(FILE_DATA_BLURHASH_KEY.to_string()),
            value: Set(hash.clone()),
            ..Default::default()
        };

        file_data_model
            .save(self.db())
            .await
            .map_err(|e| anyhow!("Failed to save blurhash to db: {}", e.to_string()))?;

        logger::trace!("Saved blurhash to db");

        Ok(hash)
    }
}
