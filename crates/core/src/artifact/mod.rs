pub mod assessed;
pub mod common;
pub mod evidence;
pub mod layer;
pub mod staged;

pub use assessed::AssessedArtifact;
pub use common::{Diagnostic, DiagnosticLevel, Producer};
pub use evidence::{
    ArtifactValue, Assertion, AssertionStyle, Call, CallRole, Callee, Confidence, Evidence,
    EvidenceArtifact, LiteralClass, Matcher, Module, ModuleBundle, ModuleKind, Parameter,
    ResolutionStatus, Source, TestCase, TestModuleLink, ValueKind,
};
pub use layer::{Assessment, AssessmentKind, AssessmentLayer, AssessmentSeverity, LayerProducer};
pub use staged::{
    AssessedModuleEvidenceArtifact, BundleTest, Evaluator, EvaluatorInfo, Finding, FindingLayer,
    FindingLevel, FindingSubject, Lineage, LineageProducer, ModuleEvidenceArtifact,
    ParsedEvidenceArtifact, StagedEvidence, StagedModuleBundle, StagedTestModuleLink, SubjectKind,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactKind {
    Evidence(EvidenceArtifact),
    Assessed(AssessedArtifact),
    ParsedEvidence(ParsedEvidenceArtifact),
    ModuleEvidence(ModuleEvidenceArtifact),
    AssessedModuleEvidence(AssessedModuleEvidenceArtifact),
}
