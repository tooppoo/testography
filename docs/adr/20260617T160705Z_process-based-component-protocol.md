# Use Process-Based Component Protocol for Parsers and Evaluators

- Status: Accepted
- Created: 2026-06-17T16:07:05Z

## Context

testography core is intended to be implemented in Rust and distributed as a standalone binary. The core should not require users to install a language runtime such as Node.js, Python, Go, PHP, or .NET in order to run testography itself.

At the same time, testography needs to integrate with parsers and evaluators that may depend on language-specific ecosystems. For example, a TypeScript parser may reasonably use `tsc`, `node`, or TypeScript compiler APIs. A Go parser may use Go tooling. A PHP parser may use PHP ecosystem libraries. An LLM-based evaluator may depend on an external provider SDK, API credentials, or a local model runner.

For this reason, parser and evaluator implementations should not be forced into Rust. A strong Rust trait-based interface would provide good compile-time type consistency inside the core, but it would also make cross-language parser development more expensive. In many cases, the most practical parser for a programming language is likely to be built using that language's own compiler, analyzer, or testing ecosystem.

The integration mechanism therefore needs to balance two competing priorities:

- robust interface enforcement between the core and submodules;
- low-friction integration with independently implemented parsers and evaluators.

For v0, testography prioritizes ease of integration and minimal implementation constraints over the strongest possible interface-level enforcement.

This decision should be recorded as an ADR because it defines the primary extension boundary of testography. It affects parser design, evaluator design, protocol versioning, validation strategy, dependency declaration, packaging, and future support for other component execution models such as WebAssembly.

## Decision

testography v0 will use a process-based component protocol for parsers and evaluators.

The core will invoke parser and evaluator components as executable processes. These components may be implemented in any language and may use any runtime or external tooling required by that component. The only mandatory integration requirement is that the component accepts the expected JSON input and returns the expected JSON output according to the relevant JSON Schema contract.

In other words, JSON Schema is the process boundary contract between testography core and external components.

The core is responsible for:

- invoking configured parser and evaluator executables;
- validating component input and output against the relevant JSON Schemas;
- applying additional domain validation after schema validation;
- handling process failures, invalid JSON, schema violations, unsupported inputs, warnings, and skipped components in a structured way;
- preserving provenance about which component produced which output.

Parser and evaluator components are responsible for:

- implementing the required process protocol;
- accepting and producing the specified JSON payloads;
- declaring their required runtime, commands, environment variables, files, or other dependencies;
- checking whether their own dependencies are available in the current execution environment;
- reporting structured errors and warnings rather than relying only on exit codes.

The implementation language, runtime, and internal architecture of a parser or evaluator are intentionally outside the core contract.

For example, all of the following should be valid component implementation strategies as long as the JSON contract is satisfied:

```text
node tools/testography-ts-parser/index.js
python -m testography_pytest_parser
go-testography-parser
php tools/testography-php-parser/parse.php
testography-evaluator-rules
node tools/testography-llm-evaluator/index.js
```

WebAssembly plugin support remains a future option. It may be introduced later as an additional component runner for sandboxed or more tightly controlled components. It is not the primary v0 integration mechanism.

Rust trait-based integration may still be used for core-internal or built-in components, but it will not be the standard external parser/evaluator integration mechanism for v0.

## Alternatives Considered

### WebAssembly component protocol

A WebAssembly-based plugin model would provide a more uniform runtime boundary and could support stronger host-controlled sandboxing. It could also make capability-based execution more explicit, especially for filesystem, environment variable, and network access.

This was not selected for v0 because it would impose additional authoring requirements on component developers. Component authors would need to target WebAssembly, use the appropriate WASI and component tooling, generate bindings, package components correctly, and debug through a Wasm runtime. Support for WebAssembly varies across language ecosystems, and some language-specific tooling may not be easy to use from inside a Wasm component.

This is especially relevant for parsers. testography parsers may need to reuse existing compiler, analyzer, test runner, or package ecosystem tooling. Requiring Wasm support from the beginning would make such integrations harder.

WebAssembly remains a plausible future runner type, especially for components where sandboxing is more important than direct reuse of existing toolchains.

### Rust trait-based integration

A Rust trait-based plugin model would provide strong type consistency, low runtime overhead, and tight integration with the Rust core. It would also reduce the amount of serialization and schema validation needed at runtime for built-in components.

This was not selected as the standard external component mechanism because it would strongly bias parser and evaluator implementations toward Rust. For language-specific parsers, this creates a cross-language implementation cost. A TypeScript parser is often best implemented with TypeScript tooling. A Go parser is often best implemented with Go tooling. A PHP parser is often best implemented with PHP ecosystem libraries.

If Rust traits were used as the main external extension model, testography would either need to reimplement many language-specific parsers in Rust or build wrappers that call external tools anyway. The latter would effectively return to a process-based design while adding extra internal coupling.

Rust traits remain appropriate for internal abstractions and built-in components that are compiled with the core.

### Single-language parser and evaluator implementation

Another option would be to require all official parsers and evaluators to be implemented in Rust and shipped with the core binary.

This was not selected because it would make the core larger, increase the maintenance burden, and reduce the ability to reuse mature language-specific tooling. It would also make experimental or project-specific evaluators harder to add.

## Consequences

### Positive Consequences

- Parser and evaluator authors can use the implementation language and runtime most suitable for their target domain.
- Existing compiler, analyzer, test framework, and LLM tooling can be reused directly.
- testography core can remain a standalone Rust binary without directly depending on Node.js, Python, Go, PHP, LLM SDKs, or local model runners.
- JSON Schema provides a stable, inspectable, language-neutral contract between the core and external components.
- The external component ecosystem can evolve without recompiling or relinking the core.
- WebAssembly support can still be added later as an additional runner type without invalidating the v0 process-based protocol.

### Negative Consequences

- Process startup, JSON serialization, JSON parsing, and schema validation introduce runtime overhead.
- JSON Schema can validate the structural shape of exchanged data, but it cannot fully guarantee domain-level correctness.
- Interface robustness is weaker than a Rust trait-based model because component correctness is checked at runtime rather than compile time.
- Component dependency checks may vary in quality because each component is responsible for declaring and checking its own runtime requirements.
- Process-based components are not sandboxed by default. Running third-party components must be treated as running arbitrary executable code.
- Reproducibility depends not only on testography core but also on component versions, component runtimes, external tools, environment variables, and, for LLM evaluators, model/provider behavior.

### Neutral Consequences

- JSON Schema should be treated as the process boundary contract, not as the full internal domain model of the core.
- The core should perform domain validation after schema validation before accepting parser or evaluator output as normalized internal data.
- Components should support structured introspection and environment checks so users can understand which dependencies are required and which are missing.
- Optional components and optional component dependencies should be modeled separately.
- The v0 design should avoid assuming that process-based components are security-isolated.
- Future runner types, such as `wasm` or `builtin`, can coexist with the process-based runner if the component protocol remains explicit.
