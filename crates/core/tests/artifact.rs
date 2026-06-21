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

// ── staged artifact: parsed_evidence ─────────────────────────────────────────

#[test]
fn valid_parsed_evidence_loads_and_roundtrips() {
    let path = fixture("parsed_evidence/valid.json");
    let kind = read_artifact(&path).expect("should load valid parsed_evidence artifact");

    let ArtifactKind::ParsedEvidence(artifact) = kind else {
        panic!("expected ParsedEvidence variant");
    };

    assert_eq!(artifact.schema_version, "0.0.1");
    assert_eq!(artifact.artifact_type, "parsed_evidence");

    let json = serde_json::to_string(&artifact).expect("serialize should succeed");
    let kind2 = parse_artifact(&json).expect("re-parsed artifact should be valid");
    assert!(matches!(kind2, ArtifactKind::ParsedEvidence(_)));
}

#[test]
fn parsed_evidence_with_module_bundles_at_top_level_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {},
        "module_bundles": []
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "should have schema violation");
        }
        other => panic!(
            "expected SchemaViolation for module_bundles in parsed_evidence, got {:?}",
            other
        ),
    }
}

#[test]
fn parsed_evidence_with_assessment_layers_at_top_level_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {},
        "assessment_layers": []
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "should have schema violation");
        }
        other => panic!(
            "expected SchemaViolation for assessment_layers in parsed_evidence, got {:?}",
            other
        ),
    }
}

#[test]
fn parsed_evidence_link_missing_id_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"test_ref": "test-001", "module_ref": "mod-001"}
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(
                !violations.is_empty(),
                "should have schema violation for missing id"
            );
        }
        other => panic!("expected SchemaViolation, got {:?}", other),
    }
}

#[test]
fn parsed_evidence_duplicate_link_id_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"id": "link-dup", "test_ref": "test-001", "module_ref": "mod-001"},
                {"id": "link-dup", "test_ref": "test-001", "module_ref": "mod-001"}
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(
                    |v| matches!(v, ReferenceViolation::DuplicateId { id } if id == "link-dup")
                ),
                "expected DuplicateId for link-dup"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn parsed_evidence_broken_test_ref_in_link_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-MISSING", "module_ref": "mod-001"}
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "test_module_link.test_ref" && id == "test-MISSING"
                )),
                "expected BrokenRef for test-MISSING"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn parsed_evidence_broken_module_ref_in_link_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-MISSING"}
            ]
        }
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "test_module_link.module_ref" && id == "mod-MISSING"
                )),
                "expected BrokenRef for mod-MISSING"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── staged artifact: module_evidence ─────────────────────────────────────────

#[test]
fn valid_module_evidence_loads_and_roundtrips() {
    let path = fixture("module_evidence/valid.json");
    let kind = read_artifact(&path).expect("should load valid module_evidence artifact");

    let ArtifactKind::ModuleEvidence(artifact) = kind else {
        panic!("expected ModuleEvidence variant");
    };

    assert_eq!(artifact.schema_version, "0.0.1");
    assert_eq!(artifact.artifact_type, "module_evidence");
    assert_eq!(artifact.module_bundles.len(), 1);
    assert_eq!(artifact.module_bundles[0].tests.len(), 1);

    let json = serde_json::to_string(&artifact).expect("serialize should succeed");
    let kind2 = parse_artifact(&json).expect("re-parsed artifact should be valid");
    assert!(matches!(kind2, ArtifactKind::ModuleEvidence(_)));
}

#[test]
fn module_evidence_with_assessment_layers_at_top_level_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": []
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "should have schema violation");
        }
        other => panic!(
            "expected SchemaViolation for assessment_layers in module_evidence, got {:?}",
            other
        ),
    }
}

#[test]
fn module_evidence_missing_module_bundles_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {}
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "module_bundles is required");
        }
        other => panic!("expected SchemaViolation, got {:?}", other),
    }
}

