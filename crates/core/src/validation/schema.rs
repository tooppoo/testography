use jsonschema::{Resource, Validator};
use serde_json::Value;

use super::SchemaViolation;
use crate::validation::ArtifactError;

const EVIDENCE_SCHEMA_STR: &str =
    include_str!("../../../../schemas/testography-evidence/testography-evidence.v0.json");
const ASSESSED_SCHEMA_STR: &str = include_str!(
    "../../../../schemas/testography-assessed-artifact/testography-assessed-artifact.v0.json"
);
const LAYER_SCHEMA_STR: &str = include_str!(
    "../../../../schemas/testography-assessment-layer/testography-assessment-layer.v0.json"
);

const EVIDENCE_SCHEMA_URL: &str = "https://raw.githubusercontent.com/tooppoo/testography/main/schemas/testography-evidence/testography-evidence.v0.json";
const LAYER_SCHEMA_URL: &str = "https://raw.githubusercontent.com/tooppoo/testography/main/schemas/testography-assessment-layer/testography-assessment-layer.v0.json";

fn build_evidence_validator() -> Result<Validator, ArtifactError> {
    let schema: Value = serde_json::from_str(EVIDENCE_SCHEMA_STR)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))?;
    jsonschema::options()
        .build(&schema)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))
}

fn build_assessed_validator() -> Result<Validator, ArtifactError> {
    let evidence_schema: Value = serde_json::from_str(EVIDENCE_SCHEMA_STR)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))?;
    let layer_schema: Value = serde_json::from_str(LAYER_SCHEMA_STR)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))?;
    let assessed_schema: Value = serde_json::from_str(ASSESSED_SCHEMA_STR)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))?;

    let evidence_resource = Resource::from_contents(evidence_schema)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))?;
    let layer_resource = Resource::from_contents(layer_schema)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))?;

    jsonschema::options()
        .with_resource(EVIDENCE_SCHEMA_URL, evidence_resource)
        .with_resource(LAYER_SCHEMA_URL, layer_resource)
        .build(&assessed_schema)
        .map_err(|e| ArtifactError::SchemaCompile(e.to_string()))
}

fn collect_violations(validator: &Validator, instance: &Value) -> Vec<SchemaViolation> {
    validator
        .iter_errors(instance)
        .map(|e| SchemaViolation {
            path: e.instance_path.to_string(),
            message: e.to_string(),
        })
        .collect()
}

pub fn validate_evidence_schema(instance: &Value) -> Result<Vec<SchemaViolation>, ArtifactError> {
    let validator = build_evidence_validator()?;
    Ok(collect_violations(&validator, instance))
}

pub fn validate_assessed_schema(instance: &Value) -> Result<Vec<SchemaViolation>, ArtifactError> {
    let validator = build_assessed_validator()?;
    Ok(collect_violations(&validator, instance))
}
