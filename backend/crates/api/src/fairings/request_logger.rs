use std::collections::HashMap;

use rocket::{
    async_trait,
    fairing::{Fairing, Info, Kind},
    Data, Request, Response,
};
use tracing::info_span;

pub const SPAN_TARGET: &str = "request";

pub struct RequestLogger {
    span: tracing::Span,
}

impl RequestLogger {
    pub fn new(span: tracing::Span) -> Self {
        Self { span }
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        Self::new(info_span!("request"))
    }
}

#[async_trait]
impl Fairing for RequestLogger {
    fn info(&self) -> Info {
        Info {
            name: "Logger",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let _ = self.span.enter();

        let headers = request.headers();
        let id = headers.get_one("x-request-id");
        let _timer = request.local_cache(RequestTimer::default);

        logger::trace!(
            target: SPAN_TARGET,
            id = id,
            "type" = "start",
        );
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let _ = self.span.enter();

        let status = response.status();
        let headers = request.headers();
        let id = headers.get_one("x-request-id");
        let method = request.method().as_str();
        let uri = request.uri();
        let duration = format!("{:?}", request.local_cache(RequestTimer::default).elapsed());

        let headers = headers
            .iter()
            .map(|header| (header.name().to_string(), header.value().to_string()))
            .filter(|(name, _)| name != "x-request-id")
            .filter(|(name, _)| !name.starts_with("sec-"))
            .fold(HashMap::new(), |mut acc, (name, value)| {
                acc.insert(name, value);
                acc
            });

        logger::info!(
            target: SPAN_TARGET,
            id = id,
            method = ?method,
            uri = ?uri.to_string(),
            status = ?status.code.to_string(),
            from = ?request.client_ip().map(|x| x.to_string()).unwrap_or_default(),
            headers = ?headers,
            took = ?duration,
        );
    }
}

struct RequestTimer(std::time::Instant);

impl RequestTimer {
    fn new() -> Self {
        Self(std::time::Instant::now())
    }

    fn elapsed(&self) -> std::time::Duration {
        self.0.elapsed()
    }
}

impl Default for RequestTimer {
    fn default() -> Self {
        Self::new()
    }
}
