use crate::artifact::{AssessmentLayer, EvidenceArtifact};

use super::ComponentResult;

pub struct EvaluatorInput {
    pub artifact: EvidenceArtifact,
    pub config: Option<serde_json::Value>,
}

pub trait Evaluator {
    fn evaluate(&self, input: EvaluatorInput) -> ComponentResult<AssessmentLayer>;
}
