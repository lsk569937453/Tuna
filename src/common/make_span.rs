use axum::http;
use http::header::HeaderValue;
use uuid::Uuid;
#[derive(Clone)]
pub struct RequestIdSpan;

impl<B> tower_http::trace::MakeSpan<B> for RequestIdSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        tracing::info_span!(
            target: "access_log",
            "access_log",
            log = %format!("|{}|{}|{:?}", request.method(),  request.uri(), request.version()),
        )
    }
}
