use std::sync::Arc;

use crate::ExplainResult;
use anyhow::Result;
use sqlx::SqlitePool;

pub mod no_index_match_analyzer;

pub trait Analyzer {
    /// the output struct coming from analyzer
    type Output: AbstractAnalyzeResult;

    async fn process(&self, data: Arc<ExplainResult>, pool: Arc<SqlitePool>) -> Result<()> {
        let result = self.analyse(data)?;
        self.store(result, pool).await?;
        Ok(())
    }

    /// how the analyzer analysis the explain result coming from mysql, and emit its own analysis result
    fn analyse(&self, data: Arc<ExplainResult>) -> Result<Self::Output>;

    /// for the further analysis and presentation, we need to define how the analysis result store into self hosted database
    async fn store(&self, analysis_result: Self::Output, pool: Arc<SqlitePool>) -> Result<()>;
}

pub trait AbstractAnalyzeResult {
    fn name(&self) -> String;
    fn pass(&self) -> bool;
    fn msg(&self) -> Option<String>;
}
