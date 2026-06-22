use crate::artifact::ParsedEvidenceArtifact;
use crate::artifact::staged::StagedEvidence;
use crate::component::ComponentResult;
use crate::component::parser::{Parser, ParserInput};
use crate::validation::ACCEPTED_SCHEMA_VERSION;

pub struct BuiltinParser;

impl Parser for BuiltinParser {
    fn parse(&self, _input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact> {
        Ok(ParsedEvidenceArtifact {
            schema_version: ACCEPTED_SCHEMA_VERSION.to_string(),
            artifact_type: "parsed_evidence".to_string(),
            evidence: StagedEvidence {
                test_cases: vec![],
                modules: vec![],
                test_module_links: vec![],
            },
        })
    }
}
