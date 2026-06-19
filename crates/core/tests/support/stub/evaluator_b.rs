use tgraphy_core::artifact::layer::LayerProducerKind;
use tgraphy_core::artifact::layer::{Assessment, AssessmentKind, AssessmentLayer, LayerProducer};
use tgraphy_core::component::{ComponentResult, Evaluator, EvaluatorInput};
use tgraphy_core::validation::ACCEPTED_SCHEMA_VERSION;

pub struct StubEvaluatorB;

impl Evaluator for StubEvaluatorB {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<AssessmentLayer> {
        Ok(AssessmentLayer {
            schema_version: ACCEPTED_SCHEMA_VERSION.to_string(),
            id: "stub-layer-b".to_string(),
            producer: LayerProducer {
                name: "stub-evaluator-b".to_string(),
                version: "0.0.0".to_string(),
                kind: LayerProducerKind::Static,
                extra: Default::default(),
            },
            assessments: vec![Assessment {
                id: "stub-assessment-b-001".to_string(),
                kind: AssessmentKind::DiagnosticNote,
                statement: "stub assessment from evaluator b".to_string(),
                rule_id: None,
                category: None,
                status: None,
                severity: None,
                confidence: None,
                evidence_refs: None,
                assessment_refs: None,
                data: None,
                extensions: None,
            }],
            diagnostics: None,
            extensions: None,
        })
    }
}
