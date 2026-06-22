mod support;

use support::stub::{StubEvaluatorA, StubEvaluatorB, StubParser, StubReporter};
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::pipeline::{
    PipelineError, collect_step, evaluate_step, report_step, transform_step,
};
use tgraphy_core::{ArtifactKind, ComponentError};

// ── helpers ───────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("tgraphy_pipeline_{}_{}", std::process::id(), name))
}

fn builtin_registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    r.register_parser("builtin", Box::new(BuiltinParser));
    r.register_evaluator("builtin", Box::new(BuiltinEvaluator));
    r.register_reporter("builtin", Box::new(BuiltinReporter));
    r
}

// ── collect step ──────────────────────────────────────────────────────────────

#[test]
fn collect_writes_parsed_evidence_artifact() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("collect_output.json");

    let result = collect_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "collect should succeed: {:?}", result);

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    assert!(
        matches!(artifact, ArtifactKind::ParsedEvidence(_)),
        "collect should write parsed_evidence, got {:?}",
        artifact
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn collect_overwrites_existing_output() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("collect_overwrite.json");

    std::fs::write(&output, b"old content").unwrap();

    let result = collect_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "collect should overwrite existing output");

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    assert!(matches!(artifact, ArtifactKind::ParsedEvidence(_)));

    let _ = std::fs::remove_file(&output);
}

#[test]
fn collect_fails_with_same_path_exact_match() {
    let registry = builtin_registry();
    let path = fixture("valid_evidence.json");

    let result = collect_step(&registry, "builtin", &path, &path);
    assert!(
        matches!(result, Err(PipelineError::SamePath { .. })),
        "collect with same input/output should fail with SamePath: {:?}",
        result
    );
}

#[test]
fn collect_fails_with_same_path_dot_prefix() {
    let registry = builtin_registry();
    let input = std::path::PathBuf::from("a.json");
    let output = std::path::PathBuf::from("./a.json");

    let result = collect_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::SamePath { .. })),
        "a.json and ./a.json should be treated as same path: {:?}",
        result
    );
}

#[test]
fn collect_fails_with_same_path_via_parent_dir() {
    let registry = builtin_registry();
    let input = std::path::PathBuf::from("./dir/../a.json");
    let output = std::path::PathBuf::from("a.json");

    let result = collect_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::SamePath { .. })),
        "./dir/../a.json and a.json should be treated as same path: {:?}",
        result
    );
}

#[test]
fn collect_does_not_fail_for_distinct_paths() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("collect_distinct.json");

    if let Err(PipelineError::SamePath { .. }) = collect_step(&registry, "builtin", &input, &output)
    {
        panic!("distinct paths should not trigger SamePath error");
    }

    let _ = std::fs::remove_file(&output);
}

#[test]
fn collect_fails_with_component_error_for_unsupported_parser() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("collect_unsupported.json");

    let result = collect_step(&registry, "nonexistent-parser", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::Component(
                ComponentError::NotFoundComponent { .. }
            ))
        ),
        "unknown parser should produce Component(NotFoundComponent): {:?}",
        result
    );
}

// ── transform step ────────────────────────────────────────────────────────────

