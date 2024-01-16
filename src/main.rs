use anyhow::Result;
use dotenv::dotenv;
use futures_util::StreamExt;
use log::info;
use mysql_async::binlog::events::{QueryEvent, WriteRowsEvent};
use mysql_async::binlog::EventType;
use mysql_async::prelude::{Query, Queryable, WithParams};
use mysql_async::{BinlogStreamRequest, Opts};

#[derive(Debug)]
struct BinLogRow {
    log_name: String,
    file_size: u64,
    encrypted: String,
}

#[derive(Debug)]
struct ExplainResult {
    id: u64,
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

    while let Some(Ok(event)) = binlog.next().await {
        let eventtype = event.header().event_type()?;
        match eventtype {
            EventType::QUERY_EVENT => {
                let query_event: QueryEvent = event.read_event()?;

                let query = query_event.query().trim().to_string();
                info!("[{}:{}] get event, `{}`", query_event.thread_id(), query_event.execution_time(), &query);
                if query.ne("BEGIN") && query.ne("COMMIT") {
                    let explain_sql = format!("EXPLAIN {}", &query);
                    info!("explaining sql: {}", &query);
                    let explain_result = explain_sql
                        .with(())
                        .map(
                            &mut explain_conn,
                            |(id, select_type, table, partitions, _type, possible_keys, key, key_len, _ref, rows, filtered, extra)| ExplainResult {
                                id,
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
                            },
                        )
                        .await?;
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
