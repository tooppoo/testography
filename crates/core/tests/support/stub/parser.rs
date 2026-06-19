use tgraphy_core::EvidenceArtifact;
use tgraphy_core::artifact::common::Producer;
use tgraphy_core::artifact::evidence::{
    Assertion, AssertionStyle, Call, CallRole, Callee, Confidence, Evidence, LinkRelationship,
    Module, ModuleKind, ResolutionStatus, Source, TestCase, TestModuleLink,
};
use tgraphy_core::component::{ComponentResult, Parser, ParserInput};
use tgraphy_core::validation::ACCEPTED_SCHEMA_VERSION;

pub struct StubParser;

impl Parser for StubParser {
    fn parse(&self, _input: ParserInput) -> ComponentResult<EvidenceArtifact> {
        Ok(EvidenceArtifact {
            schema_version: ACCEPTED_SCHEMA_VERSION.to_string(),
            artifact_type: "evidence".to_string(),
            producer: Producer {
                name: "stub-parser".to_string(),
                version: "0.0.0".to_string(),
                kind: None,
                extensions: None,
            },
            evidence: Evidence {
                test_cases: Some(vec![TestCase {
                    id: "stub-test-001".to_string(),
                    name: "stub test case".to_string(),
                    source: Source {
                        file: "stub/test.ts".to_string(),
                        line: None,
                        column: None,
                        language: None,
                        text: None,
                        text_hash: None,
                        extensions: None,
                    },
                    suite: None,
                    calls: Some(vec![Call {
                        id: "stub-call-001".to_string(),
                        role: CallRole::AssertionTargetCall,
                        callee: Callee {
                            text: "stubFunction".to_string(),
                            resolution_status: ResolutionStatus::Unresolved,
                            confidence: None,
                            resolved_module_id: None,
                            resolved_symbol: None,
                        },
                        source: None,
                        extensions: None,
                    }]),
                    parameters: None,
                    assertions: Some(vec![Assertion {
                        id: "stub-assertion-001".to_string(),
                        style: AssertionStyle::ExpectMatcher,
                        framework: None,
                        matcher: None,
                        target_call_refs: Some(vec!["stub-call-001".to_string()]),
                        target_expression: None,
                        expected: None,
                        source: None,
                        extensions: None,
                    }]),
                    mocks: None,
                    fixtures: None,
                    extensions: None,
                }]),
                modules: Some(vec![Module {
                    id: "stub-module-001".to_string(),
                    kind: ModuleKind::Function,
                    path: Some("stub/subject.ts".to_string()),
                    qualified_name: Some("stubFunction".to_string()),
                    container: None,
                    language: None,
                    extensions: None,
                }]),
                test_module_links: Some(vec![TestModuleLink {
                    test_id: "stub-test-001".to_string(),
                    module_id: "stub-module-001".to_string(),
                    relationship: LinkRelationship::AssertionTarget,
                    confidence: Confidence::High,
                    basis: None,
                    evidence_refs: Some(vec!["stub-call-001".to_string()]),
                    extensions: None,
                }]),
                module_bundles: None,
                extensions: None,
            },
            diagnostics: None,
            project: None,
            extensions: None,
        })
    }
}
