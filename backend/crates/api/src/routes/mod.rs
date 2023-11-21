use std::{collections::HashMap, sync::Arc};

use rocket::{http::Status, Route, State};
use sea_orm::{prelude::*, DatabaseConnection};
use serde_json::json;

use crate::AppRoutes;

mod file;
mod page_data;

type RouteBase = String;
type RouteList = Vec<(RouteBase, Vec<Route>)>;

#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/routes")]
pub fn routes(routes: &State<AppRoutes>) -> serde_json::Value {
    serde_json::json!(routes
        .0
        .iter()
        .map(|r| format!("[{}] {}", r.method, r.uri))
        .collect::<Vec<_>>())
}

#[get("/indexed")]
pub async fn indexed(db: &State<Arc<DatabaseConnection>>) -> Result<serde_json::Value, Status> {
    let indexed = entity::files::Entity::find()
        .find_with_related(entity::file_data::Entity)
        .all(db.as_ref())
        .await
        .map_err(|e| {
            logger::error!("failed to get indexed files: {}", e);
            Status::InternalServerError
        })?;

    let metas = entity::file_metadata::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|e| {
            logger::error!("failed to get indexed files: {}", e);
            Status::InternalServerError
        })?
        .into_iter()
        .fold(HashMap::new(), |mut acc, x| {
            acc.insert(x.file_id, x);
            acc
        });

    Ok(serde_json::json!(indexed
        .into_iter()
        .map(|(file, data)| {
            let meta = metas.get(&file.id);
            let data = data
                .into_iter()
                .map(|x| {
                    json!({
                        "id": x.id,
                        "key": x.key,
                        "value": x.value,
                        "meta": serde_json::from_str::<serde_json::Value>(&x.meta).ok(),
                        "createdAt": x.created_at,
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "file": file,
                "data": data,
                "meta": meta,
            })
        })
        .collect::<Vec<_>>()))
}

pub(super) fn get() -> RouteList {
    let mut joined = vec![];

    joined.push(("/".into(), routes![index, indexed, routes]));
    joined.append(&mut resolve_get("/page-data", page_data::get()));
    joined.append(&mut resolve_get("/file", file::get()));

    joined
}

pub(crate) fn resolve_get<TBase>(with_base: TBase, old_list: RouteList) -> RouteList
where
    TBase: Into<RouteBase>,
{
    let with_base = with_base.into();
    let with_base = with_base.trim_end_matches('/');

    old_list
        .into_iter()
        .map(|(old_base, routes)| {
            let old_base = old_base.trim_start_matches('/');
            let new_base = format!("{}/{}", with_base, old_base);

            (new_base, routes)
        })
        .collect()
}
