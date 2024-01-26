use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use dotenv::dotenv;
use futures_util::StreamExt;
use log::{debug, info, warn};
use mysql_async::binlog::events::{QueryEvent, WriteRowsEvent};
use mysql_async::binlog::EventType;
use mysql_async::prelude::{Query, Queryable, WithParams};
use mysql_async::{BinlogStreamRequest, Opts as MysqlOpts};
use sqlx::sqlite::SqlitePool;
use sqlx::{Connection, Pool, Sqlite};
use uuid::Uuid;

use crate::analyzers::no_index_match_analyzer::NoIndexMatchAnalyzer;
use crate::analyzers::Analyzer;
use crate::cli::Opts;
use crate::domain::ExplainResult;
use clap::Parser;

mod analyzers;
mod domain;
mod routes;
mod templates;

mod cli;

#[derive(Debug)]
struct BinLogRow {
    log_name: String,
    file_size: u64,
    encrypted: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    pretty_env_logger::init();
    info!("Language security officer is launching...");

    let opts = Opts::parse();

    //todo: cli flag to enable persistent storage and sql query histories
    let sqlite_pool = if let Some(db_path) = opts.db {
        info!("using file {} as persistent sqlite storage", db_path.display());
        Arc::new(SqlitePool::connect(&format!("sqlite://{}", db_path.display())).await?)
    } else {
        warn!("using memory as sqlite storage, will erase all data when restart");
        Arc::new(SqlitePool::connect("sqlite::memory:").await?)
    };

    info!("running sqlite migrations...");
    sqlx::migrate!("./migrations").run(&*sqlite_pool).await?;

    tokio::spawn(mysql_bin_log_listener(sqlite_pool.clone()));

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(routes::common::index))
        .route("/txn/:txn_uuid", get(routes::common::txn_detail))
        .with_state(sqlite_pool.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    info!("starting web server...");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn mysql_bin_log_listener(sqlite_pool: Arc<Pool<Sqlite>>) -> Result<()> {
    let opts = MysqlOpts::from_url(&std::env::var("SQL_DSN").expect("SQL_DSN must be set"))?;
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
    info!("starting listening bin log events...");
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
                        let mut explain_result = explain_sql
                            .with(())
                            .map(
                                &mut explain_conn,
                                |(explain_id, select_type, table, partitions, _type, possible_keys, key, key_len, _ref, rows, filtered, extra)| ExplainResult {
                                    id: Uuid::new_v4().to_string(),
                                    query: sql.to_owned(),
                                    txn_uuid: txn_uuid.map(|it| it.to_string()),
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
                        let explain_result = explain_result.pop().expect("cannot get explain result");

                        debug!("explain_result: {:?}", &explain_result);

                        sqlx::query!(
                            r#"
                            INSERT INTO explains (id, query, txn_uuid, explain_id, select_type, "table", partitions, _type, possible_keys, key,
                                                  key_len, _ref, rows, filtered, extra, record_time)
                            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
                            "#,
                            explain_result.id,
                            explain_result.query,
                            explain_result.txn_uuid,
                            explain_result.explain_id,
                            explain_result.select_type,
                            explain_result.table,
                            explain_result.partitions,
                            explain_result._type,
                            explain_result.possible_keys,
                            explain_result.key,
                            explain_result.key_len,
                            explain_result._ref,
                            explain_result.rows,
                            explain_result.filtered,
                            explain_result.explain_id,
                            explain_result.record_time
                        )
                        .execute(&*sqlite_pool)
                        .await?;
                        debug!("save explain result [id:{}]", &explain_result.id);

                        debug!("dispatch analyzers for explain result [id:{}]", &explain_result.id);
                        NoIndexMatchAnalyzer {}.analyse(&explain_result);
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

/// get timestamp millis and format it as i64
/// function `as_millis` return u128 as data format,
/// and we store it into i64,
/// the max number of i64 is `9223372036854775807`, should be safe for a while
fn get_current_timestamp_millis() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_millis() as i64
}
