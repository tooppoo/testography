use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::staged::{
    AssessedModuleEvidenceArtifact, FindingLayer, StagedEvidence, StagedModuleBundle,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParserInput {
    pub source_paths: Vec<PathBuf>,
    pub config: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct EvaluatorInput {
    pub evidence: StagedEvidence,
    #[serde(default)]
    pub module_bundles: Vec<StagedModuleBundle>,
    #[serde(default)]
    pub assessment_layers: Vec<FindingLayer>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterInput {
    pub artifact: AssessedModuleEvidenceArtifact,
    pub config: Option<serde_json::Value>,
}

/// Output envelope returned by process-based reporter binaries via stdout.
///
/// `extension` must be a non-empty lowercase alphanumeric string (e.g. "md", "json").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterOutput {
    pub extension: String,
    pub content: String,
}
