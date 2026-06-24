use tgraphy_evaluator_rust_static::{EvaluatorInput, FindingLevel, SubjectKind, evaluate};

// ── helpers ───────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust")
        .join(name)
}

fn load_input(fixture_name: &str) -> EvaluatorInput {
    let json = std::fs::read_to_string(fixture(fixture_name)).expect("fixture should be readable");
    serde_json::from_str(&json).expect("fixture should deserialize as EvaluatorInput")
}

fn validate_assessed_module_evidence_schema(json: &str) -> Result<(), String> {
    use jsonschema::{Resource, Validator};

    let parsed_evidence_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../schemas/parsed_evidence/parsed_evidence.v0.json"
    ))
    .expect("parsed_evidence schema should parse");
    let module_evidence_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../schemas/module_evidence/module_evidence.v0.json"
    ))
    .expect("module_evidence schema should parse");
    let assessed_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../schemas/assessed_module_evidence/assessed_module_evidence.v0.json"
    ))
    .expect("assessed_module_evidence schema should parse");

    let pe_resource = Resource::from_contents(parsed_evidence_schema)
        .expect("parsed_evidence resource should build");
    let me_resource = Resource::from_contents(module_evidence_schema)
        .expect("module_evidence resource should build");

    let validator: Validator = jsonschema::options()
        .with_resource(
            "https://raw.githubusercontent.com/tooppoo/testography/main/schemas/parsed_evidence/parsed_evidence.v0.json",
            pe_resource,
        )
        .with_resource(
            "https://raw.githubusercontent.com/tooppoo/testography/main/schemas/module_evidence/module_evidence.v0.json",
            me_resource,
        )
        .build(&assessed_schema)
        .map_err(|e| format!("schema failed to compile: {e}"))?;

    let instance: serde_json::Value =
        serde_json::from_str(json).expect("artifact JSON should parse");
    let errors = validator
        .iter_errors(&instance)
        .map(|e| e.to_string())
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

// ── match cases ───────────────────────────────────────────────────────────────

#[test]
fn assert_predicate_only_produces_finding() {
    let layer = evaluate(load_input("assert_predicate_only.json"));

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
    let layer = evaluate(load_input("assert_with_message.json"));

    assert_eq!(
        layer.findings.len(),
        1,
        "assert!(predicate, \"message\") should still produce a finding"
    );
    assert_eq!(
        layer.findings[0].rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
}

#[test]
fn assert_with_comparison_expression_produces_finding() {
    let layer = evaluate(load_input("assert_with_comparison.json"));

    assert_eq!(
        layer.findings.len(),
        1,
        "assert!(actual == expected) should produce a finding in v0"
    );
    assert_eq!(
        layer.findings[0].rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
}

// ── non-match cases ───────────────────────────────────────────────────────────

#[test]
fn assert_eq_does_not_produce_finding() {
    let layer = evaluate(load_input("assert_eq.json"));
    assert_eq!(
        layer.findings.len(),
        0,
        "assert_eq! should not produce a predicate-only finding"
    );
}

#[test]
fn assert_ne_does_not_produce_finding() {
    let layer = evaluate(load_input("assert_ne.json"));
    assert_eq!(
        layer.findings.len(),
        0,
        "assert_ne! should not produce a predicate-only finding"
    );
}

// ── finding structure ─────────────────────────────────────────────────────────

#[test]
fn finding_has_stable_rule_id() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert_eq!(
        layer.findings[0].rule_id.as_deref(),
        Some("rust.assert.predicate_only_assertion")
    );
}

#[test]
fn finding_id_is_stable_and_derived_from_assertion_id() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert!(
        layer.findings[0].id.contains("rust-assertion-0001"),
        "finding id should reference the assertion id, got: {}",
        layer.findings[0].id
    );
}

#[test]
fn finding_has_confidence() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert!(
        layer.findings[0].confidence.is_some(),
        "finding should have a confidence value"
    );
}

#[test]
fn finding_level_is_info() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert!(
        matches!(layer.findings[0].level, FindingLevel::Info),
        "finding level should be info"
    );
}

