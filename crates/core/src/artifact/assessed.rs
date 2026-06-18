use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::common::{Diagnostic, Producer};
use super::evidence::Evidence;
use super::layer::AssessmentLayer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssessedArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub producer: Producer,
    pub evidence: Evidence,
    pub assessment_layers: Vec<AssessmentLayer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<Diagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}
