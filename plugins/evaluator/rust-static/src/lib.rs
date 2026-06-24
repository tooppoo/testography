use tgraphy_types::{EvaluatorInfo, Finding, FindingSubject};

pub use tgraphy_types::{EvaluatorInput, FindingLayer, FindingLevel, SubjectKind};

const EVALUATOR_ID: &str = "rust-static";
const EVALUATOR_VERSION: &str = env!("CARGO_PKG_VERSION");
const RULE_PREDICATE_ONLY: &str = "rust.assert.predicate_only_assertion";

pub fn evaluate(input: EvaluatorInput) -> FindingLayer {
    let mut findings = Vec::new();

    for test_case in &input.evidence.test_cases {
        for assertion in test_case.assertions.as_deref().unwrap_or(&[]) {
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

    FindingLayer {
        id: "rust-static-layer".to_string(),
        evaluator: EvaluatorInfo {
            id: EVALUATOR_ID.to_string(),
            version: Some(EVALUATOR_VERSION.to_string()),
        },
        findings,
        summary: None,
    }
}
