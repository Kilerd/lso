use crate::domain::{AnalyseResult, ExplainResult};

#[derive(Debug)]
pub struct ExplainResultWithAnalysis {
    pub explain: ExplainResult,
    pub analysis: Vec<AnalyseResult>,
}
