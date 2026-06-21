use std::collections::HashSet;

use crate::artifact::assessed::AssessedArtifact;
use crate::artifact::evidence::{Evidence, EvidenceArtifact, ResolutionStatus};
use crate::artifact::staged::{
    AssessedModuleEvidenceArtifact, ModuleEvidenceArtifact, ParsedEvidenceArtifact, StagedEvidence,
    SubjectKind,
};

use super::ReferenceViolation;

/// IDs collected from all evidence items in an Evidence struct.
struct EvidenceIds {
    test_case_ids: HashSet<String>,
    module_ids: HashSet<String>,
    /// Only call IDs — used to validate `call_ref` and `target_call_refs`.
    call_ids: HashSet<String>,
    /// All granular evidence IDs: call, parameter, assertion IDs — used for
    /// general evidence_refs validation and cross-type duplicate detection.
    evidence_item_ids: HashSet<String>,
}

impl EvidenceIds {
    fn collect(evidence: &Evidence) -> (Self, Vec<ReferenceViolation>) {
        let mut test_case_ids = HashSet::new();
        let mut module_ids = HashSet::new();
        let mut call_ids = HashSet::new();
        let mut evidence_item_ids = HashSet::new();
        let mut violations = vec![];

        for tc in evidence.test_cases.iter().flatten() {
            if !test_case_ids.insert(tc.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId { id: tc.id.clone() });
            }
            for call in tc.calls.iter().flatten() {
                if !evidence_item_ids.insert(call.id.clone()) {
                    violations.push(ReferenceViolation::DuplicateId {
                        id: call.id.clone(),
                    });
                }
                // Track call IDs separately regardless of whether a duplicate
                // was reported; reference checks still need to know the ID exists.
                call_ids.insert(call.id.clone());
            }
            for param in tc.parameters.iter().flatten() {
                if !evidence_item_ids.insert(param.id.clone()) {
                    violations.push(ReferenceViolation::DuplicateId {
                        id: param.id.clone(),
                    });
                }
            }
            for assertion in tc.assertions.iter().flatten() {
                if !evidence_item_ids.insert(assertion.id.clone()) {
                    violations.push(ReferenceViolation::DuplicateId {
                        id: assertion.id.clone(),
                    });
                }
            }
        }

        for module in evidence.modules.iter().flatten() {
            if !module_ids.insert(module.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId {
                    id: module.id.clone(),
                });
            }
        }

        (
            Self {
                test_case_ids,
                module_ids,
                call_ids,
                evidence_item_ids,
            },
            violations,
        )
    }
}

fn check_evidence_refs(evidence: &Evidence, ids: &EvidenceIds) -> Vec<ReferenceViolation> {
    let mut violations = vec![];

    for tc in evidence.test_cases.iter().flatten() {
        for call in tc.calls.iter().flatten() {
            // When a callee is resolved, its resolved_module_id must point to a
            // known module.
            if call.callee.resolution_status == ResolutionStatus::Resolved
                && let Some(ref mid) = call.callee.resolved_module_id
                && !ids.module_ids.contains(mid.as_str())
            {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "callee.resolved_module_id".to_string(),
                    id: mid.clone(),
                });
            }
        }

        for param in tc.parameters.iter().flatten() {
            // call_ref must point to a call, not just any evidence item.
            if let Some(ref call_ref) = param.call_ref
                && !ids.call_ids.contains(call_ref.as_str())
            {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "parameter.call_ref".to_string(),
                    id: call_ref.clone(),
                });
            }
        }

        for assertion in tc.assertions.iter().flatten() {
            // target_call_refs must point to calls, not parameters or assertions.
            for target_ref in assertion.target_call_refs.iter().flatten() {
                if !ids.call_ids.contains(target_ref.as_str()) {
                    violations.push(ReferenceViolation::BrokenRef {
                        field: "assertion.target_call_refs".to_string(),
                        id: target_ref.clone(),
                    });
                }
            }
        }
    }

    for link in evidence.test_module_links.iter().flatten() {
        if !ids.test_case_ids.contains(link.test_id.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "test_module_link.test_id".to_string(),
                id: link.test_id.clone(),
            });
        }
        if !ids.module_ids.contains(link.module_id.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "test_module_link.module_id".to_string(),
                id: link.module_id.clone(),
            });
        }
        for ev_ref in link.evidence_refs.iter().flatten() {
            if !ids.evidence_item_ids.contains(ev_ref.as_str()) {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "test_module_link.evidence_refs".to_string(),
                    id: ev_ref.clone(),
                });
            }
        }
    }

    for bundle in evidence.module_bundles.iter().flatten() {
        if !ids.module_ids.contains(bundle.module_id.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "module_bundle.module_id".to_string(),
                id: bundle.module_id.clone(),
            });
        }
        for ev_ref in bundle.evidence_refs.iter().flatten() {
            if !ids.evidence_item_ids.contains(ev_ref.as_str()) {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "module_bundle.evidence_refs".to_string(),
                    id: ev_ref.clone(),
                });
            }
        }
    }

    violations
}

