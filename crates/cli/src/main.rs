use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use tgraphy_core::component::ComponentRegistry;
use tgraphy_core::component::builtin::{BuiltinEvaluator, BuiltinParser, BuiltinReporter};
use tgraphy_core::component::process::{ProcessConfig, ProcessParser};
use tgraphy_core::pipeline::{
    PipelineError, collect_step, evaluate_step, report_step, transform_step,
};

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
}

fn rust_parser_command() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("tgraphy-parser-rust")))
        .unwrap_or_else(|| PathBuf::from("tgraphy-parser-rust"))
        .to_string_lossy()
        .into_owned()
}

fn default_registry() -> ComponentRegistry {
    let mut registry = ComponentRegistry::new();
    registry.register_parser("builtin", Box::new(BuiltinParser));
    registry.register_parser(
        "rust",
        Box::new(ProcessParser {
            config: ProcessConfig {
                command: rust_parser_command(),
                args: vec![],
            },
        }),
    );
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
        Command::Transform { input, output } => transform_step(&input, &output),
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