#[test]
fn module_evidence_bundle_with_missing_module_ref_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-001"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-MISSING",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "module_bundle.module_ref" && id == "mod-MISSING"
                )),
                "expected BrokenRef for mod-MISSING"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_uncovered_module_is_reported() {
    // mod-002 exists in modules but has no bundle.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [
                {"id": "mod-001", "kind": "file"},
                {"id": "mod-002", "kind": "file"}
            ],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-001"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field.contains("not covered") && id == "mod-002"
                )),
                "expected BrokenRef for uncovered mod-002, got {:?}",
                violations
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_uncovered_link_is_reported() {
    // link-002 exists in test_module_links but is not referenced by any bundle test.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-001"},
                {"id": "link-002", "test_ref": "test-001", "module_ref": "mod-001"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field.contains("not covered") && id == "link-002"
                )),
                "expected BrokenRef for uncovered link-002, got {:?}",
                violations
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_duplicate_link_ref_across_bundles_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [
                {"id": "mod-001", "kind": "file"},
                {"id": "mod-002", "kind": "file"}
            ],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-001"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            },
            {
                "module_ref": "mod-002",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::DuplicateId { id } if id == "link-001"
                )),
                "expected DuplicateId for link-001 used in two bundles"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_test_ref_mismatch_with_resolved_link_is_reported() {
    // bundle_test.test_ref = "test-001" but link-001.test_ref = "test-002"
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [
                {"id": "test-001", "name": "t1", "source": {"file": "a.ts"}},
                {"id": "test-002", "name": "t2", "source": {"file": "a.ts"}}
            ],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-002", "module_ref": "mod-001"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, .. }
                        if field.contains("test_ref mismatch")
                )),
                "expected BrokenRef for test_ref mismatch, got {:?}",
                violations
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_module_ref_mismatch_with_resolved_link_is_reported() {
    // bundle module_ref = "mod-001" but link-001.module_ref = "mod-002"
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [
                {"id": "mod-001", "kind": "file"},
                {"id": "mod-002", "kind": "file"}
            ],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-002"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            },
            {
                "module_ref": "mod-002",
                "tests": []
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, .. }
                        if field.contains("module_ref mismatch")
                )),
                "expected BrokenRef for module_ref mismatch, got {:?}",
                violations
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_same_test_ref_in_multiple_bundles_via_distinct_links_is_accepted() {
    // test-001 links to both mod-001 and mod-002 through distinct links — allowed.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [
                {"id": "mod-001", "kind": "file"},
                {"id": "mod-002", "kind": "file"}
            ],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-001"},
                {"id": "link-002", "test_ref": "test-001", "module_ref": "mod-002"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            },
            {
                "module_ref": "mod-002",
                "tests": [{"test_ref": "test-001", "link_ref": "link-002"}]
            }
        ]
    }"#;
    assert!(
        parse_artifact(json).is_ok(),
        "same test_ref in multiple bundles via distinct links should be accepted"
    );
}

#[test]
fn module_evidence_duplicate_module_ref_in_bundles_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": []
        },
        "module_bundles": [
            {"module_ref": "mod-001", "tests": []},
            {"module_ref": "mod-001", "tests": []}
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::DuplicateId { id } if id == "mod-001"
                )),
                "expected DuplicateId for duplicate module_ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn module_evidence_module_with_no_links_has_empty_tests_bundle() {
    // mod-001 has no links — represented by a bundle with empty tests array.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "module_evidence",
        "evidence": {
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": []
        },
        "module_bundles": [
            {"module_ref": "mod-001", "tests": []}
        ]
    }"#;
    assert!(
        parse_artifact(json).is_ok(),
        "module with no links should be represented as bundle with empty tests"
    );
}

// ── staged artifact: assessed_module_evidence ─────────────────────────────────

#[test]
fn valid_assessed_module_evidence_loads_and_roundtrips() {
    let path = fixture("assessed_module_evidence/valid.json");
    let kind = read_artifact(&path).expect("should load valid assessed_module_evidence artifact");

    let ArtifactKind::AssessedModuleEvidence(artifact) = kind else {
        panic!("expected AssessedModuleEvidence variant");
    };

    assert_eq!(artifact.schema_version, "0.0.1");
    assert_eq!(artifact.artifact_type, "assessed_module_evidence");
    assert_eq!(artifact.assessment_layers.len(), 1);
    assert_eq!(artifact.assessment_layers[0].findings.len(), 1);

    let json = serde_json::to_string(&artifact).expect("serialize should succeed");
    let kind2 = parse_artifact(&json).expect("re-parsed artifact should be valid");
    assert!(matches!(kind2, ArtifactKind::AssessedModuleEvidence(_)));
}

