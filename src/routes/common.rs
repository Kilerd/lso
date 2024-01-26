use crate::domain::ExplainResult;
use crate::templates::{HtmlTemplate, IndexTemplate, TxnDetailTemplate};
use axum::extract::{Path, State};
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

pub(crate) async fn txn_detail(Path(param): Path<(String,)>, State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    let explains = sqlx::query_as!(ExplainResult, r"SELECT * from explains where txn_uuid = ? order by id desc", param.0)
        .fetch_all(&*pool)
        .await
        .expect("cannot read records");
    HtmlTemplate(TxnDetailTemplate { uuid: param.0, explains })
}
