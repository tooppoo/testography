# Summary

- Modules: 1
- Assessment layers: 1
- Findings: 1

# Modules

## mod-001

- tc-001 (link: link-001)

# Assessment Layers

## rust-static-0 (rust-static v0.0.1)

Findings: 1

# Findings

## rust.assert.predicate_only_assertion:a-001

- **Layer**: rust-static-0
- **Level**: info
- **Rule**: rust.assert.predicate_only_assertion
- **Confidence**: high
- **Message**: assert!(...) is a predicate-only assertion.
- **Subjects**:
  - assertion ref=a-001
- **Rationale**: Use assert_eq! to capture structured evidence.

