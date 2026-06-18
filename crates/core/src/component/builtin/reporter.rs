use crate::component::reporter::{ReportOutput, Reporter, ReporterInput};
use crate::component::ComponentResult;

pub struct BuiltinReporter;

impl Reporter for BuiltinReporter {
    fn report(&self, _input: ReporterInput) -> ComponentResult<ReportOutput> {
        Ok(ReportOutput {
            format: "json".to_string(),
            content: vec![],
        })
    }
}