#[test]
fn transform_writes_module_evidence_from_parsed_evidence() {
    let input = fixture("parsed_evidence/valid.json");
    let output = temp_path("transform_output.json");

    let result = transform_step(&input, &output);
    assert!(result.is_ok(), "transform should succeed: {:?}", result);

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    let ArtifactKind::ModuleEvidence(me) = artifact else {
        panic!("expected module_evidence artifact");
    };
    assert_eq!(
        me.module_bundles.len(),
        1,
        "should have one bundle per module"
    );
    assert_eq!(
        me.module_bundles[0].tests.len(),
        1,
        "module should have one test entry from the link"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn transform_preserves_evidence_data() {
    let input = fixture("parsed_evidence/valid.json");
    let output = temp_path("transform_preserves.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::ParsedEvidence(original_pe) = original else {
        panic!()
    };

    transform_step(&input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::ModuleEvidence(me) = result else {
        panic!("expected module_evidence");
    };
    assert_eq!(
        me.evidence, original_pe.evidence,
        "evidence must be preserved through transform"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn transform_rejects_legacy_evidence_with_stage_error() {
    let input = fixture("valid_evidence.json");
    let output = temp_path("transform_reject_evidence.json");

    let result = transform_step(&input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "evidence",
                ..
            })
        ),
        "transform should reject evidence with UnexpectedArtifactType: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn transform_rejects_module_evidence_with_stage_error() {
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("transform_reject_me.json");

    let result = transform_step(&input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "module_evidence",
                ..
            })
        ),
        "transform should reject module_evidence with UnexpectedArtifactType: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn transform_rejects_assessed_module_evidence_with_stage_error() {
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("transform_reject_ame.json");

    let result = transform_step(&input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "assessed_module_evidence",
                ..
            })
        ),
        "transform should reject assessed_module_evidence with UnexpectedArtifactType: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn transform_fails_with_same_path() {
    let path = fixture("parsed_evidence/valid.json");

    let result = transform_step(&path, &path);
    assert!(
        matches!(result, Err(PipelineError::SamePath { .. })),
        "transform with same input/output should fail with SamePath: {:?}",
        result
    );
}

#[test]
fn transform_fails_with_io_error_for_missing_input() {
    let input = temp_path("transform_missing.json");
    let output = temp_path("transform_missing_out.json");

    let result = transform_step(&input, &output);
    assert!(
        matches!(result, Err(PipelineError::Io(_))),
        "missing input should produce IO error: {:?}",
        result
    );
}

// ── evaluate step: module_evidence input ──────────────────────────────────────

#[test]
fn evaluate_with_module_evidence_input_writes_assessed_module_evidence() {
    let registry = builtin_registry();
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("evaluate_from_module_evidence.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "evaluate should succeed: {:?}", result);

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    let ArtifactKind::AssessedModuleEvidence(assessed) = artifact else {
        panic!("expected assessed_module_evidence artifact");
    };
    assert_eq!(assessed.assessment_layers.len(), 1);

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_with_module_evidence_input_preserves_evidence_and_bundles() {
    let registry = builtin_registry();
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("evaluate_preserves_module_evidence.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::ModuleEvidence(original_me) = original else {
        panic!()
    };

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::AssessedModuleEvidence(assessed) = result else {
        panic!("expected assessed_module_evidence");
    };
    assert_eq!(
        assessed.evidence, original_me.evidence,
        "evidence must be preserved"
    );
    assert_eq!(
        assessed.module_bundles, original_me.module_bundles,
        "module_bundles must be preserved"
    );

    let _ = std::fs::remove_file(&output);
}

// ── evaluate step: assessed_module_evidence input (chaining) ─────────────────

#[test]
fn evaluate_with_assessed_module_evidence_input_appends_layer() {
    let registry = builtin_registry();
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("evaluate_from_assessed_module_evidence.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::AssessedModuleEvidence(original_assessed) = original else {
        panic!()
    };
    let original_layer_count = original_assessed.assessment_layers.len();

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::AssessedModuleEvidence(updated) = result else {
        panic!("expected assessed_module_evidence");
    };
    assert_eq!(
        updated.assessment_layers.len(),
        original_layer_count + 1,
        "evaluate should append exactly one new finding layer"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_with_assessed_module_evidence_input_preserves_existing_layers() {
    let registry = builtin_registry();
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("evaluate_preserves_layers.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::AssessedModuleEvidence(original_assessed) = original else {
        panic!()
    };

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::AssessedModuleEvidence(updated) = result else {
        panic!()
    };

    for (i, layer) in original_assessed.assessment_layers.iter().enumerate() {
        assert_eq!(
            updated.assessment_layers[i].id, layer.id,
            "existing layer {i} should be preserved"
        );
    }

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_with_assessed_module_evidence_passes_existing_layers_to_evaluator() {
    // StubEvaluatorA ignores input, but we verify the pipeline passes assessment_layers
    // through by confirming the output contains both the original and new layers.
    let registry = builtin_registry();
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("evaluate_layers_passed.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::AssessedModuleEvidence(original_assessed) = original else {
        panic!()
    };

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::AssessedModuleEvidence(updated) = result else {
        panic!()
    };

    assert!(
        updated.assessment_layers.len() > original_assessed.assessment_layers.len(),
        "output must contain more layers than input"
    );

    let _ = std::fs::remove_file(&output);
}

// ── evaluate step: stage rejection tests ─────────────────────────────────────

#[test]
fn evaluate_rejects_parsed_evidence_with_stage_error() {
    let registry = builtin_registry();
    let input = fixture("parsed_evidence/valid.json");
    let output = temp_path("evaluate_reject_parsed_evidence.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "parsed_evidence",
                ..
            })
        ),
        "evaluate should reject parsed_evidence with UnexpectedArtifactType: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_rejects_legacy_evidence_with_stage_error() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("evaluate_reject_evidence.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "evidence",
                ..
            })
        ),
        "evaluate should reject evidence with UnexpectedArtifactType: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_rejects_legacy_assessed_artifact_with_stage_error() {
    let registry = builtin_registry();
    let input = fixture("valid_assessed.json");
    let output = temp_path("evaluate_reject_assessed.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "assessed_artifact",
                ..
            })
        ),
        "evaluate should reject assessed_artifact with UnexpectedArtifactType: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

