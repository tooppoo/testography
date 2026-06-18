use tgraphy_core::io::{
    read_artifact, serialize_assessed, serialize_evidence, write_assessed, write_bytes,
    write_evidence,
};
use tgraphy_core::{ArtifactError, ArtifactKind, ReferenceViolation, parse_artifact};

// ── helpers ──────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("tgraphy_test_{}_{}", std::process::id(), name))
}

// ── valid load / roundtrip ────────────────────────────────────────────────────

#[test]
fn valid_evidence_loads_and_roundtrips() {
    let path = fixture("valid_evidence.json");
    let kind = read_artifact(&path).expect("should load valid evidence artifact");

    let ArtifactKind::Evidence(artifact) = kind else {
        panic!("expected Evidence variant");
    };

    assert_eq!(artifact.schema_version, "0.0.1");
    assert_eq!(artifact.artifact_type, "evidence");

    // Serialize and re-parse to confirm roundtrip.
    let bytes = serialize_evidence(&artifact).expect("serialize should succeed");
    let json = std::str::from_utf8(&bytes).unwrap();
    let kind2 = parse_artifact(json).expect("re-parsed artifact should be valid");
    assert!(matches!(kind2, ArtifactKind::Evidence(_)));
}

#[test]
fn valid_assessed_loads_and_roundtrips() {
    let path = fixture("valid_assessed.json");
    let kind = read_artifact(&path).expect("should load valid assessed artifact");

    let ArtifactKind::Assessed(artifact) = kind else {
        panic!("expected Assessed variant");
    };

    assert_eq!(artifact.schema_version, "0.0.1");
    assert_eq!(artifact.artifact_type, "assessed_artifact");
    assert_eq!(artifact.assessment_layers.len(), 1);
    assert_eq!(artifact.assessment_layers[0].assessments.len(), 1);

    // Roundtrip.
    let bytes = serialize_assessed(&artifact).expect("serialize should succeed");
    let json = std::str::from_utf8(&bytes).unwrap();
    let kind2 = parse_artifact(json).expect("re-parsed artifact should be valid");
    assert!(matches!(kind2, ArtifactKind::Assessed(_)));
}

// ── schema_version checks ─────────────────────────────────────────────────────

#[test]
fn accepted_schema_version_is_0_0_1() {
    let json = include_str!("fixtures/valid_evidence.json");
    assert!(parse_artifact(json).is_ok());
}

#[test]
fn unknown_schema_version_is_rejected() {
    let json = r#"{
        "schema_version": "9.9.9",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {}
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::UnknownSchemaVersion { found }) => {
            assert_eq!(found, "9.9.9");
        }
        other => panic!("expected UnknownSchemaVersion, got {:?}", other),
    }
}

#[test]
fn v0_schema_version_is_rejected() {
    let json = r#"{
        "schema_version": "v0",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {}
    }"#;
    assert!(matches!(
        parse_artifact(json),
        Err(ArtifactError::UnknownSchemaVersion { .. })
    ));
}

#[test]
fn missing_schema_version_is_rejected() {
    let json = r#"{
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {}
    }"#;
    assert!(matches!(
        parse_artifact(json),
        Err(ArtifactError::UnknownSchemaVersion { .. })
    ));
}

// ── artifact_type checks ──────────────────────────────────────────────────────

#[test]
fn unknown_artifact_type_is_rejected_before_schema_validation() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "unknown_type",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {}
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::UnknownArtifactType { found }) => {
            assert_eq!(found, "unknown_type");
        }
        other => panic!("expected UnknownArtifactType, got {:?}", other),
    }
}

#[test]
fn missing_artifact_type_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {}
    }"#;
    assert!(matches!(
        parse_artifact(json),
        Err(ArtifactError::UnknownArtifactType { .. })
    ));
}

// ── schema validation errors ──────────────────────────────────────────────────

#[test]
fn invalid_artifact_json_produces_clear_validation_error() {
    // Missing required `producer` field.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "evidence": {}
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "should have at least one violation");
        }
        other => panic!("expected SchemaViolation, got {:?}", other),
    }
}

// ── JSON output stability ─────────────────────────────────────────────────────

#[test]
fn serialization_is_deterministic() {
    let path = fixture("valid_evidence.json");
    let ArtifactKind::Evidence(artifact) = read_artifact(&path).expect("should load") else {
        panic!()
    };

    let out1 = serialize_evidence(&artifact).unwrap();
    let out2 = serialize_evidence(&artifact).unwrap();
    assert_eq!(out1, out2, "serialization must be deterministic");
}

#[test]
fn serialization_has_no_wall_clock_dependency() {
    // Serialize twice in quick succession; output must not differ.
    let json = include_str!("fixtures/valid_evidence.json");
    let ArtifactKind::Evidence(a) = parse_artifact(json).unwrap() else {
        panic!()
    };
    let s1 = serialize_evidence(&a).unwrap();
    let s2 = serialize_evidence(&a).unwrap();
    assert_eq!(s1, s2);
}

// ── diagnostics ───────────────────────────────────────────────────────────────

