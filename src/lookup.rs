use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
struct CapcodeCsv {
    #[serde(rename = "capcode")]
    capcode: String,
    #[serde(rename = "dienst")]
    service: String,
    #[serde(rename = "regio")]
    region: String,
    #[serde(rename = "plaats")]
    place: String,
    #[serde(rename = "omschrijving")]
    description: String,
    #[serde(rename = "afkorting")]
    short: String,
}

#[derive(Debug, Clone)]
pub struct CapcodeInfo {
    pub code: String,
    pub service: String,
    pub region: String,
    pub place: String,
    pub description: String,
    pub short: String,
}

#[derive(Debug, Default)]
pub struct Lookup {
    capcodes: HashMap<String, CapcodeInfo>,
    abbreviations: HashMap<String, String>,
    abbreviations_no_space: HashMap<String, String>,
}

impl Lookup {
    pub fn load(capcode_path: &Path, abbreviations_path: &Path) -> Result<Self> {
        let capcodes = load_capcodes(capcode_path)?;
        let (abbreviations, abbreviations_no_space) = load_abbreviations(abbreviations_path)?;
        Ok(Lookup {
            capcodes,
            abbreviations,
            abbreviations_no_space,
        })
    }

    pub fn resolve_capcode(&self, code: &str) -> Option<&CapcodeInfo> {
        let key = normalize_code(code);
        self.capcodes.get(&key)
    }

    pub fn expand_abbreviation(&self, token: &str) -> Option<&String> {
        if let Some(hit) = self.abbreviations.get(token) {
            return Some(hit);
        }
        let normalized = token.replace(' ', "");
        if normalized.is_empty() {
            return None;
        }
        self.abbreviations_no_space.get(&normalized)
    }
}

fn load_capcodes(path: &Path) -> Result<HashMap<String, CapcodeInfo>> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(file);

    let mut map = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        if record.len() < 6 {
            continue;
        }
        let info = CapcodeInfo {
            code: record[0].trim_matches('"').to_string(),
            service: record[1].trim_matches('"').to_string(),
            region: record[2].trim_matches('"').to_string(),
            place: record[3].trim_matches('"').to_string(),
            description: record[4].trim_matches('"').to_string(),
            short: record[5].trim_matches('"').to_string(),
        };
        let key = normalize_code(&info.code);
        map.insert(key, info);
    }
    Ok(map)
}

fn normalize_code(code: &str) -> String {
    let trimmed = code.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn load_abbreviations(path: &Path) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
    let mut map = HashMap::new();
    let mut map_no_space = HashMap::new();
    let content = std::fs::read_to_string(path)?;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((abbr, rest)) = line.split_once(':') {
            let key = abbr.trim();
            let value = rest.trim();
            if !key.is_empty() && !value.is_empty() {
                map.insert(key.to_string(), value.to_string());
                let normalized = key.replace(' ', "");
                if !normalized.is_empty() {
                    map_no_space.insert(normalized, value.to_string());
                }
            }
        }
    }
    Ok((map, map_no_space))
}
