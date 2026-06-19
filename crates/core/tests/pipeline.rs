mod support;

use support::stub::{StubEvaluatorA, StubEvaluatorB, StubParser, StubReporter};
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::pipeline::{PipelineError, collect_step, evaluate_step, report_step};
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
fn collect_writes_evidence_artifact() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("collect_output.json");

    let result = collect_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "collect should succeed: {:?}", result);

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    assert!(matches!(artifact, ArtifactKind::Evidence(_)));

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
    assert!(matches!(artifact, ArtifactKind::Evidence(_)));

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

    // Only check same-path detection; the step may fail for other reasons.
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

// ── evaluate step: evidence input ─────────────────────────────────────────────

#[test]
fn evaluate_with_evidence_input_writes_assessed_artifact() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("evaluate_from_evidence.json");

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "evaluate should succeed: {:?}", result);

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    let ArtifactKind::Assessed(assessed) = artifact else {
        panic!("expected assessed artifact");
    };
    assert_eq!(assessed.assessment_layers.len(), 1);

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_with_evidence_input_preserves_evidence_data() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("evaluate_preserves_evidence.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::Evidence(original_ev) = original else {
        panic!()
    };

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::Assessed(assessed) = result else {
        panic!("expected assessed");
    };
    assert_eq!(
        assessed.evidence, original_ev.evidence,
        "evidence data must be preserved"
    );

    let _ = std::fs::remove_file(&output);
}

// ── evaluate step: assessed input ─────────────────────────────────────────────

#[test]
fn evaluate_with_assessed_input_appends_layer() {
    let registry = builtin_registry();
    let input = fixture("valid_assessed.json");
    let output = temp_path("evaluate_from_assessed.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::Assessed(original_assessed) = original else {
        panic!()
    };
    let original_layer_count = original_assessed.assessment_layers.len();

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::Assessed(updated) = result else {
        panic!("expected assessed");
    };
    assert_eq!(
        updated.assessment_layers.len(),
        original_layer_count + 1,
        "evaluate should append exactly one new assessment layer"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn evaluate_with_assessed_input_preserves_existing_layers() {
    let registry = builtin_registry();
    let input = fixture("valid_assessed.json");
    let output = temp_path("evaluate_preserves_layers.json");

    let original = tgraphy_core::io::read_artifact(&input).unwrap();
    let ArtifactKind::Assessed(original_assessed) = original else {
        panic!()
    };

    evaluate_step(&registry, "builtin", &input, &output).unwrap();

    let result = tgraphy_core::io::read_artifact(&output).unwrap();
    let ArtifactKind::Assessed(updated) = result else {
        panic!()
    };

    let original_layers = &original_assessed.assessment_layers;
    for (i, layer) in original_layers.iter().enumerate() {
        assert_eq!(
            updated.assessment_layers[i].id, layer.id,
            "existing layer {i} should be preserved"
        );
    }

    let _ = std::fs::remove_file(&output);
}

// ── evaluate step: rejection and errors ───────────────────────────────────────

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
        br#"{"schema_version":"0.0.1","artifact_type":"unknown_type","producer":{"name":"x","version":"0.1.0"},"evidence":{}}"#,
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
    let input = fixture("valid_evidence.json");
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
    let path = fixture("valid_evidence.json");

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
    let input = fixture("valid_evidence.json");
    let output = temp_path("evaluate_overwrite.json");

    std::fs::write(&output, b"old content").unwrap();

    let result = evaluate_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "evaluate should overwrite existing output");

    let artifact = tgraphy_core::io::read_artifact(&output).expect("output should be readable");
    assert!(matches!(artifact, ArtifactKind::Assessed(_)));

    let _ = std::fs::remove_file(&output);
}

// ── report step ───────────────────────────────────────────────────────────────