pub fn check_evidence_integrity(artifact: &EvidenceArtifact) -> Vec<ReferenceViolation> {
    let (ids, mut violations) = EvidenceIds::collect(&artifact.evidence);
    violations.extend(check_evidence_refs(&artifact.evidence, &ids));
    violations
}

pub fn check_assessed_integrity(artifact: &AssessedArtifact) -> Vec<ReferenceViolation> {
    let (ids, mut violations) = EvidenceIds::collect(&artifact.evidence);
    violations.extend(check_evidence_refs(&artifact.evidence, &ids));

    // Collect all assessment IDs across all layers for cross-layer refs.
    let mut assessment_ids: HashSet<String> = HashSet::new();
    let mut layer_ids: HashSet<String> = HashSet::new();

    for layer in &artifact.assessment_layers {
        if !layer_ids.insert(layer.id.clone()) {
            violations.push(ReferenceViolation::DuplicateId {
                id: layer.id.clone(),
            });
        }
        for assessment in &layer.assessments {
            if !assessment_ids.insert(assessment.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId {
                    id: assessment.id.clone(),
                });
            }
        }
    }

    for layer in &artifact.assessment_layers {
        for assessment in &layer.assessments {
            for ev_ref in assessment.evidence_refs.iter().flatten() {
                if !ids.evidence_item_ids.contains(ev_ref.as_str()) {
                    violations.push(ReferenceViolation::BrokenRef {
                        field: "assessment.evidence_refs".to_string(),
                        id: ev_ref.clone(),
                    });
                }
            }
            for a_ref in assessment.assessment_refs.iter().flatten() {
                if !assessment_ids.contains(a_ref.as_str()) {
                    violations.push(ReferenceViolation::BrokenRef {
                        field: "assessment.assessment_refs".to_string(),
                        id: a_ref.clone(),
                    });
                }
            }
        }
    }

    violations
}

// ── Staged artifact integrity ────────────────────────────────────────────────

/// IDs collected from a `StagedEvidence` struct.
struct StagedEvidenceIds {
    test_case_ids: HashSet<String>,
    module_ids: HashSet<String>,
    link_ids: HashSet<String>,
    /// Call IDs only — used to validate `call_ref` and `target_call_refs`.
    call_ids: HashSet<String>,
    /// All granular item IDs (calls, parameters, assertions) — used for `evidence_refs`.
    evidence_item_ids: HashSet<String>,
}

impl StagedEvidenceIds {
    fn collect(evidence: &StagedEvidence) -> (Self, Vec<ReferenceViolation>) {
        let mut test_case_ids = HashSet::new();
        let mut module_ids = HashSet::new();
        let mut link_ids = HashSet::new();
        let mut call_ids = HashSet::new();
        let mut evidence_item_ids = HashSet::new();
        let mut violations = vec![];

        for tc in evidence.test_cases.iter().flatten() {
            if !test_case_ids.insert(tc.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId { id: tc.id.clone() });
            }
            for call in tc.calls.iter().flatten() {
                if !evidence_item_ids.insert(call.id.clone()) {
                    violations.push(ReferenceViolation::DuplicateId {
                        id: call.id.clone(),
                    });
                }
                call_ids.insert(call.id.clone());
            }
            for param in tc.parameters.iter().flatten() {
                if !evidence_item_ids.insert(param.id.clone()) {
                    violations.push(ReferenceViolation::DuplicateId {
                        id: param.id.clone(),
                    });
                }
            }
            for assertion in tc.assertions.iter().flatten() {
                if !evidence_item_ids.insert(assertion.id.clone()) {
                    violations.push(ReferenceViolation::DuplicateId {
                        id: assertion.id.clone(),
                    });
                }
            }
        }

        for module in evidence.modules.iter().flatten() {
            if !module_ids.insert(module.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId {
                    id: module.id.clone(),
                });
            }
        }

        for link in evidence.test_module_links.iter().flatten() {
            if !link_ids.insert(link.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId {
                    id: link.id.clone(),
                });
            }
        }

        (
            Self {
                test_case_ids,
                module_ids,
                link_ids,
                call_ids,
                evidence_item_ids,
            },
            violations,
        )
    }
}

