use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::component::evaluator::{Evaluator, EvaluatorInput};
use tgraphy_core::component::parser::ParserInput;
use tgraphy_core::component::process::{
    ProcessConfig, ProcessEvaluator, ProcessParser, ProcessReporter,
};
use tgraphy_core::component::reporter::{Reporter, ReporterInput};
use tgraphy_core::component::{ComponentError, ComponentRegistry};
use tgraphy_core::{ArtifactError, ArtifactKind, ComponentResult};

// ── helpers ───────────────────────────────────────────────────────────────────

fn empty_parser_input() -> ParserInput {
    ParserInput {
        source_paths: vec![],
        config: None,
    }
}

fn process_config(command: &str) -> ProcessConfig {
    ProcessConfig {
        command: command.to_string(),
        args: vec![],
    }
}

fn minimal_assessed_module_evidence() -> tgraphy_core::AssessedModuleEvidenceArtifact {
    let json = r#"{"schema_version":"0.0.1","artifact_type":"assessed_module_evidence","evidence":{},"module_bundles":[],"assessment_layers":[]}"#;
    match tgraphy_core::parse_artifact(json).unwrap() {
        ArtifactKind::AssessedModuleEvidence(a) => a,
        _ => panic!("expected assessed_module_evidence"),
    }
}

fn minimal_evaluator_input() -> EvaluatorInput {
    use tgraphy_core::artifact::staged::StagedEvidence;
    EvaluatorInput {
        evidence: StagedEvidence {
            test_cases: vec![],
            modules: vec![],
            test_module_links: vec![],
            extensions: None,
        },
        module_bundles: vec![],
        assessment_layers: vec![],
        config: None,
    }
}

// ── builtin component resolution ──────────────────────────────────────────────

#[test]
fn builtin_parser_resolves_from_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register_parser("builtin", Box::new(BuiltinParser));

    let parser = registry.resolve_parser("builtin");
    assert!(parser.is_ok(), "builtin parser should resolve");
}

#[test]
fn builtin_evaluator_resolves_from_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register_evaluator("builtin", Box::new(BuiltinEvaluator));

    let evaluator = registry.resolve_evaluator("builtin");
    assert!(evaluator.is_ok(), "builtin evaluator should resolve");
}

#[test]
fn builtin_reporter_resolves_from_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register_reporter("builtin", Box::new(BuiltinReporter));

    let reporter = registry.resolve_reporter("builtin");
    assert!(reporter.is_ok(), "builtin reporter should resolve");
}

#[test]
fn builtin_parser_returns_parsed_evidence_artifact() {
    let parser = BuiltinParser;
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(result.is_ok(), "builtin parser should produce a result");
    let artifact = result.unwrap();
    assert_eq!(artifact.artifact_type, "parsed_evidence");
}

// ── process adapter representation ───────────────────────────────────────────

#[test]
fn process_parser_can_be_registered_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register_parser(
        "ts-parser",
        Box::new(ProcessParser {
            config: process_config("node tools/parser/index.js"),
        }),
    );

    let result = registry.resolve_parser("ts-parser");
    assert!(result.is_ok(), "process parser should resolve by name");
}

#[test]
fn process_evaluator_can_be_registered_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register_evaluator(
        "vitest-static",
        Box::new(ProcessEvaluator {
            config: process_config("testography-evaluator-vitest"),
        }),
    );

    let result = registry.resolve_evaluator("vitest-static");
    assert!(result.is_ok(), "process evaluator should resolve by name");
}

#[test]
fn process_reporter_can_be_registered_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register_reporter(
        "markdown",
        Box::new(ProcessReporter {
            name: "markdown".to_string(),
            config: process_config("testography-reporter-markdown"),
        }),
    );

    let result = registry.resolve_reporter("markdown");
    assert!(result.is_ok(), "process reporter should resolve by name");
}

// ── unsupported component name ────────────────────────────────────────────────

#[test]
fn resolving_unknown_parser_name_returns_error() {
    let registry = ComponentRegistry::new();
    let result = registry.resolve_parser("nonexistent");
    assert!(
        matches!(result, Err(ComponentError::NotFoundComponent { .. })),
        "unknown parser name should return NotFoundComponent error"
    );
}

#[test]
fn resolving_unknown_evaluator_name_returns_error() {
    let registry = ComponentRegistry::new();
    let result = registry.resolve_evaluator("nonexistent");
    assert!(
        matches!(result, Err(ComponentError::NotFoundComponent { .. })),
        "unknown evaluator name should return NotFoundComponent error"
    );
}

#[test]
fn resolving_unknown_reporter_name_returns_error() {
    let registry = ComponentRegistry::new();
    let result = registry.resolve_reporter("nonexistent");
    assert!(
        matches!(result, Err(ComponentError::NotFoundComponent { .. })),
        "unknown reporter name should return NotFoundComponent error"
    );
}

// ── structured component failure ─────────────────────────────────────────────

