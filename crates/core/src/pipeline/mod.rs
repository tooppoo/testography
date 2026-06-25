use std::path::{Component, Path, PathBuf};

use crate::artifact::staged::{
    BundleTest, Lineage, LineageProducer, StagedEvidence, StagedModuleBundle,
};
use crate::artifact::{
    AssessedModuleEvidenceArtifact, ModuleEvidenceArtifact, ParsedEvidenceArtifact,
};
use crate::component::evaluator::EvaluatorInput;
use crate::component::parser::ParserInput;
use crate::component::reporter::ReporterInput;
use crate::component::{ComponentError, ComponentRegistry};
use crate::io::{
    read_artifact, write_assessed_module_evidence, write_bytes, write_module_evidence,
    write_parsed_evidence,
};
use crate::validation::ArtifactError;
use crate::{ArtifactKind, parse_artifact};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("I/O error: {0}")]
    Io(std::io::Error),

    #[error("artifact validation error: {0}")]
    ArtifactValidation(ArtifactError),

    #[error("component error: {0}")]
    Component(ComponentError),

    #[error("input and output resolve to the same path: {path:?}")]
    SamePath { path: PathBuf },

    #[error("pipeline stage contract error at {step}: expected {expected}, found {found}")]
    UnexpectedArtifactType {
        step: &'static str,
        expected: &'static str,
        found: &'static str,
    },

    #[error(
        "report-output contract failure: invalid extension {extension:?} from reporter '{reporter}': {message}"
    )]
    InvalidReportExtension {
        reporter: String,
        extension: String,
        message: String,
    },

    #[error("run step '{step}' failed: {source}")]
    StepFailed {
        step: &'static str,
        #[source]
        source: Box<PipelineError>,
    },

    #[error("invalid pipeline configuration: {message}")]
    InvalidPipelineConfig { message: String },
}

fn map_artifact_error(err: ArtifactError) -> PipelineError {
    match err {
        ArtifactError::Io(e) => PipelineError::Io(e),
        other => PipelineError::ArtifactValidation(other),
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return canonical;
    }
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let mut result = PathBuf::new();
    for component in abs.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other),
        }
    }
    result
}

fn check_same_path(input: &Path, output: &Path) -> Result<(), PipelineError> {
    let norm_input = normalize_path(input);
    let norm_output = normalize_path(output);
    if norm_input == norm_output {
        return Err(PipelineError::SamePath { path: norm_input });
    }
    Ok(())
}

fn validate_module_evidence(artifact: &ModuleEvidenceArtifact) -> Result<(), PipelineError> {
    let json = serde_json::to_string(artifact)
        .map_err(|e| PipelineError::ArtifactValidation(ArtifactError::ParseJson(e)))?;
    parse_artifact(&json).map_err(map_artifact_error)?;
    Ok(())
}

fn validate_parsed_evidence(artifact: &ParsedEvidenceArtifact) -> Result<(), PipelineError> {
    let json = serde_json::to_string(artifact)
        .map_err(|e| PipelineError::ArtifactValidation(ArtifactError::ParseJson(e)))?;
    parse_artifact(&json).map_err(map_artifact_error)?;
    Ok(())
}

fn validate_assessed_module_evidence(
    artifact: &AssessedModuleEvidenceArtifact,
) -> Result<(), PipelineError> {
    let json = serde_json::to_string(artifact)
        .map_err(|e| PipelineError::ArtifactValidation(ArtifactError::ParseJson(e)))?;
    parse_artifact(&json).map_err(map_artifact_error)?;
    Ok(())
}

/// Run the collect step: invoke the named parser with `input`, validate the
/// produced parsed_evidence artifact, and write it to `output`.
pub fn collect_step(
    registry: &ComponentRegistry,
    parser_name: &str,
    input: &Path,
    output: &Path,
) -> Result<(), PipelineError> {
    check_same_path(input, output)?;

    let parser = registry
        .resolve_parser(parser_name)
        .map_err(PipelineError::Component)?;

    let parser_input = ParserInput {
        source_paths: vec![input.to_path_buf()],
        config: None,
    };

    let artifact = parser
        .parse(parser_input)
        .map_err(PipelineError::Component)?;

    validate_parsed_evidence(&artifact)?;

    write_parsed_evidence(&artifact, output).map_err(map_artifact_error)?;

    Ok(())
}

/// Derive one bundle per module from the existing test-module links.
///
/// Bundles are sorted by `module_ref`; tests within each bundle are sorted by
/// (`test_ref`, `link_ref`) so that the output is deterministic regardless of
/// the input order of `modules` and `test_module_links`.
fn derive_module_bundles(evidence: &StagedEvidence) -> Vec<StagedModuleBundle> {
    let mut bundles: Vec<StagedModuleBundle> = evidence
        .modules
        .iter()
        .map(|module| {
            let mut tests: Vec<BundleTest> = evidence
                .test_module_links
                .iter()
                .filter(|link| link.module_ref == module.id)
                .map(|link| BundleTest {
                    test_ref: link.test_ref.clone(),
                    link_ref: link.id.clone(),
                })
                .collect();
            tests.sort_by(|a, b| {
                a.test_ref
                    .cmp(&b.test_ref)
                    .then(a.link_ref.cmp(&b.link_ref))
            });
            StagedModuleBundle {
                module_ref: module.id.clone(),
                tests,
            }
        })
        .collect();
    bundles.sort_by(|a, b| a.module_ref.cmp(&b.module_ref));
    bundles
}