// ── evaluate step: other error cases ─────────────────────────────────────────

#[test]
fn evaluate_rejects_invalid_json_input() {
    let registry = builtin_registry();
    let input = temp_path("evaluate_invalid.json");
    let output = temp_path("evaluate_invalid_out.json");

    std::fs::write(&input, b"not valid json").unwrap();

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::ArtifactValidation(_))),
        "invalid JSON should return ArtifactValidation error: {:?}",
        result
    );

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_rejects_unknown_artifact_type() {
    let registry = builtin_registry();
    let input = temp_path("evaluate_unknown_type.json");
    let output = temp_path("evaluate_unknown_type_out.json");

    std::fs::write(
        &input,
        br#"{"schema_version":"0.0.1","artifact_type":"unknown_type","evidence":{}}"#,
    )
    .unwrap();

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::ArtifactValidation(_))),
        "unknown artifact type should return ArtifactValidation error: {:?}",
        result
    );

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_fails_with_io_error_for_missing_input() {
    let registry = builtin_registry();
    let input = temp_path("evaluate_missing_input.json");
    let output = temp_path("evaluate_missing_output.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::Io(_))),
        "missing input file should produce IO error: {:?}",
        result
    );
}

#[test]
fn evaluate_fails_with_component_error_for_unsupported_evaluator() {
    let registry = builtin_registry();
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("evaluate_unsupported.json");

    let result = evaluate_step(&registry, "nonexistent-evaluator", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::Component(
                ComponentError::NotFoundComponent { .. }
            ))
        ),
        "unknown evaluator should produce Component(NotFoundComponent): {:?}",
        result
    );
}

#[test]
fn evaluate_fails_with_same_path() {
    let registry = builtin_registry();
    let path = fixture("module_evidence/valid.json");

    let result = evaluate_step(&registry, "builtin", &path, &path);
    assert!(
        matches!(result, Err(PipelineError::SamePath { .. })),
        "evaluate with same input/output should fail with SamePath: {:?}",
        result
    );
}

#[test]
fn evaluate_overwrites_existing_output() {
    let registry = builtin_registry();
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("evaluate_overwrite.json");

    std::fs::write(&output, b"old content").unwrap();

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "evaluate should overwrite existing output");

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    assert!(matches!(artifact, ArtifactKind::AssessedModuleEvidence(_)));

    let _ = std::fs::remove_file(&output);
}

// ── report step ───────────────────────────────────────────────────────────────