#[test]
fn assessed_module_evidence_missing_assessment_layers_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": []
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "assessment_layers is required");
        }
        other => panic!("expected SchemaViolation, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_empty_assessment_layers_is_accepted() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": []
    }"#;
    assert!(
        parse_artifact(json).is_ok(),
        "empty assessment_layers should be accepted"
    );
}

#[test]
fn assessed_module_evidence_duplicate_layer_id_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-dup",
                "evaluator": {"id": "ev-001"},
                "findings": []
            },
            {
                "id": "layer-dup",
                "evaluator": {"id": "ev-001"},
                "findings": []
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(
                    |v| matches!(v, ReferenceViolation::DuplicateId { id } if id == "layer-dup")
                ),
                "expected DuplicateId for layer-dup"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_duplicate_finding_id_within_layer_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {"id": "finding-dup", "level": "info", "message": "a"},
                    {"id": "finding-dup", "level": "warning", "message": "b"}
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::DuplicateId { id } if id == "finding-dup"
                )),
                "expected DuplicateId for finding-dup"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_finding_level_error_does_not_make_artifact_schema_invalid() {
    // findings[].level = "error" is a severity concept, not a schema validation failure.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {"id": "finding-001", "level": "error", "message": "critical issue found"}
                ]
            }
        ]
    }"#;
    assert!(
        parse_artifact(json).is_ok(),
        "finding with level=error should not make the artifact schema-invalid"
    );
}

#[test]
fn assessed_module_evidence_invalid_finding_level_is_rejected() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {"id": "finding-001", "level": "critical", "message": "bad level"}
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::SchemaViolation(violations)) => {
            assert!(!violations.is_empty(), "invalid level should be rejected");
        }
        other => panic!(
            "expected SchemaViolation for invalid level, got {:?}",
            other
        ),
    }
}

// ── staged integrity: internal cross-refs inside parsed_evidence (P1) ─────────

#[test]
fn parsed_evidence_broken_resolved_module_id_in_call_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
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
                                "text": "fn",
                                "resolution_status": "resolved",
                                "resolved_module_id": "mod-MISSING"
                            }
                        }
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
                        if field == "callee.resolved_module_id" && id == "mod-MISSING"
                )),
                "expected BrokenRef for mod-MISSING in staged evidence"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn parsed_evidence_broken_parameter_call_ref_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "parameters": [
                        {
                            "id": "param-001",
                            "argument_index": 0,
                            "value_kind": "string_literal",
                            "call_ref": "call-MISSING"
                        }
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
                "expected BrokenRef for call-MISSING in parameter.call_ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn parsed_evidence_broken_assertion_target_call_ref_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "test_cases": [
                {
                    "id": "test-001",
                    "name": "t1",
                    "source": {"file": "a.ts"},
                    "assertions": [
                        {
                            "id": "assertion-001",
                            "style": "expect_matcher",
                            "target_call_refs": ["call-MISSING"]
                        }
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
                        if field == "assertion.target_call_refs" && id == "call-MISSING"
                )),
                "expected BrokenRef for call-MISSING in assertion.target_call_refs"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn parsed_evidence_broken_link_evidence_refs_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "parsed_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {
                    "id": "link-001",
                    "test_ref": "test-001",
                    "module_ref": "mod-001",
                    "evidence_refs": ["ev-MISSING"]
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
                        if field == "test_module_link.evidence_refs" && id == "ev-MISSING"
                )),
                "expected BrokenRef for ev-MISSING in test_module_link.evidence_refs"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

// ── staged integrity: finding subject refs in assessed_module_evidence (P2) ───

