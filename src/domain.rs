use sqlx::FromRow;
use uuid::Uuid;

/// explain result for target sql query, all analysis are based on the EXPLAIN execution from mysql,
///
/// REF: https://dev.to/amitiwary999/get-useful-information-from-mysql-explain-2i97
#[derive(Debug, FromRow)]
pub struct ExplainResult {
    pub id: String,

    /// raw query sql
    pub query: String,

    /// unique uuid for each txn, used for analysing queries within one transaction. mark it as optional cause some queries are not executed in a valid transaction
    pub txn_uuid: Option<String>,

    // explain info
    pub explain_id: i64,
    pub select_type: String,
    pub table: String,
    pub partitions: Option<String>,
    pub _type: String,
    pub possible_keys: Option<String>,
    pub key: Option<String>,
    pub key_len: Option<i64>,
    pub _ref: Option<String>,
    pub rows: Option<i64>,
    pub filtered: Option<f64>,
    pub extra: Option<String>,

    // extra meta
    pub record_time: i64,
}

/// analyse result
#[derive(Debug, FromRow)]
pub struct AnalyseResult {
    pub id: i32,

    pub explain_id: String,

    pub name: String,
    pub pass: bool,
    pub msg: Option<String>,
}
