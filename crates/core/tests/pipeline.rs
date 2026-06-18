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
                ComponentError::UnsupportedComponent { .. }
            ))
        ),
        "unknown parser should produce Component(UnsupportedComponent): {:?}",
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
                ComponentError::UnsupportedComponent { .. }
            ))
        ),
        "unknown evaluator should produce Component(UnsupportedComponent): {:?}",
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
                ComponentError::UnsupportedComponent { .. }
            ))
        ),
        "unknown reporter should produce Component(UnsupportedComponent): {:?}",
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
    let component_err = PipelineError::Component(ComponentError::UnsupportedComponent {
        message: "no such component".to_string(),
    });

    assert!(matches!(io_err, PipelineError::Io(_)));
    assert!(matches!(artifact_err, PipelineError::ArtifactValidation(_)));
    assert!(matches!(component_err, PipelineError::Component(_)));
}
