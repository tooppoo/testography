use tgraphy_core::artifact::staged::{
    Evaluator as EvaluatorInfo, Finding, FindingLayer, FindingLevel,
};
use tgraphy_core::component::{ComponentResult, Evaluator, EvaluatorInput};

pub struct StubEvaluatorA;

impl Evaluator for StubEvaluatorA {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        Ok(FindingLayer {
            id: "stub-layer-a".to_string(),
            evaluator: EvaluatorInfo {
                id: "stub-evaluator-a".to_string(),
                version: Some("0.0.0".to_string()),
            },
            findings: vec![Finding {
                id: "stub-finding-a-001".to_string(),
                rule_id: None,
                level: FindingLevel::Info,
                confidence: None,
                message: "stub finding from evaluator a".to_string(),
                subjects: vec![],
                rationale: None,
            }],
            summary: None,
        })
    }
}
