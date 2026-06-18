use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::component::parser::ParserInput;
use tgraphy_core::component::process::{
    ProcessConfig, ProcessEvaluator, ProcessParser, ProcessReporter,
};
use tgraphy_core::component::{ComponentError, ComponentRegistry};
use tgraphy_core::{ArtifactError, ComponentResult};

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
fn builtin_parser_returns_evidence_artifact() {
    let parser = BuiltinParser;
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(result.is_ok(), "builtin parser should produce a result");
    let artifact = result.unwrap();
    assert_eq!(artifact.artifact_type, "evidence");
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
        matches!(result, Err(ComponentError::UnsupportedComponent { .. })),
        "unknown parser name should return UnsupportedComponent error"
    );
}

#[test]
fn resolving_unknown_evaluator_name_returns_error() {
    let registry = ComponentRegistry::new();
    let result = registry.resolve_evaluator("nonexistent");
    assert!(
        matches!(result, Err(ComponentError::UnsupportedComponent { .. })),
        "unknown evaluator name should return UnsupportedComponent error"
    );
}

#[test]
fn resolving_unknown_reporter_name_returns_error() {
    let registry = ComponentRegistry::new();
    let result = registry.resolve_reporter("nonexistent");
    assert!(
        matches!(result, Err(ComponentError::UnsupportedComponent { .. })),
        "unknown reporter name should return UnsupportedComponent error"
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
fn process_parser_returns_structured_failure() {
    let parser = ProcessParser {
        config: process_config("nonexistent-command"),
    };
    let result = tgraphy_core::component::parser::Parser::parse(&parser, empty_parser_input());
    assert!(
        matches!(result, Err(ComponentError::InternalError { .. })),
        "process parser should return InternalError before execution is implemented"
    );
}
