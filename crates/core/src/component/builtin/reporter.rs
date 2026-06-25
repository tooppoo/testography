use crate::component::ComponentResult;
use crate::component::reporter::{ReportOutput, Reporter, ReporterInput};

pub struct BuiltinReporter;

impl Reporter for BuiltinReporter {
    fn report(&self, input: ReporterInput) -> ComponentResult<ReportOutput> {
        let layer_count = input.artifact.assessment_layers.len();
        let module_count = input.artifact.module_bundles.len();
        let finding_count: usize = input
            .artifact
            .assessment_layers
            .iter()
            .map(|l| l.findings.len())
            .sum();

        let text = format!(
            "assessed_module_evidence: {module_count} module(s), {layer_count} layer(s), {finding_count} finding(s)\n"
        );

        Ok(ReportOutput {
            format: "text".to_string(),
            content: text.into_bytes(),
        })
    }
}
