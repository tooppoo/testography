---
name: code-fix
description: Ensure implementation changes are verified with make check, fix failures until success, and handle coverage failures by adding tests or narrowly excluding verification-only code using cargo-llvm-cov recommended mechanisms.
---

# make-check-verification

## Purpose

Ensure that every implementation change is verified by the repository's standard check command, and that coverage failures are resolved by improving tests unless there is a justified reason to exclude verification-only code from coverage.

## When to use this skill

Use this skill whenever you modify implementation code, tests, build configuration, schemas, fixtures, or coverage-related configuration in this repository.

## Required verification loop

After changing implementation or test code, always run:

```sh
make check
```

If `make check` fails:

1. Read the error message carefully.
2. Identify the concrete failing category, such as:

   * formatting failure
   * lint failure
   * compile failure
   * test failure
   * schema or fixture validation failure
   * coverage failure
3. Fix the cause indicated by the error.
4. Run `make check` again.
5. Continue this loop until `make check` succeeds.

Do not treat the task as complete while `make check` is still failing.

If `make check` cannot be executed because of an environment limitation, missing tool, missing permission, network restriction, or external service failure, stop and report:

* the exact command attempted
* the exact error output
* whether the failure is caused by the repository code or by the execution environment
* the remaining unverified risk

## Coverage policy

If coverage is low or a coverage threshold fails, first assume the correct response is to add or improve tests.

Prefer adding tests that cover observable behavior rather than tests that merely execute lines for the sake of increasing coverage.

When adding tests, prioritize:

1. public behavior and CLI-visible behavior
2. parser / evaluator / reporter boundaries
3. error cases and validation failures
4. regression cases related to the change being made
5. edge cases that are likely to break silently

Do not lower the coverage threshold merely to make the check pass.

## Coverage exclusion policy

Coverage exclusion is allowed only when the uncovered code is primarily verification-only, scaffolding-only, generated, or structurally unsuitable for meaningful behavioral testing.

Examples that may be eligible:

* builtin module implementations used mainly as validation scaffolding
* generated code
* test-only adapters
* code that exists only to support fixture validation
* unreachable defensive branches that cannot be exercised without corrupting invariants

Before excluding code from coverage, confirm that adding a meaningful test would not be the better fix.

When excluding code, keep the exclusion as narrow as possible.

Preferred order:

1. Exclude file patterns from the cargo-llvm-cov report when an entire file or directory is verification-only.
2. Exclude a specific module or function only when file-level exclusion would be too broad.

For file-level exclusion, use cargo-llvm-cov's `--ignore-filename-regex` mechanism through the repository's Makefile or coverage command configuration.

Example:

```sh
cargo llvm-cov --ignore-filename-regex 'src/builtin/.*'
```

For function or module-level exclusion, do not use a bare `#[coverage(off)]` attribute.

Use cargo-llvm-cov's cfg-gated form:

```rust
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg_attr(coverage_nightly, coverage(off))]
mod verification_only_builtin {
    // ...
}
```

or:

```rust
#[cfg_attr(coverage_nightly, coverage(off))]
fn verification_only_helper() {
    // ...
}
```

If `coverage` or `coverage_nightly` cfg warnings occur on Rust 1.80+, add the appropriate `unexpected_cfgs` configuration to `Cargo.toml` rather than suppressing the warning broadly:

```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(coverage,coverage_nightly)'] }
```

Every coverage exclusion must be accompanied by a brief code comment or configuration comment explaining why exclusion is preferable to adding tests.

## Completion criteria

The task is complete only when:

* the requested implementation change is made
* relevant tests are added or updated when behavior changes
* coverage failures are resolved by tests or justified narrow exclusions
* `make check` succeeds
* the final report states what was changed and that `make check` passed

Do not claim success unless `make check` was actually run successfully.
