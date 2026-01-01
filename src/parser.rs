use chrono::DateTime;
use regex::Regex;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid FLEX format: {0}")]
    InvalidFormat(String),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Missing field: {0}")]
    MissingField(String),
}

#[derive(Debug, Clone)]
pub struct P2000Message {
    pub protocol: String,
    pub timestamp: DateTime<chrono::Local>,
    pub radio_address: String,
    pub frequency: String,
    pub capcodes: Vec<String>,
    pub message_type: String,
    pub content: String,
    // Parsed fields
    pub priority: Option<String>,
    pub incident_code: Option<String>,
    pub location: String,
    pub units: Vec<String>,
}

impl fmt::Display for P2000Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} - {}",
            self.timestamp.format("%H:%M:%S"),
            self.priority.as_deref().unwrap_or("?"),
            self.content
        )
    }
}

pub struct Parser {
    priority_regex: Regex,
    incident_code_regex: Regex,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            // Matches priority at the start: P1, P2, P3, A0, A1, A2, B
            priority_regex: Regex::new(r"^([PA]\d|B)\s").unwrap(),
            // Matches incident codes like BDH-07, BRT-03, etc.
            incident_code_regex: Regex::new(r"\b([A-Z]{2,3}-\d{2})\b").unwrap(),
        }
    }

    pub fn parse_line(&self, line: &str) -> Result<P2000Message, ParseError> {
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() < 7 {
            return Err(ParseError::InvalidFormat(format!(
                "Expected at least 7 fields, got {}",
                parts.len()
            )));
        }

        let protocol = parts[0].to_string();
        let timestamp_str = parts[1];
        let radio_address = parts[2].to_string();
        let frequency = parts[3].to_string();
        let capcodes_str = parts[4];
        let message_type = parts[5].to_string();
        let content = parts[6..].join("|").to_string();

        // Parse timestamp
        let timestamp = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|ndt| {
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc)
                    .with_timezone(&chrono::Local)
            })
            .ok_or_else(|| ParseError::InvalidTimestamp(timestamp_str.to_string()))?;

        // Parse capcodes
        let capcodes: Vec<String> = if capcodes_str.is_empty() {
            Vec::new()
        } else {
            capcodes_str.split_whitespace().map(|s| s.to_string()).collect()
        };

        // Parse priority from content
        let priority = self
            .priority_regex
            .find(&content)
            .map(|m| m.as_str().trim().to_string());

        // Parse incident code from content
        let incident_code = self
            .incident_code_regex
            .find(&content)
            .map(|m| m.as_str().to_string());

        // Extract location - usually after the incident code/description
        let location = extract_location(&content);

        // Extract unit codes from capcodes
        let units = parse_unit_codes(&capcodes);

        Ok(P2000Message {
            protocol,
            timestamp,
            radio_address,
            frequency,
            capcodes,
            message_type,
            content,
            priority,
            incident_code,
            location,
            units,
        })
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_location(content: &str) -> String {
    // Location is typically after the incident code and description
    // We'll look for the last segment that doesn't look like a code

    let parts: Vec<&str> = content.split_whitespace().collect();

    // Start from the end and collect meaningful location parts
    let mut location_parts = Vec::new();

    for part in parts.iter().rev() {
        // Stop if we hit a code-like pattern (all digits or numeric codes)
        if part.len() <= 6 && part.chars().all(|c| c.is_numeric()) {
            break;
        }

        // Skip priority markers and short codes
        if part.len() <= 3
            || (part.len() > 3 && part.chars().all(|c| c.is_numeric() || c == '-'))
        {
            continue;
        }

        location_parts.push(*part);
    }

    location_parts.reverse();
    location_parts.join(" ")
}

fn parse_unit_codes(capcodes: &[String]) -> Vec<String> {
    // In this format, capcodes are device IDs
    // We'll return them as-is; they could be looked up in a database
    capcodes.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message() {
        let parser = Parser::new();
        let line = "FLEX|2026-01-01 20:14:32|1600/2/K/A|03.091|002029575 001503282 001503289 001503900|ALN|P 2 BDH-07 Ongeval (los object) Gangetje Leiden 169252";

        let msg = parser.parse_line(line).expect("Failed to parse");
        assert_eq!(msg.protocol, "FLEX");
        assert_eq!(msg.radio_address, "1600/2/K/A");
        assert_eq!(msg.frequency, "03.091");
        assert_eq!(msg.priority, Some("P 2".to_string()));
        assert_eq!(msg.incident_code, Some("BDH-07".to_string()));
    }
}
