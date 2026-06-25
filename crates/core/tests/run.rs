mod support;

use support::stub::{StubEvaluatorA, StubEvaluatorB, StubParser, StubReporter};
use tgraphy_core::ArtifactKind;
use tgraphy_core::artifact::staged::FindingLayer;
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::{ComponentError, ComponentResult, Evaluator, EvaluatorInput};
use tgraphy_core::component::{ReportOutput, Reporter, ReporterInput};
use tgraphy_core::pipeline::{PipelineError, run_pipeline};

// ── helpers ───────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn temp_output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tgraphy_run_{}_{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).expect("could not create temp dir");
    dir
}

fn stub_registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    r.register_parser("stub-parser", Box::new(StubParser));
    r.register_evaluator("stub-evaluator-a", Box::new(StubEvaluatorA));
    r.register_evaluator("stub-evaluator-b", Box::new(StubEvaluatorB));
    r.register_reporter("stub-reporter", Box::new(StubReporter));
    r
}

fn run_with_stubs(dir: &std::path::Path, evaluators: &[&str]) -> Result<(), PipelineError> {
    let input = fixture("valid_evidence.json");
    let evaluator_names: Vec<String> = evaluators.iter().map(|s| s.to_string()).collect();
    run_pipeline(
        &stub_registry(),
        &input,
        "stub-parser",
        &evaluator_names,
        "stub-reporter",
        dir,
    )
}

// ── reporter stub with configurable extension ─────────────────────────────────

struct ExtReporter(String);

impl Reporter for ExtReporter {
    fn report(&self, _input: ReporterInput) -> ComponentResult<ReportOutput> {
        Ok(ReportOutput {
            format: "test".to_string(),
            extension: self.0.clone(),
            content: b"test report content".to_vec(),
        })
    }
}

fn run_with_extension(dir: &std::path::Path, extension: &str) -> Result<(), PipelineError> {
    let mut r = ComponentRegistry::new();
    r.register_parser("stub-parser", Box::new(StubParser));
    r.register_evaluator("stub-evaluator-a", Box::new(StubEvaluatorA));
    r.register_reporter("ext-reporter", Box::new(ExtReporter(extension.to_string())));

    let input = fixture("valid_evidence.json");
    run_pipeline(
        &r,
        &input,
        "stub-parser",
        &["stub-evaluator-a".to_string()],
        "ext-reporter",
        dir,
    )
}

// ── failure stubs ─────────────────────────────────────────────────────────────

struct FailingEvaluator;

impl Evaluator for FailingEvaluator {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        Err(ComponentError::ExecutionFailed {
            message: "stub evaluator failure".to_string(),
        })
    }
}

struct FailingReporter;

impl Reporter for FailingReporter {
    fn report(&self, _input: ReporterInput) -> ComponentResult<ReportOutput> {
        Err(ComponentError::ExecutionFailed {
            message: "stub reporter failure".to_string(),
        })
    }
}

// ── successful run ────────────────────────────────────────────────────────────

