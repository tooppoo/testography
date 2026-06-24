use serde::{Deserialize, Serialize};

const EVALUATOR_ID: &str = "rust-static";
const EVALUATOR_VERSION: &str = env!("CARGO_PKG_VERSION");
const RULE_PREDICATE_ONLY: &str = "rust.assert.predicate_only_assertion";

// ── Input types (deserialized from stdin) ────────────────────────────────────

#[derive(Deserialize)]
pub struct EvaluatorInput {
    pub evidence: StagedEvidence,
    #[serde(default)]
    pub module_bundles: Vec<serde_json::Value>,
    #[serde(default)]
    pub assessment_layers: Vec<serde_json::Value>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct StagedEvidence {
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
}

#[derive(Deserialize)]
pub struct TestCase {
    pub id: String,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
}

#[derive(Deserialize)]
pub struct Assertion {
    pub id: String,
    pub matcher: Option<Matcher>,
}

#[derive(Deserialize)]
pub struct Matcher {
    pub name: Option<String>,
}

// ── Output types (serialized to stdout) ──────────────────────────────────────

#[derive(Serialize)]
pub struct FindingLayer {
    pub id: String,
    pub evaluator: EvaluatorInfo,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Serialize)]
pub struct EvaluatorInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Serialize)]
pub struct Finding {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    pub level: FindingLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<FindingSubject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingLevel {
    Info,
    #[allow(dead_code)]
    Warning,
    #[allow(dead_code)]
    Error,
}

#[derive(Serialize)]
pub struct FindingSubject {
    pub kind: SubjectKind,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub entity_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Assertion,
}

// ── Evaluation logic ──────────────────────────────────────────────────────────

pub fn evaluate(input: EvaluatorInput) -> FindingLayer {
    let mut findings = Vec::new();

    for test_case in &input.evidence.test_cases {
        for assertion in &test_case.assertions {
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
