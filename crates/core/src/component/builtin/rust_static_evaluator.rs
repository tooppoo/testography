use crate::artifact::staged::{
    Evaluator as EvaluatorInfo, Finding, FindingLayer, FindingLevel, FindingSubject, SubjectKind,
};
use crate::component::ComponentResult;
use crate::component::evaluator::{Evaluator, EvaluatorInput};

pub struct RustStaticEvaluator;

const EVALUATOR_ID: &str = "rust-static";
const RULE_PREDICATE_ONLY: &str = "rust.assert.predicate_only_assertion";

impl Evaluator for RustStaticEvaluator {
    fn evaluate(&self, input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        let mut findings = Vec::new();

        for test_case in &input.evidence.test_cases {
            let Some(assertions) = &test_case.assertions else {
                continue;
            };

            for assertion in assertions {
                let is_plain_assert =
                    assertion.matcher.as_ref().and_then(|m| m.name.as_deref()) == Some("assert");

                if is_plain_assert {
                    findings.push(Finding {
                        id: format!("{}:{}", RULE_PREDICATE_ONLY, assertion.id),
                        rule_id: Some(RULE_PREDICATE_ONLY.to_string()),
                        level: FindingLevel::Info,
                        confidence: Some("high".to_string()),
                        message: "assert!(...) is a predicate-only assertion. \
                            Unlike assert_eq! or assert_ne!, it does not record a structured \
                            expected value as evidence. The assertion may still be correct; \
                            this finding records that no explicit expected value is captured."
                            .to_string(),
                        subjects: vec![FindingSubject {
                            kind: SubjectKind::Assertion,
                            entity_ref: Some(assertion.id.clone()),
                            path: None,
                        }],
                        rationale: Some(
                            "An assert!(...) assertion does not separate the actual and expected \
                            values as structured evidence. Using assert_eq! or assert_ne! captures \
                            both values and improves diagnostic output when the assertion fails."
                                .to_string(),
                        ),
                    });
                }
            }
        }

        Ok(FindingLayer {
            id: "rust-static-layer".to_string(),
            evaluator: EvaluatorInfo {
                id: EVALUATOR_ID.to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            },
            findings,
            summary: None,
        })
    }
}
