use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use quote::ToTokens;

use crate::artifact::ParsedEvidenceArtifact;
use crate::artifact::evidence::{
    ArtifactValue, Assertion, AssertionStyle, Call, CallRole, Callee, LiteralClass, Matcher,
    Module, ModuleKind, Parameter, ResolutionStatus, Source, TestCase, ValueKind,
};
use crate::artifact::staged::{StagedEvidence, StagedTestModuleLink};
use crate::component::parser::{Parser, ParserInput};
use crate::component::{ComponentError, ComponentResult};
use crate::validation::ACCEPTED_SCHEMA_VERSION;

pub struct RustParser;

impl Parser for RustParser {
    fn parse(&self, input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact> {
        let mut ctx = ParseContext::new();
        let mut source_paths = input.source_paths;
        source_paths.sort();

        for path in &source_paths {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let source =
                std::fs::read_to_string(path).map_err(|err| ComponentError::ExecutionFailed {
                    message: format!("failed to read Rust source {}: {err}", path.display()),
                })?;
            ctx.process_file(path, &source)?;
        }

        Ok(ParsedEvidenceArtifact {
            schema_version: ACCEPTED_SCHEMA_VERSION.to_string(),
            artifact_type: "parsed_evidence".to_string(),
            evidence: ctx.into_evidence(),
        })
    }
}

struct ParseContext {
    counter: usize,
    test_cases: Vec<TestCase>,
    modules: Vec<Module>,
    test_module_links: Vec<StagedTestModuleLink>,
    module_by_qname: BTreeMap<String, String>,
}

struct TestEvidence {
    calls: Vec<Call>,
    parameters: Vec<Parameter>,
    assertions: Vec<Assertion>,
    diagnostics: Vec<ParserDiagnostic>,
}

#[derive(Clone)]
struct ParserDiagnostic {
    code: &'static str,
    message: String,
}

impl ParseContext {
    fn new() -> Self {
        Self {
            counter: 0,
            test_cases: Vec::new(),
            modules: Vec::new(),
            test_module_links: Vec::new(),
            module_by_qname: BTreeMap::new(),
        }
    }

    fn next_id(&mut self, kind: &str) -> String {
        self.counter += 1;
        format!("rust-{kind}-{:04}", self.counter)
    }

    fn ensure_module(&mut self, qualified_name: &str) -> String {
        if let Some(id) = self.module_by_qname.get(qualified_name) {
            return id.clone();
        }

        let id = self.next_id("module");
        self.modules.push(Module {
            id: id.clone(),
            kind: ModuleKind::Function,
            path: None,
            qualified_name: Some(qualified_name.to_string()),
            container: None,
            language: Some("rust".to_string()),
            extensions: None,
        });
        self.module_by_qname
            .insert(qualified_name.to_string(), id.clone());
        id
    }

    fn into_evidence(self) -> StagedEvidence {
        StagedEvidence {
            test_cases: self.test_cases,
            modules: self.modules,
            test_module_links: self.test_module_links,
        }
    }

    fn process_file(&mut self, path: &Path, source: &str) -> ComponentResult<()> {
        let file = syn::parse_file(source).map_err(|err| ComponentError::ExecutionFailed {
            message: format!("failed to parse Rust source {}: {err}", path.display()),
        })?;

        let module_path = Vec::new();
        self.process_items(path, &file.items, &module_path);
        Ok(())
    }

    fn process_items(&mut self, path: &Path, items: &[syn::Item], module_path: &[String]) {
        let mut use_map = BTreeMap::new();
        for item in items {
            if let syn::Item::Use(u) = item {
                collect_use_tree(&u.tree, "", &mut use_map);
            }
        }

        for item in items {
            match item {
                syn::Item::Fn(f) if is_test_fn(f) => {
                    self.process_test_fn(path, f, module_path, &use_map);
                }
                syn::Item::Mod(m) => {
                    if let Some((_, nested_items)) = &m.content {
                        let mut nested_path = module_path.to_vec();
                        nested_path.push(m.ident.to_string());
                        self.process_items(path, nested_items, &nested_path);
                    }
                }
                _ => {}
            }
        }
    }

