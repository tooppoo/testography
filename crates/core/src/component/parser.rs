use std::path::PathBuf;

use crate::artifact::ParsedEvidenceArtifact;

use super::ComponentResult;

pub struct ParserInput {
    pub source_paths: Vec<PathBuf>,
    pub config: Option<serde_json::Value>,
}

pub trait Parser {
    fn parse(&self, input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact>;
}
