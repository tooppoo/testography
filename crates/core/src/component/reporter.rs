pub use tgraphy_types::ReporterInput;

use super::ComponentResult;

#[derive(Debug)]
pub struct ReportOutput {
    pub format: String,
    pub content: Vec<u8>,
}

pub trait Reporter {
    fn report(&self, input: ReporterInput) -> ComponentResult<ReportOutput>;
}
