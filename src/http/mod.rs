pub mod routes;
pub mod sse;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
    http::{StatusCode, Request, HeaderMap},
    response::IntoResponse,
    middleware::{self, Next},
    extract::State,
    Json,
};
use rust_embed::RustEmbed;
use tokio::sync::RwLock;
use serde::Deserialize;

use crate::config::Config;
use crate::engine::index_manager::IndexManager;
use routes::{
    AppState,
    list_indexes, create_index, get_index, delete_index, commit_index, get_segments,
    search_index, get_metrics, metrics_stream, get_config, set_config,
    auth_status, delete_by_query, set_index_settings,
};

#[derive(RustEmbed)]
#[folder = "dashboard/.output/public"]
struct DashboardAssets;

async fn serve_static(path: String) -> impl IntoResponse {
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    match DashboardAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [("content-type", mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => {
            match DashboardAssets::get("index.html") {
                Some(content) => (
                    StatusCode::OK,
                    [("content-type", "text/html")],
                    content.data.into_owned(),
                )
                    .into_response(),
                None => (StatusCode::NOT_FOUND, "Not found").into_response(),
            }
        }
    }
}

fn extract_key(headers: &HeaderMap, uri: &axum::http::Uri) -> String {
    // 1. Cookie: tantex_key=<value>
    if let Some(cookie_header) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for part in cookie_header.split(';') {
            if let Some(val) = part.trim().strip_prefix("tantex_key=") {
                return val.to_string();
            }
        }
    }
    // 2. X-Api-Key header
    if let Some(val) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
        return val.to_string();
    }
    // 3. Authorization: Bearer <key>
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(key) = auth.strip_prefix("Bearer ") {
            return key.to_string();
        }
    }
    // 4. ?api_key=<key> query parameter (for EventSource which can't set headers)
    if let Some(query) = uri.query() {
        for pair in query.split('&') {
            if let Some(val) = pair.strip_prefix("api_key=") {
                return val.to_string();
            }
        }
    }
    String::new()
}

async fn auth_middleware(
    State((config, _)): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, (StatusCode, &'static str)> {
    let path = request.uri().path().to_string();

    // Auth status and login endpoints are always public
    if path == "/api/auth/status" || path == "/api/auth/login" || !path.starts_with("/api/") {
        return Ok(next.run(request).await);
    }

    let api_key = {
        let cfg = config.read().await;
        cfg.api_key.clone()
    };

    match api_key {
        None => Ok(next.run(request).await),
        Some(expected) => {
            let provided = extract_key(request.headers(), request.uri());
            if provided == expected {
                Ok(next.run(request).await)
            } else {
                Err((StatusCode::UNAUTHORIZED, "API key required"))
            }
        }
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    key: String,
}

async fn login(
    State((config, _)): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let api_key = {
        let cfg = config.read().await;
        cfg.api_key.clone()
    };

    match api_key {
        None => {
            // No auth configured — login always succeeds
            (
                StatusCode::OK,
                [("content-type", "application/json")],
                r#"{"success":true}"#,
            ).into_response()
        }
        Some(expected) if expected == payload.key => {
            let cookie = format!(
                "tantex_key={}; HttpOnly; SameSite=Strict; Path=/",
                payload.key
            );
            (
                StatusCode::OK,
                [
                    ("content-type", "application/json"),
                    ("set-cookie", &cookie),
                ],
                r#"{"success":true}"#,
            ).into_response()
        }
        Some(_) => (
            StatusCode::UNAUTHORIZED,
            [("content-type", "application/json")],
            r#"{"error":"Invalid key"}"#,
        ).into_response(),
    }
}

async fn logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            ("content-type", "application/json"),
            ("set-cookie", "tantex_key=; Max-Age=0; Path=/"),
        ],
        r#"{"success":true}"#,
    ).into_response()
}

pub async fn start_http_server(
    config: Arc<RwLock<Config>>,
    index_manager: Arc<RwLock<IndexManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let port = config.read().await.http_port;

    let state: AppState = (config, index_manager);

    let app = Router::new()
        // Auth
        .route("/api/auth/status", get(auth_status))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        // Index routes
        .route("/api/indexes", get(list_indexes).post(create_index))
        .route("/api/indexes/{name}", get(get_index).delete(delete_index))
        .route("/api/indexes/{name}/commit", post(commit_index))
        .route("/api/indexes/{name}/segments", get(get_segments))
        .route("/api/indexes/{name}/search", post(search_index))
        .route("/api/indexes/{name}/delete", post(delete_by_query))
        .route("/api/indexes/{name}/settings", post(set_index_settings))
        // Metrics
        .route("/api/metrics", get(get_metrics))
        .route("/api/metrics/stream", get(metrics_stream))
        // Config
        .route("/api/config", get(get_config).post(set_config))
        // Auth middleware applies to all /api/* routes
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        // Static files fallback (embedded)
        .fallback(|uri: axum::http::Uri| serve_static(uri.path().to_string()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    log::info!("HTTP server listening on :{}", port);

    axum::serve(
        listener,
        app.into_make_service(),
    )
    .await?;

    Ok(())
}
