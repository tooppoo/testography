pub mod assessed;
pub mod common;
pub mod evidence;
pub mod layer;

pub use assessed::AssessedArtifact;
pub use common::{Diagnostic, DiagnosticLevel, Producer};
pub use evidence::{
    ArtifactValue, Assertion, AssertionStyle, Call, CallRole, Callee, Confidence, Evidence,
    EvidenceArtifact, LiteralClass, Matcher, Module, ModuleBundle, ModuleKind, Parameter,
    ResolutionStatus, Source, TestCase, TestModuleLink, ValueKind,
};
pub use layer::{Assessment, AssessmentKind, AssessmentLayer, AssessmentSeverity, LayerProducer};

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactKind {
    Evidence(EvidenceArtifact),
    Assessed(AssessedArtifact),
}
