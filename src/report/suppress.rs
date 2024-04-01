use std::fs::read_to_string;
use std::mem::take;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::helpers::TigerHashMap;
use crate::report::errors::Errors;
use crate::report::ErrorKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SuppressionKey {
    pub key: ErrorKey,
    pub message: String,
}

pub type Suppression = Vec<SuppressionLocation>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct SuppressionLocation {
    pub path: String,
    pub line: Option<String>,
    pub tag: Option<String>,
}

/// This picks out the fields we need from the json reports
#[derive(Deserialize)]
struct JsonReport {
    key: ErrorKey,
    message: String,
    locations: Vec<SuppressionLocation>,
}

pub fn suppress_from_json(fullpath: &Path) -> Result<()> {
    let reports: Vec<JsonReport> = serde_json::from_str(&read_to_string(fullpath)?)?;
    let mut suppress: TigerHashMap<SuppressionKey, Vec<Suppression>> = TigerHashMap::default();
    for mut report in reports {
        let locations = take(&mut report.locations);
        let suppressionkey = SuppressionKey { key: report.key, message: report.message };
        if let Some(v) = suppress.get_mut(&suppressionkey) {
            v.push(locations);
        } else {
            suppress.insert(suppressionkey, vec![locations]);
        }
    }
    Errors::get_mut().suppress = suppress;
    Ok(())
}