/// Run the transform step: read `input` (parsed_evidence), derive module bundles
/// from existing test-module links, and write `module_evidence` to `output`.
///
/// Rejects all other artifact stages as pipeline stage contract errors.
pub fn transform_step(input: &Path, output: &Path) -> Result<(), PipelineError> {
    check_same_path(input, output)?;

    let artifact = read_artifact(input).map_err(map_artifact_error)?;

    let parsed = match artifact {
        ArtifactKind::ParsedEvidence(p) => p,
        ArtifactKind::Evidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "transform",
                expected: "parsed_evidence",
                found: "evidence",
            });
        }
        ArtifactKind::Assessed(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "transform",
                expected: "parsed_evidence",
                found: "assessed_artifact",
            });
        }
        ArtifactKind::ModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "transform",
                expected: "parsed_evidence",
                found: "module_evidence",
            });
        }
        ArtifactKind::AssessedModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "transform",
                expected: "parsed_evidence",
                found: "assessed_module_evidence",
            });
        }
    };

    let module_bundles = derive_module_bundles(&parsed.evidence);

    let module_evidence = ModuleEvidenceArtifact {
        schema_version: parsed.schema_version,
        artifact_type: "module_evidence".to_string(),
        evidence: parsed.evidence,
        module_bundles,
        lineage: Lineage::DerivedFrom {
            input_artifact_type: "parsed_evidence".to_string(),
            input_producer: LineageProducer {
                name: "module-bundle-transform".to_string(),
                version: None,
                kind: Some("transform".to_string()),
            },
        },
    };

    validate_module_evidence(&module_evidence)?;

    write_module_evidence(&module_evidence, output).map_err(map_artifact_error)?;

    Ok(())
}

/// Run the evaluate step: read `input` (module_evidence or assessed_module_evidence),
/// invoke the named evaluator, append the returned finding layer, and write an
/// assessed_module_evidence artifact to `output`.
///
/// Rejects parsed_evidence, evidence, and assessed_artifact as pipeline stage
/// contract errors.
pub fn evaluate_step(
    registry: &ComponentRegistry,
    evaluator_name: &str,
    input: &Path,
    output: &Path,
) -> Result<(), PipelineError> {
    check_same_path(input, output)?;

    let artifact = read_artifact(input).map_err(map_artifact_error)?;

    let evaluator = registry
        .resolve_evaluator(evaluator_name)
        .map_err(PipelineError::Component)?;

    let (schema_version, evidence, module_bundles, existing_layers) = match artifact {
        ArtifactKind::ModuleEvidence(ref m) => (
            m.schema_version.clone(),
            m.evidence.clone(),
            m.module_bundles.clone(),
            vec![],
        ),
        ArtifactKind::AssessedModuleEvidence(ref a) => (
            a.schema_version.clone(),
            a.evidence.clone(),
            a.module_bundles.clone(),
            a.assessment_layers.clone(),
        ),
        ArtifactKind::ParsedEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "evaluate",
                expected: "module_evidence or assessed_module_evidence",
                found: "parsed_evidence",
            });
        }
        ArtifactKind::Evidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "evaluate",
                expected: "module_evidence or assessed_module_evidence",
                found: "evidence",
            });
        }
        ArtifactKind::Assessed(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "evaluate",
                expected: "module_evidence or assessed_module_evidence",
                found: "assessed_artifact",
            });
        }
    };

    let new_layer = evaluator
        .evaluate(EvaluatorInput {
            evidence: evidence.clone(),
            module_bundles: module_bundles.clone(),
            assessment_layers: existing_layers.clone(),
            config: None,
        })
        .map_err(PipelineError::Component)?;

    let mut layers = existing_layers;
    layers.push(new_layer);

    let assessed = AssessedModuleEvidenceArtifact {
        schema_version,
        artifact_type: "assessed_module_evidence".to_string(),
        evidence,
        module_bundles,
        assessment_layers: layers,
    };

    validate_assessed_module_evidence(&assessed)?;

    write_assessed_module_evidence(&assessed, output).map_err(map_artifact_error)?;

    Ok(())
}