    fn process_test_fn(
        &mut self,
        path: &Path,
        f: &syn::ItemFn,
        module_path: &[String],
        use_map: &BTreeMap<String, String>,
    ) {
        let test_id = self.next_id("test");
        let mut evidence = TestEvidence {
            calls: Vec::new(),
            parameters: Vec::new(),
            assertions: Vec::new(),
            diagnostics: Vec::new(),
        };

        for stmt in &f.block.stmts {
            self.process_stmt(stmt, use_map, module_path, &mut evidence);
        }

        let resolved_links = evidence
            .calls
            .iter()
            .filter_map(|call| {
                call.callee
                    .resolved_module_id
                    .as_ref()
                    .map(|module_id| (module_id.clone(), call.id.clone()))
            })
            .collect::<Vec<_>>();

        self.test_cases.push(TestCase {
            id: test_id.clone(),
            name: f.sig.ident.to_string(),
            source: Source {
                file: path.to_string_lossy().to_string(),
                line: None,
                column: None,
                language: Some("rust".to_string()),
                text: None,
                text_hash: None,
                extensions: None,
            },
            suite: if module_path.is_empty() {
                None
            } else {
                Some(module_path.to_vec())
            },
            calls: none_if_empty(evidence.calls),
            parameters: none_if_empty(evidence.parameters),
            assertions: none_if_empty(evidence.assertions),
            mocks: None,
            fixtures: None,
            extensions: diagnostics_extensions(&evidence.diagnostics),
        });

        let mut linked_modules = BTreeSet::new();
        for (module_id, call_id) in resolved_links {
            if linked_modules.insert(module_id.clone()) {
                let link_id = self.next_id("link");
                self.test_module_links.push(StagedTestModuleLink {
                    id: link_id,
                    test_ref: test_id.clone(),
                    module_ref: module_id,
                    relationship: Some("assertion_target".to_string()),
                    confidence: Some("high".to_string()),
                    basis: vec!["assertion_macro".to_string()],
                    evidence_refs: vec![call_id],
                });
            }
        }
    }

