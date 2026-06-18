# Treat Reporter Output as Rendered Output

- Status: Accepted
- Created: 2026-06-18T14:32:21Z

## Context

Testography v0 defines `collect`, `evaluate`, and `report` as separate pipeline steps.

`collect` and `evaluate` produce JSON artifacts that are validated by JSON Schema and passed to later steps.

`report` consumes an assessed artifact, but its purpose is rendering a user-facing or machine-facing report rather than producing another pipeline artifact.

Future reporters may emit Markdown, plain text, JSON, HTML, or other rendered formats. Requiring reporter output to always be a JSON artifact would over-constrain reporter implementations and blur the distinction between validated pipeline artifacts and rendered outputs.

## Decision

- Reporters must consume assessed artifacts.
- Reporter input must be validated before rendering.
- Reporter output must be treated as rendered output, not as a pipeline artifact.
- Reporter output is not required to be a JSON artifact.
- Reporter output format may vary by reporter implementation.

## Alternatives Considered

### Require report output to be a JSON artifact

Rejected for v0.

This would make report output uniformly machine-readable, but it would over-constrain reporters that naturally produce Markdown or other rendered formats.

### Treat reporters as another artifact-producing pipeline stage

Rejected for v0.

This would make the pipeline shape more uniform, but it would blur the distinction between validated intermediate artifacts and final rendered outputs.

### Define only Markdown report output in v0

Rejected for this ADR.

Markdown may be a useful initial reporter output format, but the reporter contract should not require all reporters to produce Markdown.

## Consequences

### Positive Consequences

- Reporters can produce formats appropriate to their use case.
- Markdown, text, JSON, HTML, or other rendered outputs can be supported without changing the core artifact model.
- The core can keep JSON Schema validation focused on pipeline artifacts.
- The distinction between intermediate artifacts and final rendered outputs remains clear.

### Negative Consequences

- Downstream tools cannot assume that report output is a schema-validated JSON artifact.
- Consumers that need machine-readable reports must select a reporter that explicitly produces machine-readable output.
- Report output validation, if needed, must be handled by reporter-specific logic rather than the core artifact validator.

### Neutral Consequences

- A JSON reporter may still exist, but its JSON output is a rendered report format, not a required pipeline artifact.
- Reporter-specific output contracts can be documented separately when concrete reporters are added.

## Related

- #4
- #9

## Notes

Pipeline handoff, assessed artifact model, and evaluator output contract are covered in `Use File-Based Pipeline Artifact Handoff`.
