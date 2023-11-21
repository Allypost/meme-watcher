use config::CONFIG;
use entity::files;
use file_watcher::{thumb::ThumbDimensions, FileWatcher};
use rocket::{
    http::{hyper::header, Header, Status},
    request::FromParam,
    State,
};
use sea_orm::prelude::*;

use crate::{helpers::range_responder::RangeResponder, routes::RouteList};

#[get("/<ulid>")]
pub async fn serve_file(
    db: &State<std::sync::Arc<DatabaseConnection>>,
    ulid: &str,
) -> Result<RangeResponder<tokio::fs::File>, Status> {
    let db_file = files::Entity::find()
        .filter(files::Column::Ulid.eq(ulid.to_uppercase()))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            logger::error!(err = ?e, "Failed to get file");

            Status::InternalServerError
        })?;

    let db_file = match db_file {
        Some(x) => x,
        None => return Err(Status::NotFound),
    };

    let base_path = &CONFIG.app.directory;
    let file_path = base_path.join(&db_file.path);

    let mut responder = RangeResponder::from_path(&file_path).await.map_err(|e| {
        logger::error!(err = ?e, "Failed to open file");

        Status::NotFound
    })?;

    responder
        .add_header(Header::new(
            header::ETAG.as_str(),
            format!("\"{}\"", db_file.hash),
        ))
        .add_header(Header::new(
            header::CACHE_CONTROL.as_str(),
            "public, max-age=31536000, immutable",
        ))
        .add_header(Header::new(header::PRAGMA.as_str(), "public"));

    Ok(responder)
}

#[derive(Debug, PartialEq)]
pub enum ThumbSize {
    Thumb,
    Poster,
    Specific(ThumbDimensions),
}

impl<'r> FromParam<'r> for ThumbSize {
    type Error = Status;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match param {
            "thumb" | "thumbnail" => Ok(ThumbSize::Thumb),
            "poster" => Ok(ThumbSize::Poster),
            param => {
                let items = param.split_once('x').and_then(|x| {
                    let (w, h) = x;

                    match (w.parse::<u32>(), h.parse::<u32>()) {
                        (Ok(w), Ok(h)) => Some(ThumbSize::Specific(ThumbDimensions {
                            width: w,
                            height: h,
                        })),
                        _ => None,
                    }
                });

                match items {
                    Some(x) => Ok(x),
                    None => Err(Status::BadRequest),
                }
            }
        }
    }
}

impl From<ThumbSize> for file_watcher::thumb::ThumbSize {
    fn from(size: ThumbSize) -> Self {
        match size {
            ThumbSize::Thumb => file_watcher::thumb::ThumbSize::Thumb,
            ThumbSize::Poster => file_watcher::thumb::ThumbSize::Poster,
            ThumbSize::Specific(dimensions) => file_watcher::thumb::ThumbSize::Specific(dimensions),
        }
    }
}

#[get("/<ulid>/<thumb_size>")]
pub async fn get_thumbnail<'o>(
    fw: &State<std::sync::Arc<FileWatcher>>,
    ulid: &str,
    thumb_size: ThumbSize,
) -> Result<RangeResponder<tokio::fs::File>, Status> {
    let res_file = fw
        .get_or_generate_thumb(ulid, thumb_size.into())
        .await
        .map_err(|e| {
            logger::warn!(err = ?e, "Failed to get thumb");

            Status::NotFound
        })?;

    let file_path = CONFIG.app.metadata_directory.join(&res_file.path);

    let mut responder = RangeResponder::from_path(&file_path).await.map_err(|e| {
        logger::error!(err = ?e, "Failed to open file");

        Status::NotFound
    })?;

    responder
        .add_header(Header::new(
            header::ETAG.as_str(),
            format!("\"{}\"", res_file.meta.hash),
        ))
        .add_header(Header::new(
            header::CACHE_CONTROL.as_str(),
            "public, max-age=31536000, immutable",
        ))
        .add_header(Header::new(header::PRAGMA.as_str(), "public"));

    Ok(responder)
}

pub(super) fn get() -> RouteList {
    let mut joined = vec![];

    joined.push(("/".into(), routes![serve_file, get_thumbnail]));

    joined
}
