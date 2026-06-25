use serde::{Deserialize, Serialize};

pub use tgraphy_types::EvaluatorInfo as Evaluator;
pub use tgraphy_types::staged::*;

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
    DerivedFrom {
        input_artifact_type: String,
        input_producer: LineageProducer,
    },
    Origin {
        source: String,
    },
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
