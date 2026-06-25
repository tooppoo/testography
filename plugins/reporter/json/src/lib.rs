pub use tgraphy_types::{AssessedModuleEvidenceArtifact, ReporterInput};

pub fn render(input: ReporterInput) -> Vec<u8> {
    let mut out = serde_json::to_vec_pretty(&input.artifact)
        .expect("AssessedModuleEvidenceArtifact should always serialize");
    out.push(b'\n');
    out
}
