use std::path::{Component, Path, PathBuf};

use crate::artifact::{AssessedArtifact, EvidenceArtifact};
use crate::component::evaluator::EvaluatorInput;
use crate::component::parser::ParserInput;
use crate::component::reporter::ReporterInput;
use crate::component::{ComponentError, ComponentRegistry};
use crate::io::{read_artifact, write_assessed, write_bytes, write_evidence};
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

    #[error("unexpected artifact type: expected {expected}, found {found}")]
    UnexpectedArtifactType {
        expected: &'static str,
        found: &'static str,
    },
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

fn validate_evidence(artifact: &EvidenceArtifact) -> Result<(), PipelineError> {
    let json = serde_json::to_string(artifact)
        .map_err(|e| PipelineError::ArtifactValidation(ArtifactError::ParseJson(e)))?;
    parse_artifact(&json).map_err(map_artifact_error)?;
    Ok(())
}

fn validate_assessed(artifact: &AssessedArtifact) -> Result<(), PipelineError> {
    let json = serde_json::to_string(artifact)
        .map_err(|e| PipelineError::ArtifactValidation(ArtifactError::ParseJson(e)))?;
    parse_artifact(&json).map_err(map_artifact_error)?;
    Ok(())
}

fn assessed_from_evidence(
    evidence: &EvidenceArtifact,
    layer: crate::artifact::AssessmentLayer,
) -> AssessedArtifact {
    AssessedArtifact {
        schema_version: evidence.schema_version.clone(),
        artifact_type: "assessed_artifact".to_string(),
        producer: evidence.producer.clone(),
        evidence: evidence.evidence.clone(),
        assessment_layers: vec![layer],
        diagnostics: evidence.diagnostics.clone(),
        project: evidence.project.clone(),
        extensions: evidence.extensions.clone(),
    }
}

fn evidence_from_assessed(assessed: &AssessedArtifact) -> EvidenceArtifact {
    EvidenceArtifact {
        schema_version: assessed.schema_version.clone(),
        artifact_type: "evidence".to_string(),
        producer: assessed.producer.clone(),
        evidence: assessed.evidence.clone(),
        diagnostics: assessed.diagnostics.clone(),
        project: assessed.project.clone(),
        extensions: assessed.extensions.clone(),
    }
}

/// Run the collect step: invoke the named parser with `input`, validate the
/// produced evidence artifact, and write it to `output`.
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

    let evidence = parser
        .parse(parser_input)
        .map_err(PipelineError::Component)?;

    validate_evidence(&evidence)?;

    write_evidence(&evidence, output).map_err(map_artifact_error)?;

    Ok(())
}

/// Run the evaluate step: read `input` (evidence or assessed artifact), invoke
/// the named evaluator, append the returned assessment layer, and write an
/// assessed artifact to `output`.
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

    let assessed = match artifact {
        ArtifactKind::Evidence(ev) => {
            let layer = evaluator
                .evaluate(EvaluatorInput {
                    artifact: ev.clone(),
                    config: None,
                })
                .map_err(PipelineError::Component)?;
            assessed_from_evidence(&ev, layer)
        }
        ArtifactKind::Assessed(ref assessed) => {
            let ev = evidence_from_assessed(assessed);
            let layer = evaluator
                .evaluate(EvaluatorInput {
                    artifact: ev,
                    config: None,
                })
                .map_err(PipelineError::Component)?;
            let mut updated = assessed.clone();
            updated.assessment_layers.push(layer);
            updated
        }
        ArtifactKind::ParsedEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "evidence or assessed_artifact",
                found: "parsed_evidence",
            });
        }
        ArtifactKind::ModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "evidence or assessed_artifact",
                found: "module_evidence",
            });
        }
        ArtifactKind::AssessedModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "evidence or assessed_artifact",
                found: "assessed_module_evidence",
            });
        }
    };

    validate_assessed(&assessed)?;

    write_assessed(&assessed, output).map_err(map_artifact_error)?;

    Ok(())
}

/// Run the report step: read `input` (must be an assessed artifact), invoke
/// the named reporter, and write the rendered output to `output`.
pub fn report_step(
    registry: &ComponentRegistry,
    reporter_name: &str,
    input: &Path,
    output: &Path,
) -> Result<(), PipelineError> {
    check_same_path(input, output)?;

    let artifact = read_artifact(input).map_err(map_artifact_error)?;

    let assessed = match artifact {
        ArtifactKind::Assessed(a) => a,
        ArtifactKind::Evidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "assessed_artifact",
                found: "evidence",
            });
        }
        ArtifactKind::ParsedEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "assessed_artifact",
                found: "parsed_evidence",
            });
        }
        ArtifactKind::ModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "assessed_artifact",
                found: "module_evidence",
            });
        }
        ArtifactKind::AssessedModuleEvidence(_) => {
            return Err(PipelineError::UnexpectedArtifactType {
                expected: "assessed_artifact",
                found: "assessed_module_evidence",
            });
        }
    };

    let reporter = registry
        .resolve_reporter(reporter_name)
        .map_err(PipelineError::Component)?;

    let report_output = reporter
        .report(ReporterInput {
            artifact: assessed,
            config: None,
        })
        .map_err(PipelineError::Component)?;

    write_bytes(output, &report_output.content).map_err(map_artifact_error)?;

    Ok(())
}
