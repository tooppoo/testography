pub mod integrity;
pub mod schema;

pub const ACCEPTED_SCHEMA_VERSION: &str = "0.0.1";

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaViolation {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceViolation {
    DuplicateId { id: String },
    BrokenRef { field: String, id: String },
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    ParseJson(#[from] serde_json::Error),

    #[error("unknown schema_version: {found:?}")]
    UnknownSchemaVersion { found: String },

    #[error("unknown artifact_type: {found:?}")]
    UnknownArtifactType { found: String },

    #[error("schema validation failed")]
    SchemaViolation(Vec<SchemaViolation>),

    #[error("reference integrity failed")]
    ReferenceIntegrity(Vec<ReferenceViolation>),

    #[error("schema compilation error: {0}")]
    SchemaCompile(String),
}
