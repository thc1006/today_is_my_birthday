use axum::{
    Router,
    body::Body,
    http::{HeaderName, HeaderValue, Request, Response, StatusCode, header},
    middleware::{self, Next},
    response::IntoResponse,
};
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing::info;

// Pre-defined security header names and values (compiled at build time, no per-request parsing)
static REFERRER_POLICY: HeaderName = HeaderName::from_static("referrer-policy");
static X_XSS_PROTECTION: HeaderName = HeaderName::from_static("x-xss-protection");
static PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");

static NOSNIFF: HeaderValue = HeaderValue::from_static("nosniff");
static SAMEORIGIN: HeaderValue = HeaderValue::from_static("SAMEORIGIN");
static REFERRER_POLICY_VALUE: HeaderValue = HeaderValue::from_static("strict-origin-when-cross-origin");
static XSS_PROTECTION_VALUE: HeaderValue = HeaderValue::from_static("1; mode=block");
static PERMISSIONS_POLICY_VALUE: HeaderValue = HeaderValue::from_static("geolocation=(), microphone=(), camera=()");

/// Security headers middleware
async fn security_headers(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Security headers (using pre-built static values for performance)
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, NOSNIFF.clone());
    headers.insert(header::X_FRAME_OPTIONS, SAMEORIGIN.clone());
    headers.insert(REFERRER_POLICY.clone(), REFERRER_POLICY_VALUE.clone());
    headers.insert(X_XSS_PROTECTION.clone(), XSS_PROTECTION_VALUE.clone());
    headers.insert(PERMISSIONS_POLICY.clone(), PERMISSIONS_POLICY_VALUE.clone());

    response
}

/// Embedded 404 HTML content (compiled into the binary for performance)
static NOT_FOUND_HTML: &str = include_str!("../static/404.html");

/// Custom 404 handler - serves embedded 404.html
async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        NOT_FOUND_HTML,
    )
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Serve static files with fallback to 404
    let serve_dir = ServeDir::new("/app/static")
        .append_index_html_on_directories(true)
        .fallback(axum::routing::get(handle_404));

    // Build application router
    let app = Router::new()
        .fallback_service(serve_dir)
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(security_headers));

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server starting on http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