fn check_staged_evidence_refs(
    evidence: &StagedEvidence,
    ids: &StagedEvidenceIds,
) -> Vec<ReferenceViolation> {
    let mut violations = vec![];

    for tc in evidence.test_cases.iter().flatten() {
        for call in tc.calls.iter().flatten() {
            if call.callee.resolution_status == ResolutionStatus::Resolved
                && let Some(ref mid) = call.callee.resolved_module_id
                && !ids.module_ids.contains(mid.as_str())
            {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "callee.resolved_module_id".to_string(),
                    id: mid.clone(),
                });
            }
        }
        for param in tc.parameters.iter().flatten() {
            if let Some(ref call_ref) = param.call_ref
                && !ids.call_ids.contains(call_ref.as_str())
            {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "parameter.call_ref".to_string(),
                    id: call_ref.clone(),
                });
            }
        }
        for assertion in tc.assertions.iter().flatten() {
            for target_ref in assertion.target_call_refs.iter().flatten() {
                if !ids.call_ids.contains(target_ref.as_str()) {
                    violations.push(ReferenceViolation::BrokenRef {
                        field: "assertion.target_call_refs".to_string(),
                        id: target_ref.clone(),
                    });
                }
            }
        }
    }

    for link in evidence.test_module_links.iter().flatten() {
        if !ids.test_case_ids.contains(link.test_ref.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "test_module_link.test_ref".to_string(),
                id: link.test_ref.clone(),
            });
        }
        if !ids.module_ids.contains(link.module_ref.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "test_module_link.module_ref".to_string(),
                id: link.module_ref.clone(),
            });
        }
        for ev_ref in link.evidence_refs.iter().flatten() {
            if !ids.evidence_item_ids.contains(ev_ref.as_str()) {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "test_module_link.evidence_refs".to_string(),
                    id: ev_ref.clone(),
                });
            }
        }
    }

    violations
}

pub fn check_parsed_evidence_integrity(
    artifact: &ParsedEvidenceArtifact,
) -> Vec<ReferenceViolation> {
    let (ids, mut violations) = StagedEvidenceIds::collect(&artifact.evidence);
    violations.extend(check_staged_evidence_refs(&artifact.evidence, &ids));
    violations
}

pub fn check_module_evidence_integrity(
    artifact: &ModuleEvidenceArtifact,
) -> Vec<ReferenceViolation> {
    let (ids, mut violations) = StagedEvidenceIds::collect(&artifact.evidence);
    violations.extend(check_staged_evidence_refs(&artifact.evidence, &ids));
    violations.extend(check_module_bundles(
        &artifact.evidence,
        &artifact.module_bundles,
        &ids,
    ));
    violations
}

pub fn check_assessed_module_evidence_integrity(
    artifact: &AssessedModuleEvidenceArtifact,
) -> Vec<ReferenceViolation> {
    let (ids, mut violations) = StagedEvidenceIds::collect(&artifact.evidence);
    violations.extend(check_staged_evidence_refs(&artifact.evidence, &ids));
    violations.extend(check_module_bundles(
        &artifact.evidence,
        &artifact.module_bundles,
        &ids,
    ));
    violations.extend(check_finding_layers(&artifact.assessment_layers, &ids));
    violations
}

