use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer,
    services::ServeDir,
};
use tracing::info;

/// Security headers middleware
async fn security_headers(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Security headers
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        "SAMEORIGIN".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    response
}

/// Custom 404 handler - serves 404.html
async fn handle_404() -> impl IntoResponse {
    let content = tokio::fs::read_to_string("/app/static/404.html")
        .await
        .unwrap_or_else(|_| "404 Not Found".to_string());

    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        content,
    )
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
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