    fn process_stmt(
        &mut self,
        stmt: &syn::Stmt,
        use_map: &BTreeMap<String, String>,
        module_path: &[String],
        evidence: &mut TestEvidence,
    ) {
        match stmt {
            syn::Stmt::Macro(stmt_mac) => {
                self.process_macro(&stmt_mac.mac, use_map, module_path, evidence);
            }
            syn::Stmt::Expr(expr, _) => {
                self.process_expr(expr, use_map, module_path, evidence);
            }
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    self.process_expr(&init.expr, use_map, module_path, evidence);
                }
            }
            syn::Stmt::Item(_) => {}
        }
    }

    fn process_expr(
        &mut self,
        expr: &syn::Expr,
        use_map: &BTreeMap<String, String>,
        module_path: &[String],
        evidence: &mut TestEvidence,
    ) {
        match expr {
            syn::Expr::Macro(em) => self.process_macro(&em.mac, use_map, module_path, evidence),
            syn::Expr::Block(block) => {
                self.process_block(&block.block, use_map, module_path, evidence)
            }
            syn::Expr::Unsafe(block) => {
                self.process_block(&block.block, use_map, module_path, evidence)
            }
            syn::Expr::If(expr_if) => {
                self.process_expr(&expr_if.cond, use_map, module_path, evidence);
                self.process_block(&expr_if.then_branch, use_map, module_path, evidence);
                if let Some((_, else_expr)) = &expr_if.else_branch {
                    self.process_expr(else_expr, use_map, module_path, evidence);
                }
            }
            _ => {}
        }
    }

    fn process_block(
        &mut self,
        block: &syn::Block,
        use_map: &BTreeMap<String, String>,
        module_path: &[String],
        evidence: &mut TestEvidence,
    ) {
        for stmt in &block.stmts {
            self.process_stmt(stmt, use_map, module_path, evidence);
        }
    }

    fn process_macro(
        &mut self,
        mac: &syn::Macro,
        use_map: &BTreeMap<String, String>,
        module_path: &[String],
        evidence: &mut TestEvidence,
    ) {
        let Some(name) = macro_name(mac) else {
            return;
        };

        match name.as_str() {
            "assert" | "assert_eq" | "assert_ne" => {
                let assertion =
                    self.process_assertion_macro(&name, mac, use_map, module_path, evidence);
                evidence.assertions.push(assertion);
            }
            _ => evidence.diagnostics.push(ParserDiagnostic {
                code: "rust_parser.unsupported_macro",
                message: format!("unsupported macro {name}! was not expanded"),
            }),
        }
    }

    fn process_assertion_macro(
        &mut self,
        name: &str,
        mac: &syn::Macro,
        use_map: &BTreeMap<String, String>,
        module_path: &[String],
        evidence: &mut TestEvidence,
    ) -> Assertion {
        let assertion_id = self.next_id("assertion");
        let args = match parse_macro_args(mac) {
            Some(args) => args,
            None => {
                return Assertion {
                    id: assertion_id,
                    style: AssertionStyle::Unknown,
                    framework: None,
                    matcher: Some(Matcher {
                        name: Some(name.to_string()),
                        arguments: None,
                    }),
                    target_call_refs: None,
                    target_expression: None,
                    expected: None,
                    source: None,
                    extensions: Some(single_diagnostic_extension(ParserDiagnostic {
                        code: "rust_parser.unparsed_assertion",
                        message: format!("could not parse {name}! arguments"),
                    })),
                };
            }
        };

        let target = args.first();
        let mut target_call_refs = Vec::new();
        if let Some(expr) = target
            && let Some(call) =
                self.extract_assertion_target_call(expr, use_map, module_path, evidence)
        {
            target_call_refs.push(call.id.clone());
            evidence.calls.push(call);
        }

        let expected = match name {
            "assert_eq" | "assert_ne" => args.get(1).map(extract_artifact_value),
            _ => None,
        };

        Assertion {
            id: assertion_id,
            style: AssertionStyle::AssertFunction,
            framework: None,
            matcher: Some(Matcher {
                name: Some(name.to_string()),
                arguments: None,
            }),
            target_call_refs: none_if_empty(target_call_refs),
            target_expression: target.map(expr_to_string),
            expected,
            source: None,
            extensions: None,
        }
    }

    fn extract_assertion_target_call(
        &mut self,
        expr: &syn::Expr,
        use_map: &BTreeMap<String, String>,
        module_path: &[String],
        evidence: &mut TestEvidence,
    ) -> Option<Call> {
        let syn::Expr::Call(call_expr) = peel_parens(expr) else {
            return None;
        };

        let callee_text = path_to_string(&call_expr.func)?;
        let (resolution_status, resolved_qname) =
            resolve_callee(&callee_text, use_map, module_path);
        let resolved_module_id = resolved_qname
            .as_ref()
            .map(|qname| self.ensure_module(qname));
        let call_extensions = unresolved_call_extensions(&callee_text, &resolution_status);
        let call_id = self.next_id("call");

        for (index, arg) in call_expr.args.iter().enumerate() {
            let value = extract_artifact_value(arg);
            let array_items = value
                .array_items
                .map(|items| items.into_iter().map(|item| *item).collect());
            evidence.parameters.push(Parameter {
                id: self.next_id("param"),
                argument_index: index as u64,
                value_kind: value.value_kind,
                call_ref: Some(call_id.clone()),
                literal_class: value.literal_class,
                object_shape: unbox_object_shape(value.object_shape),
                array_items,
                origin: value.origin,
                syntax: value.syntax,
                extensions: value.extensions,
            });
        }

        Some(Call {
            id: call_id,
            role: CallRole::AssertionTargetCall,
            callee: Callee {
                text: callee_text,
                resolution_status,
                confidence: None,
                resolved_module_id,
                resolved_symbol: None,
            },
            source: None,
            extensions: call_extensions,
        })
    }
}

fn is_test_fn(f: &syn::ItemFn) -> bool {
    f.attrs.iter().any(|a| a.path().is_ident("test"))
}

