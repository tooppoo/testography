use std::path::PathBuf;

use tgraphy_core::artifact::evidence::{LiteralClass, ResolutionStatus, ValueKind};
use tgraphy_core::component::builtin::RustParser;
use tgraphy_core::component::parser::{Parser, ParserInput};

fn fixture(name: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust")
        .join(name)
}

fn run(paths: Vec<PathBuf>) -> tgraphy_core::ParsedEvidenceArtifact {
    Parser::parse(
        &RustParser,
        ParserInput {
            source_paths: paths,
            config: None,
        },
    )
    .unwrap()
}

#[test]
fn simple_assertion_produces_schema_valid_evidence() {
    let artifact = run(vec![fixture("simple_assertion.rs")]);
    assert_eq!(artifact.artifact_type, "parsed_evidence");
    assert_eq!(artifact.schema_version, "0.0.1");
    assert_eq!(artifact.evidence.test_cases.len(), 1);

    let json = serde_json::to_string(&artifact).expect("should serialize");
    assert!(
        tgraphy_core::parse_artifact(&json).is_ok(),
        "schema validation failed for {json}"
    );
}

#[test]
fn simple_assertion_extracts_call_parameters_expected_and_link() {
    let artifact = run(vec![fixture("simple_assertion.rs")]);
    let evidence = &artifact.evidence;
    let tc = &evidence.test_cases[0];

    assert_eq!(tc.name, "test_add_returns_sum");
    let calls = tc.calls.as_ref().expect("should have calls");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].callee.text, "add");
    assert_eq!(
        calls[0].callee.resolution_status,
        ResolutionStatus::Resolved
    );
    assert_eq!(
        calls[0].callee.resolved_module_id.as_deref(),
        Some(evidence.modules[0].id.as_str())
    );

    let parameters = tc.parameters.as_ref().expect("should have parameters");
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0].argument_index, 0);
    assert_eq!(
        parameters[0].call_ref.as_deref(),
        Some(calls[0].id.as_str())
    );
    assert_eq!(parameters[0].value_kind, ValueKind::NumberLiteral);
    assert_eq!(
        parameters[0].literal_class,
        Some(LiteralClass::PositiveNumber)
    );
    assert_eq!(parameters[1].argument_index, 1);
    assert_eq!(
        parameters[1].call_ref.as_deref(),
        Some(calls[0].id.as_str())
    );

    let assertions = tc.assertions.as_ref().expect("should have assertions");
    assert_eq!(
        assertions[0]
            .matcher
            .as_ref()
            .and_then(|m| m.name.as_deref()),
        Some("assert_eq")
    );
    assert_eq!(
        assertions[0].target_expression.as_deref(),
        Some("add (1 , 2)")
    );
    assert_eq!(
        assertions[0].target_call_refs.as_deref(),
        Some([calls[0].id.clone()].as_slice())
    );
    assert_eq!(
        assertions[0]
            .expected
            .as_ref()
            .and_then(|v| v.literal_class.clone()),
        Some(LiteralClass::PositiveNumber)
    );

    assert_eq!(evidence.modules.len(), 1);
    assert_eq!(
        evidence.modules[0].qualified_name.as_deref(),
        Some("crate::calculator::add")
    );
    assert_eq!(evidence.test_module_links.len(), 1);
    assert_eq!(evidence.test_module_links[0].test_ref, tc.id);
    assert_eq!(
        evidence.test_module_links[0].module_ref,
        evidence.modules[0].id
    );
    assert_eq!(
        evidence.test_module_links[0].evidence_refs,
        vec![calls[0].id.clone()]
    );
}

#[test]
fn direct_path_resolves_without_use_import() {
    let artifact = run(vec![fixture("direct_path.rs")]);
    let tc = &artifact.evidence.test_cases[0];
    let calls = tc.calls.as_ref().expect("should have calls");

    assert_eq!(tc.name, "test_multiply_via_path");
    assert_eq!(calls[0].callee.text, "crate::calculator::multiply");
    assert_eq!(
        calls[0].callee.resolution_status,
        ResolutionStatus::Resolved
    );
    assert_eq!(
        artifact.evidence.modules[0].qualified_name.as_deref(),
        Some("crate::calculator::multiply")
    );
    assert_eq!(
        artifact.evidence.test_module_links[0]
            .relationship
            .as_deref(),
        Some("assertion_target")
    );
}

