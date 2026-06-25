use tgraphy_reporter_json::{ReporterInput, render};

fn load_input(fixture_name: &str) -> ReporterInput {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture_name);
    let json = std::fs::read_to_string(&path).expect("fixture should be readable");
    serde_json::from_str(&json).expect("fixture should deserialize as ReporterInput")
}

// ── output shape ──────────────────────────────────────────────────────────────

#[test]
fn render_outputs_artifact_not_wrapper() {
    let input = load_input("minimal_input.json");
    let output = render(input);
    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");

    assert!(
        parsed.get("artifact").is_none(),
        "output should not contain the ReporterInput wrapper 'artifact' key"
    );
    assert_eq!(
        parsed.get("artifact_type").and_then(|v| v.as_str()),
        Some("assessed_module_evidence"),
        "output should be the artifact itself"
    );
}

#[test]
fn render_output_ends_with_newline() {
    let input = load_input("minimal_input.json");
    let output = render(input);
    assert!(
        output.ends_with(b"\n"),
        "output should end with a trailing newline"
    );
}

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
fn render_preserves_assessment_layers() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");

    let layers = parsed["assessment_layers"]
        .as_array()
        .expect("assessment_layers should be an array");
    assert_eq!(layers.len(), 1, "should preserve one assessment layer");
    assert_eq!(layers[0]["id"], "rust-static-0");
    assert_eq!(layers[0]["evaluator"]["id"], "rust-static");
}

#[test]
fn render_preserves_module_bundles() {
    let input = load_input("with_findings_input.json");
    let output = render(input);
    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");

    let bundles = parsed["module_bundles"]
        .as_array()
        .expect("module_bundles should be an array");
    assert_eq!(bundles.len(), 1);
    assert_eq!(bundles[0]["module_ref"], "mod-001");
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

#[test]
fn render_minimal_produces_expected_fixture() {
    let input = load_input("minimal_input.json");
    let output = render(input);
    let text = std::str::from_utf8(&output).expect("valid UTF-8");

    let expected = "{\n  \"schema_version\": \"0.0.1\",\n  \"artifact_type\": \"assessed_module_evidence\",\n  \"evidence\": {},\n  \"module_bundles\": [],\n  \"assessment_layers\": []\n}\n";
    assert_eq!(
        text, expected,
        "minimal render should match expected fixture"
    );
}
