mod pages;

use axum::{
    Router,
    body::Body,
    http::{HeaderName, HeaderValue, Request, Response, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use std::{net::SocketAddr, path::Path};
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing::{error, info};

// Pre-defined security header names and values (compiled at build time, no per-request parsing)
static REFERRER_POLICY: HeaderName = HeaderName::from_static("referrer-policy");
static X_XSS_PROTECTION: HeaderName = HeaderName::from_static("x-xss-protection");
static PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");

static NOSNIFF: HeaderValue = HeaderValue::from_static("nosniff");
static SAMEORIGIN: HeaderValue = HeaderValue::from_static("SAMEORIGIN");
static REFERRER_POLICY_VALUE: HeaderValue =
    HeaderValue::from_static("strict-origin-when-cross-origin");
static XSS_PROTECTION_VALUE: HeaderValue = HeaderValue::from_static("1; mode=block");
static PERMISSIONS_POLICY_VALUE: HeaderValue =
    HeaderValue::from_static("geolocation=(), microphone=(), camera=()");

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

static NO_CACHE: HeaderValue = HeaderValue::from_static("no-cache");

/// Always send the current content, and make browsers check back for pages.
///
/// Conditional request headers are dropped, so static files are never answered with 304. After
/// a rollback they carry an older Last-Modified, and a 304 would keep browsers on the newer build.
///
/// Pages get `no-cache`, so a deploy shows up on the next visit instead of whenever the
/// browser's own guess about the cache lifetime runs out.
async fn serve_current_files(mut request: Request<Body>, next: Next) -> Response<Body> {
    let headers = request.headers_mut();
    for name in [
        header::IF_MODIFIED_SINCE,
        header::IF_NONE_MATCH,
        header::IF_UNMODIFIED_SINCE,
        header::IF_MATCH,
    ] {
        headers.remove(name);
    }
    // A range is only safe to serve when its If-Range still matches; send the whole file instead.
    if headers.remove(header::IF_RANGE).is_some() {
        headers.remove(header::RANGE);
    }

    let mut response = next.run(request).await;
    let is_page = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    if is_page {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, NO_CACHE.clone());
    }
    response
}

fn page(rendered: askama::Result<String>, status: StatusCode) -> Response<Body> {
    match rendered {
        Ok(html) => (status, Html(html)).into_response(),
        Err(err) => {
            error!("rendering a page failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn home() -> Response<Body> {
    page(pages::home(), StatusCode::OK)
}

async fn pe() -> Response<Body> {
    page(pages::pe(), StatusCode::OK)
}

async fn not_found(uri: Uri) -> Response<Body> {
    page(pages::not_found(uri.path()), StatusCode::NOT_FOUND)
}

/// `thc1006-web render <dir>` writes the pages as files, so CI can run an HTML validator on them.
fn render_to(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir.join("PE"))?;
    std::fs::write(dir.join("index.html"), pages::home()?)?;
    std::fs::write(dir.join("PE/index.html"), pages::pe()?)?;
    std::fs::write(dir.join("404.html"), pages::not_found("/404.html")?)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Parse the data files first: bad data should stop the server, not break a page later.
    pages::load();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("render") {
        let Some(dir) = args.get(1) else {
            error!("usage: thc1006-web render <dir>");
            std::process::exit(2);
        };
        if let Err(err) = render_to(Path::new(dir)) {
            error!("render failed: {err}");
            std::process::exit(1);
        }
        info!("Pages written to {dir}");
        return;
    }

    // Pages come from the templates; the stylesheet and images are served from /app/static.
    let static_files = ServeDir::new("/app/static").fallback(get(not_found));

    let app = Router::new()
        .route("/", get(home))
        .route("/index.html", get(home))
        .route("/PE", get(|| async { Redirect::temporary("/PE/") }))
        .route("/PE/", get(pe))
        .route("/PE/index.html", get(pe))
        .fallback_service(static_files)
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(serve_current_files))
        .layer(middleware::from_fn(security_headers));

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server starting on http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