#[test]
fn component_error_is_distinct_from_artifact_error() {
    // ComponentError and ArtifactError are separate types; this test confirms
    // they are not conflated and can be handled independently.
    let component_err: ComponentResult<()> = Err(ComponentError::ExecutionFailed {
        message: "process exited with code 1".to_string(),
    });
    let artifact_err: Result<(), ArtifactError> = Err(ArtifactError::UnknownArtifactType {
        found: "unknown".to_string(),
    });

    assert!(matches!(
        component_err,
        Err(ComponentError::ExecutionFailed { .. })
    ));
    assert!(matches!(
        artifact_err,
        Err(ArtifactError::UnknownArtifactType { .. })
    ));
}

#[test]
fn process_parser_returns_error_for_missing_command() {
    let parser = ProcessParser {
        config: process_config("nonexistent-command"),
    };
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "missing command should return InternalError"
    );
}

#[test]
fn process_parser_returns_error_for_non_zero_exit() {
    let parser = ProcessParser {
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 1".to_string()],
        },
    };
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "non-zero exit should return InternalError"
    );
}

#[test]
fn process_parser_returns_error_for_invalid_json_output() {
    let parser = ProcessParser {
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "cat > /dev/null; echo 'not json'".to_string(),
            ],
        },
    };
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "invalid JSON output should return InternalError"
    );
}

#[test]
fn process_parser_returns_parsed_evidence_for_valid_output() {
    let minimal_json = r#"{"schema_version":"0.0.1","artifact_type":"parsed_evidence","evidence":{"test_cases":[],"modules":[],"test_module_links":[]}}"#;
    let cmd = format!("cat > /dev/null; printf '%s' '{}'", minimal_json);
    let parser = ProcessParser {
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), cmd],
        },
    };
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(
        result.is_ok(),
        "valid JSON output should succeed: {result:?}"
    );
    assert_eq!(result.unwrap().artifact_type, "parsed_evidence");
}

#[test]
fn process_evaluator_returns_error_for_missing_command() {
    let evaluator = ProcessEvaluator {
        config: process_config("nonexistent-command"),
    };
    let result = Evaluator::evaluate(&evaluator, minimal_evaluator_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "missing command should return InternalError"
    );
}

#[test]
fn process_evaluator_returns_error_for_non_zero_exit() {
    let evaluator = ProcessEvaluator {
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 1".to_string()],
        },
    };
    let result = Evaluator::evaluate(&evaluator, minimal_evaluator_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "non-zero exit should return InternalError"
    );
}

#[test]
fn process_evaluator_returns_error_for_invalid_json_output() {
    let evaluator = ProcessEvaluator {
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "cat > /dev/null; echo 'not json'".to_string(),
            ],
        },
    };
    let result = Evaluator::evaluate(&evaluator, minimal_evaluator_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "invalid JSON output should return InternalError"
    );
}

#[test]
fn process_evaluator_returns_finding_layer_for_valid_output() {
    let layer_json = r#"{"id":"layer-1","evaluator":{"id":"test-eval"},"findings":[]}"#;
    let cmd = format!("cat > /dev/null; printf '%s' '{}'", layer_json);
    let evaluator = ProcessEvaluator {
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), cmd],
        },
    };
    let result = Evaluator::evaluate(&evaluator, minimal_evaluator_input());
    assert!(
        result.is_ok(),
        "valid JSON output should succeed: {result:?}"
    );
    assert_eq!(result.unwrap().id, "layer-1");
}

#[test]
fn process_reporter_returns_error_for_missing_command() {
    let reporter = ProcessReporter {
        name: "test".to_string(),
        config: process_config("nonexistent-command"),
    };
    let result = Reporter::report(
        &reporter,
        ReporterInput {
            artifact: minimal_assessed_module_evidence(),
            config: None,
        },
    );
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "missing command should return InternalError"
    );
}

#[test]
fn process_reporter_returns_error_for_non_zero_exit() {
    let reporter = ProcessReporter {
        name: "test".to_string(),
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 1".to_string()],
        },
    };
    let result = Reporter::report(
        &reporter,
        ReporterInput {
            artifact: minimal_assessed_module_evidence(),
            config: None,
        },
    );
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "non-zero exit should return InternalError"
    );
}

#[test]
fn process_reporter_returns_stdout_bytes_for_successful_exit() {
    let reporter = ProcessReporter {
        name: "echo-reporter".to_string(),
        config: ProcessConfig {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "cat > /dev/null; printf 'hello report'".to_string(),
            ],
        },
    };
    let result = Reporter::report(
        &reporter,
        ReporterInput {
            artifact: minimal_assessed_module_evidence(),
            config: None,
        },
    );
    assert!(
        result.is_ok(),
        "successful process should return Ok: {result:?}"
    );
    let output = result.unwrap();
    assert_eq!(output.content, b"hello report");
    assert_eq!(output.format, "echo-reporter");
}
