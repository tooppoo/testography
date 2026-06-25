use std::collections::HashSet;
use std::path::{Path, PathBuf};

use kdl::KdlDocument;
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::process::{
    ProcessConfig, ProcessEvaluator, ProcessParser, ProcessReporter,
};

#[derive(Debug)]
pub enum ConfigError {
    InvalidKdl(String),
    MissingCommand { kind: String, name: String },
    DuplicateName { kind: String, name: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidKdl(msg) => {
                write!(f, "invalid testography.kdl: {msg}")
            }
            ConfigError::MissingCommand { kind, name } => write!(
                f,
                "missing required 'command' for {kind} '{name}' in testography.kdl"
            ),
            ConfigError::DuplicateName { kind, name } => {
                write!(f, "duplicate {kind} name '{name}' in testography.kdl")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn find_worktree_root() -> Option<PathBuf> {
    std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()))
}

pub fn register_from_config(
    registry: &mut ComponentRegistry,
    worktree_root: &Path,
) -> Result<(), ConfigError> {
    let config_path = worktree_root.join("testography.kdl");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let doc: KdlDocument = content
        .parse()
        .map_err(|e: kdl::KdlError| ConfigError::InvalidKdl(e.to_string()))?;

    let mut parser_names: HashSet<String> = HashSet::new();
    let mut evaluator_names: HashSet<String> = HashSet::new();
    let mut reporter_names: HashSet<String> = HashSet::new();

    for components_node in doc
        .nodes()
        .iter()
        .filter(|n| n.name().value() == "components")
    {
        let Some(children) = components_node.children() else {
            continue;
        };

        for node in children.nodes() {
            let kind = node.name().value();
            if kind == "parser" {
                let name = node
                    .entries()
                    .first()
                    .and_then(|e| e.value().as_string())
                    .ok_or_else(|| {
                        ConfigError::InvalidKdl("parser node requires a name argument".to_string())
                    })?
                    .to_string();

                if !parser_names.insert(name.clone()) {
                    return Err(ConfigError::DuplicateName {
                        kind: "parser".to_string(),
                        name,
                    });
                }

                let process = parse_process_block(node, "parser", &name)?;
                let command = resolve_command(worktree_root, process.command);

                registry.register_parser(
                    &name,
                    Box::new(ProcessParser {
                        config: ProcessConfig {
                            command,
                            args: process.args,
                        },
                    }),
                );
            } else if kind == "evaluator" {
                let name = node
                    .entries()
                    .first()
                    .and_then(|e| e.value().as_string())
                    .ok_or_else(|| {
                        ConfigError::InvalidKdl(
                            "evaluator node requires a name argument".to_string(),
                        )
                    })?
                    .to_string();

                if !evaluator_names.insert(name.clone()) {
                    return Err(ConfigError::DuplicateName {
                        kind: "evaluator".to_string(),
                        name,
                    });
                }

                let process = parse_process_block(node, "evaluator", &name)?;
                let command = resolve_command(worktree_root, process.command);

                registry.register_evaluator(
                    &name,
                    Box::new(ProcessEvaluator {
                        config: ProcessConfig {
                            command,
                            args: process.args,
                        },
                    }),
                );
            } else if kind == "reporter" {
                let name = node
                    .entries()
                    .first()
                    .and_then(|e| e.value().as_string())
                    .ok_or_else(|| {
                        ConfigError::InvalidKdl(
                            "reporter node requires a name argument".to_string(),
                        )
                    })?
                    .to_string();

                if !reporter_names.insert(name.clone()) {
                    return Err(ConfigError::DuplicateName {
                        kind: "reporter".to_string(),
                        name,
                    });
                }

                let process = parse_process_block(node, "reporter", &name)?;
                let command = resolve_command(worktree_root, process.command);

                registry.register_reporter(
                    &name,
                    Box::new(ProcessReporter {
                        name: name.clone(),
                        config: ProcessConfig {
                            command,
                            args: process.args,
                        },
                    }),
                );
            }
        }
    }

    Ok(())
}

struct RawProcess {
    command: String,
    args: Vec<String>,
}

fn parse_process_block(
    kind_node: &kdl::KdlNode,
    kind: &str,
    name: &str,
) -> Result<RawProcess, ConfigError> {
    let node_children = kind_node.children().ok_or_else(|| {
        ConfigError::InvalidKdl(format!("{kind} '{name}' requires a child block"))
    })?;

    let process_node = node_children.get("process").ok_or_else(|| {
        ConfigError::InvalidKdl(format!("{kind} '{name}' requires a 'process' block"))
    })?;

    let process_children = process_node.children().ok_or_else(|| {
        ConfigError::InvalidKdl(format!(
            "{kind} '{name}' process block must have child nodes"
        ))
    })?;

    let command_node =
        process_children
            .get("command")
            .ok_or_else(|| ConfigError::MissingCommand {
                kind: kind.to_string(),
                name: name.to_string(),
            })?;

    let command = command_node
        .entries()
        .first()
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| ConfigError::MissingCommand {
            kind: kind.to_string(),
            name: name.to_string(),
        })?
        .to_string();