#[test]
fn diagnostics_are_represented_in_evidence() {
    let path = fixture("valid_evidence.json");
    let ArtifactKind::Evidence(artifact) = read_artifact(&path).expect("should load") else {
        panic!()
    };

    let diags = artifact.diagnostics.expect("fixture has diagnostics");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "evidence collected");
    assert_eq!(diags[0].code.as_deref(), Some("evidence.collected"));
}

#[test]
fn diagnostics_include_severity_code_message() {
    use tgraphy_core::artifact::DiagnosticLevel;

    let path = fixture("valid_evidence.json");
    let ArtifactKind::Evidence(artifact) = read_artifact(&path).expect("should load") else {
        panic!()
    };

    let d = &artifact.diagnostics.unwrap()[0];
    assert!(matches!(d.level, DiagnosticLevel::Info));
    assert!(d.code.is_some());
    assert!(!d.message.is_empty());
}

// ── reference integrity: duplicate IDs ───────────────────────────────────────

#[test]
fn duplicate_call_id_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "calls": [
                        {"id": "call-dup", "role": "direct_call",
                         "callee": {"text": "f", "resolution_status": "resolved"}},
                        {"id": "call-dup", "role": "direct_call",
                         "callee": {"text": "g", "resolution_status": "resolved"}}
                    ]
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(
                    |v| matches!(v, ReferenceViolation::DuplicateId { id } if id == "call-dup")
                ),
                "expected DuplicateId for call-dup"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── reference integrity: broken refs ─────────────────────────────────────────

#[test]
fn broken_call_ref_in_parameter_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "calls": [
                        {"id": "call-001", "role": "direct_call",
                         "callee": {"text": "f", "resolution_status": "resolved"}}
                    ],
                    "parameters": [
                        {"id": "param-001", "argument_index": 0,
                         "value_kind": "string_literal", "call_ref": "call-MISSING"}
                    ]
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "parameter.call_ref" && id == "call-MISSING"
                )),
                "expected BrokenRef for call-MISSING"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn broken_target_call_ref_in_assertion_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "assertions": [
                        {"id": "assertion-001", "style": "expect_matcher",
                         "target_call_refs": ["call-MISSING"]}
                    ]
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "assertion.target_call_refs" && id == "call-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn broken_evidence_ref_in_assessment_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_artifact",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "assertions": [
                        {"id": "assertion-001", "style": "expect_matcher"}
                    ]
                }
            ]
        },
        "assessment_layers": [
            {
                "schema_version": "0.0.1",
                "id": "layer-001",
                "producer": {"name": "x", "version": "0.1.0", "kind": "static"},
                "assessments": [
                    {
                        "id": "assessment-001",
                        "kind": "static_rule_match",
                        "statement": "test",
                        "evidence_refs": ["assertion-MISSING"]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "assessment.evidence_refs" && id == "assertion-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── reference integrity: call_ref / target_call_refs must be call IDs ─────────

#[test]
fn call_ref_pointing_to_assertion_id_is_rejected() {
    // param.call_ref = "assertion-001" — assertion IDs must not be accepted.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "assertions": [
                        {"id": "assertion-001", "style": "expect_matcher"}
                    ],
                    "parameters": [
                        {"id": "param-001", "argument_index": 0,
                         "value_kind": "string_literal", "call_ref": "assertion-001"}
                    ]
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "parameter.call_ref" && id == "assertion-001"
                )),
                "call_ref pointing to assertion ID should be rejected"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn target_call_refs_pointing_to_param_id_is_rejected() {
    // assertion.target_call_refs = ["param-001"] — parameter IDs must not be accepted.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "calls": [
                        {"id": "call-001", "role": "direct_call",
                         "callee": {"text": "f", "resolution_status": "resolved"}}
                    ],
                    "parameters": [
                        {"id": "param-001", "argument_index": 0, "value_kind": "string_literal",
                         "call_ref": "call-001"}
                    ],
                    "assertions": [
                        {"id": "assertion-001", "style": "expect_matcher",
                         "target_call_refs": ["param-001"]}
                    ]
                }
            ],
            "modules": []
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "assertion.target_call_refs" && id == "param-001"
                )),
                "target_call_refs pointing to param ID should be rejected"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── reference integrity: callee.resolved_module_id ───────────────────────────

#[test]
fn resolved_callee_with_unknown_module_id_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "calls": [
                        {
                            "id": "call-001",
                            "role": "assertion_target_call",
                            "callee": {
                                "text": "createUser",
                                "resolution_status": "resolved",
                                "resolved_module_id": "symbol:src/MISSING.ts#fn"
                            }
                        }
                    ]
                }
            ],
            "modules": []
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "callee.resolved_module_id"
                            && id == "symbol:src/MISSING.ts#fn"
                )),
                "resolved callee pointing to unknown module should be rejected"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn unresolved_callee_with_no_module_id_is_accepted() {
    // resolution_status != resolved → resolved_module_id is not checked.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "calls": [
                        {
                            "id": "call-001",
                            "role": "direct_call",
                            "callee": {
                                "text": "someFunc",
                                "resolution_status": "unresolved"
                            }
                        }
                    ]
                }
            ]
        }
    }"#;
    assert!(parse_artifact(json).is_ok());
}

