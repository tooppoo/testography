# Testography

Static evidence maps for test code.

Testography is a language-neutral test analysis pipeline. It parses test code into structured evidence, groups that evidence by target module, lets pluggable evaluators add assessment layers, and keeps reporting as an independent output layer.

The CLI is named `tgraphy`.

```sh
tgraphy collect
tgraphy evaluate
tgraphy report
```

## Why Testography?

Generated tests often look plausible while hiding important problems:

- the test name does not match what the test actually calls
- the assertion is too broad
- many tests cover the same parameter class
- mocked dependencies erase the behavior supposedly under test
- tests are organized by file, while the meaningful unit is the target module
- coverage increases, but the test intent remains unclear

Testography does not try to execute tests or replace your test runner. Instead, it statically analyzes test code and builds evidence that can be inspected, evaluated, and rendered.

The core idea is simple:

```text
test code
  -> evidence
  -> assessment layers
  -> reports
```

## Core principles

### Evidence first

Parsers extract evidence. They do not judge test quality.

Examples of evidence:

- which module or symbol is called
- where the call appears
- what source text produced the call
- what arguments are passed
- what assertion matcher is used
- what mocks, spies, stubs, fixtures, or setup blocks appear
- which test appears to be linked to which target module

Examples of assessment:

- the assertion is probably underspecified
- the parameter set only covers one equivalence class
- two tests are likely redundant
- the test name overstates what is actually verified
- the target module is ambiguous

Evidence and assessment are separate artifacts.

### JSON is the source of truth

Testography uses JSON as the canonical artifact format.

TOON may be used as a compact LLM-facing projection, but TOON is not the canonical format. In particular, the LLM evaluator may convert JSON evidence and prior assessments into TOON internally, send that to an LLM, validate the returned JSON, and append the result as another assessment layer.

### Language-specific parsing, language-neutral evidence

Source parsing is language-specific.

For example, a TypeScript parser may use TypeScript-specific tooling to resolve imports, symbols, calls, assertions, and parameters.

After evidence is produced, later stages should not depend on the source language unless explicitly configured to do so.

```text
TypeScript parser
  -> language-neutral evidence JSON
  -> evaluator pipeline
  -> assessed artifact
  -> reporter layer
```

### Module-centered aggregation

Testography does not treat the test file as the primary unit of analysis.

The primary aggregation unit is the target module or target symbol.

This means:

- tests in different files can be grouped under the same target module
- tests in the same file can be split across different target modules
- one test may be linked to multiple modules
- unresolved or ambiguous targets are preserved rather than guessed away

### Evaluators are pluggable

Evaluators add assessment layers.

They can be generic, language-specific, test-framework-specific, project-specific, or LLM-based.

Examples:

```text
generic-static evaluator
typescript-static evaluator
vitest-static evaluator
project-domain evaluator
llm evaluator
```

Evaluators can be combined in arbitrary order.

```text
evidence
  -> generic-static evaluator
  -> vitest-static evaluator
  -> project-domain evaluator
  -> llm evaluator
  -> assessed artifact
```

A later evaluator may read the results of earlier evaluators. This allows static evaluators to improve LLM accuracy, reduce token usage, or provide structured hints before higher-level assessment.

### Reporting is separate from evaluation

Reporters do not add new assessments.

They render an already assessed artifact into one or more output formats.

The same assessment artifact may produce:

- Markdown
- JSON
- HTML
- SARIF
- GitHub comments
- CI summaries

## Pipeline model

Testography uses an artifact pipeline model.

This is inspired by UNIX pipelines, but it does not require native stdin/stdout chaining. Each stage receives the previous artifact, transforms or augments it, and passes the result to the next stage.

```text
Parser
  -> Evidence Artifact

Transformer
  -> Module Evidence Bundles

Evaluator Pipeline
  -> Assessed Artifact

Reporter Layer
  -> Reports
```

## Concepts

### Evidence Artifact

An evidence artifact contains structured facts extracted from test code.

It should be deterministic and reproducible.

Example:

```json
{
  "schema_version": "0.1.0",
  "artifact_type": "evidence",
  "evidence": {
    "test_cases": [],
    "modules": [],
    "test_module_links": []
  }
}
```

### Test Case Evidence

A test case evidence entry represents a discovered test case and the static facts extracted from it.

It may contain:

- test name
- source location
- calls
- parameters
- assertions
- mocks
- fixtures
- source spans
- source text snippets

Example:

```json
{
  "id": "test-001",
  "name": "rejects empty name",
  "source": {
    "file": "tests/user.test.ts",
    "line": 12,
    "language": "typescript"
  },
  "calls": [
    {
      "id": "call-001",
      "role": "assertion_target_call",
      "callee": {
        "text": "createUser",
        "resolution_status": "resolved",
        "resolved_module_id": "symbol:src/user/createUser.ts#createUser"
      },
      "source": {
        "text": "createUser({ name: \"\" })",
        "text_hash": "sha256:..."
      }
    }
  ],
  "parameters": [
    {
      "id": "param-001",
      "call_ref": "call-001",
      "argument_index": 0,
      "value_kind": "object_literal",
      "object_shape": {
        "name": {
          "value_kind": "string_literal",
          "literal_class": "empty_string"
        }
      },
      "origin": "inline_literal"
    }
  ],
  "assertions": [
    {
      "id": "assertion-001",
      "style": "expect_matcher",
      "matcher": {
        "name": "toThrow",
        "arguments": []
      },
      "target_call_refs": ["call-001"],
      "source": {
        "text": "expect(() => createUser({ name: \"\" })).toThrow()"
      }
    }
  ]
}
```

### Test Module Link

A test module link connects a test case to one or more target modules.

The link is not a quality judgment. It records why the tool believes the test is related to that module.

Example:

```json
{
  "test_id": "test-001",
  "module_id": "symbol:src/user/createUser.ts#createUser",
  "relationship": "assertion_target",
  "confidence": "high",
  "basis": [
    "call resolves to src/user/createUser.ts#createUser",
    "call appears inside assertion target"
  ]
}
```

### Module Evidence Bundle

A module evidence bundle groups all relevant test evidence for a target module.

Example:

```json
{
  "module_id": "symbol:src/user/createUser.ts#createUser",
  "tests": [
    {
      "test_id": "test-001",
      "relationship": "assertion_target",
      "evidence_refs": ["call-001", "param-001", "assertion-001"]
    }
  ]
}
```

### Assessment Layer

An evaluator appends an assessment layer.

The original evidence should remain unchanged.

Example:

```json
{
  "id": "layer-vitest-static-001",
  "producer": {
    "name": "vitest-static",
    "version": "0.1.0",
    "kind": "static"
  },
  "assessments": [
    {
      "id": "assessment-001",
      "kind": "static_rule_match",
      "rule_id": "vitest.toThrow.no_expected_argument",
      "statement": "toThrow is used without an expected error argument.",
      "evidence_refs": ["assertion-001"],
      "confidence": "high"
    }
  ]
}
```

An LLM evaluator may then refer to this prior assessment:

```json
{
  "id": "assessment-llm-001",
  "kind": "llm_assessment",
  "category": "assertion_specificity",
  "status": "possibly_underspecified",
  "statement": "The assertion may be underspecified because it checks only that some error is thrown.",
  "evidence_refs": ["assertion-001"],
  "assessment_refs": ["assessment-001"],
  "confidence": "medium"
}
```

## Evaluator filtering

Evaluator capabilities cannot be fully expressed by static configuration alone.

Each evaluator may provide a filter interface that inspects the actual artifact and returns an applicability result.

The core uses this result to decide which evidence should be passed to the evaluator and which evidence should be treated as unexpected.

Conceptual result shape:

```ts
type ApplicabilityResult =
  | {
      ok: true;
      value: {
        status: "applicable";
        accepted_refs: string[];
        diagnostics?: Diagnostic[];
      };
    }
  | {
      ok: true;
      value: {
        status: "partial";
        accepted_refs: string[];
        unexpected_refs: string[];
        diagnostics?: Diagnostic[];
      };
    }
  | {
      ok: true;
      value: {
        status: "not_applicable";
        unexpected_refs: string[];
        diagnostics?: Diagnostic[];
      };
    }
  | {
      ok: false;
      error: {
        code:
          | "invalid_artifact"
          | "schema_mismatch"
          | "missing_required_field"
          | "internal_error";
        message: string;
        refs?: string[];
      };
    };
```

Unexpected evidence is not necessarily an error.

Default behavior is warning. It can be configured per evaluator.