    let args = match process_children.get("args") {
        Some(args_node) => args_node
            .entries()
            .iter()
            .filter(|e| e.name().is_none())
            .filter_map(|e| e.value().as_string().map(|s| s.to_string()))
            .collect(),
        None => vec![],
    };

    Ok(RawProcess { command, args })
}

fn resolve_command(worktree_root: &Path, command: String) -> String {
    let path = PathBuf::from(&command);
    if path.is_absolute() {
        command
    } else {
        worktree_root.join(path).to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    fn make_config_dir(kdl: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let mut f =
            std::fs::File::create(dir.path().join("testography.kdl")).expect("create config");
        f.write_all(kdl.as_bytes()).expect("write config");
        dir
    }

    #[test]
    fn registers_process_parser_from_config() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process {
      command "./target/debug/tgraphy-parser-rust"
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");

        assert!(
            registry.resolve_parser("rust").is_ok(),
            "rust parser should be registered"
        );
    }

    #[test]
    fn registers_parser_with_args() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process {
      command "/usr/bin/my-parser"
      args "--foo" "bar"
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");
        assert!(registry.resolve_parser("rust").is_ok());
    }

    #[test]
    fn no_config_file_is_ok() {
        let dir = tempfile::tempdir().expect("temp dir");
        let mut registry = ComponentRegistry::new();
        assert!(register_from_config(&mut registry, dir.path()).is_ok());
    }

    #[test]
    fn config_parser_overrides_builtin() {
        use tgraphy_core::component::builtin::BuiltinParser;

        let dir = make_config_dir(
            r#"components {
  parser "builtin" {
    process {
      command "/usr/bin/my-parser"
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        registry.register_parser("builtin", Box::new(BuiltinParser));
        register_from_config(&mut registry, dir.path()).expect("load config");
        assert!(registry.resolve_parser("builtin").is_ok());
    }

    #[test]
    fn duplicate_parser_name_is_error() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process { command "./a" args }
  }
  parser "rust" {
    process { command "./b" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(&err, ConfigError::DuplicateName { kind, name } if kind == "parser" && name == "rust"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn missing_command_is_error() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process {
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(&err, ConfigError::MissingCommand { kind, name } if kind == "parser" && name == "rust"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn invalid_kdl_is_error() {
        let dir = make_config_dir("{ this is not valid kdl !!!}}}");
        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(err, ConfigError::InvalidKdl(_)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn relative_command_resolved_from_worktree_root() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process {
      command "target/debug/tgraphy-parser-rust"
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");
        assert!(registry.resolve_parser("rust").is_ok());
    }

    #[test]
    fn args_omitted_means_empty() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process {
      command "/usr/bin/parser"
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");
        assert!(registry.resolve_parser("rust").is_ok());
    }

    #[test]
    fn parsers_from_multiple_components_blocks_are_all_registered() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process { command "/usr/bin/rust-parser" args }
  }
}
components {
  parser "go" {
    process { command "/usr/bin/go-parser" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");

        assert!(
            registry.resolve_parser("rust").is_ok(),
            "rust should be registered"
        );
        assert!(
            registry.resolve_parser("go").is_ok(),
            "go should be registered"
        );
    }

    #[test]
    fn duplicate_name_across_components_blocks_is_error() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process { command "/usr/bin/a" args }
  }
}
components {
  parser "rust" {
    process { command "/usr/bin/b" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(&err, ConfigError::DuplicateName { kind, name } if kind == "parser" && name == "rust"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn registers_process_evaluator_from_config() {
        let dir = make_config_dir(
            r#"components {
  evaluator "rust-static" {
    process {
      command "./target/release/tgraphy-evaluator-rust-static"
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");

        assert!(
            registry.resolve_evaluator("rust-static").is_ok(),
            "rust-static evaluator should be registered"
        );
    }

    #[test]
    fn duplicate_evaluator_name_is_error() {
        let dir = make_config_dir(
            r#"components {
  evaluator "rust-static" {
    process { command "./a" args }
  }
  evaluator "rust-static" {
    process { command "./b" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(&err, ConfigError::DuplicateName { kind, name } if kind == "evaluator" && name == "rust-static"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parser_and_evaluator_registered_from_same_components_block() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process { command "/usr/bin/rust-parser" args }
  }
  evaluator "rust-static" {
    process { command "/usr/bin/rust-static-evaluator" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");

        assert!(
            registry.resolve_parser("rust").is_ok(),
            "rust parser should be registered"
        );
        assert!(
            registry.resolve_evaluator("rust-static").is_ok(),
            "rust-static evaluator should be registered"
        );
    }

    #[test]
    fn registers_process_reporter_from_config() {
        let dir = make_config_dir(
            r#"components {
  reporter "markdown" {
    process {
      command "./target/release/tgraphy-reporter-markdown"
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");

        assert!(
            registry.resolve_reporter("markdown").is_ok(),
            "markdown reporter should be registered"
        );
    }

    #[test]
    fn duplicate_reporter_name_is_error() {
        let dir = make_config_dir(
            r#"components {
  reporter "markdown" {
    process { command "./a" args }
  }
  reporter "markdown" {
    process { command "./b" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(&err, ConfigError::DuplicateName { kind, name } if kind == "reporter" && name == "markdown"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn config_reporter_overrides_builtin() {
        use tgraphy_core::component::builtin::BuiltinReporter;

        let dir = make_config_dir(
            r#"components {
  reporter "builtin" {
    process {
      command "/usr/bin/my-reporter"
      args
    }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        registry.register_reporter("builtin", Box::new(BuiltinReporter));
        register_from_config(&mut registry, dir.path()).expect("load config");
        assert!(registry.resolve_reporter("builtin").is_ok());
    }

    #[test]
    fn parser_evaluator_and_reporter_registered_from_same_components_block() {
        let dir = make_config_dir(
            r#"components {
  parser "rust" {
    process { command "/usr/bin/rust-parser" args }
  }
  evaluator "rust-static" {
    process { command "/usr/bin/rust-static-evaluator" args }
  }
  reporter "markdown" {
    process { command "/usr/bin/tgraphy-reporter-markdown" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        register_from_config(&mut registry, dir.path()).expect("load config");

        assert!(
            registry.resolve_parser("rust").is_ok(),
            "rust parser should be registered"
        );
        assert!(
            registry.resolve_evaluator("rust-static").is_ok(),
            "rust-static evaluator should be registered"
        );
        assert!(
            registry.resolve_reporter("markdown").is_ok(),
            "markdown reporter should be registered"
        );
    }

    #[test]
    fn duplicate_reporter_name_across_components_blocks_is_error() {
        let dir = make_config_dir(
            r#"components {
  reporter "json" {
    process { command "/usr/bin/a" args }
  }
}
components {
  reporter "json" {
    process { command "/usr/bin/b" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(&err, ConfigError::DuplicateName { kind, name } if kind == "reporter" && name == "json"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn reporter_missing_name_argument_is_error() {
        let dir = make_config_dir(
            r#"components {
  reporter {
    process { command "/usr/bin/reporter" args }
  }
}"#,
        );

        let mut registry = ComponentRegistry::new();
        let err = register_from_config(&mut registry, dir.path()).expect_err("should error");
        assert!(
            matches!(err, ConfigError::InvalidKdl(_)),
            "reporter without name should produce InvalidKdl error: {err}"
        );
    }
}
