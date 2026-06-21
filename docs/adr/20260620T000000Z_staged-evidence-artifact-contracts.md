# Define staged evidence artifact contracts

- Status: Accepted
- Created: 2026-06-20T00:00:00Z

## Context

Testography components exchange artifacts through JSON-based contracts. The core pipeline needs to distinguish the output of parsers, module-bundle transforms, evaluators, and reporters without relying on ambiguous or stage-dependent interpretation of a single evidence object.

The initial evidence model was sufficient for parser-produced primary evidence, but module-centered derived data and evaluator-produced assessment data introduce additional concerns:

- parser-produced evidence must remain distinguishable from transform-produced derived data;
- evaluator output must be readable by reporters without requiring evaluator-specific rule knowledge;
- schema validation must reject artifacts from the wrong pipeline stage;
- reference integrity must remain explicit enough for AI, humans, and tooling to inspect the artifact;
- module-centered views must not silently omit parser-produced test-module relationships.

This decision should be recorded as an ADR because it defines a long-lived artifact contract between independently implemented components. It affects parser output, transform output, evaluator output, reporter input, JSON Schema files, Rust artifact types, fixtures, and validation behavior. Keeping this only in an issue would make the architectural intent harder to recover after the schema details evolve.

## Decision

Define three staged evidence artifact contracts:

1. `parsed_evidence`
2. `module_evidence`
3. `assessed_module_evidence`

`parsed_evidence` is the parser output artifact. It contains parser-produced primary evidence.

```json
{
  "artifact_type": "parsed_evidence",
  "evidence": {
    "test_cases": [],
    "modules": [],
    "test_module_links": []
  }
}
```

`module_evidence` is the module-bundle transform output artifact. It preserves parser-produced `evidence` and adds transform-produced `module_bundles` at the artifact top level.

```json
{
  "artifact_type": "module_evidence",
  "evidence": {
    "test_cases": [],
    "modules": [],
    "test_module_links": []
  },
  "module_bundles": []
}
```

`assessed_module_evidence` is the evaluator output and reporter input artifact. It preserves parser-produced `evidence`, transform-produced `module_bundles`, and adds evaluator-produced `assessment_layers` at the artifact top level.

```json
{
  "artifact_type": "assessed_module_evidence",
  "evidence": {
    "test_cases": [],
    "modules": [],
    "test_module_links": []
  },
  "module_bundles": [],
  "assessment_layers": []
}
```

The following structure rules apply:

- `evidence` is parser-produced primary evidence.
- `module_bundles` is transform-produced module-centered derived data.
- `assessment_layers` is evaluator-produced assessment data.
- `module_bundles` must not be nested inside `evidence`.
- `assessed_module_evidence` must not contain a nested `module_evidence` wrapper.
- Each artifact stage has its own JSON Schema and Rust artifact type.
- Validation dispatches by `artifact_type`.
- Each artifact schema is closed by default using `additionalProperties: false`.

`test_module_links[]` entries now carry a required `id` field that enables stable cross-referencing from `module_bundles[].tests[].link_ref`. The field names `test_ref` and `module_ref` replace the old `test_id` / `module_id` to clarify their role as references.

`module_evidence` represents a total module-centered derived view of `parsed_evidence` in v0. Partial or filtered module-bundle views are out of scope.

This means:

- every module in `evidence.modules[]` is represented by exactly one module bundle;
- a module with no linked tests is represented by a module bundle with an empty `tests` array;
- every `evidence.test_module_links[].id` appears exactly once as a `module_bundles[].tests[].link_ref`;
- `link_ref` values are globally unique across all module bundles;
- the same `test_ref` may appear in multiple module bundles only through distinct `test_module_links` entries.

`assessment_layers` is defined as a minimal evaluator result container. It is not intended to encode evaluator rule semantics in this ADR. The reporter can rely on each assessment layer containing evaluator identity and a list of findings. Finding severity is distinct from schema validity: a finding with `level = error` represents evaluator severity and does not make the artifact schema-invalid by itself.

## Schema files and Rust type mapping

| Artifact type             | Schema file                                              | Rust type                      |
|---------------------------|----------------------------------------------------------|--------------------------------|
| `parsed_evidence`         | `schemas/parsed_evidence/parsed_evidence.v0.json`        | `ParsedEvidenceArtifact`       |
| `module_evidence`         | `schemas/module_evidence/module_evidence.v0.json`        | `ModuleEvidenceArtifact`       |
| `assessed_module_evidence`| `schemas/assessed_module_evidence/assessed_module_evidence.v0.json` | `AssessedModuleEvidenceArtifact` |

