use std::{collections::HashMap, path::Path};

use entity::{file_data, files, files_tags};
use rocket::{http::Status, State};
use sea_orm::{prelude::*, IntoSimpleExpr, QueryOrder, QuerySelect};
use serde::Serialize;
use serde_json::json;
use typeshare::typeshare;

use crate::{
    helpers::{order, pagination::Pagination},
    routes::RouteList,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
struct PageDataIndexItemDataItem {
    key: String,
    value: String,
    meta: serde_json::Value,
}

impl From<&file_data::Model> for PageDataIndexItemDataItem {
    fn from(x: &file_data::Model) -> Self {
        Self {
            key: x.key.clone(),
            value: x.value.clone(),
            meta: x.meta.parse().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
struct PageDataIndexItem {
    id: String,
    name: String,
    file_size: Option<String>,
    file_type: Option<String>,
    created: Option<String>,
    modified: Option<String>,
    tags: Vec<i32>,
    data: Vec<PageDataIndexItemDataItem>,
}

impl From<files::Model> for PageDataIndexItem {
    fn from(x: files::Model) -> Self {
        Self {
            id: x.ulid.to_lowercase(),
            name: Path::new(&x.path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            file_size: x.file_size.map(|x| x.to_string()),
            file_type: x.file_type,
            created: x.file_ctime,
            modified: x.file_mtime,
            ..Default::default()
        }
    }
}

impl PageDataIndexItem {
    fn with_tags(mut self, tags: Vec<i32>) -> Self {
        self.tags = tags;
        self
    }

    fn with_data(mut self, data: Vec<PageDataIndexItemDataItem>) -> Self {
        self.data = data;
        self
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
struct PageDataIndex {
    items: Vec<PageDataIndexItem>,
    pagination: Pagination,
}

#[derive(Debug, Serialize, Default, FromFormField, Clone)]
#[typeshare]
pub enum PageDataIndexOrderBy {
    #[default]
    Modified,
    Created,
    Size,
    Id,
}

impl PageDataIndexOrderBy {
    fn into_simple_expr(self) -> impl IntoSimpleExpr {
        match self {
            PageDataIndexOrderBy::Modified => files::Column::FileMtime.into_simple_expr(),
            PageDataIndexOrderBy::Created => files::Column::FileCtime.into_simple_expr(),
            PageDataIndexOrderBy::Size => files::Column::FileSize.into_simple_expr(),
            PageDataIndexOrderBy::Id => files::Column::Id.into_simple_expr(),
        }
    }
}

#[get("/?<pagination>&<order>")]
pub async fn index(
    db: &State<std::sync::Arc<DatabaseConnection>>,
    pagination: Option<Pagination>,
    order: Option<order::Order<PageDataIndexOrderBy>>,
) -> Result<serde_json::Value, Status> {
    let mut pagination = pagination.unwrap_or_default().with_defaults();
    let order = order.unwrap_or_default();
    let per_page = pagination.per_page();

    let items = files::Entity::find()
        .order_by(order.by().into_simple_expr(), order.direction().into())
        .limit(per_page)
        .offset(pagination.offset())
        .all(db.as_ref())
        .await
        .map_err(|e| {
            logger::error!(err = ?e, "failed to get files");
            Status::InternalServerError
        })?;

    let total_items = files::Entity::find()
        .count(db.as_ref())
        .await
        .map_err(|_| Status::InternalServerError)?;

    pagination.set_total_pages(total_items);

    let file_ids = items.iter().map(|x| x.id).collect::<Vec<_>>();

    let files_tags = files_tags::Entity::find()
        .filter(files_tags::Column::FileId.is_in(file_ids.clone()))
        .all(db.as_ref())
        .await
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .fold(HashMap::new(), |mut acc: HashMap<i32, Vec<_>>, x| {
            acc.entry(x.file_id).or_default().push(x.tag_id);
            acc
        });

    let file_data = file_data::Entity::find()
        .filter(file_data::Column::FileId.is_in(file_ids))
        .all(db.as_ref())
        .await
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .fold(HashMap::new(), |mut acc: HashMap<i32, Vec<_>>, x| {
            acc.entry(x.file_id).or_default().push(x);
            acc
        });

    let items = items
        .into_iter()
        .map(|x| {
            let id = x.id;

            let mut item = PageDataIndexItem::from(x);

            if let Some(tags) = files_tags.get(&id) {
                item = item.with_tags(tags.clone());
            }

            if let Some(data) = file_data.get(&id) {
                item = item.with_data(data.iter().map(Into::into).collect());
            }

            item
        })
        .collect::<Vec<_>>();

    Ok(json!(PageDataIndex { items, pagination }))
}

pub fn get() -> RouteList {
    vec![("/index".into(), routes![index])]
}
