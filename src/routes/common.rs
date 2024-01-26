use crate::domain::{AnalyseResult, ExplainResult};
use crate::templates::{HtmlTemplate, IndexTemplate, JournalTemplate, TxnDetailTemplate};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use itertools::Itertools;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::response::ExplainResultWithAnalysis;

pub(crate) async fn index(State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    let analyse_results = sqlx::query_as!(AnalyseResult, r"select * from analyzer_results where pass = false;")
        .fetch_all(&*pool)
        .await
        .expect("cannot read analyse result");
    let mut analysis_group: HashMap<String, Vec<AnalyseResult>> = HashMap::default();
    for (key, group) in &analyse_results
        .into_iter()
        .sorted_by_key(|it| it.explain_id.to_owned())
        .group_by(|it| it.explain_id.to_owned())
    {
        analysis_group.insert(key, group.collect_vec());
    }

    let explains = sqlx::query_as!(
        ExplainResult,
        r"SELECT * from explains where id in (select DISTINCT explain_id from analyzer_results where pass = false) order by record_time desc"
    )
    .fetch_all(&*pool)
    .await
    .expect("cannot read records");
    let explains = explains
        .into_iter()
        .map(|explain| ExplainResultWithAnalysis {
            analysis: analysis_group.remove(&explain.id).unwrap_or_default(),
            explain: explain,
        })
        .collect_vec();
    HtmlTemplate(IndexTemplate { explains })
}

pub(crate) async fn journals(State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    let explains = sqlx::query_as!(ExplainResult, r"SELECT * from explains order by record_time desc")
        .fetch_all(&*pool)
        .await
        .expect("cannot read records");
    HtmlTemplate(JournalTemplate { explains })
}

pub(crate) async fn txn_detail(Path(param): Path<(String,)>, State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    let explains = sqlx::query_as!(ExplainResult, r"SELECT * from explains where txn_uuid = ? order by record_time desc", param.0)
        .fetch_all(&*pool)
        .await
        .expect("cannot read records");
    HtmlTemplate(TxnDetailTemplate { uuid: param.0, explains })
}
