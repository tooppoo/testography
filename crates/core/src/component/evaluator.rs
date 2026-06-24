use tgraphy_types::FindingLayer;

use super::ComponentResult;

pub use tgraphy_types::EvaluatorInput;

pub trait Evaluator {
    fn evaluate(&self, input: EvaluatorInput) -> ComponentResult<FindingLayer>;
}
