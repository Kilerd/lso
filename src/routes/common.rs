use crate::domain::ExplainResult;
use crate::templates::{HtmlTemplate, IndexTemplate};
use axum::extract::State;
use axum::response::IntoResponse;
use sqlx::SqlitePool;
use std::sync::Arc;

pub(crate) async fn index(State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    let explains = sqlx::query_as!(ExplainResult, r"SELECT * from explains order by id desc")
        .fetch_all(&*pool)
        .await
        .expect("cannot read records");
    HtmlTemplate(IndexTemplate { explains })
}