```json
{
  "evaluators": [
    {
      "name": "vitest-static",
      "unexpected_evidence_policy": "warn",
      "invalid_input_policy": "error"
    }
  ]
}
```

Supported unexpected evidence policies:

```text
warn
ignore
error
```

## LLM evaluator

The LLM evaluator consumes JSON artifacts, not raw source files.

Internally, it may project selected evidence and prior assessments into TOON.

```text
JSON artifact
  -> TOON projection
  -> LLM
  -> JSON assessment
  -> schema validation
  -> appended assessment layer
```

The LLM evaluator should:

- use evidence IDs and assessment IDs
- distinguish facts from interpretations
- avoid treating prior static assessments as final truth
- return JSON that conforms to the assessment schema
- append a new assessment layer rather than modifying evidence

## Reporter layer

Reporters consume assessed artifacts and render them.

They do not add assessments.

Example:

```sh
tgraphy report --format markdown
tgraphy report --format html
tgraphy report --format json
```

A report may include:

- module summary
- test evidence summary
- assertion quality assessments
- parameter coverage assessments
- duplicate candidates
- unresolved target warnings
- evaluator diagnostics
- recommendations

## CLI sketch

```sh
# Parse source/test code and produce evidence
tgraphy collect --parser typescript --out .testography/evidence.json

# Transform test-case evidence into module-centered bundles
tgraphy transform module-bundle \
  --in .testography/evidence.json \
  --out .testography/module-bundles.json

# Run evaluator pipeline
tgraphy evaluate \
  --in .testography/module-bundles.json \
  --evaluator generic-static \
  --out .testography/assessed-1.json

tgraphy evaluate \
  --in .testography/assessed-1.json \
  --evaluator vitest-static \
  --out .testography/assessed-2.json

tgraphy evaluate \
  --in .testography/assessed-2.json \
  --evaluator llm \
  --out .testography/assessed-final.json

# Render reports
tgraphy report \
  --in .testography/assessed-final.json \
  --format markdown \
  --out .testography/report.md

tgraphy report \
  --in .testography/assessed-final.json \
  --format html \
  --out .testography/report.html
```

For convenience:

```sh
tgraphy run
```

## Configuration sketch

```json
{
  "schema_version": "0.1.0",
  "parser": {
    "name": "typescript",
    "config": {
      "tsconfig": "tsconfig.json",
      "test_frameworks": ["vitest"]
    }
  },
  "transforms": [
    {
      "name": "module-bundle"
    }
  ],
  "evaluators": [
    {
      "name": "generic-static",
      "unexpected_evidence_policy": "warn",
      "invalid_input_policy": "error"
    },
    {
      "name": "vitest-static",
      "unexpected_evidence_policy": "warn",
      "invalid_input_policy": "error"
    },
    {
      "name": "llm",
      "config": {
        "input_projection": "toon",
        "consume_prior_assessments": true
      }
    }
  ],
  "reporters": [
    {
      "name": "markdown",
      "out": ".testography/report.md"
    },
    {
      "name": "html",
      "out": ".testography/report.html"
    },
    {
      "name": "json",
      "out": ".testography/report.json"
    }
  ]
}
```

## MVP scope

Initial scope:

- TypeScript parser
- Vitest/Jest-style test syntax
- static extraction only
- JSON evidence artifact
- module-centered aggregation
- pluggable evaluator pipeline
- LLM evaluator with TOON projection
- Markdown and JSON reporters

Initial parser support:

- `describe`
- `test`
- `it`
- `test.each`
- `expect`
- `vi.mock`
- `jest.mock`
- direct function calls
- simple method calls
- inline literals
- local const literals
- object and array literals

Out of scope for MVP:

- runtime coverage
- per-test execution tracing
- full dependency injection resolution
- dynamic import resolution
- complete type-flow analysis
- automatic deletion or rewriting of tests
- treating coverage as a quality score

## Non-goals

Testography is not:

- a test runner
- a coverage tool
- a replacement for Vitest, Jest, pytest, Go test, or Cargo test
- a linter that directly fails code by default
- a tool that claims tests are good merely because they exist
- a tool that treats LLM output as unquestionable truth

## Design summary

```text
Parsers produce evidence.
Transformers reorganize evidence.
Evaluators append assessment layers.
LLM evaluators may use TOON internally.
Reporters render assessed artifacts.
JSON remains the source of truth.
```

## License

TBD