#[test]
fn report_with_assessed_input_writes_output() {
    let registry = builtin_registry();
    let input = fixture("valid_assessed.json");
    let output = temp_path("report_output.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "report should succeed: {:?}", result);
    assert!(output.exists(), "report output file should exist");

    let _ = std::fs::remove_file(&output);
}

#[test]
fn report_rejects_evidence_artifact_input() {
    let registry = builtin_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("report_reject_evidence.txt");

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(
        matches!(result, Err(PipelineError::UnexpectedArtifactType { .. })),
        "report should reject evidence artifact: {:?}",
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
    let input = fixture("valid_assessed.json");
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
    let path = fixture("valid_assessed.json");

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
    let input = fixture("valid_assessed.json");
    let output = temp_path("report_overwrite.txt");

    std::fs::write(&output, b"old content").unwrap();

    let result = report_step(&registry, "builtin", &input, &output);
    assert!(result.is_ok(), "report should overwrite existing output");

    let _ = std::fs::remove_file(&output);
}

// ── end-to-end: collect → evaluate → report ───────────────────────────────────

#[test]
fn collect_evaluate_report_end_to_end() {
    let registry = builtin_registry();
    let source = fixture("valid_evidence.json");
    let evidence_out = temp_path("e2e_evidence.json");
    let assessed_out = temp_path("e2e_assessed.json");
    let report_out = temp_path("e2e_report.txt");

    collect_step(&registry, "builtin", &source, &evidence_out).expect("collect should succeed");

    let evidence = tgraphy_core::io::read_artifact(&evidence_out).unwrap();
    assert!(matches!(evidence, ArtifactKind::Evidence(_)));

    evaluate_step(&registry, "builtin", &evidence_out, &assessed_out)
        .expect("evaluate should succeed");

    let assessed = tgraphy_core::io::read_artifact(&assessed_out).unwrap();
    assert!(matches!(assessed, ArtifactKind::Assessed(_)));

    report_step(&registry, "builtin", &assessed_out, &report_out).expect("report should succeed");

    assert!(report_out.exists(), "report output file should exist");

    let _ = std::fs::remove_file(&evidence_out);
    let _ = std::fs::remove_file(&assessed_out);
    let _ = std::fs::remove_file(&report_out);
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
fn stub_collect_matches_evidence_fixture() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let output = temp_path("stub_collect_evidence.json");

    collect_step(&registry, "stub-parser", &input, &output).expect("stub collect should succeed");

    let actual = read_json_file(&output);
    let expected = read_json_file(&pipeline_fixture("evidence.json"));
    assert_eq!(
        actual, expected,
        "collect output should match evidence fixture"
    );

    let _ = std::fs::remove_file(&output);
}

#[test]
fn stub_first_evaluate_matches_assessed_first_fixture() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let evidence_out = temp_path("stub_first_ev.json");
    let assessed_out = temp_path("stub_first_assessed.json");

    collect_step(&registry, "stub-parser", &input, &evidence_out).expect("collect should succeed");
    evaluate_step(&registry, "stub-evaluator-a", &evidence_out, &assessed_out)
        .expect("first evaluate should succeed");

    let actual = read_json_file(&assessed_out);
    let expected = read_json_file(&pipeline_fixture("assessed_first.json"));
    assert_eq!(
        actual, expected,
        "first evaluate output should match assessed_first fixture"
    );

    let _ = std::fs::remove_file(&evidence_out);
    let _ = std::fs::remove_file(&assessed_out);
}

#[test]
fn stub_second_evaluate_matches_assessed_second_fixture() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let evidence_out = temp_path("stub_second_ev.json");
    let first_out = temp_path("stub_second_first.json");
    let second_out = temp_path("stub_second_assessed.json");

    collect_step(&registry, "stub-parser", &input, &evidence_out).expect("collect should succeed");
    evaluate_step(&registry, "stub-evaluator-a", &evidence_out, &first_out)
        .expect("first evaluate should succeed");
    evaluate_step(&registry, "stub-evaluator-b", &first_out, &second_out)
        .expect("second evaluate should succeed");

    let actual = read_json_file(&second_out);
    let expected = read_json_file(&pipeline_fixture("assessed_second.json"));
    assert_eq!(
        actual, expected,
        "second evaluate output should match assessed_second fixture"
    );

    let _ = std::fs::remove_file(&evidence_out);
    let _ = std::fs::remove_file(&first_out);
    let _ = std::fs::remove_file(&second_out);
}