#[test]
fn report_with_assessed_module_evidence_input_writes_output() {
    let registry = builtin_registry();
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("report_output.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "report should succeed: {:?}", result);
    assert!(output.exists(), "report output file should exist");

    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_rejects_legacy_evidence_artifact() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("report_reject_evidence.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "evidence",
                ..
            })
        ),
        "report should reject evidence artifact: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_rejects_legacy_assessed_artifact() {
    let registry = builtin_registry();
    let input = fixture("valid_assessed.json");
    let output = temp_path("report_reject_assessed.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "assessed_artifact",
                ..
            })
        ),
        "report should reject assessed_artifact: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_rejects_parsed_evidence_artifact() {
    let registry = builtin_registry();
    let input = fixture("parsed_evidence/valid.json");
    let output = temp_path("report_reject_parsed_evidence.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "parsed_evidence",
                ..
            })
        ),
        "report should reject parsed_evidence: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_rejects_module_evidence_artifact() {
    let registry = builtin_registry();
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("report_reject_module_evidence.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::UnexpectedArtifactType {
                found: "module_evidence",
                ..
            })
        ),
        "report should reject module_evidence: {:?}",
        result
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_rejects_invalid_json_input() {
    let registry = builtin_registry();
    let input = temp_path("report_invalid.json");
    let output = temp_path("report_invalid_out.txt");

    std::fs::write(&input, b"not valid json").unwrap();

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::ArtifactValidation(_))),
        "invalid JSON should return ArtifactValidation error: {:?}",
        result
    );

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_fails_with_io_error_for_missing_input() {
    let registry = builtin_registry();
    let input = temp_path("report_missing_input.json");
    let output = temp_path("report_missing_output.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::Io(_))),
        "missing input file should produce IO error: {:?}",
        result
    );
}

#[test]
fn report_fails_with_component_error_for_unsupported_reporter() {
    let registry = builtin_registry();
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("report_unsupported.txt");

    let result = report_step(&registry, "nonexistent-reporter", &input, &output);
    assert!(
        matches!(
            result,
            Err(PipelineError::Component(
                ComponentError::NotFoundComponent { .. }
            ))
        ),
        "unknown reporter should produce Component(NotFoundComponent): {:?}",
        result
    );
}

#[test]
fn report_fails_with_same_path() {
    let registry = builtin_registry();
    let path = fixture("assessed_module_evidence/valid.json");

    let result = report_step(&registry, "builtin", &path, &path);
    assert!(
        matches!(result, Err(PipelineError::SamePath { .. })),
        "report with same input/output should fail with SamePath: {:?}",
        result
    );
}

#[test]
fn report_overwrites_existing_output() {
    let registry = builtin_registry();
    let input = fixture("assessed_module_evidence/valid.json");
    let output = temp_path("report_overwrite.txt");

    std::fs::write(&output, b"old content").unwrap();

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "report should overwrite existing output");

    let _ = std::fs::remove_file(&output);
}

// ── stub fixture components: pipeline integration ─────────────────────────────

fn pipeline_fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pipeline")
        .join(name)
}

fn stub_registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    r.register_parser("stub-parser", Box::new(StubParser));
    r.register_evaluator("stub-evaluator-a", Box::new(StubEvaluatorA));
    r.register_evaluator("stub-evaluator-b", Box::new(StubEvaluatorB));
    r.register_reporter("stub-reporter", Box::new(StubReporter));
    r
}

fn read_json_file(path: &std::path::Path) -> serde_json::Value {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse JSON from {}: {}", path.display(), e))
}

#[test]
fn stub_collect_matches_parsed_evidence_fixture() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("stub_collect_parsed_evidence.json");

    collect_step(&registry, "stub-parser", &input, &output).expect("stub collect should succeed");

    let actual = read_json_file(&output);
    let expected = read_json_file(&pipeline_fixture("parsed_evidence.json"));
    assert_eq!(
        actual, expected,
        "collect output should match parsed_evidence fixture"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn stub_collect_transform_matches_module_evidence_from_stub_fixture() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let parsed_out = temp_path("stub_transform_parsed.json");
    let module_out = temp_path("stub_transform_module.json");

    collect_step(&registry, "stub-parser", &input, &parsed_out).expect("collect should succeed");
    transform_step(&parsed_out, &module_out).expect("transform should succeed");

    let actual = read_json_file(&module_out);
    let expected = read_json_file(&pipeline_fixture("module_evidence_from_stub.json"));
    assert_eq!(
        actual, expected,
        "collect+transform output should match module_evidence_from_stub fixture"
    );

    let _ = std::fs::remove_file(&parsed_out);
    let _ = std::fs::remove_file(&module_out);
}

