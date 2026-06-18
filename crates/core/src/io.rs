use std::path::Path;

use crate::artifact::{AssessedArtifact, EvidenceArtifact};
use crate::validation::ArtifactError;
use crate::{ArtifactKind, parse_artifact};

/// Write bytes to a file, creating parent directories if needed.
pub fn write_bytes(path: &Path, data: &[u8]) -> Result<(), ArtifactError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(ArtifactError::Io)?;
    }
    std::fs::write(path, data).map_err(ArtifactError::Io)
}

/// Parse and validate an artifact from a file path, dispatching by `artifact_type`.
pub fn read_artifact(path: &Path) -> Result<ArtifactKind, ArtifactError> {
    let json = std::fs::read_to_string(path).map_err(ArtifactError::Io)?;
    parse_artifact(&json)
}

/// Serialize an EvidenceArtifact to deterministic pretty JSON.
pub fn serialize_evidence(artifact: &EvidenceArtifact) -> Result<Vec<u8>, ArtifactError> {
    let mut bytes = serde_json::to_vec_pretty(artifact).map_err(ArtifactError::ParseJson)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Serialize an AssessedArtifact to deterministic pretty JSON.
pub fn serialize_assessed(artifact: &AssessedArtifact) -> Result<Vec<u8>, ArtifactError> {
    let mut bytes = serde_json::to_vec_pretty(artifact).map_err(ArtifactError::ParseJson)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Write an EvidenceArtifact to a file as deterministic pretty JSON.
pub fn write_evidence(artifact: &EvidenceArtifact, path: &Path) -> Result<(), ArtifactError> {
    let bytes = serialize_evidence(artifact)?;
    write_bytes(path, &bytes)
}

/// Write an AssessedArtifact to a file as deterministic pretty JSON.
pub fn write_assessed(artifact: &AssessedArtifact, path: &Path) -> Result<(), ArtifactError> {
    let bytes = serialize_assessed(artifact)?;
    write_bytes(path, &bytes)
}