#[test]
fn run_pipeline_creates_all_intermediate_artifacts() {
    let dir = temp_output_dir("creates_all");
    let result = run_with_stubs(&dir, &["stub-evaluator-a"]);
    assert!(result.is_ok(), "run should succeed: {result:?}");

    assert!(
        dir.join("parsed_evidence.json").exists(),
        "parsed_evidence.json should exist"
    );
    assert!(
        dir.join("module_evidence.json").exists(),
        "module_evidence.json should exist"
    );
    assert!(
        dir.join("assessed_module_evidence.json").exists(),
        "assessed_module_evidence.json should exist"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_report_uses_reporter_defined_extension() {
    let dir = temp_output_dir("ext_from_reporter");
    run_with_stubs(&dir, &["stub-evaluator-a"]).expect("run should succeed");

    assert!(
        dir.join("report.md").exists(),
        "report.md should exist (extension comes from stub reporter, not core mapping)"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_new_reporter_extension_does_not_require_core_mapping() {
    let dir = temp_output_dir("new_ext");

    let mut r = ComponentRegistry::new();
    r.register_parser("stub-parser", Box::new(StubParser));
    r.register_evaluator("stub-evaluator-a", Box::new(StubEvaluatorA));
    r.register_reporter("txt-reporter", Box::new(ExtReporter("txt".to_string())));

    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &r,
        &input,
        "stub-parser",
        &["stub-evaluator-a".to_string()],
        "txt-reporter",
        &dir,
    );
    assert!(
        result.is_ok(),
        "run with txt reporter should succeed: {result:?}"
    );
    assert!(dir.join("report.txt").exists(), "report.txt should exist");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_overwrites_existing_output_on_rerun() {
    let dir = temp_output_dir("overwrite");
    run_with_stubs(&dir, &["stub-evaluator-a"]).expect("first run should succeed");
    let first_content = std::fs::read(dir.join("parsed_evidence.json")).unwrap();

    run_with_stubs(&dir, &["stub-evaluator-a"]).expect("second run should succeed");
    let second_content = std::fs::read(dir.join("parsed_evidence.json")).unwrap();

    assert_eq!(
        first_content, second_content,
        "artifact should be overwritten with identical content"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_creates_output_dir_if_missing() {
    let base = temp_output_dir("create_dir_base");
    let dir = base.join("nested").join("subdir");

    assert!(!dir.exists(), "nested dir should not exist yet");

    let result = run_with_stubs(&dir, &["stub-evaluator-a"]);
    assert!(
        result.is_ok(),
        "run should succeed after creating output dir: {result:?}"
    );
    assert!(dir.exists(), "output dir should have been created");

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn run_pipeline_intermediate_artifacts_have_correct_stages() {
    let dir = temp_output_dir("stages");
    run_with_stubs(&dir, &["stub-evaluator-a"]).expect("run should succeed");

    let parsed = tgraphy_core::io::read_artifact(&dir.join("parsed_evidence.json")).unwrap();
    assert!(
        matches!(parsed, ArtifactKind::ParsedEvidence(_)),
        "must be parsed_evidence"
    );

    let module = tgraphy_core::io::read_artifact(&dir.join("module_evidence.json")).unwrap();
    assert!(
        matches!(module, ArtifactKind::ModuleEvidence(_)),
        "must be module_evidence"
    );

    let assessed =
        tgraphy_core::io::read_artifact(&dir.join("assessed_module_evidence.json")).unwrap();
    assert!(
        matches!(assessed, ArtifactKind::AssessedModuleEvidence(_)),
        "must be assessed_module_evidence"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── evaluator chaining ────────────────────────────────────────────────────────

#[test]
fn run_pipeline_with_two_evaluators_preserves_layer_order() {
    let dir = temp_output_dir("two_evals");
    run_with_stubs(&dir, &["stub-evaluator-a", "stub-evaluator-b"]).expect("run should succeed");

    let artifact = tgraphy_core::io::read_artifact(&dir.join("assessed_module_evidence.json"))
        .expect("assessed artifact should be readable");
    let ArtifactKind::AssessedModuleEvidence(assessed) = artifact else {
        panic!("expected assessed_module_evidence");
    };

    assert_eq!(
        assessed.assessment_layers.len(),
        2,
        "should have two layers"
    );
    assert_eq!(
        assessed.assessment_layers[0].id, "stub-layer-a",
        "first layer must be from evaluator-a"
    );
    assert_eq!(
        assessed.assessment_layers[1].id, "stub-layer-b",
        "second layer must be from evaluator-b"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_with_two_evaluators_preserves_existing_layers() {
    let dir = temp_output_dir("preserve_layers");
    run_with_stubs(&dir, &["stub-evaluator-a", "stub-evaluator-b"]).expect("run should succeed");

    let artifact = tgraphy_core::io::read_artifact(&dir.join("assessed_module_evidence.json"))
        .expect("artifact should be readable");
    let ArtifactKind::AssessedModuleEvidence(assessed) = artifact else {
        panic!("expected assessed_module_evidence");
    };

    assert!(
        assessed
            .assessment_layers
            .iter()
            .any(|l| l.id == "stub-layer-a"),
        "layer-a must be preserved after evaluator-b runs"
    );
    assert!(
        assessed
            .assessment_layers
            .iter()
            .any(|l| l.id == "stub-layer-b"),
        "layer-b must be present"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── extension validation ──────────────────────────────────────────────────────

#[test]
fn run_pipeline_accepts_extension_md() {
    let dir = temp_output_dir("ext_md");
    let result = run_with_extension(&dir, "md");
    assert!(result.is_ok(), "md extension should be valid: {result:?}");
    assert!(dir.join("report.md").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_accepts_extension_json() {
    let dir = temp_output_dir("ext_json");
    let result = run_with_extension(&dir, "json");
    assert!(result.is_ok(), "json extension should be valid: {result:?}");
    assert!(dir.join("report.json").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_accepts_extension_txt() {
    let dir = temp_output_dir("ext_txt");
    let result = run_with_extension(&dir, "txt");
    assert!(result.is_ok(), "txt extension should be valid: {result:?}");
    assert!(dir.join("report.txt").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_extension_with_leading_dot() {
    let dir = temp_output_dir("ext_leading_dot");
    let result = run_with_extension(&dir, ".md");
    assert!(
        matches!(result, Err(PipelineError::InvalidReportExtension { .. })),
        ".md should be invalid: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_empty_extension() {
    let dir = temp_output_dir("ext_empty");
    let result = run_with_extension(&dir, "");
    assert!(
        matches!(result, Err(PipelineError::InvalidReportExtension { .. })),
        "empty extension should be invalid: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_extension_with_slash() {
    let dir = temp_output_dir("ext_slash");
    let result = run_with_extension(&dir, "a/b");
    assert!(
        matches!(result, Err(PipelineError::InvalidReportExtension { .. })),
        "a/b should be invalid: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_extension_with_backslash() {
    let dir = temp_output_dir("ext_backslash");
    let result = run_with_extension(&dir, "a\\b");
    assert!(
        matches!(result, Err(PipelineError::InvalidReportExtension { .. })),
        "a\\\\b should be invalid: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_extension_with_traversal() {
    let dir = temp_output_dir("ext_traversal");
    let result = run_with_extension(&dir, "../x");
    assert!(
        matches!(result, Err(PipelineError::InvalidReportExtension { .. })),
        "../x should be invalid: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_extension_that_looks_like_filename() {
    let dir = temp_output_dir("ext_filename");
    let result = run_with_extension(&dir, "report.md");
    assert!(
        matches!(result, Err(PipelineError::InvalidReportExtension { .. })),
        "report.md should be invalid (extension only, not filename): {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_invalid_extension_error_is_distinct_from_other_errors() {
    let dir = temp_output_dir("ext_error_class");
    let result = run_with_extension(&dir, ".md");
    assert!(matches!(
        result,
        Err(PipelineError::InvalidReportExtension { .. })
    ));
    assert!(!matches!(result, Err(PipelineError::ArtifactValidation(_))));
    assert!(!matches!(result, Err(PipelineError::Component(_))));
    let _ = std::fs::remove_dir_all(&dir);
}

// ── failure behavior ──────────────────────────────────────────────────────────

fn assert_step_failed(result: &Result<(), PipelineError>, expected_step: &str) {
    let Err(PipelineError::StepFailed { step, .. }) = result else {
        panic!("expected StepFailed error for step '{expected_step}', got: {result:?}");
    };
    assert_eq!(
        *step, expected_step,
        "expected step '{expected_step}', got '{step}'"
    );
}

fn inner_error(result: &Result<(), PipelineError>) -> &PipelineError {
    let Err(PipelineError::StepFailed { source, .. }) = result else {
        panic!("expected StepFailed, got: {result:?}");
    };
    source.as_ref()
}

#[test]
fn run_pipeline_keeps_prior_artifacts_on_evaluator_failure() {
    let dir = temp_output_dir("keep_on_eval_fail");

    let mut r = stub_registry();
    r.register_evaluator("failing-evaluator", Box::new(FailingEvaluator));

    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &r,
        &input,
        "stub-parser",
        &["failing-evaluator".to_string()],
        "stub-reporter",
        &dir,
    );

    assert_step_failed(&result, "evaluate");
    assert!(matches!(inner_error(&result), PipelineError::Component(_)));
    assert!(
        dir.join("parsed_evidence.json").exists(),
        "parsed_evidence.json should remain"
    );
    assert!(
        dir.join("module_evidence.json").exists(),
        "module_evidence.json should remain"
    );
    assert!(
        !dir.join("assessed_module_evidence.json").exists(),
        "assessed artifact should not exist after eval failure"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_keeps_prior_artifacts_on_reporter_failure() {
    let dir = temp_output_dir("keep_on_report_fail");

    let mut r = stub_registry();
    r.register_reporter("failing-reporter", Box::new(FailingReporter));

    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &r,
        &input,
        "stub-parser",
        &["stub-evaluator-a".to_string()],
        "failing-reporter",
        &dir,
    );

    assert_step_failed(&result, "report");
    assert!(matches!(inner_error(&result), PipelineError::Component(_)));
    assert!(
        dir.join("parsed_evidence.json").exists(),
        "parsed_evidence.json should remain"
    );
    assert!(
        dir.join("module_evidence.json").exists(),
        "module_evidence.json should remain"
    );
    assert!(
        dir.join("assessed_module_evidence.json").exists(),
        "assessed artifact should remain after reporter failure"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_collect_failure_names_collect_step() {
    let dir = temp_output_dir("collect_step_name");
    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &stub_registry(),
        &input,
        "nonexistent-parser",
        &["stub-evaluator-a".to_string()],
        "stub-reporter",
        &dir,
    );
    assert_step_failed(&result, "collect");
    assert!(
        matches!(inner_error(&result), PipelineError::Component(_)),
        "inner error should be Component: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_evaluate_failure_names_evaluate_step() {
    let dir = temp_output_dir("evaluate_step_name");
    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &stub_registry(),
        &input,
        "stub-parser",
        &["nonexistent-evaluator".to_string()],
        "stub-reporter",
        &dir,
    );
    assert_step_failed(&result, "evaluate");
    assert!(
        matches!(inner_error(&result), PipelineError::Component(_)),
        "inner error should be Component: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_report_failure_names_report_step() {
    let dir = temp_output_dir("report_step_name");
    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &stub_registry(),
        &input,
        "stub-parser",
        &["stub-evaluator-a".to_string()],
        "nonexistent-reporter",
        &dir,
    );
    assert_step_failed(&result, "report");
    assert!(
        matches!(inner_error(&result), PipelineError::Component(_)),
        "inner error should be Component: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_step_error_message_includes_step_name_and_inner_class() {
    let dir = temp_output_dir("error_message");
    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &stub_registry(),
        &input,
        "nonexistent-parser",
        &["stub-evaluator-a".to_string()],
        "stub-reporter",
        &dir,
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("collect"),
        "error message should name the failed step: {msg}"
    );
    assert!(
        msg.contains("component"),
        "error message should include error class: {msg}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_pipeline_rejects_empty_evaluator_list() {
    let dir = temp_output_dir("empty_evaluators");
    let input = fixture("valid_evidence.json");
    let result = run_pipeline(
        &stub_registry(),
        &input,
        "stub-parser",
        &[],
        "stub-reporter",
        &dir,
    );
    assert!(
        matches!(result, Err(PipelineError::InvalidPipelineConfig { .. })),
        "empty evaluators should return InvalidPipelineConfig: {result:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
