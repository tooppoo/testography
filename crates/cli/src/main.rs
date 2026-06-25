use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::pipeline::{collect_step, evaluate_step, report_step, transform_step};
use tgraphy_core::run_pipeline;

mod config;

#[derive(Parser)]
#[command(name = "tgraphy", about = "Testography pipeline CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Collect evidence using a parser (produces parsed_evidence)
    Collect {
        #[arg(long)]
        parser: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Transform parsed_evidence into module_evidence
    Transform {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Evaluate module_evidence or assessed_module_evidence using an evaluator
    Evaluate {
        #[arg(long)]
        evaluator: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Generate a report from assessed_module_evidence
    Report {
        #[arg(long)]
        reporter: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run the full collect → transform → evaluate → report pipeline
    Run {
        #[arg(long)]
        parser: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long = "evaluator", required = true)]
        evaluators: Vec<String>,
        #[arg(long)]
        reporter: String,
        #[arg(long, default_value = ".testography")]
        output_dir: PathBuf,
    },
}

fn build_registry() -> Result<ComponentRegistry, config::ConfigError> {
    let mut registry = ComponentRegistry::new();
    registry.register_parser("builtin", Box::new(BuiltinParser));
    registry.register_evaluator("builtin", Box::new(BuiltinEvaluator));
    registry.register_reporter("builtin", Box::new(BuiltinReporter));

    if let Some(root) = config::find_worktree_root() {
        config::register_from_config(&mut registry, &root)?;
    }

    Ok(registry)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let registry = build_registry()?;

    match cli.command {
        Command::Collect {
            parser,
            input,
            output,
        } => collect_step(&registry, &parser, &input, &output)?,
        Command::Transform { input, output } => transform_step(&input, &output)?,
        Command::Evaluate {
            evaluator,
            input,
            output,
        } => evaluate_step(&registry, &evaluator, &input, &output)?,
        Command::Report {
            reporter,
            input,
            output,
        } => report_step(&registry, &reporter, &input, &output)?,
        Command::Run {
            parser,
            input,
            evaluators,
            reporter,
            output_dir,
        } => run_pipeline(
            &registry,
            &input,
            &parser,
            &evaluators,
            &reporter,
            &output_dir,
        )?,
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}
