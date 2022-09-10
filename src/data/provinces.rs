use fnv::{FnvHashMap, FnvHashSet};
use image::{DynamicImage, Rgb};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::block::{Block, BlockOrValue};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::parse::csv::{parse_csv, read_csv};
use crate::pdxfile::PdxFile;
use crate::token::{Loc, Token};

pub type ProvId = u32;

#[derive(Clone, Debug, Default)]
pub struct Provinces {
    /// Colors in the provinces.png
    colors: FnvHashSet<Rgb<u8>>,

    /// Provinces defined in definition.csv.
    /// CK3 requires uninterrupted indices starting at 0, but we want to be able to warn
    /// and continue if they're not, so it's a hashmap.
    provinces: FnvHashMap<ProvId, Province>,

    /// Kept and used for error reporting
    definition_csv: Option<FileEntry>,

    adjacencies: Vec<Adjacency>,

    impassable: FnvHashSet<ProvId>,
}

impl Provinces {
    fn parse_definition(&mut self, csv: &[Token]) {
        if let Some(province) = Province::parse(csv) {
            if self.provinces.contains_key(&province.id) {
                error(
                    &province.comment,
                    ErrorKey::Duplicate,
                    "duplicate entry for this province id",
                );
            }
            self.provinces.insert(province.id, province);
        }
    }

    pub fn load_impassable(&mut self, block: &Block) {
        enum Expecting {
            Range,
            List,
            Nothing,
        }

        let mut expecting = Expecting::Nothing;
        for (k, _, v) in block.iter_items() {
            if let Some(key) = k {
                if key.is("sea_zones")
                    || key.is("river_provinces")
                    || key.is("impassable_mountains")
                {
                    if let BlockOrValue::Token(t) = v {
                        if t.is("LIST") {
                            expecting = Expecting::List;
                        } else if t.is("RANGE") {
                            expecting = Expecting::Range;
                        } else {
                            expecting = Expecting::Nothing;
                        }
                    }
                }
            } else if let BlockOrValue::Block(b) = v {
                if matches!(expecting, Expecting::Range) {
                    let vec = b.get_values();
                    if vec.len() != 2 {
                        error(b, ErrorKey::Validation, "invalid RANGE");
                        expecting = Expecting::Nothing;
                        continue;
                    }
                    let from = vec[0].as_str().parse::<ProvId>();
                    let to = vec[1].as_str().parse::<ProvId>();
                    if from.is_err() || to.is_err() {
                        error(b, ErrorKey::Validation, "invalid RANGE");
                        expecting = Expecting::Nothing;
                        continue;
                    }
                    for provid in from.unwrap()..=to.unwrap() {
                        self.impassable.insert(provid);
                    }
                } else if matches!(expecting, Expecting::List) {
                    for token in b.get_values() {
                        let provid = token.as_str().parse::<ProvId>();
                        if let Ok(provid) = provid {
                            self.impassable.insert(provid);
                        } else {
                            error(b, ErrorKey::Validation, "invalid LIST");
                            break;
                        }
                    }
                }
                expecting = Expecting::Nothing;
            }
        }
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token) {
        if let Ok(provid) = key.parse::<ProvId>() {
            if !self.provinces.contains_key(&provid) {
                let msg = format!("province {} not defined in map_data/definition.csv", provid);
                error(item, ErrorKey::MissingItem, &msg);
            }
        } else {
            error(item, ErrorKey::Validation, "province id should be numeric");
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Ok(provid) = key.parse::<ProvId>() {
            self.provinces.contains_key(&provid)
        } else {
            false
        }
    }

    #[allow(clippy::unused_self)]
    pub fn validate(&self, _data: &Everything) {
        // TODO: validate adjacencies
    }
}

impl FileHandler for Provinces {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if entry.path().components().count() == 2 {
            match &*entry.filename().to_string_lossy() {
                "adjacencies.csv" => {
                    let content = match read_csv(fullpath) {
                        Ok(content) => content,
                        Err(e) => {
                            error(
                                entry,
                                ErrorKey::ReadError,
                                &format!("could not read file: {:#}", e),
                            );
                            return;
                        }
                    };
                    let mut seen_terminator = false;
                    for csv in parse_csv(entry, 1, &content) {
                        if csv[0].is("-1") {
                            seen_terminator = true;
                        } else if seen_terminator {
                            warn(
                                &csv[0],
                                ErrorKey::ParseError,
                                "the line with all `-1;` should be the last line in the file",
                            );
                            break;
                        } else {
                            self.adjacencies.extend(Adjacency::parse(&csv));
                        }
                    }
                    if !seen_terminator {
                        error(
                            entry,
                            ErrorKey::ParseError,
                            "CK3 needs a line with all `-1;` at the end of this file",
                        );
                    }
                }
                "definition.csv" => {
                    self.definition_csv = Some(entry.clone());
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
                    for csv in parse_csv(entry, 0, &content) {
                        self.parse_definition(&csv);
                    }
                }
                "provinces.png" => {
                    let img = match image::open(fullpath) {
                        Ok(img) => img,
                        Err(e) => {
                            error(
                                entry,
                                ErrorKey::ReadError,
                                &format!("could not read `{}`: {:#}", entry.path().display(), e),
                            );
                            return;
                        }
                    };
                    if let DynamicImage::ImageRgb8(img) = img {
                        for pixel in img.pixels() {
                            self.colors.insert(*pixel);
                        }
                    } else {
                        error(
                            entry,
                            ErrorKey::ImageFormat,
                            &format!(
                                "`{}` has wrong color format `{:?}`, should be Rgb8",
                                entry.path().display(),
                                img.color()
                            ),
                        );
                    }
                }
                "default.map" => {
                    let block = match PdxFile::read(entry, fullpath) {
                        Some(block) => block,
                        None => return,
                    };
                    self.load_impassable(&block);
                }
                _ => (),
            }
        }
    }

