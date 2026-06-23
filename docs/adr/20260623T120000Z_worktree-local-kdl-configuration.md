# Use Worktree-Local KDL Configuration for Component Registry

- Status: Accepted
- Created: 2026-06-23T12:00:00Z

## Context

testography v0 uses process-based components as the primary extension boundary for parsers and evaluators. The core invokes external components as processes, while JSON and JSON Schema define the process boundary contract between the core and those components.

This allows parser and evaluator implementations to use the language runtime, compiler tooling, test framework, or provider SDK that best fits the target domain. However, it also means the core needs a way to register process-based components without hard-coding concrete plugin executable paths in the CLI.

The Rust parser was introduced as a process-based parser under `plugins/parser/rust`, but keeping the CLI responsible for constructing or knowing the `tgraphy-parser-rust` execution path preserves an unwanted coupling between the CLI and a specific plugin implementation. The CLI should resolve component names such as `rust`, but the executable command associated with that component name should come from configuration.

This decision defines how projects bind component names to process-based plugins. It affects CLI behavior, worktree behavior, configuration syntax, path resolution, test strategy, and future plugin registry design.

## Decision

testography v0 uses a worktree-local KDL configuration file named `testography.kdl` to register process-based components into `ComponentRegistry`.

The CLI continues to accept component names through flags such as:

```text
tgraphy collect --parser rust ...
```

The meaning of `rust` is resolved through the component registry. When `testography.kdl` exists in the current Git worktree root, the CLI loads it and registers the configured components before resolving the requested component name.

The CLI must not hard-code the concrete execution path for any plugin binary.

### Configuration file location

The configuration file name is `testography.kdl`. testography resolves it from the current Git worktree root using `git rev-parse --show-toplevel`.

The worktree root means the root of the current working tree, not the common `.git` directory. When Git worktrees are used, each worktree may have its own `testography.kdl`, and testography treats that file as the local configuration for that worktree.

If `testography.kdl` is not present, testography proceeds with only builtin components registered.

### Configuration format

The configuration format is KDL. Parser definitions are nested under a `components` node, grouped by component kind.

A minimal parser configuration:

```kdl
components {
  parser "rust" {
    process {
      command "./target/debug/tgraphy-parser-rust"
      args
    }
  }
}
```

When arguments are needed, `args` uses KDL node arguments:

```kdl
components {
  parser "rust" {
    process {
      command "./target/debug/tgraphy-parser-rust"
      args "--example" "value"
    }
  }
}
```

If `args` is omitted or has no values, it is treated as an empty argv list.

### Path resolution

Relative `command` paths are resolved from the current Git worktree root. Absolute paths are used as-is.

`args` values are passed to the process as-is. testography v0 does not interpret `args` as paths, because argument semantics belong to the plugin being invoked.

### Built-in component precedence

Built-in components are treated as test and development fallback components. When a component is defined in `testography.kdl`, that config-defined component takes precedence over a built-in component with the same name.

### Configuration validation

For v0, configuration validation covers the minimal structure read by the registry loader:

- invalid KDL syntax;
- missing required `command`;
- duplicate component names within the same component kind.

These failures produce clear error messages and do not panic. Plugin-specific configuration schema validation is out of scope for v0.

## Alternatives Considered

### Hard-code known plugin executables in the CLI

The CLI could special-case known plugin names such as `rust` and internally map them to executable paths. This was not selected because it couples the CLI to specific plugin implementations. Adding, replacing, or moving plugins would require CLI changes even when the process protocol remains stable.

### Require explicit config path for every command

Another option would be to require users to pass a configuration path explicitly with `--config testography.kdl`. This was not selected for v0 because it adds repetitive CLI overhead. An explicit `--config` option may be added later.

### Resolve configuration from the common Git directory

Git worktrees share a common Git directory. testography could resolve configuration from that common location to create one shared configuration across all linked worktrees. This was not selected because it would make worktree-local experimentation harder. A user may intentionally change `testography.kdl` in one worktree while preserving another version in a different worktree.

### Use JSON or TOML for configuration

JSON was not selected because it is noisy for human-authored configuration, especially for nested component definitions and command argument lists. TOML was not selected because KDL provides a concise tree structure with natural support for named nodes and node arguments, which maps well to component definitions such as `parser "rust"` and argv-style definitions such as `args "--example" "value"`.

### Treat built-in components as higher priority than config components

Built-in components could override config-defined components. This was not selected because the purpose of the configuration file is to let the project define which process-based components are registered. If a config-defined component and a built-in component have the same name, using the built-in version would make the configuration misleading.

## Consequences

### Positive Consequences

- CLI commands keep the existing component-name based user experience such as `--parser rust`.
- Concrete process commands are moved out of the CLI and into project configuration.
- Worktree-local configuration supports isolated experimentation across Git worktrees.
- Relative command paths are deterministic because they are resolved from the current Git worktree root.
- Config-defined components can replace built-in fallback components without changing CLI code.

### Negative Consequences

- Users need a valid `testography.kdl` for project-specific plugin registration.
- Configuration parsing and validation introduce a new failure mode before component execution.
- Worktree-local behavior may surprise users who expected one shared configuration across linked worktrees.
- KDL support adds a configuration parsing dependency and requires documentation.

### Neutral Consequences

- Plugin-specific configuration schema validation remains outside v0.
- `args` are treated as opaque argv values and are not path-resolved by the core.
- Built-in components continue to exist primarily as test and development fallback.
- Future configuration features such as explicit `--config`, user/global config, plugin discovery, or `tgraphy run` integration can be added later without changing the v0 registry-loading principle.