fn collect_use_tree(tree: &syn::UseTree, prefix: &str, map: &mut BTreeMap<String, String>) {
    match tree {
        syn::UseTree::Path(p) => {
            let next = join_path(prefix, &p.ident.to_string());
            collect_use_tree(&p.tree, &next, map);
        }
        syn::UseTree::Name(n) => {
            map.insert(n.ident.to_string(), join_path(prefix, &n.ident.to_string()));
        }
        syn::UseTree::Rename(r) => {
            map.insert(
                r.rename.to_string(),
                join_path(prefix, &r.ident.to_string()),
            );
        }
        syn::UseTree::Group(g) => {
            for item in &g.items {
                collect_use_tree(item, prefix, map);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

struct MacroArgs(syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>);

impl syn::parse::Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(syn::punctuated::Punctuated::parse_terminated(input)?))
    }
}

fn parse_macro_args(mac: &syn::Macro) -> Option<Vec<syn::Expr>> {
    mac.parse_body::<MacroArgs>()
        .ok()
        .map(|args| args.0.into_iter().collect())
}

fn macro_name(mac: &syn::Macro) -> Option<String> {
    mac.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn path_to_string(expr: &syn::Expr) -> Option<String> {
    let syn::Expr::Path(ep) = peel_parens(expr) else {
        return None;
    };

    Some(
        ep.path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
    )
}

fn resolve_callee(
    text: &str,
    use_map: &BTreeMap<String, String>,
    module_path: &[String],
) -> (ResolutionStatus, Option<String>) {
    if text.starts_with("crate::") {
        return (ResolutionStatus::Resolved, Some(text.to_string()));
    }

    if text.starts_with("super::") {
        if let Some(qname) = normalize_super_path(text, module_path) {
            return (ResolutionStatus::Resolved, Some(qname));
        }
        return (ResolutionStatus::Unresolved, None);
    }

    if let Some(qname) = use_map.get(text)
        && qname.starts_with("crate::")
    {
        return (ResolutionStatus::Resolved, Some(qname.clone()));
    }

    if let Some(qname) = use_map.get(text)
        && qname.starts_with("super::")
        && let Some(normalized) = normalize_super_path(qname, module_path)
    {
        return (ResolutionStatus::Resolved, Some(normalized));
    }

    // Prefix-qualified call: `use crate::math; math::add(1, 2)`.
    // If the first path segment maps to a crate-rooted path in use_map,
    // append the remaining segments to form the full qualified name.
    if let Some((first, rest)) = text.split_once("::") {
        if let Some(prefix_qname) = use_map.get(first)
            && prefix_qname.starts_with("crate::")
        {
            return (
                ResolutionStatus::Resolved,
                Some(format!("{prefix_qname}::{rest}")),
            );
        }

        if let Some(prefix_qname) = use_map.get(first)
            && prefix_qname.starts_with("super::")
            && let Some(normalized_prefix) = normalize_super_path(prefix_qname, module_path)
        {
            return (
                ResolutionStatus::Resolved,
                Some(format!("{normalized_prefix}::{rest}")),
            );
        }
    }

    (ResolutionStatus::Unresolved, None)
}

fn normalize_super_path(text: &str, module_path: &[String]) -> Option<String> {
    let mut segments = text.split("::").collect::<Vec<_>>();
    let super_count = segments
        .iter()
        .take_while(|segment| **segment == "super")
        .count();
    if super_count == 0 || super_count > module_path.len() {
        return None;
    }

    segments.drain(0..super_count);
    if segments.is_empty() {
        return None;
    }

    let mut normalized = vec!["crate".to_string()];
    normalized.extend(
        module_path[..module_path.len() - super_count]
            .iter()
            .cloned(),
    );
    normalized.extend(segments.into_iter().map(str::to_string));
    Some(normalized.join("::"))
}

fn extract_artifact_value(expr: &syn::Expr) -> ArtifactValue {
    match peel_parens(expr) {
        syn::Expr::Lit(lit) => literal_value(&lit.lit, Some(expr_to_string(expr))),
        syn::Expr::Array(array) if array.elems.is_empty() => ArtifactValue {
            value_kind: ValueKind::ArrayLiteral,
            literal_class: Some(LiteralClass::EmptyArray),
            object_shape: None,
            array_items: Some(Vec::new()),
            origin: None,
            syntax: Some(expr_to_string(expr)),
            extensions: None,
        },
        syn::Expr::Path(_) => ArtifactValue {
            value_kind: ValueKind::Identifier,
            literal_class: None,
            object_shape: None,
            array_items: None,
            origin: None,
            syntax: Some(expr_to_string(expr)),
            extensions: None,
        },
        syn::Expr::Call(_) => ArtifactValue {
            value_kind: ValueKind::CallExpression,
            literal_class: None,
            object_shape: None,
            array_items: None,
            origin: None,
            syntax: Some(expr_to_string(expr)),
            extensions: None,
        },
        syn::Expr::MethodCall(_) | syn::Expr::Field(_) => ArtifactValue {
            value_kind: ValueKind::MemberExpression,
            literal_class: None,
            object_shape: None,
            array_items: None,
            origin: None,
            syntax: Some(expr_to_string(expr)),
            extensions: None,
        },
        _ => ArtifactValue {
            value_kind: ValueKind::Unknown,
            literal_class: None,
            object_shape: None,
            array_items: None,
            origin: None,
            syntax: Some(expr_to_string(expr)),
            extensions: None,
        },
    }
}

fn literal_value(lit: &syn::Lit, syntax: Option<String>) -> ArtifactValue {
    match lit {
        syn::Lit::Str(s) => {
            let value = s.value();
            let literal_class = if value.is_empty() {
                LiteralClass::EmptyString
            } else if value.trim().is_empty() {
                LiteralClass::WhitespaceString
            } else {
                LiteralClass::NonEmptyString
            };
            ArtifactValue {
                value_kind: ValueKind::StringLiteral,
                literal_class: Some(literal_class),
                object_shape: None,
                array_items: None,
                origin: None,
                syntax,
                extensions: None,
            }
        }
        syn::Lit::Int(i) => ArtifactValue {
            value_kind: ValueKind::NumberLiteral,
            literal_class: Some(match i.base10_parse::<i64>() {
                Ok(0) => LiteralClass::Zero,
                Ok(n) if n > 0 => LiteralClass::PositiveNumber,
                Ok(_) => LiteralClass::NegativeNumber,
                Err(_) => LiteralClass::Integer,
            }),
            object_shape: None,
            array_items: None,
            origin: None,
            syntax,
            extensions: None,
        },
        syn::Lit::Float(_) => ArtifactValue {
            value_kind: ValueKind::NumberLiteral,
            literal_class: Some(LiteralClass::Float),
            object_shape: None,
            array_items: None,
            origin: None,
            syntax,
            extensions: None,
        },
        syn::Lit::Bool(b) => ArtifactValue {
            value_kind: ValueKind::BooleanLiteral,
            literal_class: Some(if b.value {
                LiteralClass::True
            } else {
                LiteralClass::False
            }),
            object_shape: None,
            array_items: None,
            origin: None,
            syntax,
            extensions: None,
        },
        _ => ArtifactValue {
            value_kind: ValueKind::LanguageSpecificLiteral,
            literal_class: None,
            object_shape: None,
            array_items: None,
            origin: None,
            syntax,
            extensions: None,
        },
    }
}

fn peel_parens(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Paren(paren) => peel_parens(&paren.expr),
        syn::Expr::Group(group) => peel_parens(&group.expr),
        _ => expr,
    }
}

fn expr_to_string(expr: &syn::Expr) -> String {
    expr.to_token_stream().to_string()
}

fn join_path(prefix: &str, segment: &str) -> String {
    if prefix.is_empty() {
        segment.to_string()
    } else {
        format!("{prefix}::{segment}")
    }
}

fn none_if_empty<T>(items: Vec<T>) -> Option<Vec<T>> {
    if items.is_empty() { None } else { Some(items) }
}

fn diagnostics_extensions(
    diagnostics: &[ParserDiagnostic],
) -> Option<BTreeMap<String, serde_json::Value>> {
    if diagnostics.is_empty() {
        None
    } else {
        let values = diagnostics
            .iter()
            .map(diagnostic_json)
            .collect::<Vec<serde_json::Value>>();
        Some(BTreeMap::from([(
            "diagnostics".to_string(),
            serde_json::Value::Array(values),
        )]))
    }
}

fn single_diagnostic_extension(
    diagnostic: ParserDiagnostic,
) -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([(
        "diagnostics".to_string(),
        serde_json::Value::Array(vec![diagnostic_json(&diagnostic)]),
    )])
}

fn diagnostic_json(diagnostic: &ParserDiagnostic) -> serde_json::Value {
    serde_json::json!({
        "code": diagnostic.code,
        "message": diagnostic.message,
    })
}

fn unresolved_call_extensions(
    callee_text: &str,
    resolution_status: &ResolutionStatus,
) -> Option<BTreeMap<String, serde_json::Value>> {
    if matches!(resolution_status, ResolutionStatus::Unresolved) {
        Some(single_diagnostic_extension(ParserDiagnostic {
            code: "rust_parser.unresolved_call",
            message: format!("could not resolve assertion target call {callee_text}"),
        }))
    } else {
        None
    }
}

fn unbox_object_shape(
    value: Option<BTreeMap<String, Box<ArtifactValue>>>,
) -> Option<BTreeMap<String, ArtifactValue>> {
    value.map(|shape| {
        shape
            .into_iter()
            .map(|(name, value)| (name, *value))
            .collect()
    })
}
