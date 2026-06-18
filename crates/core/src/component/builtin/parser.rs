use crate::artifact::EvidenceArtifact;
use crate::artifact::common::Producer;
use crate::artifact::evidence::Evidence;
use crate::component::ComponentResult;
use crate::component::parser::{Parser, ParserInput};
use crate::validation::ACCEPTED_SCHEMA_VERSION;

pub struct BuiltinParser;

impl Parser for BuiltinParser {
    fn parse(&self, _input: ParserInput) -> ComponentResult<EvidenceArtifact> {
        Ok(EvidenceArtifact {
            schema_version: ACCEPTED_SCHEMA_VERSION.to_string(),
            artifact_type: "evidence".to_string(),
            producer: Producer {
                name: "builtin".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                kind: None,
                extensions: None,
            },
            evidence: Evidence {
                test_cases: None,
                modules: None,
                test_module_links: None,
                module_bundles: None,
                extensions: None,
            },
            diagnostics: None,
            project: None,
            extensions: None,
        })
    }
}
