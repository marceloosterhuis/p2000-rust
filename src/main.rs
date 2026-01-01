mod parser;
mod reader;
mod tui;

use anyhow::Result;
use std::path::Path;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

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
    tui::run_tui(messages).await.map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
