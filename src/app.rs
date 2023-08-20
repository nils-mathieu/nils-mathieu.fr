use std::convert::Infallible;
use std::future::Ready;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, Request, Response, StatusCode};
use axum::Router;

use tower_service::Service;

/// Creates a [`Router`] that serves the application.
pub fn app<S>() -> Router<S>
where
    S: 'static + Send + Sync + Clone,
{
    Router::new()
        .fallback_service(error_404_service())
        .route_service("/", home_page_service())
        .nest_service("/static", static_files_service())
}

/// Returns the service that's responsible for serving static files.
fn static_files_service() -> tower_http::services::ServeDir<StaticFilesFallback> {
    tower_http::services::ServeDir::new("www")
        .append_index_html_on_directories(false)
        .call_fallback_on_method_not_allowed(true)
        .fallback(StaticFilesFallback)
}

/// Returns the service that's responsible for serving the home page.
fn home_page_service() -> tower_http::services::ServeFile {
    tower_http::services::ServeFile::new("www/index.html")
}

/// Returns the fallback service used when a static file cannot be found.
fn error_404_service() -> tower_http::services::ServeFile {
    tower_http::services::ServeFile::new("www/404.html")
}

/// The fallback service used when a static file cannot be found.
#[derive(Debug, Clone, Copy)]
pub struct StaticFilesFallback;

impl Service<Request<Body>> for StaticFilesFallback {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let code = match *req.method() {
            Method::GET => StatusCode::NOT_FOUND,
            _ => StatusCode::METHOD_NOT_ALLOWED,
        };

        let message = match code {
            StatusCode::NOT_FOUND => "not found",
            StatusCode::METHOD_NOT_ALLOWED => "method not allowed",
            _ => unreachable!(),
        };

        let response = Response::builder()
            .status(code)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(message))
            .unwrap();

        std::future::ready(Ok(response))
    }
}
