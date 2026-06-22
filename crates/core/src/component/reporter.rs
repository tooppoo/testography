use crate::artifact::AssessedModuleEvidenceArtifact;

use super::ComponentResult;

pub struct ReporterInput {
    pub artifact: AssessedModuleEvidenceArtifact,
    pub config: Option<serde_json::Value>,
}

pub struct ReportOutput {
    pub format: String,
    pub content: Vec<u8>,
}

pub trait Reporter {
    fn report(&self, input: ReporterInput) -> ComponentResult<ReportOutput>;
}