#[test]
fn assessed_module_evidence_subject_ref_to_missing_test_case_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "info",
                        "message": "test",
                        "subjects": [{"kind": "test_case", "ref": "test-MISSING"}]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "finding_subject.ref" && id == "test-MISSING"
                )),
                "expected BrokenRef for test-MISSING subject ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_subject_ref_to_missing_module_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "warning",
                        "message": "test",
                        "subjects": [{"kind": "module", "ref": "mod-MISSING"}]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "finding_subject.ref" && id == "mod-MISSING"
                )),
                "expected BrokenRef for mod-MISSING subject ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_subject_ref_to_missing_link_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "error",
                        "message": "test",
                        "subjects": [{"kind": "test_module_link", "ref": "link-MISSING"}]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    ReferenceViolation::BrokenRef { field, id }
                        if field == "finding_subject.ref" && id == "link-MISSING"
                )),
                "expected BrokenRef for link-MISSING subject ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_artifact_subject_without_ref_is_accepted() {
    // artifact-level subject needs no ref.
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "info",
                        "message": "artifact-level finding",
                        "subjects": [{"kind": "artifact"}]
                    }
                ]
            }
        ]
    }"#;
    assert!(
        parse_artifact(json).is_ok(),
        "artifact subject without ref should be accepted"
    );
}

#[test]
fn assessed_module_evidence_valid_subject_refs_are_accepted() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {
            "test_cases": [{"id": "test-001", "name": "t1", "source": {"file": "a.ts"}}],
            "modules": [{"id": "mod-001", "kind": "file"}],
            "test_module_links": [
                {"id": "link-001", "test_ref": "test-001", "module_ref": "mod-001"}
            ]
        },
        "module_bundles": [
            {
                "module_ref": "mod-001",
                "tests": [{"test_ref": "test-001", "link_ref": "link-001"}]
            }
        ],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "info",
                        "message": "all valid refs",
                        "subjects": [
                            {"kind": "test_case", "ref": "test-001"},
                            {"kind": "module", "ref": "mod-001"},
                            {"kind": "test_module_link", "ref": "link-001"},
                            {"kind": "artifact"}
                        ]
                    }
                ]
            }
        ]
    }"#;
    assert!(
        parse_artifact(json).is_ok(),
        "valid subject refs should all be accepted"
    );
}

// ── staged integrity: entity subjects missing ref (P2 follow-up) ─────────────

#[test]
fn assessed_module_evidence_test_case_subject_without_ref_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "info",
                        "message": "missing ref on test_case subject",
                        "subjects": [{"kind": "test_case"}]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations
                    .iter()
                    .any(|v| matches!(v, ReferenceViolation::MissingRef { field } if field == "finding_subject.ref")),
                "expected MissingRef for test_case subject without ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_module_subject_without_ref_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "warning",
                        "message": "missing ref on module subject",
                        "subjects": [{"kind": "module"}]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations
                    .iter()
                    .any(|v| matches!(v, ReferenceViolation::MissingRef { field } if field == "finding_subject.ref")),
                "expected MissingRef for module subject without ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}

#[test]
fn assessed_module_evidence_link_subject_without_ref_is_reported() {
    let json = r#"{
        "schema_version": "0.0.1",
        "artifact_type": "assessed_module_evidence",
        "evidence": {},
        "module_bundles": [],
        "assessment_layers": [
            {
                "id": "layer-001",
                "evaluator": {"id": "ev-001"},
                "findings": [
                    {
                        "id": "finding-001",
                        "level": "error",
                        "message": "missing ref on test_module_link subject",
                        "subjects": [{"kind": "test_module_link"}]
                    }
                ]
            }
        ]
    }"#;
    match parse_artifact(json) {
        Err(ArtifactError::ReferenceIntegrity(violations)) => {
            assert!(
                violations
                    .iter()
                    .any(|v| matches!(v, ReferenceViolation::MissingRef { field } if field == "finding_subject.ref")),
                "expected MissingRef for test_module_link subject without ref"
            );
        }
        other => panic!("expected ReferenceIntegrity, got {:?}", other),
    }
}