#[test]
fn finding_subject_kind_is_assertion() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert!(
        matches!(layer.findings[0].subjects[0].kind, SubjectKind::Assertion),
        "subject kind should be assertion"
    );
}

#[test]
fn finding_message_does_not_claim_assert_is_always_invalid() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    let lower = layer.findings[0].message.to_lowercase();
    assert!(
        !lower.contains("invalid") && !lower.contains("incorrect") && !lower.contains("wrong"),
        "message should not claim the assertion is invalid/incorrect/wrong: {}",
        layer.findings[0].message
    );
}

#[test]
fn evaluator_id_is_rust_static() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert_eq!(layer.evaluator.id, "rust-static");
}

// ── schema validity ───────────────────────────────────────────────────────────

#[test]
fn output_layer_produces_schema_valid_assessed_module_evidence() {
    let input_json = std::fs::read_to_string(fixture("assert_predicate_only.json"))
        .expect("fixture should be readable");
    let input_value: serde_json::Value =
        serde_json::from_str(&input_json).expect("fixture should parse");

    let layer = evaluate(load_input("assert_predicate_only.json"));
    let layer_value = serde_json::to_value(&layer).expect("layer should serialize");

    let assessed = serde_json::json!({
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": input_value["evidence"],
        "module_bundles": input_value["module_bundles"],
        "assessment_layers": [layer_value]
    });

    let json = serde_json::to_string(&assessed).expect("should serialize");
    assert!(
        validate_assessed_module_evidence_schema(&json).is_ok(),
        "evaluator output should be schema-valid assessed_module_evidence: {json}"
    );
}

// ── chaining ──────────────────────────────────────────────────────────────────

#[test]
fn chaining_input_deserializes_and_produces_finding() {
    let layer = evaluate(load_input("chaining_input.json"));

    assert_eq!(layer.findings.len(), 1, "chaining input has one assert!()");
    assert_eq!(layer.evaluator.id, "rust-static");
}

// ── regression: assert! with expected field ───────────────────────────────────

#[test]
fn assert_with_expected_does_not_produce_finding() {
    let layer = evaluate(load_input("assert_with_expected.json"));
    assert_eq!(
        layer.findings.len(),
        0,
        "assert! with an expected field set should not produce a predicate-only finding"
    );
}

// ── layer identity ────────────────────────────────────────────────────────────

#[test]
fn layer_id_first_run_is_evaluator_id_indexed_at_zero() {
    let layer = evaluate(load_input("assert_predicate_only.json"));
    assert_eq!(
        layer.id, "rust-static-0",
        "first run (no prior rust-static layers) should produce id 'rust-static-0'"
    );
}

#[test]
fn layer_id_does_not_collide_with_existing_layer_ids() {
    let input = load_input("chaining_input.json");
    let existing_ids: Vec<String> = input
        .assessment_layers
        .iter()
        .map(|l| l.id.clone())
        .collect();
    let layer = evaluate(input);
    assert!(
        !existing_ids.iter().any(|id| id == &layer.id),
        "new layer id '{}' must not collide with existing ids: {:?}",
        layer.id,
        existing_ids
    );
}

#[test]
fn layer_id_increments_when_same_evaluator_ran_before() {
    let layer = evaluate(load_input("rerun_input.json"));
    assert_eq!(
        layer.id, "rust-static-1",
        "second run (one prior rust-static-0 layer) should produce id 'rust-static-1'"
    );
}

#[test]
fn layer_id_skips_index_occupied_by_foreign_evaluator() {
    // "rust-static-0" exists but belongs to a different evaluator — must not reuse it
    let layer = evaluate(load_input("foreign_id_conflict_input.json"));
    assert_eq!(
        layer.id, "rust-static-1",
        "rust-static-0 is taken by another evaluator; next available id is rust-static-1"
    );
}

#[test]
fn layer_id_fills_gap_when_zero_index_is_free() {
    // only "rust-static-1" exists — the gap at index 0 should be filled
    let layer = evaluate(load_input("gap_fill_input.json"));
    assert_eq!(
        layer.id, "rust-static-0",
        "rust-static-0 is free even though rust-static-1 exists; gap should be filled"
    );
}
