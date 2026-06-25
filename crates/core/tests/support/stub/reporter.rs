use tgraphy_core::component::{ComponentResult, ReportOutput, Reporter, ReporterInput};

pub struct StubReporter;

impl Reporter for StubReporter {
    fn report(&self, input: ReporterInput) -> ComponentResult<ReportOutput> {
        let artifact = &input.artifact;
        let mut lines: Vec<String> = vec![
            "# Testography Report".to_string(),
            "".to_string(),
            "## Evidence".to_string(),
            "".to_string(),
        ];

        let test_count = artifact.evidence.test_cases.len();
        let module_count = artifact.evidence.modules.len();
        lines.push(format!("- test cases: {}", test_count));
        lines.push(format!("- modules: {}", module_count));
        lines.push("".to_string());
        lines.push("## Assessment Layers".to_string());

        for layer in &artifact.assessment_layers {
            lines.push("".to_string());
            lines.push(format!("### {}", layer.id));
            lines.push("".to_string());
            for finding in &layer.findings {
                lines.push(format!("- {}: {}", finding.id, finding.message));
            }
        }

        let mut content = lines.join("\n");
        content.push('\n');

        Ok(ReportOutput {
            format: "markdown".to_string(),
            extension: "md".to_string(),
            content: content.into_bytes(),
        })
    }
}
