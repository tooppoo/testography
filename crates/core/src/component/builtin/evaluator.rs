use crate::artifact::staged::{Evaluator as EvaluatorInfo, FindingLayer};
use crate::component::ComponentResult;
use crate::component::evaluator::{Evaluator, EvaluatorInput};

pub struct BuiltinEvaluator;

impl Evaluator for BuiltinEvaluator {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        Ok(FindingLayer {
            id: "builtin-layer".to_string(),
            evaluator: EvaluatorInfo {
                id: "builtin".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            },
            findings: vec![],
            summary: None,
        })
    }
}
