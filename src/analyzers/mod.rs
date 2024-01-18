use crate::ExplainResult;
use anyhow::Result;

pub mod no_index_match_analyzer;

pub trait Analyzer {
    /// the output struct coming from analyzer
    type Output: Passable;

    /// how the analyzer analysis the explain result coming from mysql, and emit its own analysis result
    fn analyse(&self, data: ExplainResult) -> Result<Self::Output>;

    /// for the further analysis and presentation, we need to define how the analysis result store into self hosted database
    fn store(&self, analysis_result: Self::Output) -> Result<()>;
}

pub trait Passable {
    fn pass(&self) -> bool;
}
