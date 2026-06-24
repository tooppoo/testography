use tgraphy_core::component::builtin::RustStaticEvaluator;
use tgraphy_core::component::evaluator::{Evaluator, EvaluatorInput};
use tgraphy_core::{ArtifactKind, parse_artifact};

// ── helpers ───────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust")
        .join(name)
}

fn evaluator_input_from_module_evidence(path: &std::path::Path) -> EvaluatorInput {
    let json = std::fs::read_to_string(path).expect("fixture should be readable");
    let artifact = parse_artifact(&json).expect("fixture should be valid");
    let ArtifactKind::ModuleEvidence(m) = artifact else {
        panic!("expected ModuleEvidence fixture");
    };
    EvaluatorInput {
        evidence: m.evidence,
        module_bundles: m.module_bundles,
        assessment_layers: vec![],
        config: None,
    }
}

fn evaluator_input_from_assessed(path: &std::path::Path) -> EvaluatorInput {
    let json = std::fs::read_to_string(path).expect("fixture should be readable");
    let artifact = parse_artifact(&json).expect("fixture should be valid");
    let ArtifactKind::AssessedModuleEvidence(a) = artifact else {
        panic!("expected AssessedModuleEvidence fixture");
    };
    EvaluatorInput {
        evidence: a.evidence,
        module_bundles: a.module_bundles,
        assessment_layers: a.assessment_layers,
        config: None,
    }
}

// ── match cases ───────────────────────────────────────────────────────────────

#[test]
fn assert_predicate_only_produces_finding() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    assert_eq!(layer.findings.len(), 1);
    let finding = &layer.findings[0];
    assert_eq!(
        finding.rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
    assert_eq!(
        finding.subjects.len(),
        1,
        "finding should reference the assertion"
    );
    assert_eq!(
        finding.subjects[0].entity_ref.as_deref(),
        Some("rust-assertion-0001")
    );
}

#[test]
fn assert_with_diagnostic_message_produces_finding() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_with_message.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    assert_eq!(
        layer.findings.len(),
        1,
        "assert!(predicate, \"message\") should still produce a finding"
    );
    let finding = &layer.findings[0];
    assert_eq!(
        finding.rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
}

#[test]
fn assert_with_comparison_expression_produces_finding() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_with_comparison.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    assert_eq!(
        layer.findings.len(),
        1,
        "assert!(actual == expected) should produce a finding in v0 (no predicate classification)"
    );
    let finding = &layer.findings[0];
    assert_eq!(
        finding.rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
}

// ── non-match cases ───────────────────────────────────────────────────────────

#[test]
fn assert_eq_does_not_produce_finding() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_eq.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    assert_eq!(
        layer.findings.len(),
        0,
        "assert_eq! should not produce a predicate-only finding"
    );
}

#[test]
fn assert_ne_does_not_produce_finding() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_ne.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    assert_eq!(
        layer.findings.len(),
        0,
        "assert_ne! should not produce a predicate-only finding"
    );
}

// ── finding structure ─────────────────────────────────────────────────────────

#[test]
fn finding_has_stable_rule_id() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let finding = &layer.findings[0];
    assert_eq!(
        finding.rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
}

#[test]
fn finding_has_stable_id_derived_from_assertion_id() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let finding = &layer.findings[0];
    assert!(
        finding.id.contains("rust-assertion-0001"),
        "finding id should reference the assertion id, got: {}",
        finding.id
    );
}

#[test]
fn finding_has_confidence() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let finding = &layer.findings[0];
    assert!(
        finding.confidence.is_some(),
        "finding should have a confidence value"
    );
}

#[test]
fn finding_message_does_not_claim_assert_is_always_invalid() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let message = &layer.findings[0].message;
    let lower = message.to_lowercase();
    assert!(
        !lower.contains("invalid") && !lower.contains("incorrect") && !lower.contains("wrong"),
        "message should not claim the assertion is invalid/incorrect/wrong: {message}"
    );
}

#[test]
fn finding_subject_kind_is_assertion() {
    use tgraphy_core::artifact::staged::SubjectKind;

    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let subject = &layer.findings[0].subjects[0];
    assert_eq!(
        subject.kind,
        SubjectKind::Assertion,
        "subject kind should be assertion"
    );
}

// ── schema validity ───────────────────────────────────────────────────────────

#[test]
fn output_is_schema_valid_assessed_module_evidence() {
    use tgraphy_core::artifact::staged::AssessedModuleEvidenceArtifact;

    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let original_evidence = input.evidence.clone();
    let original_bundles = input.module_bundles.clone();

    let layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let assessed = AssessedModuleEvidenceArtifact {
        schema_version: "0.0.1".to_string(),
        artifact_type: "assessed_module_evidence".to_string(),
        evidence: original_evidence,
        module_bundles: original_bundles,
        assessment_layers: vec![layer],
    };

    let json = serde_json::to_string(&assessed).expect("serialize should succeed");
    let result = parse_artifact(&json);
    assert!(
        result.is_ok(),
        "evaluator output should be schema-valid assessed_module_evidence: {result:?}"
    );
}

// ── pipeline behavior ─────────────────────────────────────────────────────────

#[test]
fn input_evidence_is_unchanged_after_evaluate() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let evidence_before = input.evidence.clone();

    RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    // Re-load to confirm the input fixture itself was not mutated.
    let input2 = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    assert_eq!(
        evidence_before, input2.evidence,
        "evaluate should not mutate the input evidence"
    );
}

#[test]
fn input_module_bundles_are_unchanged_after_evaluate() {
    let input = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    let bundles_before = input.module_bundles.clone();

    RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    let input2 = evaluator_input_from_module_evidence(&fixture("assert_predicate_only.json"));
    assert_eq!(
        bundles_before, input2.module_bundles,
        "evaluate should not mutate the input module_bundles"
    );
}

#[test]
fn chaining_preserves_existing_layers_and_appends_one_new_layer() {
    let input = evaluator_input_from_assessed(&fixture("chaining_input.json"));
    let prior_layer_count = input.assessment_layers.len();
    assert_eq!(
        prior_layer_count, 1,
        "fixture should have exactly one prior layer"
    );

    let new_layer = RustStaticEvaluator
        .evaluate(input)
        .expect("evaluate should succeed");

    // Simulate what the pipeline does: append the new layer.
    let input2 = evaluator_input_from_assessed(&fixture("chaining_input.json"));
    let mut all_layers = input2.assessment_layers.clone();
    all_layers.push(new_layer);

    assert_eq!(
        all_layers.len(),
        2,
        "chaining should preserve the prior layer and append exactly one new layer"
    );
    assert_eq!(
        all_layers[0].id, "prior-layer-001",
        "prior layer should be preserved at index 0"
    );
    assert_eq!(
        all_layers[1].evaluator.id, "rust-static",
        "new layer should be from rust-static evaluator"
    );
}
