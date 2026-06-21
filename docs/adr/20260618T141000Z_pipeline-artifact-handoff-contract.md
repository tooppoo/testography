# Use File-Based Pipeline Artifact Handoff

- Status: Superseded by [20260621T000000Z_staged-pipeline-artifact-handoff-contract.md](20260621T000000Z_staged-pipeline-artifact-handoff-contract.md)
- Created: 2026-06-18T14:10:00Z

## Context

Testography v0 executes the parser → evaluator → reporter flow as separate `collect`, `evaluate`, and `report` steps.

Each step must hand off an artifact to the next step and validate that artifact against a JSON Schema.

The pipeline was initially described using a UNIX pipeline metaphor, but v0 does not require native stdin/stdout chaining.

Applying multiple evaluators sequentially requires `evaluate` to accept not only unassessed evidence artifacts but also artifacts that already carry assessment layers. At the same time, allowing evaluators to return a full assessed artifact would give external components the opportunity to destructively overwrite evidence data.

## Decision

- Pipeline steps must communicate through file-based artifact handoff in v0.
- Native stdin/stdout chaining is not required in v0.
- An assessed artifact must carry both evidence data and assessment layers.
- `evaluate` must accept both evidence artifacts and assessed artifacts as evidence-compatible input.
- Evaluators must return assessment layers, not full assessed artifacts.
- The core pipeline must construct assessed artifacts from the input artifact and the returned assessment layer.

Conceptual model:

```text
EvidenceArtifact
  = evidence data

AssessedArtifact
  = evidence data
  + assessment layers
```

When `evaluate` receives an evidence artifact it writes an assessed artifact containing the original evidence and the new assessment layer.

When `evaluate` receives an assessed artifact it preserves the existing evidence and assessment layers, appends the new assessment layer, and writes an assessed artifact.

`evaluate` rejects inputs that are neither evidence artifacts nor assessed artifacts.

`report` accepts only assessed artifacts as input.

## Alternatives Considered

### Native stdin/stdout chaining

Rejected for v0.

The UNIX pipeline metaphor is useful conceptually, but v0 prioritises file-based handoff so that intermediate artifacts can be inspected, validated, stored, and reused between steps.

### Treat assessed artifact as evidence artifact

Rejected.

Treating `assessed_artifact` as equivalent to `evidence` would blur the meaning of `artifact_type` and make the input contract of `report` ambiguous.

### Evaluator returns a full assessed artifact

Rejected for v0.

Flexible but it would allow evaluators to destructively modify evidence data. In v0 evaluators return only assessment layers and the core constructs the assessed artifact.

## Consequences

### Positive Consequences

- Intermediate artifacts can be stored, validated, inspected, and reused between steps.
- `collect`, `evaluate`, and `report` can be implemented as file-path-based CLI commands without native stdin/stdout chaining.
- Multiple evaluators can be applied sequentially by passing assessed artifacts back into `evaluate`.
- The project can distinguish evidence artifacts from assessed artifacts while preserving the meaning of `artifact_type`.
- Evaluator responsibility is limited to producing assessment layers.
- The core owns artifact handoff normalisation and assessed artifact construction.
- The design reduces the chance that evaluators destructively modify evidence data.

### Negative Consequences

- The core pipeline must normalise two input artifact shapes for `evaluate`: evidence artifacts and assessed artifacts.
- Evaluators cannot directly rewrite assessed artifacts; evaluator implementations must express their result as an assessment layer.
- File-based handoff requires explicit temporary file or output path management when callers want one-shot execution.
- The core must own additional assembly logic for assessed artifacts.

### Neutral Consequences

- Native stdin/stdout chaining can still be introduced later, but it is not part of the v0 contract.
- Reporter output remains outside this ADR unless a separate reporter contract ADR is created.

## Related

- #2
- #3
- #4
