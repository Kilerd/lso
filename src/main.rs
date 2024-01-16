use anyhow::Result;
use dotenv::dotenv;
use futures_util::StreamExt;
use mysql_async::binlog::events::{QueryEvent, WriteRowsEvent};
use mysql_async::binlog::EventType;
use mysql_async::prelude::{Query, WithParams};
use mysql_async::{BinlogStreamRequest, Opts};

#[derive(Debug)]
struct BinLogRow {
    log_name: String,
    file_size: u64,
    encrypted: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    println!("Hello, world!");
    let opts = Opts::from_url(&std::env::var("SQL_DSN").expect("SQL_DSN must be set"))?;
    let pool = mysql_async::Pool::new(opts);
    let mut conn = pool.get_conn().await?;
    let mut bin_logs = "SHOW BINARY LOGS"
        .with(())
        .map(&mut conn, |(log_name, file_size, encrypted)| BinLogRow {
            log_name,
            file_size,
            encrypted,
        })
        .await?;
    dbg!(&bin_logs);
    let option = bin_logs.pop().expect("cannot get latest bin log file");
    let mut binlog = conn
        .get_binlog_stream(BinlogStreamRequest::new(1).with_filename(option.log_name.as_bytes()).with_pos(option.file_size))
        .await?;

    while let Some(Ok(event)) = binlog.next().await {
        let eventtype = event.header().event_type()?;
        match eventtype {
            EventType::QUERY_EVENT => {
                let query_event: QueryEvent = event.read_event()?;

                println!("get event, {}", query_event.query());
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
