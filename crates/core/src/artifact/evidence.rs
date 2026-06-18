use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::common::{Diagnostic, Producer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub producer: Producer,
    pub evidence: Evidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<Diagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_cases: Option<Vec<TestCase>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<Module>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_module_links: Option<Vec<TestModuleLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_bundles: Option<Vec<ModuleBundle>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suite: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calls: Option<Vec<Call>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<Vec<Assertion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mocks: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixtures: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallRole {
    DirectCall,
    AssertionTargetCall,
    SetupCall,
    FixtureCall,
    HelperCall,
    FactoryCall,
    MockSetupCall,
    UnknownCall,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStatus {
    Resolved,
    Unresolved,
    Ambiguous,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Callee {
    pub text: String,
    pub resolution_status: ResolutionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_module_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Call {
    pub id: String,
    pub role: CallRole,
    pub callee: Callee,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueKind {
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,
    NullLiteral,
    UndefinedLiteral,
    LanguageSpecificLiteral,
    ObjectLiteral,
    ArrayLiteral,
    Identifier,
    CallExpression,
    MemberExpression,
    TemplateLiteral,
    Closure,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiteralClass {
    EmptyString,
    WhitespaceString,
    NonEmptyString,
    Zero,
    PositiveNumber,
    NegativeNumber,
    Integer,
    Float,
    EmptyArray,
    NonEmptyArray,
    EmptyObject,
    NonEmptyObject,
    Nullish,
    True,
    False,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactValue {
    pub value_kind: ValueKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_class: Option<LiteralClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_shape: Option<BTreeMap<String, Box<ArtifactValue>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_items: Option<Vec<Box<ArtifactValue>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub id: String,
    pub argument_index: u64,
    pub value_kind: ValueKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_class: Option<LiteralClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_shape: Option<BTreeMap<String, ArtifactValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_items: Option<Vec<ArtifactValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionStyle {
    ExpectMatcher,
    AssertFunction,
    ShouldStyle,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<ArtifactValue>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    pub id: String,
    pub style: AssertionStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<Matcher>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_call_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<ArtifactValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    File,
    Symbol,
    Package,
    Class,
    Method,
    Function,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub kind: ModuleKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualified_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkRelationship {
    DirectlyCalled,
    AssertionTarget,
    SetupDependency,
    FixtureDependency,
    FactoryDependency,
    MockedDependency,
    HelperDependency,
    UnresolvedCandidate,
    AmbiguousCandidate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestModuleLink {
    pub test_id: String,
    pub module_id: String,
    pub relationship: LinkRelationship,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleBundle {
    pub module_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_ref: Option<Module>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}
