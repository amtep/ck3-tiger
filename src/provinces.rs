use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::block::{Block, Loc, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, LogPauseRaii};
use crate::everything::FileHandler;
use crate::fileset::FileEntry;
use crate::provinces::parse::{parse_csv, read_csv};

mod parse;

type ProvId = u32;

#[derive(Clone, Debug, Default)]
pub struct Provinces {
    /// Colors in the provinces.png
    colors: FnvHashSet<RGB>,
    /// Colors defined in definition.csv. Should ideally be the same values as in `colors`.
    /// CK3 requires uninterrupted indices starting at 0, but we want to be able to warn
    /// and continue if they're not, so it's a hashmap.
    province_colors: FnvHashMap<ProvId, RGB>,

    adjacencies: Vec<Adjacency>,
}

impl FileHandler for Provinces {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data")
    }

    fn config(&mut self, _config: &Block) {}

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        // let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

        if entry.path().components().count() == 2 {
            match &*entry.filename().to_string_lossy() {
                "adjacencies.csv" => {
                    let content = match read_csv(fullpath) {
                        Ok(content) => content,
                        Err(e) => {
                            error(
                                entry,
                                ErrorKey::ReadError,
                                &format!("could not read `{}`: {:#}", entry.path().display(), e),
                            );
                            return;
                        }
                    };
                    self.adjacencies = parse_csv(entry, 1, &content)
                        .filter_map(Adjacency::new)
                        .collect();
                }
                _ => (),
            }
        }
    }

    fn finalize(&mut self) {}
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Coords {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
pub struct Adjacency {
    line: Loc,
    // TODO: check from, to, and through are valid prov ids
    from: ProvId,
    to: ProvId,
    // TODO: check type is "sea" or "river_large"
    kind: Token, // sea or river_large
    through: ProvId,
    // TODO: check start and stop are map coordinates and have the right color on province.png
    // They can be -1 -1 though.
    start: Coords,
    stop: Coords,
    comment: Token,
}

impl Adjacency {
    pub fn _verify<T: FromStr>(v: &Token, msg: &str) -> Option<T> {
        let r = v.as_str().parse().ok();
        if r.is_none() {
            error(v, ErrorKey::ParseError, msg);
        }
        r
    }

    pub fn new(csv: Vec<Token>) -> Option<Self> {
        // TODO: this does panic if we get an empty line
        let line = csv[0].loc.clone();

        // TODO: warn if it's missing
        // The dummy last line
        if csv[0].as_str() == "-1" {
            return None;
        }

        if csv.len() != 9 {
            error(
                &csv[0],
                ErrorKey::ParseError,
                "wrong number of fields for this line, expected 9",
            );
            return None;
        }

        let from = Self::_verify(&csv[0], "expected province id");
        let to = Self::_verify(&csv[1], "expected province id");
        let through = Self::_verify(&csv[3], "expected province id");
        let start_x = Self::_verify(&csv[4], "expected x coordinate");
        let start_y = Self::_verify(&csv[5], "expected y coordinate");
        let stop_x = Self::_verify(&csv[6], "expected x coordinate");
        let stop_y = Self::_verify(&csv[7], "expected y coordinate");

        Some(Adjacency {
            line,
            from: from?,
            to: to?,
            kind: csv[2].clone(),
            through: through?,
            start: Coords {
                x: start_x?,
                y: start_y?,
            },
            stop: Coords {
                x: stop_x?,
                y: stop_y?,
            },
            comment: csv[8].clone(),
        })
    }
}
