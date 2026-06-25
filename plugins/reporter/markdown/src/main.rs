use std::io::{self, Read, Write};
use std::process;

use tgraphy_reporter_markdown::{ReporterInput, render};
use tgraphy_types::ReporterOutput;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin_buf = String::new();
    io::stdin().read_to_string(&mut stdin_buf)?;

    let input: ReporterInput = serde_json::from_str(&stdin_buf)?;
    let content_bytes = render(input);
    let content = String::from_utf8(content_bytes)?;

    let envelope = ReporterOutput {
        extension: "md".to_string(),
        content,
    };
    let json = serde_json::to_string(&envelope)?;
    io::stdout().write_all(json.as_bytes())?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
