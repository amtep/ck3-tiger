use std::fs::read_to_string;
use std::mem::take;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::helpers::TigerHashMap;
use crate::parse::suppress::parse_suppressions;
use crate::report::errors::Errors;
use crate::report::ErrorKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SuppressionKey {
    pub key: ErrorKey,
    pub message: String,
}

pub(crate) type Suppression = Vec<SuppressionLocation>;

/// This picks out the fields we need from the json reports.
/// It's also used by the suppression parser in `parse::suppression`.
#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
pub(crate) struct SuppressionLocation {
    pub path: String,
    pub line: Option<String>,
    pub tag: Option<String>,
}

/// This picks out the fields we need from the json reports.
/// It's also used by the suppression parser in `parse::suppression`.
#[derive(Debug, Deserialize)]
pub(crate) struct SuppressionReport {
    pub(crate) key: ErrorKey,
    pub(crate) message: String,
    pub(crate) locations: Vec<SuppressionLocation>,
}

pub fn suppress_from_file(fullpath: &Path) -> Result<()> {
    let input = read_to_string(fullpath)?;
    let reports: Vec<SuppressionReport> = if input.starts_with("[\n") {
        serde_json::from_str(&input)?
    } else {
        parse_suppressions(&input)?
    };
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
