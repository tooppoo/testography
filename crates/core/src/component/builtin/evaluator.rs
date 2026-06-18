use crate::artifact::layer::{AssessmentLayer, LayerProducer, LayerProducerKind};
use crate::component::evaluator::{Evaluator, EvaluatorInput};
use crate::component::ComponentResult;
use crate::validation::ACCEPTED_SCHEMA_VERSION;

pub struct BuiltinEvaluator;

impl Evaluator for BuiltinEvaluator {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<AssessmentLayer> {
        Ok(AssessmentLayer {
            schema_version: ACCEPTED_SCHEMA_VERSION.to_string(),
            id: "builtin-layer".to_string(),
            producer: LayerProducer {
                name: "builtin".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                kind: LayerProducerKind::Generic,
                extra: Default::default(),
            },
            assessments: vec![],
            diagnostics: None,
            extensions: None,
        })
    }
}
