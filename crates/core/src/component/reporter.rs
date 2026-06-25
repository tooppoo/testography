pub use tgraphy_types::ReporterInput;

use super::ComponentResult;

#[derive(Debug)]
pub struct ReportOutput {
    pub format: String,
    /// Reporter-defined output file extension, e.g. "md", "json", "txt".
    /// Must be non-empty lowercase alphanumeric (no leading dot, no path separators).
    pub extension: String,
    pub content: Vec<u8>,
}

pub trait Reporter {
    fn report(&self, input: ReporterInput) -> ComponentResult<ReportOutput>;
}
