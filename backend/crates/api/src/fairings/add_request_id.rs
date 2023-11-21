use rocket::{
    async_trait,
    fairing::{Fairing, Info, Kind},
    http::Header,
    Data, Request,
};
use ulid::Ulid;

pub struct AddRequestId;
#[async_trait]
impl Fairing for AddRequestId {
    fn info(&self) -> Info {
        Info {
            name: "Request ID",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        request.add_header(Header::new("x-request-id", Ulid::new().to_string()));
    }
}
