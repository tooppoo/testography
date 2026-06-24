use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::evidence::{Module, TestCase};

/// Parser-produced primary evidence, shared across all three staged artifact types.
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

/// A test-to-module link entry with a stable `id` for cross-referencing.
///
/// Replaces the old `TestModuleLink` field names (`test_id`, `module_id`) with
/// `test_ref` / `module_ref` and adds the required `id` field so that
/// `module_bundles[].tests[].link_ref` can resolve entries unambiguously.
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

/// Transform-produced module-centered view of a set of test-module links.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedModuleBundle {
    pub module_ref: String,
    pub tests: Vec<BundleTest>,
}

/// A single test entry inside a module bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleTest {
    pub test_ref: String,
    pub link_ref: String,
}

/// Producer identity recorded in artifact lineage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageProducer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Provenance chain for a staged artifact, recorded as a tagged variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Lineage {
    /// This artifact was derived from another artifact stage.
    DerivedFrom {
        input_artifact_type: String,
        input_producer: LineageProducer,
    },
    /// This artifact is the origin (produced directly from a source, e.g. a parser).
    Origin { source: String },
}

/// Evaluator identity used in a `FindingLayer`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evaluator {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Severity of a single evaluator finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingLevel {
    Info,
    Warning,
    Error,
}

/// Subject kind for a `FindingSubject`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Artifact,
    TestCase,
    Module,
    TestModuleLink,
    Assertion,
}

/// A reference to a specific entity within the artifact that a finding concerns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingSubject {
    pub kind: SubjectKind,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub entity_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A single evaluator finding within a `FindingLayer`.
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

/// One evaluator's output — a named layer of findings over the artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingLayer {
    pub id: String,
    pub evaluator: Evaluator,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

// ── The three staged artifact types ─────────────────────────────────────────

/// Parser output: primary evidence only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedEvidenceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub evidence: StagedEvidence,
}

/// Module-bundle transform output: parser evidence + module-centered derived view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEvidenceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub evidence: StagedEvidence,
    pub module_bundles: Vec<StagedModuleBundle>,
    pub lineage: Lineage,
}

/// Evaluator output / reporter input: evidence + module bundles + assessment findings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssessedModuleEvidenceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub evidence: StagedEvidence,
    pub module_bundles: Vec<StagedModuleBundle>,
    pub assessment_layers: Vec<FindingLayer>,
}
