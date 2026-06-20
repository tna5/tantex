use std::sync::Arc;
use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
};
use tokio::sync::RwLock;
use serde::Deserialize;

use crate::config::{Config, ConfigPatch};
use crate::engine::index_manager::IndexManager;
use crate::engine::writer::WriterCommand;
use crate::protocol::messages::*;

pub type AppState = (Arc<RwLock<Config>>, Arc<RwLock<IndexManager>>);

pub async fn list_indexes(
    State((_config, index_manager)): State<AppState>,
) -> Json<ListIndexesResponse> {
    let mgr = index_manager.read().await;
    Json(mgr.list_indexes())
}

pub async fn create_index(
    State((_config, index_manager)): State<AppState>,
    Json(payload): Json<CreateIndexRequest>,
) -> Result<Json<CreateIndexResponse>, (StatusCode, String)> {
    let mut mgr = index_manager.write().await;
    match mgr.create_index(&payload.name, payload.schema) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn get_index(
    State((_config, index_manager)): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<GetIndexResponse>, (StatusCode, String)> {
    let mgr = index_manager.read().await;
    match mgr.get_index(&name) {
        Ok(info) => Ok(Json(info)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

pub async fn delete_index(
    State((_config, index_manager)): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeleteIndexResponse>, (StatusCode, String)> {
    let mut mgr = index_manager.write().await;
    match mgr.delete_index(&name) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn commit_index(
    State((_config, index_manager)): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    let sender = {
        let mgr = index_manager.read().await;
        match mgr.get_managed_index(&name) {
            Ok(managed) => managed.writer.sender.clone(),
            Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
        }
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = sender.send(WriterCommand::Commit { response: tx });

    match rx.await {
        Ok(Ok(())) => Ok(Json(SuccessResponse { success: true })),
        Ok(Err(e)) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_segments(
    State((_config, index_manager)): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<GetSegmentsResponse>, (StatusCode, String)> {
    let mgr = index_manager.read().await;
    match mgr.get_segments(&name) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_search_limit() -> usize { 10 }

pub async fn search_index(
    State((_config, index_manager)): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let mgr = index_manager.read().await;
    match mgr.get_managed_index(&name) {
        Ok(managed) => match managed.searcher.search(&payload.query, payload.limit, payload.offset) {
            Ok(resp) => Ok(Json(resp)),
            Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
        },
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

#[derive(Deserialize)]
pub struct DeleteByQueryBody {
    pub query: String,
}

pub async fn delete_by_query(
    State((_config, index_manager)): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<DeleteByQueryBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (pre_count, sender) = {
        let mgr = index_manager.read().await;
        match mgr.get_managed_index(&name) {
            Ok(managed) => {
                let count = managed.searcher.search(&payload.query, 1, 0)
                    .map(|r| r.total_hits)
                    .unwrap_or(0);
                (count, managed.writer.sender.clone())
            }
            Err(e) => return Err((StatusCode::NOT_FOUND, e.to_string())),
        }
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = sender.send(WriterCommand::DeleteByQuery {
        query_str: payload.query,
        response: tx,
    });

    match rx.await {
        Ok(Ok(())) => Ok(Json(serde_json::json!({ "deleted": pre_count }))),
        Ok(Err(e)) => Err((StatusCode::BAD_REQUEST, e)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
pub struct IndexSettingsPatch {
    pub merge_target_docs: Option<usize>,
    pub max_merge_factor: Option<usize>,
    pub min_num_segments: Option<usize>,
}

pub async fn set_index_settings(
    State((config, index_manager)): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<IndexSettingsPatch>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    let (sender, default_target, default_factor, default_min) = {
        let mgr = index_manager.read().await;
        let cfg = config.read().await;
        match mgr.get_managed_index(&name) {
            Ok(managed) => (
                managed.writer.sender.clone(),
                cfg.merge_target_docs,
                cfg.max_merge_factor,
                cfg.min_num_segments,
            ),
            Err(e) => return Err((StatusCode::NOT_FOUND, e.to_string())),
        }
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = sender.send(WriterCommand::SetMergePolicy {
        target_docs: payload.merge_target_docs.unwrap_or(default_target),
        max_factor: payload.max_merge_factor.unwrap_or(default_factor),
        min_segments: payload.min_num_segments.unwrap_or(default_min),
        response: tx,
    });

    match rx.await {
        Ok(Ok(())) => Ok(Json(SuccessResponse { success: true })),
        Ok(Err(e)) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_metrics(
    State((_config, index_manager)): State<AppState>,
) -> Json<serde_json::Value> {
    let mgr = index_manager.read().await;
    let indexes = mgr.list_indexes().indexes;

    let total_docs: u64 = indexes.iter().map(|i| i.doc_count).sum();
    let total_segments: usize = indexes.iter().map(|i| i.num_segments).sum();
    let total_pending: u64 = indexes.iter().map(|i| i.pending_docs).sum();

    Json(serde_json::json!({
        "totalDocs": total_docs,
        "totalIndexes": indexes.len(),
        "totalSegments": total_segments,
        "totalPendingDocs": total_pending,
        "indexes": indexes,
    }))
}

pub async fn metrics_stream(
    State((_config, index_manager)): State<AppState>,
) -> axum::response::sse::Sse<impl futures::stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    let stream = super::sse::metrics_stream(index_manager);
    axum::response::sse::Sse::new(stream)
}

pub async fn get_config(
    State((config, _index_manager)): State<AppState>,
) -> Json<ConfigResponse> {
    let cfg = config.read().await;
    Json(ConfigResponse {
        socket_path: cfg.socket_path.clone(),
        data_dir: cfg.data_dir.clone(),
        shm_buffer_size: cfg.shm_buffer_size,
        writer_heap_size: cfg.writer_heap_size,
        auto_commit_doc_count: cfg.auto_commit_doc_count,
        auto_commit_interval_secs: cfg.auto_commit_interval_secs,
        merge_target_docs: cfg.merge_target_docs,
        max_merge_factor: cfg.max_merge_factor,
        min_num_segments: cfg.min_num_segments,
        num_indexing_threads: cfg.num_indexing_threads,
        index_threads_pct: cfg.index_threads_pct,
        hard_commit_multiplier: cfg.hard_commit_multiplier,
    })
}

pub async fn set_config(
    State((config, _index_manager)): State<AppState>,
    Json(patch): Json<SetConfigRequest>,
) -> Json<SuccessResponse> {
    let mut cfg = config.write().await;
    let config_patch = ConfigPatch {
        shm_buffer_size: patch.shm_buffer_size,
        writer_heap_size: patch.writer_heap_size,
        auto_commit_doc_count: patch.auto_commit_doc_count,
        auto_commit_interval_secs: patch.auto_commit_interval_secs,
        merge_target_docs: patch.merge_target_docs,
        max_merge_factor: patch.max_merge_factor,
        min_num_segments: patch.min_num_segments,
        num_indexing_threads: patch.num_indexing_threads,
        index_threads_pct: patch.index_threads_pct,
        hard_commit_multiplier: patch.hard_commit_multiplier,
    };
    cfg.apply_patch(config_patch);
    Json(SuccessResponse { success: true })
}

pub async fn auth_status(
    State((config, _index_manager)): State<AppState>,
) -> Json<serde_json::Value> {
    let cfg = config.read().await;
    Json(serde_json::json!({ "auth_required": cfg.api_key.is_some() }))
}
