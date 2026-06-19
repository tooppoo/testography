use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::pipeline::{PipelineError, collect_step, evaluate_step, report_step};

#[derive(Parser)]
#[command(name = "tgraphy", about = "Testography pipeline CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Collect evidence using a parser
    Collect {
        #[arg(long)]
        parser: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Evaluate an artifact using an evaluator
    Evaluate {
        #[arg(long)]
        evaluator: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Generate a report from an assessed artifact
    Report {
        #[arg(long)]
        reporter: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

fn default_registry() -> ComponentRegistry {
    let mut registry = ComponentRegistry::new();
    registry.register_parser("builtin", Box::new(BuiltinParser));
    registry.register_evaluator("builtin", Box::new(BuiltinEvaluator));
    registry.register_reporter("builtin", Box::new(BuiltinReporter));
    registry
}

fn run() -> Result<(), PipelineError> {
    let cli = Cli::parse();
    let registry = default_registry();

    match cli.command {
        Command::Collect {
            parser,
            input,
            output,
        } => collect_step(&registry, &parser, &input, &output),
        Command::Evaluate {
            evaluator,
            input,
            output,
        } => evaluate_step(&registry, &evaluator, &input, &output),
        Command::Report {
            reporter,
            input,
            output,
        } => report_step(&registry, &reporter, &input, &output),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}
