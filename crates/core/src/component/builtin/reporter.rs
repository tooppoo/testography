use crate::component::ComponentResult;
use crate::component::reporter::{ReportOutput, Reporter, ReporterInput};

pub struct BuiltinReporter;

impl Reporter for BuiltinReporter {
    fn report(&self, _input: ReporterInput) -> ComponentResult<ReportOutput> {
        Ok(ReportOutput {
            format: "json".to_string(),
            content: vec![],
        })
    }
}
