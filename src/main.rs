use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use dotenv::dotenv;
use futures_util::StreamExt;
use log::{debug, info};
use mysql_async::binlog::events::{QueryEvent, WriteRowsEvent};
use mysql_async::binlog::EventType;
use mysql_async::prelude::{Query, Queryable, WithParams};
use mysql_async::{BinlogStreamRequest, Opts};
use uuid::Uuid;

mod analyzers;

#[derive(Debug)]
struct BinLogRow {
    log_name: String,
    file_size: u64,
    encrypted: String,
}

/// explain result for target sql query, all analysis are based on the EXPLAIN execution from mysql,
///
/// REF: https://dev.to/amitiwary999/get-useful-information-from-mysql-explain-2i97
#[derive(Debug)]
struct ExplainResult {
    id: Uuid,

    /// raw query sql
    query: String,

    /// unique uuid for each txn, used for analysing queries within one transaction. mark it as optional cause some queries are not executed in a valid transaction
    txn_uuid: Option<Uuid>,

    // explain info
    explain_id: u64,
    select_type: String,
    table: String,
    partitions: Option<String>,
    _type: String,
    possible_keys: Option<String>,
    key: Option<String>,
    key_len: Option<u64>,
    _ref: Option<String>,
    rows: Option<u64>,
    filtered: Option<f64>,
    extra: Option<String>,

    // extra meta
    record_time: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::init();
    info!("Language security officer is launching...");
    let opts = Opts::from_url(&std::env::var("SQL_DSN").expect("SQL_DSN must be set"))?;
    let pool = mysql_async::Pool::new(opts);
    let mut conn = pool.get_conn().await?;
    let mut explain_conn = pool.get_conn().await?;
    let mut bin_logs = "SHOW BINARY LOGS"
        .with(())
        .map(&mut conn, |(log_name, file_size, encrypted)| BinLogRow {
            log_name,
            file_size,
            encrypted,
        })
        .await?;
    let option = bin_logs.pop().expect("cannot get latest bin log file");
    let mut binlog = conn
        .get_binlog_stream(BinlogStreamRequest::new(1).with_filename(option.log_name.as_bytes()).with_pos(option.file_size))
        .await?;

    let mut thread_txn: HashMap<u32, Uuid> = HashMap::default();
    while let Some(Ok(event)) = binlog.next().await {
        let eventtype = event.header().event_type()?;
        match eventtype {
            EventType::QUERY_EVENT => {
                let query_event: QueryEvent = event.read_event()?;
                let thread_id = query_event.thread_id();
                let query = query_event.query().trim().to_string();
                info!("[{}:{}] get event, `{}`", query_event.thread_id(), query_event.execution_time(), &query);

                match query.as_ref() {
                    "BEGIN" => {
                        // txn begin
                        thread_txn.insert(thread_id, Uuid::new_v4());
                    }
                    "COMMIT" => {
                        // commit txn
                        thread_txn.remove(&thread_id);
                    }
                    sql => {
                        let explain_sql = format!("EXPLAIN {}", &sql);

                        let current_ts = get_current_timestamp_millis();
                        let txn_uuid = thread_txn.get(&thread_id).cloned();

                        debug!("explaining sql: {}", &query);
                        let explain_result = explain_sql
                            .with(())
                            .map(
                                &mut explain_conn,
                                |(explain_id, select_type, table, partitions, _type, possible_keys, key, key_len, _ref, rows, filtered, extra)| ExplainResult {
                                    id: Uuid::new_v4(),
                                    query: sql.to_owned(),
                                    txn_uuid,
                                    explain_id,
                                    select_type,
                                    table,
                                    partitions,
                                    _type,
                                    possible_keys,
                                    key,
                                    key_len,
                                    _ref,
                                    rows,
                                    filtered,
                                    extra,

                                    record_time: current_ts,
                                },
                            )
                            .await?;

                        debug!("explain_result: {:?}", &explain_result);
                    }
                }
            }
            EventType::WRITE_ROWS_EVENT => {
                let detail_event: WriteRowsEvent = event.read_event()?;
                dbg!(detail_event);
            }
            // todo: handle bin log file switch event
            _ => {}
        }
    }
    Ok(())
}

fn get_current_timestamp_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_millis()
}
