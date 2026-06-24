use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use tgraphy_types::evidence::*;

use super::common::{Diagnostic, Producer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub producer: Producer,
    pub evidence: Evidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<Diagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_cases: Option<Vec<TestCase>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<Module>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_module_links: Option<Vec<TestModuleLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_bundles: Option<Vec<ModuleBundle>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkRelationship {
    DirectlyCalled,
    AssertionTarget,
    SetupDependency,
    FixtureDependency,
    FactoryDependency,
    MockedDependency,
    HelperDependency,
    UnresolvedCandidate,
    AmbiguousCandidate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestModuleLink {
    pub test_id: String,
    pub module_id: String,
    pub relationship: LinkRelationship,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleBundle {
    pub module_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_ref: Option<Module>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}
