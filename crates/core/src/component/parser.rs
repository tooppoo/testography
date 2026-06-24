use crate::artifact::ParsedEvidenceArtifact;

use super::ComponentResult;

pub use tgraphy_types::ParserInput;

pub trait Parser {
    fn parse(&self, input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact>;
}
