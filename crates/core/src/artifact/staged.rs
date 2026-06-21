use serde::{Deserialize, Serialize};

use super::evidence::{Module, TestCase};

/// Parser-produced primary evidence, shared across all three staged artifact types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedEvidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_cases: Option<Vec<TestCase>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<Module>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_module_links: Option<Vec<StagedTestModuleLink>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
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
    pub level: FindingLevel,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subjects: Option<Vec<FindingSubject>>,
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
