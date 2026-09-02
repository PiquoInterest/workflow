use serde::{Deserialize, Serialize};

/// Controls whether opaque payloads are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolveData {
    None,
    All,
}

/// Sort direction for cursor-based lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Options for paginated queries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub sort_order: Option<SortOrder>,
}

/// Optional lookback-window metadata returned with a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub current_lookback_days: f64,
    pub max_lookback_days: f64,
    pub current_window_start: String,
    pub max_window_start: String,
    pub upgrade_available: bool,
}

/// Standard cursor-based response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub page_info: Option<PageInfo>,
}

/// Standard structured error shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredError {
    pub message: String,
    pub stack: Option<String>,
    pub code: Option<String>,
}

/// One stream chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamChunk {
    pub index: u64,
    pub data: Vec<u8>,
}

/// Options for paginated chunk retrieval.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetChunksOptions {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Metadata about a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfoResponse {
    pub tail_index: i64,
    pub done: bool,
}

/// Cursor page of stream chunks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunksResponse {
    pub data: Vec<StreamChunk>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub done: bool,
}
