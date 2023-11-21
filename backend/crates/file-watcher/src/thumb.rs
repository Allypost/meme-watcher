use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};
use chrono::{prelude::*, DateTime};
use config::CONFIG;
use entity::{file_data, file_metadata, files};
use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};
use sea_orm::{prelude::*, Condition, Set};
use serde::{Deserialize, Serialize};
use tempfile::Builder as TempfileBuilder;
use tokio::{process::Command, task};
use tracing::instrument;
use which::which;

use crate::{
    file_metadata::FileMetadata,
    helpers::{date::parse_db_date, file::file_hash},
    FileWatcher,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileThumb {
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub meta: FileThumbMeta,
}

#[derive(Debug, Clone)]
struct ThumbGenerateResult {
    pub width: u32,
    pub height: u32,
    pub hash: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileThumbMeta {
    pub width: u32,
    pub height: u32,
    pub hash: String,
}

impl From<ThumbGenerateResult> for FileThumbMeta {
    fn from(result: ThumbGenerateResult) -> Self {
        Self {
            width: result.width,
            height: result.height,
            hash: result.hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThumbDimensions {
    pub width: u32,
    pub height: u32,
}

impl ThumbDimensions {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Default for ThumbDimensions {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
        }
    }
}

impl Display for ThumbDimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ThumbSize {
    #[default]
    Thumb,
    Poster,
    Specific(ThumbDimensions),
}

impl Display for ThumbSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thumb => write!(f, "thumbnail"),
            Self::Poster => write!(f, "poster"),
            Self::Specific(dimensions) => write!(f, "thumbnail-{dimensions}"),
        }
    }
}

impl From<ThumbSize> for ThumbDimensions {
    fn from(size: ThumbSize) -> Self {
        match size {
            ThumbSize::Thumb => ThumbDimensions::default(),
            ThumbSize::Poster => ThumbDimensions::new(300, 300),
            ThumbSize::Specific(dimensions) => dimensions,
        }
    }
}

impl FileWatcher {
    pub async fn get_or_generate_thumb(&self, ulid: &str, size: ThumbSize) -> Result<FileThumb> {
        let (db_file, db_file_meta) = files::Entity::find()
            .filter(files::Column::Ulid.eq(ulid.to_uppercase()))
            .find_also_related(file_metadata::Entity)
            .one(self.db())
            .await?
            .ok_or_else(|| anyhow!("File not found"))?;

        logger::trace!(file = ?db_file, "Found file in db");

        let thumb_key = size.to_string();

        let db_file_thumb = file_data::Entity::find()
            .filter(
                Condition::all()
                    .add(file_data::Column::FileId.eq(db_file.id))
                    .add(file_data::Column::Key.eq(&thumb_key)),
            )
            .one(self.db())
            .await?;

        if let Some(db_file_thumb) = db_file_thumb {
            logger::trace!(model = ?db_file_thumb, "Found thumb in db");

            let meta: FileThumbMeta = serde_json::from_str(&db_file_thumb.meta)?;

            let path = CONFIG.app.metadata_directory_absolute(&db_file_thumb.value);

            if path.exists() {
                return Ok(FileThumb {
                    path,
                    meta,
                    created_at: parse_db_date(&db_file.created_at)?,
                });
            }

            logger::warn!("Couldn't find thumb path from db. Deleting entry");

            db_file_thumb.delete(self.db()).await?;
        }

        logger::debug!(file = ?db_file, "Thumb not found in db, generating...");

        let db_file_meta = match db_file_meta {
            Some(db_file_meta) => db_file_meta,
            None => {
                bail!("File metadata not found");
            }
        };

        let file_path = CONFIG.app.directory_absolute(&db_file.path);
        let file_meta: FileMetadata = db_file_meta.try_into()?;
        let file_type = file_meta.file_type.unwrap_or_default();

        self.generate_thumbnail(db_file.id, &file_path, &file_type, size)
            .await
    }

    #[instrument(skip(self))]
    pub(crate) async fn generate_thumbnail(
        &self,
        file_id: i32,
        file_path: &Path,
        file_type: &str,
        size: impl Into<ThumbSize> + Debug,
    ) -> Result<FileThumb> {
        logger::trace!("Generating thumb");

        let size = size.into();
        let thumb_key = size.to_string();
        let dimensions = size.into();

        let thumb_meta = match file_type {
            t if t.starts_with("image/") => {
                self.generate_image_thumbnail(file_path, &format!("{file_id}"), dimensions)
                    .await?
            }
            t if t.starts_with("video/") => {
                self.generate_video_thumbnail(file_path, &format!("{file_id}"), dimensions)
                    .await?
            }
            _ => {
                bail!("Unsupported file type");
            }
        };

        logger::trace!(thumb = ?thumb_meta, "Generated thumb");

        let thumb_path = thumb_meta.path.clone();
        let meta: FileThumbMeta = thumb_meta.into();

        let thumb_model = file_data::ActiveModel {
            file_id: Set(file_id),
            key: Set(thumb_key),
            value: Set(thumb_path.to_string_lossy().to_string()),
            meta: Set(serde_json::to_string(&meta)?),
            ..Default::default()
        };

        let thumb_model = thumb_model.save(self.db()).await?;

        logger::trace!(model = ?thumb_model, "Saved thumb to db");

        Ok(FileThumb {
            path: thumb_path,
            meta,
            created_at: parse_db_date(thumb_model.created_at.clone().as_ref())?,
        })
    }

    #[instrument(skip(self))]
    pub(crate) async fn generate_image_thumbnail(
        &self,
        image_path: &Path,
        image_ulid: &str,
        dimensions: ThumbDimensions,
    ) -> Result<ThumbGenerateResult> {
        let thumb_path = CONFIG.app.thumbs_directory().join(format!(
            "{id}.{w}x{h}.jpeg",
            id = image_ulid,
            w = dimensions.width,
            h = dimensions.height,
        ));

        let image_path = PathBuf::from(&image_path);

        logger::debug!(thumb = ?thumb_path, file = ?image_path, "Generating a new thumbnail");

        let img = {
            let image_path = image_path.clone();
            task::spawn_blocking(move || -> Result<DynamicImage> {
                ImageReader::open(image_path)
                    .map_err(|e| anyhow!("Failed to open file: {}", e.to_string()))?
                    .with_guessed_format()
                    .map_err(|e| anyhow!("Failed to guess file format: {}", e.to_string()))?
                    .decode()
                    .map_err(|e| anyhow!("Failed to decode image: {}", e.to_string()))
            })
            .await??
        };

        logger::trace!(path = ?image_path, "Parsed image from path");

        let thumb =
            task::spawn_blocking(move || img.thumbnail(dimensions.width, dimensions.height))
                .await
                .map_err(|e| anyhow!("Failed to generate thumbnail: {}", e.to_string()))?;

        logger::trace!("Generated thumbnail");

        let (thumb_width, thumb_height) = thumb.dimensions();

        let thumb_path = task::spawn_blocking(move || thumb.save(&thumb_path).map(|()| thumb_path))
            .await?
            .map_err(|e| anyhow!("Failed to save thumbnail: {}", e.to_string()))?;

        logger::trace!(path = ?thumb_path, "Saved thumbnail");

        let thumb_hash = file_hash(&thumb_path).await?;

        logger::trace!(hash = ?thumb_hash, "Calculated thumbnail hash");

        Ok(ThumbGenerateResult {
            width: thumb_width,
            height: thumb_height,
            hash: thumb_hash,
            path: CONFIG.app.metadata_directory_relative(&thumb_path)?.into(),
        })
    }

    #[instrument(skip(self))]
    pub(crate) async fn generate_video_thumbnail(
        &self,
        video_path: &Path,
        video_ulid: &str,
        dimensions: ThumbDimensions,
    ) -> Result<ThumbGenerateResult> {
        let tmp_file = TempfileBuilder::new()
            .suffix(".jpg")
            .tempfile()
            .map_err(|e| anyhow::anyhow!("Failed to create temporary file: {}", e.to_string()))?;

        let tmp_thumb_path = self
            .extract_first_video_frame(video_path, tmp_file.path())
            .await?;

        self.generate_image_thumbnail(&tmp_thumb_path, video_ulid, dimensions)
            .await
    }

    #[instrument(skip(self))]
    pub(crate) async fn extract_first_video_frame(
        &self,
        video_path: &Path,
        extract_path: &Path,
    ) -> Result<PathBuf> {
        let ffmpeg_path = which("ffmpeg")?;

        let mut cmd = Command::new(ffmpeg_path);
        let cmd = cmd
            .arg("-hide_banner")
            .arg("-y")
            .args(["-i", video_path.to_string_lossy().to_string().as_str()])
            .args(["-vframes", "1"])
            .arg(extract_path);

        logger::trace!(cmd = ?cmd, "Running ffmpeg command");

        let output = cmd
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run ffmpeg command: {}", e.to_string()))?;

        logger::trace!(output = ?output, "ffmpeg output");

        if !output.status.success() {
            bail!("Failed to generate video thumbnail");
        }

        Ok(extract_path.to_path_buf())
    }
}
