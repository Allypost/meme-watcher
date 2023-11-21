use super::{resolve_get, RouteList};

mod index;

pub(super) fn get() -> RouteList {
    let mut joined = vec![];

    joined.append(&mut resolve_get("/", index::get()));

    joined
}
