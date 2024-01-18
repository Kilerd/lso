use crate::analyzers::{Analyzer, Passable};
use crate::ExplainResult;
use anyhow::Result;

pub struct NoIndexMatchAnalyzer;

pub struct NoIndexMatchResult {
    pass: bool,
    matched_key: Option<String>,
}

impl Analyzer for NoIndexMatchAnalyzer {
    type Output = NoIndexMatchResult;

    fn analyse(&self, data: ExplainResult) -> Result<Self::Output> {
        let pass = data.key.is_some();

        Ok(NoIndexMatchResult {
            pass,
            matched_key: data.key.clone(),
        })
    }

    fn store(&self, analysis_result: Self::Output) -> Result<()> {
        todo!()
    }
}

impl Passable for NoIndexMatchResult {
    fn pass(&self) -> bool {
        self.pass
    }
}
