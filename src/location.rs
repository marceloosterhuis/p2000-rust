use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct LocationInfo {
    pub place: String,
    pub province: String,
    pub region: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct FoundLocation {
    pub found_place: String,  // Place name found in message text
    pub info: LocationInfo,   // Municipality, province, region from data
}

#[derive(Debug, Default)]
pub struct LocationLookup {
    locations: HashMap<String, LocationInfo>,
    place_names: Vec<String>,
    place_to_wp: HashMap<String, String>,
}

impl LocationLookup {
    pub fn load(
        observations_path: &Path,
        regios_codes_path: &Path,
    ) -> Result<Self> {
        let mut locations: HashMap<String, LocationInfo> = HashMap::new();
        let mut place_names: Vec<String> = Vec::new();
        let mut place_to_wp: HashMap<String, String> = HashMap::new();
        let mut seen_places: HashSet<String> = HashSet::new();

        // Load 4pp-final-2023.csv for coordinates (if it exists)
        let coords_path = Path::new("data/4pp-final-2023.csv");
        let mut woonplaats_coords: HashMap<String, (f64, f64)> = HashMap::new();
        if coords_path.exists() {
            if let Ok(file) = File::open(coords_path) {
                let mut rdr = csv::ReaderBuilder::new().from_reader(file);
                for result in rdr.records() {
                    if let Ok(record) = result {
                        if record.len() >= 6 {
                            if let Ok(lat) = record[4].parse::<f64>() {
                                if let Ok(lon) = record[5].parse::<f64>() {
                                    // Trim quotes and whitespace from place name
                                    let woonplaats = record[1].trim_matches('"').trim().to_string();
                                    woonplaats_coords.insert(woonplaats, (lat, lon));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Load Observations.csv for province and region
        let file = File::open(observations_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() < 6 {
                continue;
            }

            let measure = record[1].trim();
            let wp_code = record[2].trim();
            let value = record[4].trim();

            let loc = locations.entry(wp_code.to_string()).or_default();

            match measure {
                "GM000C" => {
                    loc.place = value.to_string();
                    // Look up coordinates from woonplaats mapping
                    if let Some((lat, lon)) = woonplaats_coords.get(value.trim()) {
                        loc.latitude = Some(*lat);
                        loc.longitude = Some(*lon);
                    }
                    // Also add place name from Observations as a searchable name
                    let place_str = value.trim().to_string();  // Extra trim to handle spaces in CSV fields
                    if !place_str.is_empty() && !seen_places.contains(&place_str) && place_str.len() >= 3 {
                        place_to_wp.insert(place_str.clone(), wp_code.to_string());
                        place_names.push(place_str.clone());
                        seen_places.insert(place_str);
                    }
                }
                "PV0002" => loc.province = value.to_string(),
                "LD0002" => loc.region = value.to_string(),
                _ => {}
            }
        }

        // Load RegioSCodes.csv for place names and WP code mapping
        let file = File::open(regios_codes_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() < 5 {
                continue;
            }

            let wp_code = record[0].trim_matches('"').trim();
            let title = record[4].trim_matches('"').trim();  // Title is field 4 (0-indexed)

            if !wp_code.is_empty() && !title.is_empty() {
                let title_str = title.to_string();
                // Only add if we haven't seen this place name before AND it's at least 3 characters
                if !seen_places.contains(&title_str) && title_str.len() >= 3 {
                    place_to_wp.insert(title_str.clone(), wp_code.to_string());
                    place_names.push(title_str.clone());
                    seen_places.insert(title_str.clone());
                    
                    // Also try to load coordinates for this place name from 4pp
                    if let Some((lat, lon)) = woonplaats_coords.get(&title_str) {
                        if let Some(loc) = locations.get_mut(wp_code) {
                            loc.latitude = Some(*lat);
                            loc.longitude = Some(*lon);
                        }
                    }
                }
            }
        }

        // Sort place names by length (longest first) for matching priority
        place_names.sort_by(|a, b| b.len().cmp(&a.len()));

        Ok(LocationLookup {
            locations,
            place_names,
            place_to_wp,
        })
    }

    pub fn resolve(&self, wp_code: &str) -> Option<&LocationInfo> {
        self.locations.get(wp_code)
    }

    pub fn find_location_by_text(&self, text: &str) -> Option<FoundLocation> {
        let text_lower = text.to_lowercase();

        // Search for place names in order (longest first)
        for place in &self.place_names {
            let place_lower = place.to_lowercase();
            if text_lower.contains(&place_lower) {
                // Get WP code from RegioSCodes mapping
                if let Some(wp_code) = self.place_to_wp.get(place) {
                    if let Some(info) = self.locations.get(wp_code) {
                        return Some(FoundLocation {
                            found_place: place.clone(),
                            info: info.clone(),
                        });
                    }
                }
            }
        }
        None
    }

    pub fn format(&self, wp_code: &str) -> String {
        if let Some(info) = self.resolve(wp_code) {
            let mut parts = vec![];
            if !info.place.is_empty() {
                parts.push(info.place.trim().to_string());
            }
            if !info.province.is_empty() {
                parts.push(info.province.trim().to_string());
            }
            if !info.region.is_empty() {
                parts.push(info.region.trim().to_string());
            }
            parts.join(", ")
        } else {
            wp_code.to_string()
        }
    }

    pub fn format_info(&self, info: &LocationInfo) -> String {
        let mut parts = vec![];
        if !info.place.is_empty() {
            parts.push(info.place.trim().to_string());
        }
        if !info.province.is_empty() {
            parts.push(info.province.trim().to_string());
        }
        if !info.region.is_empty() {
            parts.push(info.region.trim().to_string());
        }
        parts.join(", ")
    }

    pub fn format_found_location(&self, found: &FoundLocation) -> String {
        let mut parts = vec![];
        
        // Format place: show "Found (Municipality)" if they differ, otherwise just the place
        let place_str = if found.found_place.trim() != found.info.place.trim() {
            format!("{} ({})", found.found_place.trim(), found.info.place.trim())
        } else {
            found.found_place.trim().to_string()
        };
        parts.push(place_str);
        
        if !found.info.province.is_empty() {
            parts.push(found.info.province.trim().to_string());
        }
        if !found.info.region.is_empty() {
            parts.push(found.info.region.trim().to_string());
        }
        
        // Add coordinates if available (keep full precision)
        if let (Some(lat), Some(lon)) = (found.info.latitude, found.info.longitude) {
            parts.push(format!("[{}, {}]", lat, lon));
        }
        
        parts.join(" | ")
    }
}