/// Read `input` (must be assessed_module_evidence), validate its stage, resolve
/// the named reporter, invoke it, and return the `ReportOutput`.
///
/// This is shared between `report_step` (explicit output path) and
/// `run_pipeline` (reporter-defined extension path).
fn invoke_reporter(
    registry: &ComponentRegistry,
    reporter_name: &str,
    input: &Path,
) -> Result<crate::component::reporter::ReportOutput, PipelineError> {
    let artifact = read_artifact(input).map_err(map_artifact_error)?;

    let assessed = match artifact {
        ArtifactKind::AssessedModuleEvidence(a) => a,
        ArtifactKind::Evidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "report",
                expected: "assessed_module_evidence",
                found: "evidence",
            });
        }
        ArtifactKind::Assessed(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "report",
                expected: "assessed_module_evidence",
                found: "assessed_artifact",
            });
        }
        ArtifactKind::ParsedEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "report",
                expected: "assessed_module_evidence",
                found: "parsed_evidence",
            });
        }
        ArtifactKind::ModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                step: "report",
                expected: "assessed_module_evidence",
                found: "module_evidence",
            });
        }
    };

    let reporter = registry
        .resolve_reporter(reporter_name)
        .map_err(PipelineError::Component)?;

    reporter
        .report(ReporterInput {
            artifact: assessed,
            config: None,
        })
        .map_err(PipelineError::Component)
}

/// Run the report step: read `input` (must be assessed_module_evidence), invoke
/// the named reporter, and write the rendered output to `output`.
///
/// Rejects parsed_evidence, module_evidence, evidence, and assessed_artifact as
/// pipeline stage contract errors.
pub fn report_step(
    registry: &ComponentRegistry,
    reporter_name: &str,
    input: &Path,
    output: &Path,
) -> Result<(), PipelineError> {
    check_same_path(input, output)?;

    let report_output = invoke_reporter(registry, reporter_name, input)?;
    write_bytes(output, &report_output.content).map_err(map_artifact_error)?;

    Ok(())
}

// ── extension validation ──────────────────────────────────────────────────────

fn validate_report_extension(ext: &str) -> bool {
    !ext.is_empty()
        && ext
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

// ── run pipeline ──────────────────────────────────────────────────────────────

fn wrap_step(step: &'static str, err: PipelineError) -> PipelineError {
    PipelineError::StepFailed {
        step,
        source: Box::new(err),
    }
}

/// Run the full `collect → transform → evaluate… → report` pipeline.
///
/// Intermediate artifacts are written under `output_dir` and are kept on disk
/// even when a later step fails.  The report output path is derived from the
/// reporter-defined extension carried in `ReportOutput`.
///
/// At least one evaluator name must be supplied; returns
/// `InvalidPipelineConfig` otherwise.  Each step error is wrapped in
/// `StepFailed` so the failed step name is always present in the error.
pub fn run_pipeline(
    registry: &ComponentRegistry,
    input: &Path,
    parser_name: &str,
    evaluator_names: &[String],
    reporter_name: &str,
    output_dir: &Path,
) -> Result<(), PipelineError> {
    if evaluator_names.is_empty() {
        return Err(PipelineError::InvalidPipelineConfig {
            message: "at least one evaluator is required".to_string(),
        });
    }

    std::fs::create_dir_all(output_dir).map_err(PipelineError::Io)?;

    let parsed_evidence_path = output_dir.join("parsed_evidence.json");
    let module_evidence_path = output_dir.join("module_evidence.json");
    let assessed_path = output_dir.join("assessed_module_evidence.json");
    let assessed_tmp_path = output_dir.join("assessed_module_evidence.tmp.json");

    // collect
    collect_step(registry, parser_name, input, &parsed_evidence_path)
        .map_err(|e| wrap_step("collect", e))?;

    // transform
    transform_step(&parsed_evidence_path, &module_evidence_path)
        .map_err(|e| wrap_step("transform", e))?;

    // evaluate (one or more)
    for (i, evaluator_name) in evaluator_names.iter().enumerate() {
        if i == 0 {
            evaluate_step(
                registry,
                evaluator_name,
                &module_evidence_path,
                &assessed_path,
            )
            .map_err(|e| wrap_step("evaluate", e))?;
        } else {
            evaluate_step(registry, evaluator_name, &assessed_path, &assessed_tmp_path)
                .map_err(|e| wrap_step("evaluate", e))?;
            std::fs::rename(&assessed_tmp_path, &assessed_path)
                .map_err(|e| wrap_step("evaluate", PipelineError::Io(e)))?;
        }
    }

    // report: delegate read/stage-check/resolve/invoke to invoke_reporter,
    // then validate the reporter-defined extension and write to the computed path.
    let report_output = invoke_reporter(registry, reporter_name, &assessed_path)
        .map_err(|e| wrap_step("report", e))?;

    if !validate_report_extension(&report_output.extension) {
        return Err(wrap_step(
            "report",
            PipelineError::InvalidReportExtension {
                reporter: reporter_name.to_string(),
                extension: report_output.extension.clone(),
                message: "extension must be non-empty and contain only lowercase ASCII alphanumeric characters".to_string(),
            },
        ));
    }

    let report_path = output_dir.join(format!("report.{}", report_output.extension));
    write_bytes(&report_path, &report_output.content)
        .map_err(|e| wrap_step("report", map_artifact_error(e)))?;

    Ok(())
}
