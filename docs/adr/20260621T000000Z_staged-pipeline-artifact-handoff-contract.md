# Define Staged Pipeline Artifact Handoff Contract

- Status: Accepted
- Created: 2026-06-21T00:00:00Z
- Supersedes: [20260618T141000Z_pipeline-artifact-handoff-contract.md](20260618T141000Z_pipeline-artifact-handoff-contract.md)

## Context

The existing pipeline artifact handoff ADR describes `evaluate` as accepting evidence-compatible input and assessed artifacts. That contract is too broad once parser output and transform output are represented as distinct artifact stages.

The staged evidence artifact contracts defined in [20260620T000000Z_staged-evidence-artifact-contracts.md](20260620T000000Z_staged-evidence-artifact-contracts.md) establish three named artifact stages:

- `parsed_evidence` — parser output
- `module_evidence` — module-bundle transform output
- `assessed_module_evidence` — evaluator output and reporter input

With these three distinct stages, the pipeline handoff contract must define which components may read and write each stage, which stage transitions are valid, and how invalid stage handoffs are classified.

A broad contract that allows `evaluate` to accept any "evidence-compatible" input creates an ambiguity: it does not prevent `parsed_evidence` from reaching `evaluate` unchecked. An artifact that has not been through the module-bundle transform does not carry `module_bundles` data, which `evaluate` and `report` depend on.

This decision is recorded as an ADR because it defines a long-lived contract between independently implemented pipeline components. It affects `collect`, the module-bundle transform, `evaluate`, `report`, and any tool that validates or inspects intermediate artifacts.

## Decision

Define the staged pipeline artifact handoff contract as follows.

### Stage read/write permissions

| Component              | Reads                                              | Writes                    |
|------------------------|----------------------------------------------------|---------------------------|
| `collect`              | —                                                  | `parsed_evidence`         |
| module-bundle transform| `parsed_evidence`                                  | `module_evidence`         |
| `evaluate`             | `module_evidence`, `assessed_module_evidence`      | `assessed_module_evidence`|
| `report`               | `assessed_module_evidence`                         | —                         |

### Valid artifact-stage transitions

```text
collect
  └─ writes parsed_evidence

module-bundle transform
  ├─ reads  parsed_evidence
  └─ writes module_evidence

evaluate (initial)
  ├─ reads  module_evidence
  └─ writes assessed_module_evidence

evaluate (chained)
  ├─ reads  assessed_module_evidence
  └─ writes assessed_module_evidence

report
  └─ reads  assessed_module_evidence
```

`parsed_evidence` reaching `evaluate` is a pipeline / artifact contract error, not a component execution error.

### Non-destructive evaluator output

Evaluators must return assessment layers, not full assessed artifacts. The core pipeline must:

1. Read the input artifact (`module_evidence` or `assessed_module_evidence`).
2. Run the evaluator to obtain a new assessment layer.
3. Construct a new `assessed_module_evidence` that preserves the parser-produced `evidence`, transform-produced `module_bundles`, and all existing `assessment_layers`.
4. Append the new assessment layer to `assessment_layers`.

This ensures that evaluators cannot destructively overwrite parser evidence, module-bundle data, or prior assessment layers.

### Evaluator chaining in v0

When multiple evaluators are applied sequentially, each later evaluator reads the `assessed_module_evidence` produced by the previous evaluator. The core pipeline preserves all existing `assessment_layers` and appends the new evaluator's assessment layer.

Detailed merge, replacement, and de-duplication semantics for `assessment_layers` are out of scope for v0. The minimum rule is append-only chaining.

### Artifact shape alignment

The `assessed_module_evidence` artifact does not use a nested `module_evidence` wrapper. It preserves parser-produced `evidence`, transform-produced `module_bundles`, and evaluator-produced `assessment_layers` as top-level staged fields. See [20260620T000000Z_staged-evidence-artifact-contracts.md](20260620T000000Z_staged-evidence-artifact-contracts.md) for the full artifact shape definition.

## Alternatives Considered

### Stage-ambiguous evidence

One alternative was to use a single broad evidence artifact whose pipeline stage is inferred from the presence or absence of optional later-stage fields.

This was rejected because it obscures whether the artifact has passed the module-bundle transform stage. An artifact with no `module_bundles` field could be either a `parsed_evidence` artifact that has never entered the transform, or a corrupt or partial `module_evidence` artifact. Validation cannot distinguish these cases without an explicit `artifact_type` discriminant.

Explicit staged artifact types allow the core to reject artifacts from the wrong stage and make the pipeline flow inspectable without inferring stage from optional fields.

### Allow `evaluate` to accept `parsed_evidence`

One alternative was to allow `evaluate` to accept all three artifact stages and produce output depending on input type.

This was rejected because allowing `evaluate` to process `parsed_evidence` would bypass the module-bundle transform stage silently. Reporters and evaluators depend on `module_bundles` data. If `evaluate` were allowed to accept `parsed_evidence`, a missing transform step would produce incomplete assessments rather than a clear contract error.

### Evaluator returns a full assessed artifact

One alternative was to allow evaluators to return a complete `assessed_module_evidence` artifact rather than only assessment layers.

This was rejected because it would give evaluators the opportunity to destructively overwrite parser evidence, module-bundle data, or prior assessment layers. Keeping evaluator output to assessment layers only limits evaluator responsibility and makes the non-destructive property easier to enforce.

## Consequences

### Positive Consequences

- The core can reject `parsed_evidence` at the `evaluate` boundary and classify it as a pipeline / artifact contract error rather than a component error.
- Each component's input and output contracts are explicit and independently verifiable.
- Multiple evaluators can be applied sequentially by passing `assessed_module_evidence` back into `evaluate`.
- Evaluator implementations cannot destructively overwrite evidence or prior assessment data.
- Invalid stage handoffs are distinguishable from component execution failures and artifact validation failures.

### Negative Consequences

- `evaluate` must validate the `artifact_type` of its input and handle two valid input stages explicitly.
- The module-bundle transform is a mandatory pipeline step; there is no shortcut from `parsed_evidence` to `assessed_module_evidence`.
- The core owns assessed artifact assembly and must not delegate that to evaluators.

### Neutral Consequences

- File-based artifact handoff is preserved from the superseded ADR.
- Native stdin/stdout chaining remains out of scope for v0.
- Detailed `assessment_layers` merge and de-duplication semantics remain out of scope for v0.
- Reporter output contract is defined in a separate ADR.

## Related

- #16
- [20260618T141000Z_pipeline-artifact-handoff-contract.md](20260618T141000Z_pipeline-artifact-handoff-contract.md) (superseded)
- [20260620T000000Z_staged-evidence-artifact-contracts.md](20260620T000000Z_staged-evidence-artifact-contracts.md)
- #15
