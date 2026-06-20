use std::sync::Arc;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::RwLock;

use crate::config::{Config, ConfigPatch};
use crate::engine::index_manager::IndexManager;
use crate::engine::writer::WriterCommand;
use crate::protocol::codec;
use crate::protocol::messages::*;
use crate::shm::buffer::ShmBuffer;

struct ConnectionState {
    shm_buffer: Option<Arc<ShmBuffer>>,
    session_id: String,
}

pub async fn handle_connection(
    mut stream: UnixStream,
    index_manager: Arc<RwLock<IndexManager>>,
    config: Arc<RwLock<Config>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = BytesMut::with_capacity(64 * 1024);
    let mut state = ConnectionState {
        shm_buffer: None,
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    log::debug!("New connection, session_id={}", state.session_id);

    loop {
        let n = stream.read_buf(&mut buf).await?;
        if n == 0 {
            log::debug!("Connection closed, session_id={}", state.session_id);
            break;
        }

        while let Some((msg_type, payload)) = codec::decode_frame(&mut buf) {
            let (resp_type, resp_payload) =
                handle_message(msg_type, &payload, &mut state, &index_manager, &config).await;
            let response_bytes = codec::encode_message(resp_type, &resp_payload);
            stream.write_all(&response_bytes).await?;
        }
    }

    Ok(())
}

fn error_response(msg: impl std::fmt::Display) -> (u8, Vec<u8>) {
    let err = ErrorResponse {
        code: 500,
        message: msg.to_string(),
    };
    (
        MSG_RESPONSE_ERR,
        rmp_serde::to_vec_named(&err).unwrap_or_default(),
    )
}

async fn handle_message(
    msg_type: u8,
    payload: &[u8],
    state: &mut ConnectionState,
    index_manager: &Arc<RwLock<IndexManager>>,
    config: &Arc<RwLock<Config>>,
) -> (u8, Vec<u8>) {
    match msg_type {
        MSG_CREATE_INDEX => {
            let req: CreateIndexRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let mut mgr = index_manager.write().await;
            match mgr.create_index(&req.name, req.schema) {
                Ok(resp) => (
                    MSG_RESPONSE_OK,
                    rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                ),
                Err(e) => error_response(e),
            }
        }

        MSG_DELETE_INDEX => {
            let req: DeleteIndexRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let mut mgr = index_manager.write().await;
            match mgr.delete_index(&req.name) {
                Ok(resp) => (
                    MSG_RESPONSE_OK,
                    rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                ),
                Err(e) => error_response(e),
            }
        }

        MSG_LIST_INDEXES => {
            let mgr = index_manager.read().await;
            let resp = mgr.list_indexes();
            (
                MSG_RESPONSE_OK,
                rmp_serde::to_vec_named(&resp).unwrap_or_default(),
            )
        }

        MSG_GET_INDEX => {
            let req: GetIndexRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let mgr = index_manager.read().await;
            match mgr.get_index(&req.name) {
                Ok(resp) => (
                    MSG_RESPONSE_OK,
                    rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                ),
                Err(e) => error_response(e),
            }
        }

        MSG_GET_SEGMENTS => {
            let req: GetSegmentsRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let mgr = index_manager.read().await;
            match mgr.get_segments(&req.name) {
                Ok(resp) => (
                    MSG_RESPONSE_OK,
                    rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                ),
                Err(e) => error_response(e),
            }
        }

        MSG_INIT_SHM => {
            let req: InitShmRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let size = if req.buffer_size == 0 {
                config.read().await.shm_buffer_size
            } else {
                req.buffer_size as usize
            };
            match ShmBuffer::create(&state.session_id, size) {
                Ok(shm) => {
                    let path = shm.path().to_string();
                    state.shm_buffer = Some(Arc::new(shm));
                    let resp = InitShmResponse { shm_path: path };
                    (
                        MSG_RESPONSE_OK,
                        rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                    )
                }
                Err(e) => error_response(e),
            }
        }

        MSG_INGEST_SHM => {
            let req: IngestShmRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            if let Some(ref shm) = state.shm_buffer {
                // Use zero-copy path via Arc
                let shm_arc = Arc::clone(shm);
                let sender = {
                    let mgr = index_manager.read().await;
                    match mgr.get_managed_index(&req.index) {
                        Ok(managed) => managed.writer.sender.clone(),
                        Err(e) => return error_response(e),
                    }
                };
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = sender.send(WriterCommand::AddDocumentsFromShmRef {
                    shm: shm_arc,
                    length: req.length as usize,
                    doc_count: req.doc_count,
                    response: tx,
                });
                match rx.await {
                    Ok(Ok(count)) => {
                        let resp = IngestResponse {
                            indexed: count,
                            errors: vec![],
                        };
                        (
                            MSG_RESPONSE_OK,
                            rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                        )
                    }
                    Ok(Err(e)) => error_response(e),
                    Err(e) => error_response(format!("Writer channel error: {}", e)),
                }
            } else {
                error_response("No SHM session initialized")
            }
        }

        MSG_INGEST_BATCH => {
            let req: IngestBatchRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            // Extract sender then drop the guard before awaiting
            let sender = {
                let mgr = index_manager.read().await;
                match mgr.get_managed_index(&req.index) {
                    Ok(managed) => managed.writer.sender.clone(),
                    Err(e) => return error_response(e),
                }
            };
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = sender.send(WriterCommand::AddDocuments {
                documents: req.documents,
                response: tx,
            });
            match rx.await {
                Ok(Ok(count)) => {
                    let resp = IngestResponse {
                        indexed: count,
                        errors: vec![],
                    };
                    (
                        MSG_RESPONSE_OK,
                        rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                    )
                }
                Ok(Err(e)) => error_response(e),
                Err(e) => error_response(format!("Writer channel error: {}", e)),
            }
        }

        MSG_DELETE_BY_QUERY => {
            let req: DeleteByQueryRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let (pre_count, sender) = {
                let mgr = index_manager.read().await;
                match mgr.get_managed_index(&req.index) {
                    Ok(managed) => {
                        let count = managed.searcher.search(&req.query, 1, 0)
                            .map(|r| r.total_hits)
                            .unwrap_or(0);
                        (count, managed.writer.sender.clone())
                    }
                    Err(e) => return error_response(e),
                }
            };
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = sender.send(WriterCommand::DeleteByQuery { query_str: req.query, response: tx });
            match rx.await {
                Ok(Ok(())) => {
                    let resp = serde_json::json!({ "deleted": pre_count });
                    (MSG_RESPONSE_OK, rmp_serde::to_vec_named(&resp).unwrap_or_default())
                }
                Ok(Err(e)) => error_response(e),
                Err(e) => error_response(format!("Writer channel error: {}", e)),
            }
        }

        MSG_SEARCH => {
            let req: SearchRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let mgr = index_manager.read().await;
            match mgr.get_managed_index(&req.index) {
                Ok(managed) => match managed.searcher.search(&req.query, req.limit, req.offset) {
                    Ok(resp) => (
                        MSG_RESPONSE_OK,
                        rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                    ),
                    Err(e) => error_response(e),
                },
                Err(e) => error_response(e),
            }
        }

        MSG_COMMIT => {
            let req: CommitRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            // Extract sender then drop the guard before awaiting
            let sender = {
                let mgr = index_manager.read().await;
                match mgr.get_managed_index(&req.index) {
                    Ok(managed) => managed.writer.sender.clone(),
                    Err(e) => return error_response(e),
                }
            };
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = sender.send(WriterCommand::Commit { response: tx });
            match rx.await {
                Ok(Ok(())) => {
                    let resp = SuccessResponse { success: true };
                    (
                        MSG_RESPONSE_OK,
                        rmp_serde::to_vec_named(&resp).unwrap_or_default(),
                    )
                }
                Ok(Err(e)) => error_response(e),
                Err(e) => error_response(format!("Writer channel error: {}", e)),
            }
        }

        MSG_CLOSE_SHM => {
            state.shm_buffer = None;
            let resp = SuccessResponse { success: true };
            (
                MSG_RESPONSE_OK,
                rmp_serde::to_vec_named(&resp).unwrap_or_default(),
            )
        }

        MSG_GET_CONFIG => {
            let cfg = config.read().await;
            let resp = ConfigResponse {
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
            };
            (MSG_RESPONSE_OK, rmp_serde::to_vec_named(&resp).unwrap_or_default())
        }

        MSG_SET_CONFIG => {
            let req: SetConfigRequest = match rmp_serde::from_slice(payload) {
                Ok(r) => r,
                Err(e) => return error_response(format!("Invalid request: {}", e)),
            };
            let patch = ConfigPatch {
                shm_buffer_size: req.shm_buffer_size,
                writer_heap_size: req.writer_heap_size,
                auto_commit_doc_count: req.auto_commit_doc_count,
                auto_commit_interval_secs: req.auto_commit_interval_secs,
                merge_target_docs: req.merge_target_docs,
                max_merge_factor: req.max_merge_factor,
                min_num_segments: req.min_num_segments,
                num_indexing_threads: req.num_indexing_threads,
                index_threads_pct: req.index_threads_pct,
                hard_commit_multiplier: req.hard_commit_multiplier,
            };
            {
                let mut cfg = config.write().await;
                cfg.apply_patch(patch);
                log::info!("Config updated via MSG_SET_CONFIG");
            }
            let resp = SuccessResponse { success: true };
            (MSG_RESPONSE_OK, rmp_serde::to_vec_named(&resp).unwrap_or_default())
        }

        _ => error_response(format!("Unknown message type: 0x{:02x}", msg_type)),
    }
}
