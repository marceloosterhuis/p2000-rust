mod location;
mod lookup;
mod parser;
mod reader;
mod tui;

use anyhow::Result;
use std::path::Path;
use std::env;
use crate::lookup::Lookup;
use crate::location::LocationLookup;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let capcode_path = Path::new("data/capcodelist.csv");
    let abbreviations_path = Path::new("data/abbrevations.txt");
    let observations_path = Path::new("data/Observations.csv");
    let lookup = Lookup::load(capcode_path, abbreviations_path)?;
    let regios_codes_path = Path::new("data/RegioSCodes.csv");
    let location_lookup = LocationLookup::load(observations_path, regios_codes_path)?;

    let messages = if args.len() > 1 {
        // Read from file
        let path = Path::new(&args[1]);
        reader::read_from_file(path).await?
    } else {
        // Read from stdin
        eprintln!("Reading from stdin... (or provide a file path as argument)");
        reader::read_from_stdin().await?
    };

    if messages.is_empty() {
        eprintln!("No messages to display");
        return Ok(());
    }

    eprintln!("Loaded {} messages", messages.len());
    tui::run_tui(messages, lookup, location_lookup)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