// ── io: write functions ───────────────────────────────────────────────────────

#[test]
fn write_bytes_creates_file_with_content() {
    let path = temp_path("write_bytes.bin");
    let data = b"hello testography";
    write_bytes(&path, data).expect("write_bytes should succeed");
    let actual = std::fs::read(&path).expect("should read back written file");
    let _ = std::fs::remove_file(&path);
    assert_eq!(actual, data);
}

#[test]
fn write_evidence_creates_readable_artifact() {
    let json = include_str!("fixtures/valid_evidence.json");
    let ArtifactKind::Evidence(artifact) = parse_artifact(json).unwrap() else {
        panic!("expected evidence");
    };
    let path = temp_path("write_evidence.json");
    write_evidence(&artifact, &path).expect("write_evidence should succeed");
    let re_read = read_artifact(&path).expect("re-read should succeed");
    let _ = std::fs::remove_file(&path);
    assert!(matches!(re_read, ArtifactKind::Evidence(_)));
}

#[test]
fn write_assessed_creates_readable_artifact() {
    let json = include_str!("fixtures/valid_assessed.json");
    let ArtifactKind::Assessed(artifact) = parse_artifact(json).unwrap() else {
        panic!("expected assessed");
    };
    let path = temp_path("write_assessed.json");
    write_assessed(&artifact, &path).expect("write_assessed should succeed");
    let re_read = read_artifact(&path).expect("re-read should succeed");
    let _ = std::fs::remove_file(&path);
    assert!(matches!(re_read, ArtifactKind::Assessed(_)));
}

// ── reference integrity: test_module_links ────────────────────────────────────

#[test]
fn broken_test_module_link_test_id_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {
                    "test_id": "test-MISSING",
                    "module_id": "mod-001",
                    "relationship": "directly_called",
                    "confidence": "high"
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "test_module_link.test_id" && id == "test-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn broken_test_module_link_module_id_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}
            ],
            "test_module_links": [
                {
                    "test_id": "test-001",
                    "module_id": "mod-MISSING",
                    "relationship": "directly_called",
                    "confidence": "high"
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "test_module_link.module_id" && id == "mod-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn broken_test_module_link_evidence_refs_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "calls": [
                        {
                            "id": "call-001",
                            "role": "direct_call",
                            "callee": {"text": "f", "resolution_status": "unresolved"}
                        }
                    ]
                }
            ],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {
                    "test_id": "test-001",
                    "module_id": "mod-001",
                    "relationship": "directly_called",
                    "confidence": "high",
                    "evidence_refs": ["call-MISSING"]
                }
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "test_module_link.evidence_refs" && id == "call-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── reference integrity: module_bundles ───────────────────────────────────────

#[test]
fn broken_module_bundle_module_id_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "module_bundles": [{"module_id": "mod-MISSING"}]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "module_bundle.module_id" && id == "mod-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn broken_module_bundle_evidence_refs_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "evidence",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {
            "modules": [{"id": "mod-001", "kind": "file"}],
            "module_bundles": [{"module_id": "mod-001", "evidence_refs": ["ev-MISSING"]}]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "module_bundle.evidence_refs" && id == "ev-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── reference integrity: assessed artifact ────────────────────────────────────

#[test]
fn duplicate_layer_id_in_assessed_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_artifact",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {},
        "assessment_layers": [
            {
                "schema_version": "0.0.1",
                "id": "layer-dup",
                "producer": {"name": "x", "version": "0.1.0", "kind": "static"},
                "assessments": []
            },
            {
                "schema_version": "0.0.1",
                "id": "layer-dup",
                "producer": {"name": "x", "version": "0.1.0", "kind": "static"},
                "assessments": []
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::DuplicateId { id } if id == "layer-dup"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn duplicate_assessment_id_in_assessed_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_artifact",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {},
        "assessment_layers": [
            {
                "schema_version": "0.0.1",
                "id": "layer-001",
                "producer": {"name": "x", "version": "0.1.0", "kind": "static"},
                "assessments": [
                    {"id": "assess-dup", "kind": "static_rule_match", "statement": "s"},
                    {"id": "assess-dup", "kind": "static_rule_match", "statement": "s"}
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::DuplicateId { id } if id == "assess-dup"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn broken_assessment_ref_in_assessed_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_artifact",
        "producer": {"name": "x", "version": "0.1.0"},
        "evidence": {},
        "assessment_layers": [
            {
                "schema_version": "0.0.1",
                "id": "layer-001",
                "producer": {"name": "x", "version": "0.1.0", "kind": "static"},
                "assessments": [
                    {
                        "id": "assess-001",
                        "kind": "static_rule_match",
                        "statement": "s",
                        "assessment_refs": ["assess-MISSING"]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(violations.iter().any(|v| matches!(
                v,
                ReferenceViolation::BrokenRef { field, id }
                    if field == "assessment.assessment_refs" && id == "assess-MISSING"
            )));
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}
