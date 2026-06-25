use tgraphy_reporter_markdown::{ReporterInput, render};

fn load_input(fixture_name: &str) -> ReporterInput {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture_name);
    let json = std::fs::read_to_string(&path).expect("fixture should be readable");
    serde_json::from_str(&json).expect("fixture should deserialize as ReporterInput")
}

// ── output format ─────────────────────────────────────────────────────────────

#[test]
fn render_output_is_valid_utf8() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    assert!(
        std::str::from_utf8(&output).is_ok(),
        "output should be valid UTF-8"
    );
}

#[test]
fn render_output_uses_lf_line_endings() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        !text.contains('\r'),
        "output should use LF line endings, not CRLF"
    );
}

#[test]
fn render_is_deterministic() {
    let input_a = load_input("with_findings_input.json");
    let input_b = load_input("with_findings_input.json");
    let out_a = render(input_a);
    let out_b = render(input_b);
    assert_eq!(
        out_a, out_b,
        "render should produce identical output for identical input"
    );
}

// ── section order ─────────────────────────────────────────────────────────────

#[test]
fn render_sections_appear_in_stable_order() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");

    let summary_pos = text.find("# Summary").expect("Summary section present");
    let modules_pos = text.find("# Modules").expect("Modules section present");
    let layers_pos = text
        .find("# Assessment Layers")
        .expect("Assessment Layers section present");
    let findings_pos = text.find("# Findings").expect("Findings section present");

    assert!(
        summary_pos < modules_pos,
        "Summary should appear before Modules"
    );
    assert!(
        modules_pos < layers_pos,
        "Modules should appear before Assessment Layers"
    );
    assert!(
        layers_pos < findings_pos,
        "Assessment Layers should appear before Findings"
    );
}

// ── content ───────────────────────────────────────────────────────────────────

#[test]
fn render_includes_module_ref() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("mod-001"),
        "output should include module_ref 'mod-001'"
    );
}

#[test]
fn render_includes_linked_test_ref() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("tc-001"),
        "output should include linked test_ref 'tc-001'"
    );
}

#[test]
fn render_includes_evaluator_identity() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("rust-static"),
        "output should include evaluator id 'rust-static'"
    );
}

#[test]
fn render_includes_finding_level() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(text.contains("info"), "output should include finding level");
}

#[test]
fn render_includes_finding_rule_id() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("rust.assert.predicate_only_assertion"),
        "output should include finding rule_id"
    );
}

#[test]
fn render_includes_finding_confidence() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("high"),
        "output should include finding confidence"
    );
}

#[test]
fn render_includes_finding_message() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("predicate-only assertion"),
        "output should include finding message"
    );
}

#[test]
fn render_includes_finding_subject() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("a-001"),
        "output should include finding subject ref"
    );
}

#[test]
fn render_includes_finding_rationale() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");
    assert!(
        text.contains("assert_eq!"),
        "output should include finding rationale"
    );
}

// ── exact fixture output ──────────────────────────────────────────────────────

#[test]
fn render_with_findings_produces_exact_expected_output() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let actual = std::str::from_utf8(&output).expect("valid UTF-8");

    let expected = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/with_findings_expected.md"),
    )
    .expect("expected fixture should be readable");

    assert_eq!(
        actual, expected,
        "with_findings render should match expected fixture exactly"
    );
}

#[test]
fn render_minimal_produces_expected_output() {
    let input = load_input("minimal_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");

    let expected = "\
# Summary\n\
\n\
- Modules: 0\n\
- Assessment layers: 0\n\
- Findings: 0\n\
\n\
# Modules\n\
\n\
_No modules._\n\
\n\
# Assessment Layers\n\
\n\
_No assessment layers._\n\
\n\
# Findings\n\
\n\
_No findings._\n\
\n";

    assert_eq!(
        text, expected,
        "minimal render should match expected output"
    );
}

// ── absent optional fields ────────────────────────────────────────────────────

#[test]
fn render_does_not_invent_absent_rule_id() {
    let input_json = r#"{
      "artifact": {
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
          {
            "id": "layer-0",
            "evaluator": { "id": "test-eval" },
            "findings": [
              {
                "id": "f-001",
                "level": "warning",
                "message": "something is off"
              }
            ]
          }
        ]
      },
      "config": null
    }"#;
    let input: ReporterInput = serde_json::from_str(input_json).expect("parse fixture");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");

    assert!(
        !text.contains("Rule"),
        "absent rule_id should not appear in output, got:\n{text}"
    );
    assert!(
        !text.contains("Confidence"),
        "absent confidence should not appear in output"
    );
    assert!(
        !text.contains("Rationale"),
        "absent rationale should not appear in output"
    );
    assert!(
        text.contains("something is off"),
        "message should still appear"
    );
}
