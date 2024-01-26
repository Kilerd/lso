use std::sync::Arc;

use crate::analyzers::{AbstractAnalyzeResult, Analyzer};
use crate::ExplainResult;
use anyhow::Result;
use log::debug;
use sqlx::SqlitePool;

pub struct NoIndexMatchAnalyzer;

pub struct NoIndexMatchResult {
    pass: bool,
    explain_id: String,
    matched_key: Option<String>,
}

impl Analyzer for NoIndexMatchAnalyzer {
    type Output = NoIndexMatchResult;

    fn analyse(&self, data: Arc<ExplainResult>) -> Result<Self::Output> {
        let pass = if data.select_type.eq("UPDATE") || data.select_type.eq("DELETE") {
            data.key.is_some()
        } else {
            true
        };

        Ok(NoIndexMatchResult {
            pass,
            explain_id: data.id.to_owned(),
            matched_key: data.key.clone(),
        })
    }

    async fn store(&self, analysis_result: Self::Output, pool: Arc<SqlitePool>) -> Result<()> {
        let name = analysis_result.name();
        let msg = analysis_result.msg();
        let pass = analysis_result.pass();
        debug!("insert no index match analyzer result");
        sqlx::query!(
            r#"
            INSERT INTO  "analyzer_results" ("explain_id", "name", "pass", "msg") VALUES (?, ?, ?, ?);
            "#,
            analysis_result.explain_id,
            name,
            pass,
            msg,
        )
        .execute(&*pool)
        .await?;
        Ok(())
    }
}

impl AbstractAnalyzeResult for NoIndexMatchResult {
    fn name(&self) -> String {
        "No Index Match Analyzer".to_owned()
    }
    fn pass(&self) -> bool {
        self.pass
    }
    fn msg(&self) -> Option<String> {
        if self.pass() {
            None
        } else {
            Some("explain result shows that no index is matched".to_owned())
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{analyzers::Analyzer, domain::ExplainResult};

    use super::AbstractAnalyzeResult;
    use super::NoIndexMatchAnalyzer;

    #[test]
    fn should_alway_return_pass_for_insert_sql() {
        let result = NoIndexMatchAnalyzer {}
            .analyse(Arc::new(ExplainResult {
                id: "UUID".to_owned(),
                query: "query".to_owned(),
                txn_uuid: None,
                explain_id: 1,
                select_type: "INSERT".to_owned(),
                table: "test".to_owned(),
                partitions: None,
                _type: "None".to_owned(),
                possible_keys: None,
                key: None,
                key_len: None,
                _ref: None,
                rows: None,
                filtered: None,
                extra: None,
                record_time: 0,
            }))
            .unwrap();

        assert!(result.pass());
    }
}
