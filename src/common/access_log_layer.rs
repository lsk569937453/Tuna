use axum::response::Response;
use std::time::Duration;
use tower_http::trace::OnResponse;
use tracing::Span;
#[derive(Clone, Debug)]
pub struct AccelogOnResponse;
impl<B> OnResponse<B> for AccelogOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, _: &Span) {
        let log_str = format!(
            "|{}ms|httpCode={}",
            latency.as_millis(),
            response.status().as_u16()
        );
        info!(target: "access_log", log_str);
    }
}
