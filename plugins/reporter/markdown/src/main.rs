use std::io::{self, Read, Write};
use std::process;

use tgraphy_reporter_markdown::{ReporterInput, render};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin_buf = String::new();
    io::stdin().read_to_string(&mut stdin_buf)?;

    let input: ReporterInput = serde_json::from_str(&stdin_buf)?;
    let output = render(input);

    io::stdout().write_all(&output)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
