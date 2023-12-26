use std::collections::HashMap;

use iso8601_timestamp::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct IndexAccess {
    ops: i32,
    since: Timestamp,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Index {
    name: String,
    access: IndexAccess,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct LatencyHistogramEntry {
    micros: i64,
    count: i64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct LatencyStats {
    ops: i64,
    latency: i64,
    histogram: Vec<LatencyHistogramEntry>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StorageStats {
    size: i64,
    storage_size: i64,
    total_index_size: i64,
    total_size: i64,
    index_sizes: HashMap<String, i64>,
    count: i64,
    avg_obj_size: i64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionScans {
    total: i64,
    non_tailable: i64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryExecStats {
    /// Stats regarding collection scans
    collection_scans: CollectionScans,
}

/// Collection stats
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStats {
    ns: String,
    local_time: Timestamp,
    latency_stats: HashMap<String, LatencyStats>,
    query_exec_stats: QueryExecStats,
    count: u64,
}

#[derive(Serialize, JsonSchema, Debug)]
pub struct Stats {
    pub indices: HashMap<String, Vec<Index>>,
    pub coll_stats: HashMap<String, CollectionStats>,
}
