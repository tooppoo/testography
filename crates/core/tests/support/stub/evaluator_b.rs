use tgraphy_core::artifact::staged::{
    Evaluator as EvaluatorInfo, Finding, FindingLayer, FindingLevel,
};
use tgraphy_core::component::{ComponentResult, Evaluator, EvaluatorInput};

pub struct StubEvaluatorB;

impl Evaluator for StubEvaluatorB {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        Ok(FindingLayer {
            id: "stub-layer-b".to_string(),
            evaluator: EvaluatorInfo {
                id: "stub-evaluator-b".to_string(),
                version: Some("0.0.0".to_string()),
            },
            findings: vec![Finding {
                id: "stub-finding-b-001".to_string(),
                level: FindingLevel::Info,
                message: "stub finding from evaluator b".to_string(),
                subjects: vec![],
                rationale: None,
            }],
            summary: None,
        })
    }
}
