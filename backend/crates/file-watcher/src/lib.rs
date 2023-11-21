use std::sync::Arc;

use sea_orm::prelude::*;

pub mod blurhash;
pub mod file;
pub mod file_metadata;
mod helpers;
pub mod index;
pub mod media_dimensions;
pub mod scan;
pub mod thumb;

pub struct FileWatcher {
    db: Arc<DatabaseConnection>,
    recursive: bool,
}

impl FileWatcher {
    pub fn new<T>(db: T) -> Self
    where
        T: Into<Arc<DatabaseConnection>>,
    {
        Self {
            db: db.into(),
            recursive: true,
        }
    }

    pub fn set_recursive(&mut self, recursive: bool) -> &Self {
        self.recursive = recursive;
        self
    }

    #[must_use]
    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.set_recursive(recursive);
        self
    }

    fn db(&self) -> &DatabaseConnection {
        &self.db as &DatabaseConnection
    }
}
