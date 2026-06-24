use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::evidence::{Module, TestCase};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedEvidence {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_cases: Vec<TestCase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<Module>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_module_links: Vec<StagedTestModuleLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedTestModuleLink {
    pub id: String,
    pub test_ref: String,
    pub module_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub basis: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedModuleBundle {
    pub module_ref: String,
    pub tests: Vec<BundleTest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleTest {
    pub test_ref: String,
    pub link_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatorInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Artifact,
    TestCase,
    Module,
    TestModuleLink,
    Assertion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingSubject {
    pub kind: SubjectKind,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub entity_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    pub level: FindingLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<FindingSubject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingLayer {
    pub id: String,
    pub evaluator: EvaluatorInfo,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedEvidenceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub evidence: StagedEvidence,
}
