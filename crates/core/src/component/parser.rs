use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::artifact::ParsedEvidenceArtifact;

use super::ComponentResult;

#[derive(Serialize, Deserialize)]
pub struct ParserInput {
    pub source_paths: Vec<PathBuf>,
    pub config: Option<serde_json::Value>,
}

pub trait Parser {
    fn parse(&self, input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact>;
}
