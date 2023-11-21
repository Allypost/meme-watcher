use std::{
    fs::Metadata,
    io::{Cursor, SeekFrom},
    ops::{RangeInclusive, Sub},
    path::{Path, PathBuf},
};

use chrono::{prelude::*, Duration};
use futures::executor;
use rocket::{
    self,
    http::{hyper::header, ContentType, Header, HeaderMap, Status},
    response::{self, Responder},
    Response,
};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    runtime::Handle,
};

#[derive(Debug)]
pub struct RangeResponder<R> {
    original: R,
    path: Option<PathBuf>,
    metadata: Option<Metadata>,
    additional_headers: HeaderMap<'static>,
}

impl From<std::fs::File> for RangeResponder<tokio::fs::File> {
    fn from(file: std::fs::File) -> Self {
        Self::new(tokio::fs::File::from_std(file))
    }
}

impl From<tokio::fs::File> for RangeResponder<tokio::fs::File> {
    fn from(file: tokio::fs::File) -> Self {
        Self::new(file).try_set_metadata()
    }
}

impl From<rocket::fs::NamedFile> for RangeResponder<tokio::fs::File> {
    fn from(file: rocket::fs::NamedFile) -> Self {
        let path = file.path().to_path_buf();
        Self::new(file.take_file()).with_path(&path)
    }
}

impl RangeResponder<tokio::fs::File> {
    pub async fn from_path(path: &Path) -> tokio::io::Result<Self> {
        let file = tokio::fs::File::open(path).await?;
        let new: Self = file.into();
        Ok(new.with_path(path))
    }

    fn try_set_metadata(mut self) -> Self {
        let metadata = self.metadata();

        if let Ok(metadata) = metadata {
            self.metadata = Some(metadata);
        }

        self
    }

    fn metadata(&self) -> Result<Metadata, Box<dyn std::error::Error>> {
        if self.metadata.is_some() {
            return Ok(self.metadata.clone().unwrap());
        }

        let handle = Handle::current();
        let _ = handle.enter();
        executor::block_on(self.original.metadata()).map_err(std::convert::Into::into)
    }
}

