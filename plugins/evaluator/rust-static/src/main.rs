use std::io::{self, Read};
use std::process;

use tgraphy_evaluator_rust_static::{EvaluatorInput, evaluate};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin_buf = String::new();
    io::stdin().read_to_string(&mut stdin_buf)?;

    let input: EvaluatorInput = serde_json::from_str(&stdin_buf)?;
    let layer = evaluate(input);

    serde_json::to_writer(io::stdout(), &layer)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
