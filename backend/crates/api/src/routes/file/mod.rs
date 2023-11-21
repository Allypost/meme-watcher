use super::{resolve_get, RouteList};

mod serve;

pub(super) fn get() -> RouteList {
    let mut joined = vec![];

    joined.append(&mut resolve_get("/serve", serve::get()));

    joined
}
