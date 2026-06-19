pub mod artifact;
pub mod component;
pub mod io;
pub mod pipeline;
pub mod validation;

pub use artifact::{ArtifactKind, AssessedArtifact, EvidenceArtifact};
pub use component::{ComponentError, ComponentRegistry, ComponentResult};
pub use pipeline::PipelineError;
pub use validation::{ArtifactError, ReferenceViolation, SchemaViolation};

use validation::integrity::{check_assessed_integrity, check_evidence_integrity};
use validation::schema::{validate_assessed_schema, validate_evidence_schema};

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
        other => Err(ArtifactError::UnknownArtifactType {
            found: other.to_string(),
        }),
    }
}