impl<'r, R> RangeResponder<R> {
    pub fn new(original: R) -> Self {
        Self {
            original,
            path: None,
            metadata: None,
            additional_headers: HeaderMap::new(),
        }
    }

    pub fn with_path(mut self, path: &Path) -> Self {
        self.path = Some(path.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn add_header(&mut self, header: Header<'static>) -> &mut Self {
        self.additional_headers.add(header);
        self
    }

    fn parse_range_header(
        &self,
        range_header: &str,
        file_length: u64,
    ) -> Result<RangeInclusive<u64>, Box<rocket::Response<'r>>> {
        let parsed_ranges = match http_range_header::parse_range_header(range_header) {
            Ok(x) => x,
            Err(e) => {
                return Err(Box::new(
                    self.reject_range(file_length, &Some((range_header, Some(e.into())))),
                ));
            }
        };

        let vec_range = match parsed_ranges.validate(file_length) {
            Ok(x) => x,
            Err(e) => {
                return Err(Box::new(
                    self.reject_range(file_length, &Some((range_header, Some(e.into())))),
                ));
            }
        };

        let first_range = match vec_range.first() {
            Some(x) => x,
            None => {
                return Err(Box::new(
                    Response::build()
                        .status(Status::InternalServerError)
                        .streamed_body(Cursor::new(
                            "Something went wrong while parsing the range header",
                        ))
                        .finalize(),
                ));
            }
        };

        Ok(first_range.clone())
    }

    fn reject_range(
        &self,
        file_length: u64,
        meta: &Option<(&str, Option<Box<dyn std::error::Error>>)>,
    ) -> rocket::Response<'r> {
        let headers = self.additional_headers(file_length);

        Self::reject_range_ext(file_length, meta, headers)
    }

    fn reject_range_ext(
        file_length: u64,
        meta: &Option<(&str, Option<Box<dyn std::error::Error>>)>,
        headers: HeaderMap<'r>,
    ) -> rocket::Response<'r> {
        logger::warn!(
            file_length = ?&file_length,
            meta = ?&meta,
            "Rejecting unsatisfiable range header",
        );

        let mut response = Response::build()
            .status(Status::RangeNotSatisfiable)
            .finalize();

        for x in headers.into_iter() {
            response.set_header(x);
        }

        response.set_header(Header::new(
            header::CONTENT_RANGE.as_str(),
            format!("bytes */{}", file_length),
        ));

        response
    }

    fn additional_headers<'h>(&self, _file_length: u64) -> HeaderMap<'h> {
        let mut res = self.additional_headers.clone();

        if let Some(path) = self.path.as_ref() {
            let content_type = path
                .extension()
                .and_then(|x| ContentType::from_extension(&x.to_string_lossy()));
            if let Some(content_type) = content_type {
                res.add(content_type);
            }

            res.add(Header::new(
                "X-File-Path",
                path.to_string_lossy().to_string(),
            ));
        }

        if let Some(metadata) = self.metadata.as_ref() {
            if let Ok(mtime) = metadata.modified() {
                let time: DateTime<Utc> = mtime.into();

                res.add(Header::new(
                    header::LAST_MODIFIED.as_str(),
                    time.to_rfc2822(),
                ));
            }

            if let Ok(ctime) = metadata.created() {
                let time: DateTime<Utc> = ctime.into();

                res.add(Header::new("X-Created-At", time.to_rfc2822()));
            }
        }

        res
    }

    fn respond_to_cache_headers(&self, req_headers: &HeaderMap) -> Result<(), Status> {
        if let Some(metadata) = self.metadata.as_ref() {
            let req_time = req_headers
                .get_one(header::IF_MODIFIED_SINCE.as_str())
                .and_then(|x| DateTime::parse_from_rfc2822(x).ok())
                .map(|x| x.with_timezone(&Utc));

            if let Some(req_time) = req_time {
                if let Ok(mtime) = metadata.modified() {
                    let file_time: DateTime<Utc> = mtime.into();

                    if file_time.sub(req_time) > Duration::milliseconds(10) {
                        return Err(Status::NotModified);
                    }
                }
            }
        }

        let etag = self.additional_headers.get_one(header::ETAG.as_str());

        if let Some(etag) = etag {
            let if_none_match = req_headers.get_one(header::IF_NONE_MATCH.as_str());

            if let Some(if_none_match) = if_none_match {
                if if_none_match == "*" {
                    return Err(Status::NotModified);
                }

                let if_none_match = if_none_match.split(',').map(str::trim);

                for match_etag in if_none_match {
                    if etag == match_etag {
                        return Err(Status::NotModified);
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'r> Responder<'r, 'static> for RangeResponder<tokio::fs::File> {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> response::Result<'static> {
        self.respond_to_cache_headers(request.headers())?;

        let handle = Handle::current();
        let _ = handle.enter();

        let metadata = match executor::block_on(self.original.metadata()) {
            Ok(x) => x,
            Err(e) => {
                logger::warn!(
                    err = ?e,
                    "Failed to get file metadata",
                );

                return Ok(Response::build()
                    .status(Status::InternalServerError)
                    .streamed_body(Cursor::new(
                        "Something went wrong while processing the request",
                    ))
                    .finalize());
            }
        };

        let file_length = metadata.len();
        let mut additional_headers = self.additional_headers(file_length);

        let range_header = request.headers().get_one("Range");
        let range_header = match range_header {
            None => {
                let mut response = self.original.respond_to(request)?;
                for header in additional_headers.into_iter() {
                    response.set_header(header);
                }
                return Ok(response);
            }
            Some(h) => h,
        };

        let first_range = self.parse_range_header(range_header, file_length);
        let first_range = match first_range {
            Ok(x) => x,
            Err(e) => {
                return Ok(*e);
            }
        };

        let content_range = Header::new(
            header::CONTENT_RANGE.as_str(),
            format!(
                "bytes {}-{}/{}",
                first_range.start(),
                first_range.end(),
                file_length
            ),
        );

        let content_len = first_range.end() - first_range.start() + 1;

        let mut partial_original = self.original;

        let seek_result = executor::block_on(
            partial_original.seek(SeekFrom::Start(first_range.start().to_owned())),
        );
        match seek_result {
            Ok(_) => (),
            Err(e) => {
                return Ok(Self::reject_range_ext(
                    file_length,
                    &Some((range_header, Some(e.into()))),
                    additional_headers,
                ));
            }
        }
        let partial_original = partial_original.take(content_len);

        let mut response = Response::build()
            .status(Status::PartialContent)
            .header(Header::new(
                header::CONTENT_LENGTH.as_str(),
                content_len.to_string(),
            ))
            .header(content_range)
            .streamed_body(partial_original)
            .finalize();

        additional_headers.remove(header::CONTENT_LENGTH.as_str());

        for x in additional_headers.into_iter() {
            response.set_header(x);
        }

        Ok(response)
    }
}