    fn finalize(&mut self) {
        if self.definition_csv.is_none() {
            // Shouldn't happen, it should come from vanilla if not from the mod
            eprintln!("map_data/definition.csv is missing?!?");
            return;
        }
        let definition_csv = self.definition_csv.as_ref().unwrap();

        let mut seen_colors = FnvHashMap::default();
        #[allow(clippy::cast_possible_truncation)]
        for i in 1..self.provinces.len() as u32 {
            if let Some(province) = self.provinces.get(&i) {
                if !province.valid {
                    continue;
                }
                if !self.colors.contains(&province.color) {
                    warn(
                        &province.comment,
                        ErrorKey::Validation,
                        "color is not in the provinces.png",
                    );
                } else if let Some(k) = seen_colors.get(&province.color) {
                    warn(
                        &province.comment,
                        ErrorKey::Validation,
                        &format!("color was already used for id {}", k),
                    );
                } else {
                    seen_colors.insert(province.color, i);
                }
            } else {
                error(
                    definition_csv,
                    ErrorKey::Validation,
                    &format!("province ids must be sequential, but {} is missing", i),
                );
                return;
            }
        }
        if seen_colors.len() < self.colors.len() {
            warn(
                definition_csv,
                ErrorKey::Validation,
                &format!(
                    "provinces.png contains {} colors with no provinces assigned",
                    self.colors.len() - seen_colors.len()
                ),
            );
        }
    }
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

fn _verify<T: FromStr>(v: &Token, msg: &str) -> Option<T> {
    let r = v.as_str().parse().ok();
    if r.is_none() {
        error(v, ErrorKey::ParseError, msg);
    }
    r
}

impl Adjacency {
    pub fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        let line = csv[0].loc.clone();

        if csv.len() != 9 {
            error(
                &csv[0],
                ErrorKey::ParseError,
                "wrong number of fields for this line, expected 9",
            );
            return None;
        }

        let from = _verify(&csv[0], "expected province id");
        let to = _verify(&csv[1], "expected province id");
        let through = _verify(&csv[3], "expected province id");
        let start_x = _verify(&csv[4], "expected x coordinate");
        let start_y = _verify(&csv[5], "expected y coordinate");
        let stop_x = _verify(&csv[6], "expected x coordinate");
        let stop_y = _verify(&csv[7], "expected y coordinate");

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

#[derive(Clone, Debug)]
pub struct Province {
    id: ProvId,
    valid: bool,
    color: Rgb<u8>,
    comment: Token,
}

impl Province {
    fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        let line = csv[0].loc.clone();

        if csv.len() < 5 {
            error(
                &line,
                ErrorKey::ParseError,
                "too few fields for this line, expected 5",
            );
            return None;
        }

        let id = _verify(&csv[0], "expected province id")?;
        let r = _verify(&csv[1], "expected red value")?;
        let g = _verify(&csv[2], "expected green value")?;
        let b = _verify(&csv[3], "expected blue value")?;
        let color = Rgb::from([r, g, b]);
        Some(Province {
            id,
            valid: !csv[4].as_str().is_empty(),
            color,
            comment: csv[4].clone(),
        })
    }
}
