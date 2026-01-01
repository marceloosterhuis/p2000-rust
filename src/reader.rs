use anyhow::Result;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::fs::File;
use tokio::io::AsyncBufReadExt;

use crate::parser::{P2000Message, Parser};

pub async fn read_from_file(path: &Path) -> Result<Vec<P2000Message>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let parser = Parser::new();
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = line?;
        match parser.parse_line(&line) {
            Ok(msg) => messages.push(msg),
            Err(e) => eprintln!("Warning: Failed to parse line: {}", e),
        }
    }

    Ok(messages)
}

pub async fn read_from_stdin() -> Result<Vec<P2000Message>> {
    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let parser = Parser::new();
    let mut messages = Vec::new();
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match parser.parse_line(&line) {
            Ok(msg) => messages.push(msg),
            Err(e) => eprintln!("Warning: Failed to parse line: {}", e),
        }
    }

    Ok(messages)
}
