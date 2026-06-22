pub mod artifact;
pub mod component;
pub mod io;
pub mod pipeline;
pub mod validation;

pub use artifact::{
    ArtifactKind, AssessedArtifact, AssessedModuleEvidenceArtifact, EvidenceArtifact, Lineage,
    LineageProducer, ModuleEvidenceArtifact, ParsedEvidenceArtifact,
};
pub use component::{ComponentError, ComponentRegistry, ComponentResult};
pub use pipeline::PipelineError;
pub use validation::{ArtifactError, ReferenceViolation, SchemaViolation};

use validation::integrity::{
    check_assessed_integrity, check_evidence_integrity, validate_assessed_module_evidence_refs,
    validate_module_evidence_refs, validate_parsed_evidence_refs,
};
use validation::schema::{
    validate_assessed_module_evidence_schema, validate_assessed_schema, validate_evidence_schema,
    validate_module_evidence_schema, validate_parsed_evidence_schema,
};

/// Parse and validate raw JSON as an artifact, dispatching by `artifact_type`.
pub fn parse_artifact(json: &str) -> Result<ArtifactKind, ArtifactError> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(ArtifactError::ParseJson)?;

    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ArtifactError::UnknownSchemaVersion {
            found: "<missing>".to_string(),
        })?;

    if schema_version != validation::ACCEPTED_SCHEMA_VERSION {
        return Err(ArtifactError::UnknownSchemaVersion {
            found: schema_version.to_string(),
        });
    }

    let artifact_type = value
        .get("artifact_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ArtifactError::UnknownArtifactType {
            found: "<missing>".to_string(),
        })?;

    match artifact_type {
        "evidence" => {
            let violations = validate_evidence_schema(&value)?;
            if !violations.is_empty() {
                return Err(ArtifactError::SchemaViolation(violations));
            }
            let artifact: EvidenceArtifact =
                serde_json::from_value(value).map_err(ArtifactError::ParseJson)?;
            let ref_violations = check_evidence_integrity(&artifact);
            if !ref_violations.is_empty() {
                return Err(ArtifactError::ReferenceIntegrity(ref_violations));
            }
            Ok(ArtifactKind::Evidence(artifact))
        }
        "assessed_artifact" => {
            let violations = validate_assessed_schema(&value)?;
            if !violations.is_empty() {
                return Err(ArtifactError::SchemaViolation(violations));
            }
            let artifact: AssessedArtifact =
                serde_json::from_value(value).map_err(ArtifactError::ParseJson)?;
            let ref_violations = check_assessed_integrity(&artifact);
            if !ref_violations.is_empty() {
                return Err(ArtifactError::ReferenceIntegrity(ref_violations));
            }
            Ok(ArtifactKind::Assessed(artifact))
        }
        "parsed_evidence" => {
            let violations = validate_parsed_evidence_schema(&value)?;
            if !violations.is_empty() {
                return Err(ArtifactError::SchemaViolation(violations));
            }
            let artifact: ParsedEvidenceArtifact =
                serde_json::from_value(value).map_err(ArtifactError::ParseJson)?;
            let ref_violations = validate_parsed_evidence_refs(&artifact);
            if !ref_violations.is_empty() {
                return Err(ArtifactError::ReferenceIntegrity(ref_violations));
            }
            Ok(ArtifactKind::ParsedEvidence(artifact))
        }
        "module_evidence" => {
            let violations = validate_module_evidence_schema(&value)?;
            if !violations.is_empty() {
                return Err(ArtifactError::SchemaViolation(violations));
            }
            let artifact: ModuleEvidenceArtifact =
                serde_json::from_value(value).map_err(ArtifactError::ParseJson)?;
            let ref_violations = validate_module_evidence_refs(&artifact);
            if !ref_violations.is_empty() {
                return Err(ArtifactError::ReferenceIntegrity(ref_violations));
            }
            Ok(ArtifactKind::ModuleEvidence(artifact))
        }
        "assessed_module_evidence" => {
            let violations = validate_assessed_module_evidence_schema(&value)?;
            if !violations.is_empty() {
                return Err(ArtifactError::SchemaViolation(violations));
            }
            let artifact: AssessedModuleEvidenceArtifact =
                serde_json::from_value(value).map_err(ArtifactError::ParseJson)?;
            let ref_violations = validate_assessed_module_evidence_refs(&artifact);
            if !ref_violations.is_empty() {
                return Err(ArtifactError::ReferenceIntegrity(ref_violations));
            }
            Ok(ArtifactKind::AssessedModuleEvidence(artifact))
        }
        other => Err(ArtifactError::UnknownArtifactType {
            found: other.to_string(),
        }),
    }
}
