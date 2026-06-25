use std::fmt::Write as FmtWrite;

pub use tgraphy_types::{AssessedModuleEvidenceArtifact, FindingLevel, ReporterInput, SubjectKind};

pub fn render(input: ReporterInput) -> Vec<u8> {
    let artifact = &input.artifact;
    let mut out = String::new();

    render_summary(&mut out, artifact);
    render_modules(&mut out, artifact);
    render_assessment_layers(&mut out, artifact);
    render_findings(&mut out, artifact);

    out.into_bytes()
}

fn render_summary(out: &mut String, artifact: &AssessedModuleEvidenceArtifact) {
    let module_count = artifact.module_bundles.len();
    let layer_count = artifact.assessment_layers.len();
    let finding_count: usize = artifact
        .assessment_layers
        .iter()
        .map(|l| l.findings.len())
        .sum();

    writeln!(out, "# Summary").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "- Modules: {module_count}").unwrap();
    writeln!(out, "- Assessment layers: {layer_count}").unwrap();
    writeln!(out, "- Findings: {finding_count}").unwrap();
    writeln!(out).unwrap();
}

fn render_modules(out: &mut String, artifact: &AssessedModuleEvidenceArtifact) {
    writeln!(out, "# Modules").unwrap();
    writeln!(out).unwrap();

    if artifact.module_bundles.is_empty() {
        writeln!(out, "_No modules._").unwrap();
        writeln!(out).unwrap();
        return;
    }

    for bundle in &artifact.module_bundles {
        writeln!(out, "## {}", bundle.module_ref).unwrap();
        writeln!(out).unwrap();

        if bundle.tests.is_empty() {
            writeln!(out, "_No linked tests._").unwrap();
        } else {
            for test in &bundle.tests {
                writeln!(out, "- {} (link: {})", test.test_ref, test.link_ref).unwrap();
            }
        }
        writeln!(out).unwrap();
    }
}

fn render_assessment_layers(out: &mut String, artifact: &AssessedModuleEvidenceArtifact) {
    writeln!(out, "# Assessment Layers").unwrap();
    writeln!(out).unwrap();

    if artifact.assessment_layers.is_empty() {
        writeln!(out, "_No assessment layers._").unwrap();
        writeln!(out).unwrap();
        return;
    }

    for layer in &artifact.assessment_layers {
        let evaluator_id = &layer.evaluator.id;
        let version_suffix = layer
            .evaluator
            .version
            .as_deref()
            .map(|v| format!(" v{v}"))
            .unwrap_or_default();
        writeln!(out, "## {} ({}{})", layer.id, evaluator_id, version_suffix).unwrap();
        writeln!(out).unwrap();

        if let Some(summary) = &layer.summary {
            writeln!(out, "{summary}").unwrap();
            writeln!(out).unwrap();
        }

        writeln!(out, "Findings: {}", layer.findings.len()).unwrap();
        writeln!(out).unwrap();
    }
}

fn render_findings(out: &mut String, artifact: &AssessedModuleEvidenceArtifact) {
    writeln!(out, "# Findings").unwrap();
    writeln!(out).unwrap();

    let all_findings: Vec<_> = artifact
        .assessment_layers
        .iter()
        .flat_map(|l| l.findings.iter().map(move |f| (&l.id, f)))
        .collect();

    if all_findings.is_empty() {
        writeln!(out, "_No findings._").unwrap();
        writeln!(out).unwrap();
        return;
    }

    for (layer_id, finding) in all_findings {
        let level = match finding.level {
            FindingLevel::Info => "info",
            FindingLevel::Warning => "warning",
            FindingLevel::Error => "error",
        };

        writeln!(out, "## {}", finding.id).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "- **Layer**: {layer_id}").unwrap();
        writeln!(out, "- **Level**: {level}").unwrap();

        if let Some(rule_id) = &finding.rule_id {
            writeln!(out, "- **Rule**: {rule_id}").unwrap();
        }

        if let Some(confidence) = &finding.confidence {
            writeln!(out, "- **Confidence**: {confidence}").unwrap();
        }

        writeln!(out, "- **Message**: {}", finding.message).unwrap();

        if !finding.subjects.is_empty() {
            writeln!(out, "- **Subjects**:").unwrap();
            for subject in &finding.subjects {
                let kind = match subject.kind {
                    SubjectKind::Artifact => "artifact",
                    SubjectKind::TestCase => "test_case",
                    SubjectKind::Module => "module",
                    SubjectKind::TestModuleLink => "test_module_link",
                    SubjectKind::Assertion => "assertion",
                };
                let ref_part = subject
                    .entity_ref
                    .as_deref()
                    .map(|r| format!(" ref={r}"))
                    .unwrap_or_default();
                let path_part = subject
                    .path
                    .as_deref()
                    .map(|p| format!(" path={p}"))
                    .unwrap_or_default();
                writeln!(out, "  - {kind}{ref_part}{path_part}").unwrap();
            }
        }

        if let Some(rationale) = &finding.rationale {
            writeln!(out, "- **Rationale**: {rationale}").unwrap();
        }

        writeln!(out).unwrap();
    }
}