#[test]
fn stub_report_matches_report_fixture() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let evidence_out = temp_path("stub_report_ev.json");
    let first_out = temp_path("stub_report_first.json");
    let second_out = temp_path("stub_report_second.json");
    let report_out = temp_path("stub_report.md");

    collect_step(&registry, "stub-parser", &input, &evidence_out).expect("collect should succeed");
    evaluate_step(&registry, "stub-evaluator-a", &evidence_out, &first_out)
        .expect("first evaluate should succeed");
    evaluate_step(&registry, "stub-evaluator-b", &first_out, &second_out)
        .expect("second evaluate should succeed");
    report_step(&registry, "stub-reporter", &second_out, &report_out)
        .expect("report should succeed");

    let actual = std::fs::read_to_string(&report_out).expect("report output should be readable");
    let expected = std::fs::read_to_string(&pipeline_fixture("report.md"))
        .expect("report fixture should be readable");
    assert_eq!(
        actual, expected,
        "report output should match report fixture"
    );

    let _ = std::fs::remove_file(&evidence_out);
    let _ = std::fs::remove_file(&first_out);
    let _ = std::fs::remove_file(&second_out);
    let _ = std::fs::remove_file(&report_out);
}

#[test]
fn stub_full_pipeline_collect_evaluate_evaluate_report() {
    let registry = stub_registry();
    let input = fixture("valid_evidence.json");
    let evidence_out = temp_path("stub_e2e_evidence.json");
    let first_out = temp_path("stub_e2e_first.json");
    let second_out = temp_path("stub_e2e_second.json");
    let report_out = temp_path("stub_e2e_report.md");

    collect_step(&registry, "stub-parser", &input, &evidence_out).expect("collect should succeed");

    let ArtifactKind::Evidence(_) = tgraphy_core::io::read_artifact(&evidence_out).unwrap() else {
        panic!("expected evidence artifact after collect");
    };

    evaluate_step(&registry, "stub-evaluator-a", &evidence_out, &first_out)
        .expect("first evaluate should succeed");

    let ArtifactKind::Assessed(ref first) = tgraphy_core::io::read_artifact(&first_out).unwrap()
    else {
        panic!("expected assessed artifact after first evaluate");
    };
    assert_eq!(first.assessment_layers.len(), 1);
    assert_eq!(first.assessment_layers[0].id, "stub-layer-a");

    evaluate_step(&registry, "stub-evaluator-b", &first_out, &second_out)
        .expect("second evaluate should succeed");

    let ArtifactKind::Assessed(ref second) = tgraphy_core::io::read_artifact(&second_out).unwrap()
    else {
        panic!("expected assessed artifact after second evaluate");
    };
    assert_eq!(second.assessment_layers.len(), 2);
    assert_eq!(second.assessment_layers[0].id, "stub-layer-a");
    assert_eq!(second.assessment_layers[1].id, "stub-layer-b");
    assert_eq!(
        first.evidence, second.evidence,
        "evidence must be preserved across evaluations"
    );

    report_step(&registry, "stub-reporter", &second_out, &report_out)
        .expect("report should succeed");
    assert!(report_out.exists(), "report output file should exist");

    let _ = std::fs::remove_file(&evidence_out);
    let _ = std::fs::remove_file(&first_out);
    let _ = std::fs::remove_file(&second_out);
    let _ = std::fs::remove_file(&report_out);
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

    assert!(matches!(io_err, PipelineError::Io(_)));
    assert!(matches!(artifact_err, PipelineError::ArtifactValidation(_)));
    assert!(matches!(component_err, PipelineError::Component(_)));
}