fn check_module_bundles(
    evidence: &StagedEvidence,
    bundles: &[crate::artifact::staged::StagedModuleBundle],
    ids: &StagedEvidenceIds,
) -> Vec<ReferenceViolation> {
    let mut violations = vec![];

    // Build the link map: link_id -> (test_ref, module_ref) for cross-checking.
    let mut link_map: std::collections::HashMap<&str, (&str, &str)> =
        std::collections::HashMap::new();
    for link in evidence.test_module_links.iter().flatten() {
        link_map.insert(
            link.id.as_str(),
            (link.test_ref.as_str(), link.module_ref.as_str()),
        );
    }

    // Every modules[].id must appear exactly once in module_bundles[].module_ref.
    let mut seen_module_refs: HashSet<String> = HashSet::new();
    // Every test_module_links[].id must appear exactly once in module_bundles[].tests[].link_ref.
    let mut seen_link_refs: HashSet<String> = HashSet::new();

    for bundle in bundles {
        // module_ref must resolve to a known module.
        if !ids.module_ids.contains(bundle.module_ref.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "module_bundle.module_ref".to_string(),
                id: bundle.module_ref.clone(),
            });
        }

        // module_ref must be unique across all bundles.
        if !seen_module_refs.insert(bundle.module_ref.clone()) {
            violations.push(ReferenceViolation::DuplicateId {
                id: bundle.module_ref.clone(),
            });
        }

        for test in &bundle.tests {
            // test_ref must resolve to a known test case.
            if !ids.test_case_ids.contains(test.test_ref.as_str()) {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "bundle_test.test_ref".to_string(),
                    id: test.test_ref.clone(),
                });
            }

            // link_ref must resolve to a known test_module_link.
            if !ids.link_ids.contains(test.link_ref.as_str()) {
                violations.push(ReferenceViolation::BrokenRef {
                    field: "bundle_test.link_ref".to_string(),
                    id: test.link_ref.clone(),
                });
            }

            // link_ref must be globally unique across all module bundles.
            if !seen_link_refs.insert(test.link_ref.clone()) {
                violations.push(ReferenceViolation::DuplicateId {
                    id: test.link_ref.clone(),
                });
            }

            // Cross-check: the resolved link's test_ref must match bundle_test.test_ref,
            // and the resolved link's module_ref must match the parent bundle's module_ref.
            if let Some(&(link_test_ref, link_module_ref)) = link_map.get(test.link_ref.as_str()) {
                if link_test_ref != test.test_ref.as_str() {
                    violations.push(ReferenceViolation::BrokenRef {
                        field: "bundle_test.test_ref mismatch with resolved link".to_string(),
                        id: test.link_ref.clone(),
                    });
                }
                if link_module_ref != bundle.module_ref.as_str() {
                    violations.push(ReferenceViolation::BrokenRef {
                        field: "bundle_test.link_ref module_ref mismatch with parent bundle"
                            .to_string(),
                        id: test.link_ref.clone(),
                    });
                }
            }
        }
    }

    // Every modules[].id must appear exactly once in module_bundles[].module_ref.
    for module_id in &ids.module_ids {
        if !seen_module_refs.contains(module_id.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "evidence.modules[].id not covered by any module_bundle".to_string(),
                id: module_id.clone(),
            });
        }
    }

    // Every test_module_links[].id must appear exactly once in module_bundles[].tests[].link_ref.
    for link_id in &ids.link_ids {
        if !seen_link_refs.contains(link_id.as_str()) {
            violations.push(ReferenceViolation::BrokenRef {
                field: "evidence.test_module_links[].id not covered by any bundle test".to_string(),
                id: link_id.clone(),
            });
        }
    }

    violations
}

fn check_finding_layers(
    layers: &[crate::artifact::staged::FindingLayer],
    ids: &StagedEvidenceIds,
) -> Vec<ReferenceViolation> {
    let mut violations = vec![];
    let mut layer_ids: HashSet<String> = HashSet::new();

    for layer in layers {
        if !layer_ids.insert(layer.id.clone()) {
            violations.push(ReferenceViolation::DuplicateId {
                id: layer.id.clone(),
            });
        }

        let mut finding_ids: HashSet<String> = HashSet::new();
        for finding in &layer.findings {
            if !finding_ids.insert(finding.id.clone()) {
                violations.push(ReferenceViolation::DuplicateId {
                    id: finding.id.clone(),
                });
            }
            for subject in finding.subjects.iter().flatten() {
                match (&subject.kind, &subject.entity_ref) {
                    (SubjectKind::Artifact, _) => {}
                    (_, None) => {
                        violations.push(ReferenceViolation::MissingRef {
                            field: "finding_subject.ref".to_string(),
                        });
                    }
                    (SubjectKind::TestCase, Some(r)) => {
                        if !ids.test_case_ids.contains(r.as_str()) {
                            violations.push(ReferenceViolation::BrokenRef {
                                field: "finding_subject.ref".to_string(),
                                id: r.clone(),
                            });
                        }
                    }
                    (SubjectKind::Module, Some(r)) => {
                        if !ids.module_ids.contains(r.as_str()) {
                            violations.push(ReferenceViolation::BrokenRef {
                                field: "finding_subject.ref".to_string(),
                                id: r.clone(),
                            });
                        }
                    }
                    (SubjectKind::TestModuleLink, Some(r)) => {
                        if !ids.link_ids.contains(r.as_str()) {
                            violations.push(ReferenceViolation::BrokenRef {
                                field: "finding_subject.ref".to_string(),
                                id: r.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}