#[test]
fn unresolved_call_is_preserved_without_link() {
    let artifact = run(vec![fixture("unresolved_call.rs")]);
    let evidence = &artifact.evidence;
    let calls = evidence.test_cases[0]
        .calls
        .as_ref()
        .expect("should have calls");

    assert_eq!(calls[0].callee.text, "some_external_func");
    assert_eq!(
        calls[0].callee.resolution_status,
        ResolutionStatus::Unresolved
    );
    assert!(calls[0].callee.resolved_module_id.is_none());
    assert_call_diagnostic(&calls[0], "rust_parser.unresolved_call");
    assert!(evidence.modules.is_empty(), "no module for unresolved call");
    assert!(
        evidence.test_module_links.is_empty(),
        "no link for unresolved call"
    );
}

#[test]
fn multi_assertion_extracts_all_assertions_and_deduplicates_module() {
    let artifact = run(vec![fixture("multi_assertion.rs")]);
    let tc = &artifact.evidence.test_cases[0];
    let assertions = tc.assertions.as_ref().expect("should have assertions");
    assert_eq!(assertions.len(), 3);

    let names = assertions
        .iter()
        .map(|a| {
            a.matcher
                .as_ref()
                .and_then(|m| m.name.as_deref())
                .unwrap_or("")
        })
        .collect::<Vec<_>>();
    assert!(names.contains(&"assert"));
    assert!(names.contains(&"assert_eq"));
    assert!(names.contains(&"assert_ne"));
    assert_eq!(artifact.evidence.modules.len(), 1);
    assert_eq!(
        artifact.evidence.modules[0].qualified_name.as_deref(),
        Some("crate::validator::is_valid")
    );
}

#[test]
fn multiple_top_level_test_functions_are_extracted() {
    let artifact = run(vec![fixture("multi_test.rs")]);
    let evidence = &artifact.evidence;
    let names = evidence
        .test_cases
        .iter()
        .map(|tc| tc.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(evidence.test_cases.len(), 2);
    assert!(names.contains(&"test_add"));
    assert!(names.contains(&"test_subtract"));
    assert_eq!(evidence.modules.len(), 2);
    assert_eq!(evidence.test_module_links.len(), 2);
}

#[test]
fn nested_cfg_test_module_test_is_extracted() {
    let artifact = run(vec![fixture("nested_module.rs")]);
    let evidence = &artifact.evidence;
    assert_eq!(evidence.test_cases.len(), 1);
    assert_eq!(evidence.test_cases[0].name, "nested_test_add");
    assert_eq!(
        evidence.test_cases[0].suite.as_ref(),
        Some(&vec!["tests".to_string()])
    );
    assert_eq!(
        evidence.modules[0].qualified_name.as_deref(),
        Some("crate::calculator::add")
    );
}

#[test]
fn assert_predicate_is_recorded_when_no_call_target_exists() {
    let artifact = run(vec![fixture("nested_module.rs")]);
    let tc = &artifact.evidence.test_cases[0];
    let assertions = tc.assertions.as_ref().expect("should have assertions");

    assert_eq!(assertions.len(), 2);
    assert_eq!(
        assertions[1].target_expression.as_deref(),
        Some("value > 0")
    );
    assert!(assertions[1].target_call_refs.is_none());
}

#[test]
fn empty_array_expected_value_is_recorded() {
    let artifact = run(vec![fixture("empty_array.rs")]);
    let assertion = &artifact.evidence.test_cases[0]
        .assertions
        .as_ref()
        .expect("should have assertions")[0];
    let expected = assertion.expected.as_ref().expect("should have expected");

    assert_eq!(expected.value_kind, ValueKind::ArrayLiteral);
    assert_eq!(expected.literal_class, Some(LiteralClass::EmptyArray));
    assert_eq!(expected.array_items.as_deref(), Some([].as_slice()));
}

#[test]
fn unsupported_macro_is_preserved_as_diagnostic() {
    let artifact = run(vec![fixture("unsupported_macro.rs")]);
    let extensions = artifact.evidence.test_cases[0]
        .extensions
        .as_ref()
        .expect("should have extensions");
    let diagnostics = extensions
        .get("diagnostics")
        .and_then(|value| value.as_array())
        .expect("should have diagnostics");

    assert_eq!(
        diagnostics[0].get("code").and_then(|value| value.as_str()),
        Some("rust_parser.unsupported_macro")
    );
}

#[test]
fn grouped_and_renamed_imports_resolve_and_literals_are_classified() {
    let artifact = run(vec![fixture("literals_and_imports.rs")]);
    let evidence = &artifact.evidence;
    let tc = &evidence.test_cases[0];
    let calls = tc.calls.as_ref().expect("should have calls");
    let parameters = tc.parameters.as_ref().expect("should have parameters");
    let assertions = tc.assertions.as_ref().expect("should have assertions");

    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].callee.text, "renamed_number");
    assert_eq!(
        calls[0].callee.resolution_status,
        ResolutionStatus::Resolved
    );
    assert_eq!(calls[1].callee.text, "identity");
    assert_eq!(
        calls[1].callee.resolution_status,
        ResolutionStatus::Resolved
    );
    assert_eq!(
        evidence
            .modules
            .iter()
            .map(|module| module.qualified_name.as_deref().unwrap_or(""))
            .collect::<Vec<_>>(),
        vec!["crate::values::number", "crate::values::identity"]
    );

    assert_eq!(parameters.len(), 4);
    assert_eq!(parameters[0].literal_class, Some(LiteralClass::Zero));
    assert_eq!(parameters[1].literal_class, Some(LiteralClass::Float));
    assert_eq!(parameters[2].literal_class, Some(LiteralClass::False));
    assert_eq!(
        parameters[3].literal_class,
        Some(LiteralClass::WhitespaceString)
    );
    assert_eq!(
        assertions[1]
            .expected
            .as_ref()
            .and_then(|value| value.literal_class.clone()),
        Some(LiteralClass::EmptyString)
    );
}

