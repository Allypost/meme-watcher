use std::{convert::Into, path::Path};

use anyhow::{anyhow, bail, Result};
use config::CONFIG;
use entity::{file_data, files};
use sea_orm::{prelude::*, Set};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::FileWatcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDimensions {
    pub width: u32,
    pub height: u32,
}

impl From<(u32, u32)> for MediaDimensions {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<(i64, i64)> for MediaDimensions {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn from((width, height): (i64, i64)) -> Self {
        Self {
            width: width as u32,
            height: height as u32,
        }
    }
}

impl TryFrom<file_data::Model> for MediaDimensions {
    type Error = anyhow::Error;

    fn try_from(data: file_data::Model) -> Result<Self, Self::Error> {
        if data.key != FILE_DATA_MEDIA_DIMENSIONS_KEY {
            bail!("Invalid file data key: {}", data.key);
        }

        serde_json::from_str(&data.meta).map_err(|e| {
            anyhow!(
                "Failed to deserialize media dimensions from file data: {}",
                e
            )
        })
    }
}

pub const FILE_DATA_MEDIA_DIMENSIONS_KEY: &str = "media-dimensions";

impl FileWatcher {
    #[instrument(skip(self))]
    pub async fn get_or_generate_media_dimensions(
        &self,
        ulid: &str,
    ) -> Result<Option<MediaDimensions>> {
        let db_file = files::Entity::find()
            .filter(files::Column::Ulid.eq(ulid.to_uppercase()))
            .one(self.db())
            .await?
            .ok_or_else(|| anyhow!("Could not find file with ulid: {}", ulid))?;

        let db_file_data = file_data::Entity::find()
            .filter(file_data::Column::Key.eq(FILE_DATA_MEDIA_DIMENSIONS_KEY))
            .filter(file_data::Column::FileId.eq(db_file.id))
            .one(self.db())
            .await?;

        if let Some(db_file_data) = db_file_data {
            let dims = db_file_data.try_into()?;
            return Ok(Some(dims));
        }

        let file_type = db_file.file_type.unwrap_or_default();
        let file_path = CONFIG.app.directory_absolute(&db_file.path);

        self.generate_media_dimensions(db_file.id, &file_type, &file_path)
            .await
    }

    #[instrument(skip(self))]
    pub(crate) async fn generate_media_dimensions(
        &self,
        file_id: i32,
        file_type: &str,
        file_path: &Path,
    ) -> Result<Option<MediaDimensions>> {
        let ffprobe_info = ffmpeg::ffprobe::ConfigBuilder::new()
            .with_streams(true)
            .run(file_path)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to run ffprobe to get media dimensions: {}",
                    e.to_string()
                )
            })?;

        let streams = match ffprobe_info.streams {
            Some(x) => x,
            None => {
                return Ok(None);
            }
        };

        let dims = streams.iter().find_map(|x| match (x.width, x.height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        });

        logger::trace!(?dims, "Got media dimensions from ffprobe");

        let dims: MediaDimensions = match dims {
            Some(dims) => dims.into(),
            None => {
                return Ok(None);
            }
        };

        let file_data_model = file_data::ActiveModel {
            file_id: Set(file_id),
            key: Set(FILE_DATA_MEDIA_DIMENSIONS_KEY.to_string()),
            value: Set(String::new()),
            meta: Set(serde_json::to_string(&dims)?),
            ..Default::default()
        };

        let file_data_model = file_data_model.insert(self.db()).await?;

        logger::trace!(data = ?file_data_model, "Inserted media dimensions into file data");

        Ok(Some(dims))
    }
}
