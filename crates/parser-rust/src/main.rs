use std::io::{self, Read};
use std::process;

use tgraphy_core::component::builtin::RustParser;
use tgraphy_core::component::parser::{Parser, ParserInput};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin_buf = String::new();
    io::stdin().read_to_string(&mut stdin_buf)?;

    let input: ParserInput = serde_json::from_str(&stdin_buf)?;
    let artifact = Parser::parse(&RustParser, input)?;

    serde_json::to_writer(io::stdout(), &artifact)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
