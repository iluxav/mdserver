use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::{Path as AxumPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use crate::cache::Cache;
use crate::index;
use crate::introspect;

const MARKDOWN_CT: &str = "text/markdown; charset=utf-8";

#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub cache: Arc<Cache>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/introspect", get(handle_introspect))
        .route("/*path", get(handle_path).put(handle_put))
        .with_state(state)
}

async fn handle_put(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    body: axum::body::Bytes,
) -> Response {
    let trimmed = path.trim_end_matches('/');
    if !is_md_path(trimmed) {
        return (StatusCode::METHOD_NOT_ALLOWED, "only .md files can be written\n").into_response();
    }
    let fs_path = state.root.join(trimmed);

    if let Ok(meta) = tokio::fs::metadata(&fs_path).await {
        if meta.is_dir() {
            return (StatusCode::METHOD_NOT_ALLOWED, "is a directory\n").into_response();
        }
    }

    match tokio::fs::write(&fs_path, &body).await {
        Ok(()) => {
            state.cache.files.remove(&fs_path);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "write failed\n").into_response(),
    }
}

fn is_md_path(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

async fn handle_introspect(State(state): State<AppState>) -> Response {
    match introspect::build(state.cache.clone(), state.root.clone()).await {
        Ok(tree) => Json(tree).into_response(),
        Err(_) => internal_error(),
    }
}

async fn handle_root(State(state): State<AppState>) -> Response {
    let root = state.root.clone();
    serve(&state, &root, "/").await
}

async fn handle_path(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Response {
    let trimmed = path.trim_end_matches('/');
    let fs_path = state.root.join(trimmed);
    let url_path = format!("/{}", trimmed);
    serve(&state, &fs_path, &url_path).await
}

async fn serve(state: &AppState, fs_path: &Path, url_path: &str) -> Response {
    let meta = match tokio::fs::metadata(fs_path).await {
        Ok(m) => m,
        Err(_) => return not_found(),
    };

    if meta.is_dir() {
        match index::render(&state.cache, fs_path, url_path).await {
            Ok(rendered) => (
                [(header::CONTENT_TYPE, MARKDOWN_CT)],
                rendered.as_str().to_owned(),
            )
                .into_response(),
            Err(_) => internal_error(),
        }
    } else {
        let is_md = fs_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            return not_found();
        }
        match tokio::fs::read(fs_path).await {
            Ok(bytes) => ([(header::CONTENT_TYPE, MARKDOWN_CT)], bytes).into_response(),
            Err(_) => not_found(),
        }
    }
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "not found\n").into_response()
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error\n").into_response()
}
