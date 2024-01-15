use mysql_async::{BinlogStreamRequest, Opts};
use anyhow::Result;
use dotenv::dotenv;
use futures_util::StreamExt;
use mysql_async::binlog::events::{QueryEvent, WriteRowsEvent};
use mysql_async::binlog::EventType;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    println!("Hello, world!");
    let opts = Opts::from_url(&std::env::var("SQL_DSN").expect("SQL_DSN must be set"))?;
    let pool = mysql_async::Pool::new(opts);
    let mut conn = pool.get_conn().await?;
    let mut binlog = conn.get_binlog_stream(BinlogStreamRequest::new(1)).await?;

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
            _ => {}
        }
    }
    Ok(())
}