Rust validation dispatches by `artifact_type` in `parse_artifact`. `module_evidence` and `assessed_module_evidence` schemas reference `parsed_evidence.v0.json` as a cross-schema resource for the shared `evidence` structure.

## Reference integrity policy

Uniqueness constraints that JSON Schema alone cannot express are enforced by Rust-side reference integrity validation:

- `evidence.test_module_links[].id` must be unique within the artifact.
- `module_bundles[].module_ref` must be unique across all bundles.
- `module_bundles[].tests[].link_ref` must be globally unique across all module bundles.
- `assessment_layers[].id` must be unique within the artifact.
- `assessment_layers[].findings[].id` must be unique within each layer.

Cross-field consistency is also enforced:

- `module_bundles[].tests[].test_ref` must match the `test_ref` of the resolved `test_module_links[]` entry.
- The resolved `test_module_links[]` entry's `module_ref` must match the parent `module_bundles[].module_ref`.

## Alternatives Considered

### Keep a single evidence artifact type

One option was to keep a single `evidence` artifact and allow later pipeline stages to add fields to it.

This was not selected because it would make the pipeline stage ambiguous. Parser output, transform output, and evaluator output would share a shape whose meaning depends on optional fields. That weakens validation and makes it harder for the core to reject an artifact from the wrong stage.

### Nest transform output inside `evidence`

Another option was to place `module_bundles` inside the parser-produced `evidence` object.

This was not selected because it mixes parser-produced primary evidence with transform-produced derived data. The distinction between source evidence and derived module-centered views would become harder to maintain.

### Use a nested `module_evidence` wrapper inside `assessed_module_evidence`

Another option was to make `assessed_module_evidence` contain a nested `module_evidence` wrapper plus `assessment_layers`.

This was not selected because it introduces extra nesting without adding a clear semantic boundary. The artifact already carries its stage through `artifact_type`, and top-level staged enrichment keeps the structure easier to validate, inspect, and consume.

### Allow partial or filtered module-bundle views

Another option was to allow `module_bundles` to omit some `test_module_links[]` entries.

This was not selected for v0. If omitted links are allowed, validation cannot distinguish an intentional filter from a transform bug. Evaluators and reporters reading only `module_bundles` could silently miss existing test-module relationships. A total view is safer as the default artifact contract.

Partial or filtered views may be introduced later as a separate artifact type or with explicit omission metadata.

### Allow direct findings on module bundles

Another option was to allow `assessment_layers[].findings[].subjects[]` to directly reference `module_bundle` or `module_bundle_test`.

This was not selected for v0 because module bundles do not yet have stable IDs. JSON Pointer or array-index references would depend on ordering and are not stable enough for long-term references, diffs, or regenerated artifacts. Direct bundle references can be reconsidered if stable `module_bundles[].id` values are introduced.

## Consequences

### Positive Consequences

- Parser output, transform output, and evaluator output have explicit artifact contracts.
- The core can dispatch validation by `artifact_type`.
- Schema validation can reject artifacts from the wrong stage.
- Parser-produced evidence remains distinguishable from transform-produced derived data.
- Evaluators and reporters can rely on `module_evidence` as a complete module-centered view.
- Silent data loss in the module-bundle transform is easier to detect.
- Human and AI inspection is easier because `module_bundles[].tests[]` includes explicit `test_ref` and `link_ref`.
- The model remains extensible for future evaluator layers without forcing evaluator rules into the base evidence schema.

### Negative Consequences

- The artifact contains some deliberate redundancy, especially where `module_bundles[].tests[].test_ref` repeats information reachable through `link_ref`.
- Reference integrity validation becomes more complex than JSON Schema alone can conveniently express.
- Rust-side validation is required for property-level uniqueness and cross-field consistency.
- Partial or filtered module-bundle views cannot be represented by `module_evidence` v0.
- A transform cannot emit a valid `module_evidence` artifact unless it can account for every module and every test-module link.

### Neutral Consequences

- Future stable IDs for `module_bundles[]` may be needed if findings need to target bundles directly.
- `assessment_layers` defines a common result container but does not define evaluator rules.
- CLI exit-code behavior based on assessment severity remains a separate decision.
- The schema files and Rust artifact types must remain synchronized as the artifact contracts evolve.
