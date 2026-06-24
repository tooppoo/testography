use serde::{Deserialize, Serialize};

use crate::artifact::staged::{FindingLayer, StagedEvidence, StagedModuleBundle};

use super::ComponentResult;

#[derive(Serialize, Deserialize)]
pub struct EvaluatorInput {
    pub evidence: StagedEvidence,
    pub module_bundles: Vec<StagedModuleBundle>,
    pub assessment_layers: Vec<FindingLayer>,
    pub config: Option<serde_json::Value>,
}

pub trait Evaluator {
    fn evaluate(&self, input: EvaluatorInput) -> ComponentResult<FindingLayer>;
}