#[test]
fn stub_first_evaluate_matches_assessed_module_evidence_first_fixture() {
    let registry = stub_registry();
    let input = fixture("module_evidence/valid.json");
    let output = temp_path("stub_first_assessed_module.json");

    evaluate_step(&registry, "stub-evaluator-a", &input, &output)
        .expect("first evaluate should succeed");

    let actual = read_json_file(&output);
    let expected = read_json_file(&pipeline_fixture("assessed_module_evidence_first.json"));
    assert_eq!(
        actual, expected,
        "first evaluate output should match assessed_module_evidence_first fixture"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn stub_second_evaluate_matches_assessed_module_evidence_second_fixture() {
    let registry = stub_registry();
    let first_out = temp_path("stub_second_first.json");
    let second_out = temp_path("stub_second_assessed_module.json");

    evaluate_step(
        &registry,
        "stub-evaluator-a",
        &fixture("module_evidence/valid.json"),
        &first_out,
    )
    .expect("first evaluate should succeed");
    evaluate_step(&registry, "stub-evaluator-b", &first_out, &second_out)
        .expect("second evaluate should succeed");

    let actual = read_json_file(&second_out);
    let expected = read_json_file(&pipeline_fixture("assessed_module_evidence_second.json"));
    assert_eq!(
        actual, expected,
        "second evaluate output should match assessed_module_evidence_second fixture"
    );

    let _ = std::fs::remove_file(&first_out);
    let _ = std::fs::remove_file(&second_out);
}

#[test]
fn stub_report_matches_report_staged_fixture() {
    let registry = stub_registry();
    let first_out = temp_path("stub_report_first.json");
    let second_out = temp_path("stub_report_second.json");
    let report_out = temp_path("stub_report_staged.md");

    evaluate_step(
        &registry,
        "stub-evaluator-a",
        &fixture("module_evidence/valid.json"),
        &first_out,
    )
    .expect("first evaluate should succeed");
    evaluate_step(&registry, "stub-evaluator-b", &first_out, &second_out)
        .expect("second evaluate should succeed");
    report_step(&registry, "stub-reporter", &second_out, &report_out)
        .expect("report should succeed");

    let actual = std::fs::read_to_string(&report_out).expect("report output should be readable");
    let expected = std::fs::read_to_string(pipeline_fixture("report_staged.md"))
        .expect("report_staged fixture should be readable");
    assert_eq!(
        actual, expected,
        "report output should match report_staged fixture"
    );

    let _ = std::fs::remove_file(&first_out);
    let _ = std::fs::remove_file(&second_out);
    let _ = std::fs::remove_file(&report_out);
}

#[test]
fn stub_full_pipeline_evaluate_evaluate_report() {
    let registry = stub_registry();
    let first_out = temp_path("stub_e2e_first.json");
    let second_out = temp_path("stub_e2e_second.json");
    let report_out = temp_path("stub_e2e_report.md");

    evaluate_step(
        &registry,
        "stub-evaluator-a",
        &fixture("module_evidence/valid.json"),
        &first_out,
    )
    .expect("first evaluate should succeed");

    let ArtifactKind::AssessedModuleEvidence(ref first) =
        tgraphy_core::io::read_artifact(&first_out).unwrap()
    else {
        panic!("expected assessed_module_evidence after first evaluate");
    };
    assert_eq!(first.assessment_layers.len(), 1);
    assert_eq!(first.assessment_layers[0].id, "stub-layer-a");

    evaluate_step(&registry, "stub-evaluator-b", &first_out, &second_out)
        .expect("second evaluate should succeed");

    let ArtifactKind::AssessedModuleEvidence(ref second) =
        tgraphy_core::io::read_artifact(&second_out).unwrap()
    else {
        panic!("expected assessed_module_evidence after second evaluate");
    };
    assert_eq!(second.assessment_layers.len(), 2);
    assert_eq!(second.assessment_layers[0].id, "stub-layer-a");
    assert_eq!(second.assessment_layers[1].id, "stub-layer-b");
    assert_eq!(
        first.evidence, second.evidence,
        "evidence must be preserved across evaluations"
    );
    assert_eq!(
        first.module_bundles, second.module_bundles,
        "module_bundles must be preserved across evaluations"
    );

    report_step(&registry, "stub-reporter", &second_out, &report_out)
        .expect("report should succeed");
    assert!(report_out.exists(), "report output file should exist");

    let _ = std::fs::remove_file(&first_out);
    let _ = std::fs::remove_file(&second_out);
    let _ = std::fs::remove_file(&report_out);
}

#[test]
fn collect_transform_evaluate_report_e2e() {
    let registry = stub_registry();
    let raw_input = fixture("valid_evidence.json");
    let parsed = temp_path("e2e_parsed.json");
    let module = temp_path("e2e_module.json");
    let assessed1 = temp_path("e2e_assessed1.json");
    let assessed2 = temp_path("e2e_assessed2.json");
    let report = temp_path("e2e_report.md");

    collect_step(&registry, "stub-parser", &raw_input, &parsed).expect("collect should succeed");
    assert!(matches!(
        tgraphy_core::io::read_artifact(&parsed).unwrap(),
        ArtifactKind::ParsedEvidence(_)
    ));

    transform_step(&parsed, &module).expect("transform should succeed");
    assert!(matches!(
        tgraphy_core::io::read_artifact(&module).unwrap(),
        ArtifactKind::ModuleEvidence(_)
    ));

    evaluate_step(&registry, "stub-evaluator-a", &module, &assessed1)
        .expect("first evaluate should succeed");
    let ArtifactKind::AssessedModuleEvidence(ref a1) =
        tgraphy_core::io::read_artifact(&assessed1).unwrap()
    else {
        panic!("expected assessed_module_evidence after first evaluate");
    };
    assert_eq!(a1.assessment_layers.len(), 1);
    assert_eq!(a1.assessment_layers[0].id, "stub-layer-a");

    evaluate_step(&registry, "stub-evaluator-b", &assessed1, &assessed2)
        .expect("second evaluate should succeed");
    let ArtifactKind::AssessedModuleEvidence(ref a2) =
        tgraphy_core::io::read_artifact(&assessed2).unwrap()
    else {
        panic!("expected assessed_module_evidence after second evaluate");
    };
    assert_eq!(a2.assessment_layers.len(), 2);
    assert_eq!(a2.assessment_layers[0].id, "stub-layer-a");
    assert_eq!(a2.assessment_layers[1].id, "stub-layer-b");

    report_step(&registry, "stub-reporter", &assessed2, &report).expect("report should succeed");
    assert!(report.exists(), "report output file should exist");

    let _ = std::fs::remove_file(&parsed);
    let _ = std::fs::remove_file(&module);
    let _ = std::fs::remove_file(&assessed1);
    let _ = std::fs::remove_file(&assessed2);
    let _ = std::fs::remove_file(&report);
}

// ── error category distinction ────────────────────────────────────────────────

#[test]
fn pipeline_errors_are_distinct_types() {
    let io_err = PipelineError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found",
    ));
    let artifact_err =
        PipelineError::ArtifactValidation(tgraphy_core::ArtifactError::UnknownArtifactType {
            found: "x".to_string(),
        });
    let component_err = PipelineError::Component(ComponentError::NotFoundComponent {
        message: "no such component".to_string(),
    });
    let stage_err = PipelineError::UnexpectedArtifactType {
        step: "evaluate",
        expected: "module_evidence or assessed_module_evidence",
        found: "parsed_evidence",
    };

    assert!(matches!(io_err, PipelineError::Io(_)));
    assert!(matches!(artifact_err, PipelineError::ArtifactValidation(_)));
    assert!(matches!(component_err, PipelineError::Component(_)));
    assert!(matches!(
        stage_err,
        PipelineError::UnexpectedArtifactType { .. }
    ));
}

#[test]
fn stage_error_is_distinct_from_artifact_validation_error() {
    // Wrong-stage input produces UnexpectedArtifactType, not ArtifactValidation.
    let registry = builtin_registry();
    let input = fixture("parsed_evidence/valid.json");
    let output = temp_path("stage_error_distinction.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::UnexpectedArtifactType { .. })),
        "wrong-stage input must produce UnexpectedArtifactType, not ArtifactValidation: {:?}",
        result
    );
    assert!(
        !matches!(result, Err(PipelineError::ArtifactValidation(_))),
        "wrong-stage input must not produce ArtifactValidation"
    );

    let _ = std::fs::remove_file(&output);
}
