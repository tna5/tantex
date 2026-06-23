use serde::{Deserialize, Serialize};

// Message type constants
pub const MSG_CREATE_INDEX: u8 = 0x01;
pub const MSG_DELETE_INDEX: u8 = 0x02;
pub const MSG_LIST_INDEXES: u8 = 0x03;
pub const MSG_GET_INDEX: u8 = 0x04;
pub const MSG_GET_SEGMENTS: u8 = 0x05;
pub const MSG_INGEST_BATCH: u8 = 0x10;
pub const MSG_INIT_SHM: u8 = 0x11;
pub const MSG_INGEST_SHM: u8 = 0x12;
pub const MSG_CLOSE_SHM: u8 = 0x13;
pub const MSG_SEARCH: u8 = 0x20;
pub const MSG_COMMIT: u8 = 0x21;
pub const MSG_DELETE_BY_QUERY: u8 = 0x22;
pub const MSG_GET_CONFIG: u8 = 0x30;
pub const MSG_SET_CONFIG: u8 = 0x31;
pub const MSG_RESPONSE_OK: u8 = 0x80;
pub const MSG_RESPONSE_ERR: u8 = 0x81;

/// Definition for a sub-path inside a `json` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubFieldDef {
    #[serde(default = "default_tokenizer")]
    pub tokenizer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default = "default_true")]
    pub stored: bool,
    #[serde(default = "default_true")]
    pub indexed: bool,
    #[serde(default)]
    pub fast: bool,
    #[serde(default = "default_tokenizer")]
    pub tokenizer: String,
    /// For json fields: per-path sub-field definitions (preferred API).
    /// Key = sub-path inside the json object, value = SubFieldDef.
    /// Takes precedence over `field_tokenizers` when both are present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<std::collections::BTreeMap<String, SubFieldDef>>,
    /// For json fields: per-path tokenizer overrides (legacy shorthand).
    /// Use `fields` instead. Accepted for backward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_tokenizers: Option<std::collections::BTreeMap<String, String>>,
}

fn default_true() -> bool {
    true
}

fn default_tokenizer() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub fields: Vec<FieldDefinition>,
    /// Docstore compression: "none" | "lz4" (default) | "brotli" | "snappy" | "zstd" | "zstd:<level>".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
    /// Docstore block size in bytes (default 16384). Larger blocks → better
    /// compression ratio, slower retrieval of individual docs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_size: Option<usize>,
}

// --- REQUESTS ---

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIndexRequest {
    pub name: String,
    pub schema: SchemaDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteIndexRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIndexRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSegmentsRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestBatchRequest {
    pub index: String,
    pub documents: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitShmRequest {
    pub buffer_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestShmRequest {
    pub index: String,
    pub length: u64,
    pub doc_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub index: String,
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitRequest {
    pub index: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteByQueryRequest {
    pub index: String,
    pub query: String,
}

// --- RESPONSES ---

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIndexResponse {
    pub success: bool,
    pub field_ids: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteIndexResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub doc_count: u64,
    pub num_segments: usize,
    #[serde(default)]
    pub pending_docs: u64,
    #[serde(default)]
    pub search_count: u64,
    #[serde(default)]
    pub search_latency_us: u64,
    #[serde(default)]
    pub total_docs_ingested: u64,
    #[serde(default)]
    pub raw_bytes_ingested: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListIndexesResponse {
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentInfo {
    #[serde(rename = "segment_id")]
    pub segment_id: String,
    #[serde(rename = "num_docs")]
    pub num_docs: u32,
    #[serde(rename = "num_deleted_docs")]
    pub num_deleted_docs: u32,
    #[serde(rename = "size_bytes")]
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSegmentsResponse {
    pub segments: Vec<SegmentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIndexResponse {
    pub name: String,
    pub schema: SchemaDefinition,
    pub doc_count: u64,
    pub num_segments: usize,
    #[serde(default)]
    pub pending_docs: u64,
    #[serde(default)]
    pub search_count: u64,
    #[serde(default)]
    pub search_latency_us: u64,
    #[serde(default)]
    pub total_docs_ingested: u64,
    #[serde(default)]
    pub raw_bytes_ingested: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitShmResponse {
    pub shm_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestResponse {
    pub indexed: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub score: f32,
    pub doc: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_hits: u64,
    pub hits: Vec<SearchHit>,
    /// Server-side query latency in microseconds (tantivy parse + collect + retrieve docs).
    /// Excludes socket / framing overhead.
    #[serde(default)]
    pub elapsed_us: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub socket_path: String,
    pub data_dir: String,
    pub shm_buffer_size: usize,
    pub writer_heap_size: usize,
    pub auto_commit_doc_count: usize,
    pub auto_commit_interval_secs: u64,
    pub merge_target_docs: usize,
    pub max_merge_factor: usize,
    pub min_num_segments: usize,
    pub num_indexing_threads: usize,
    pub index_threads_pct: u32,
    pub hard_commit_multiplier: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SetConfigRequest {
    pub shm_buffer_size: Option<usize>,
    pub writer_heap_size: Option<usize>,
    pub auto_commit_doc_count: Option<usize>,
    pub auto_commit_interval_secs: Option<u64>,
    pub merge_target_docs: Option<usize>,
    pub max_merge_factor: Option<usize>,
    pub min_num_segments: Option<usize>,
    pub num_indexing_threads: Option<usize>,
    pub index_threads_pct: Option<u32>,
    pub hard_commit_multiplier: Option<u32>,
}