#[test]
fn super_paths_are_resolved_relative_to_module_path_without_collapsing_modules() {
    let artifact = run(vec![fixture("super_paths.rs")]);
    let evidence = &artifact.evidence;
    assert_eq!(evidence.test_cases.len(), 2);
    assert_eq!(evidence.modules.len(), 2);
    assert_eq!(evidence.test_module_links.len(), 2);
    assert_eq!(
        evidence
            .modules
            .iter()
            .map(|module| module.qualified_name.as_deref().unwrap_or(""))
            .collect::<Vec<_>>(),
        vec!["crate::first::helper", "crate::second::helper"]
    );

    for tc in &evidence.test_cases {
        let calls = tc.calls.as_ref().expect("should preserve call");
        assert_eq!(calls[0].callee.text, "super::helper");
        assert_eq!(
            calls[0].callee.resolution_status,
            ResolutionStatus::Resolved
        );
        assert!(calls[0].extensions.is_none());
    }
}

#[test]
fn parent_use_is_not_inherited_by_nested_test_module() {
    let artifact = run(vec![fixture("parent_use_not_inherited.rs")]);
    let evidence = &artifact.evidence;
    let calls = evidence.test_cases[0]
        .calls
        .as_ref()
        .expect("should preserve call");

    assert_eq!(calls[0].callee.text, "helper");
    assert_eq!(
        calls[0].callee.resolution_status,
        ResolutionStatus::Unresolved
    );
    assert_call_diagnostic(&calls[0], "rust_parser.unresolved_call");
    assert!(evidence.modules.is_empty());
    assert!(evidence.test_module_links.is_empty());
}

#[test]
fn non_rust_paths_are_skipped() {
    let artifact = run(vec![fixture("not_rust.txt")]);
    assert!(artifact.evidence.test_cases.is_empty());
}

fn assert_call_diagnostic(call: &tgraphy_core::artifact::evidence::Call, expected_code: &str) {
    let diagnostics = call
        .extensions
        .as_ref()
        .and_then(|extensions| extensions.get("diagnostics"))
        .and_then(|value| value.as_array())
        .expect("call should have diagnostics");
    assert_eq!(
        diagnostics[0].get("code").and_then(|value| value.as_str()),
        Some(expected_code)
    );
}

#[test]
fn missing_rust_file_is_reported_as_component_error() {
    let result = Parser::parse(
        &RustParser,
        ParserInput {
            source_paths: vec![fixture("missing.rs")],
            config: None,
        },
    );

    assert!(result.is_err());
    assert!(
        format!("{:?}", result.err().unwrap()).contains("failed to read Rust source"),
        "read failure should not be silent"
    );
}

#[test]
fn parse_failure_is_reported_as_component_error() {
    let result = Parser::parse(
        &RustParser,
        ParserInput {
            source_paths: vec![fixture("parse_error.rs")],
            config: None,
        },
    );

    assert!(result.is_err());
    assert!(
        format!("{:?}", result.err().unwrap()).contains("failed to parse Rust source"),
        "parse failure should not be silent"
    );
}

#[test]
fn empty_input_produces_empty_evidence() {
    let artifact = run(vec![]);
    assert!(artifact.evidence.test_cases.is_empty());
    assert!(artifact.evidence.modules.is_empty());
    assert!(artifact.evidence.test_module_links.is_empty());
}
