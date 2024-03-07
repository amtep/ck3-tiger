use std::path::PathBuf;
use std::str::FromStr;

use fnv::{FnvHashMap, FnvHashSet};
use image::{DynamicImage, Rgb};

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::item::Item;
use crate::parse::csv::{parse_csv, read_csv};
use crate::pdxfile::PdxFile;
use crate::report::{err, fatal, report, untidy, warn, ErrorKey, Severity};
use crate::token::{Loc, Token};

pub type ProvId = u32;

#[derive(Clone, Debug, Default)]
pub struct ImperatorProvinces {
    /// Colors in the provinces.png
    colors: FnvHashSet<Rgb<u8>>,

    /// Provinces defined in definition.csv.
    /// Imperator requires uninterrupted indices starting at 0, but we want to be able to warn
    /// and continue if they're not, so it's a hashmap.
    provinces: FnvHashMap<ProvId, Province>,

    /// Kept and used for error reporting.
    definition_csv: Option<FileEntry>,

    adjacencies: Vec<Adjacency>,

    impassable: FnvHashSet<ProvId>,

    sea_or_river: FnvHashSet<ProvId>,
}

impl ImperatorProvinces {
    fn parse_definition(&mut self, csv: &[Token]) {
        if let Some(province) = Province::parse(csv) {
            if self.provinces.contains_key(&province.id) {
                err(ErrorKey::DuplicateItem)
                    .msg("duplicate entry for this province id")
                    .loc(&province.comment)
                    .push();
            }
            self.provinces.insert(province.id, province);
        }
    }

    pub fn load_impassable(&mut self, block: &Block) {
        enum Expecting<'a> {
            Range(&'a Token),
            List(&'a Token),
            Nothing,
        }

        let mut expecting = Expecting::Nothing;
        for item in block.iter_items() {
            match expecting {
                Expecting::Nothing => {
                    if let Some((key, token)) = item.expect_assignment() {
                        if key.is("sea_zones")
                            || key.is("river_provinces")
                            || key.is("impassable_terrain")
                            || key.is("uninhabitable")
                            || key.is("lakes")
                            || key.is("LAKES")
                        {
                            if token.is("LIST") {
                                expecting = Expecting::List(key);
                            } else if token.is("RANGE") {
                                expecting = Expecting::Range(key);
                            } else {
                                expecting = Expecting::Nothing;
                            }
                        } else {
                            // TODO: this has to wait until full validation
                            // let msg = format!("unexpected key `{key}`");
                            // warn(ErrorKey::UnknownField).weak().msg(msg).loc(key).push();
                        }
                    }
                }
                Expecting::Range(key) => {
                    if let Some(block) = item.expect_block() {
                        let vec: Vec<&Token> = block.iter_values().collect();
                        if vec.len() != 2 {
                            err(ErrorKey::Validation).msg("invalid RANGE").loc(block).push();
                            expecting = Expecting::Nothing;
                            continue;
                        }
                        let from = vec[0].as_str().parse::<ProvId>();
                        let to = vec[1].as_str().parse::<ProvId>();
                        if from.is_err() || to.is_err() {
                            err(ErrorKey::Validation).msg("invalid RANGE").loc(block).push();
                            expecting = Expecting::Nothing;
                            continue;
                        }
                        for provid in from.unwrap()..=to.unwrap() {
                            self.impassable.insert(provid);
                            if key.is("sea_zones") || key.is("river_provinces") {
                                self.sea_or_river.insert(provid);
                            }
                        }
                    }
                    expecting = Expecting::Nothing;
                }
                Expecting::List(key) => {
                    if let Some(block) = item.expect_block() {
                        for token in block.iter_values() {
                            let provid = token.as_str().parse::<ProvId>();
                            if let Ok(provid) = provid {
                                self.impassable.insert(provid);
                                if key.is("sea_zones") || key.is("river_provinces") {
                                    self.sea_or_river.insert(provid);
                                }
                            } else {
                                err(ErrorKey::Validation)
                                    .msg("invalid LIST item")
                                    .loc(token)
                                    .push();
                                break;
                            }
                        }
                    }
                    expecting = Expecting::Nothing;
                }
            }
        }
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token, max_sev: Severity) {
        if let Ok(provid) = key.parse::<ProvId>() {
            if !self.provinces.contains_key(&provid) {
                let msg = format!("province {provid} not defined in map_data/definition.csv");
                report(ErrorKey::MissingItem, Item::Province.severity()).msg(msg).loc(item).push();
            }
        } else {
            let msg = "province id should be numeric";
            let sev = Item::Province.severity().at_most(max_sev);
            report(ErrorKey::Validation, sev).msg(msg).loc(item).push();
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Ok(provid) = key.parse::<ProvId>() {
            self.provinces.contains_key(&provid)
        } else {
            false
        }
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.provinces.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in &self.adjacencies {
            item.validate(self);
        }
    }
}

#[derive(Debug)]
pub enum FileContent {
    Adjacencies(String),
    Definitions(String),
    Provinces(DynamicImage),
    DefaultMap(Block),
}

impl FileHandler<FileContent> for ImperatorProvinces {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<FileContent> {
        if entry.path().components().count() == 2 {
            match &*entry.filename().to_string_lossy() {
                "adjacencies.csv" => {
                    let content = match read_csv(entry.fullpath()) {
                        Ok(content) => content,
                        Err(e) => {
                            err(ErrorKey::ReadError)
                                .msg(format!("could not read file: {e:#}"))
                                .loc(entry)
                                .push();
                            return None;
                        }
                    };
                    return Some(FileContent::Adjacencies(content));
                }

                "definition.csv" => {
                    let content = match read_csv(entry.fullpath()) {
                        Ok(content) => content,
                        Err(e) => {
                            let msg =
                                format!("could not read `{}`: {:#}", entry.path().display(), e);
                            err(ErrorKey::ReadError).msg(msg).loc(entry).push();
                            return None;
                        }
                    };
                    return Some(FileContent::Definitions(content));
                }

                "provinces.png" => {
                    let img = match image::open(entry.fullpath()) {
                        Ok(img) => img,
                        Err(e) => {
                            let msg = format!("could not read `{}`: {e:#}", entry.path().display());
                            err(ErrorKey::ReadError).msg(msg).loc(entry).push();
                            return None;
                        }
                    };
                    if let DynamicImage::ImageRgb8(_) = img {
                        return Some(FileContent::Provinces(img));
                    }
                    let msg = format!(
                        "`{}` has wrong color format `{:?}`, should be Rgb8",
                        entry.path().display(),
                        img.color()
                    );
                    err(ErrorKey::ImageFormat).msg(msg).loc(entry).push();
                }

                "default.map" => {
                    return PdxFile::read_optional_bom(entry).map(FileContent::DefaultMap);
                }
                _ => (),
            }
        }
        None
    }

    fn handle_file(&mut self, entry: &FileEntry, content: FileContent) {
        match content {
            FileContent::Adjacencies(content) => {
                let mut seen_terminator = false;
                for csv in parse_csv(entry, 1, &content) {
                    if csv[0].is("-1") {
                        seen_terminator = true;
                    } else if seen_terminator {
                        let msg = "the line with all `-1;` should be the last line in the file";
                        warn(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
                        break;
                    } else {
                        self.adjacencies.extend(Adjacency::parse(&csv));
                    }
                }
                if !seen_terminator {
                    let msg = "CK3 needs a line with all `-1;` at the end of this file";
                    err(ErrorKey::ParseError).msg(msg).loc(entry).push();
                }
            }
            FileContent::Definitions(content) => {
                self.definition_csv = Some(entry.clone());
                for csv in parse_csv(entry, 0, &content) {
                    self.parse_definition(&csv);
                }
            }
            FileContent::Provinces(img) => {
                if let DynamicImage::ImageRgb8(img) = img {
                    for pixel in img.pixels() {
                        self.colors.insert(*pixel);
                    }
                }
            }
            FileContent::DefaultMap(block) => self.load_impassable(&block),
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
                if let Some(k) = seen_colors.get(&province.color) {
                    let msg = format!("color was already used for id {k}");
                    warn(ErrorKey::Colors).msg(msg).loc(&province.comment).push();
                } else {
                    seen_colors.insert(province.color, i);
                }
            } else {
                let msg = format!("province ids must be sequential, but {i} is missing");
                err(ErrorKey::Validation).msg(msg).loc(definition_csv).push();
                return;
            }
        }
        for color in &self.colors {
            if !seen_colors.contains_key(color) {
                let Rgb(rgb) = color;
                let msg = format!(
                    "definitions.csv lacks entry for color ({}, {}, {})",
                    rgb[0], rgb[1], rgb[2]
                );
                untidy(ErrorKey::Colors).msg(msg).loc(definition_csv).push();
            }
        }
    }
}

#[allow(dead_code)] // TODO
#[derive(Copy, Clone, Debug, Default)]
pub struct Coords {
    x: i32,
    y: i32,
}

#[allow(dead_code)] // TODO
#[derive(Clone, Debug)]
pub struct Adjacency {
    line: Loc,
    from: ProvId,
    to: ProvId,
    /// TODO: check type is "sea" or "river_large"
    /// sea or river_large
    kind: Token,
    through: ProvId,
    /// TODO: check start and stop are map coordinates and have the right color on province.png
    /// They can be -1 -1 though.
    start: Coords,
    stop: Coords,
    comment: Token,
}

fn _verify<T: FromStr>(v: &Token, msg: &str) -> Option<T> {
    let r = v.as_str().parse().ok();
    if r.is_none() {
        err(ErrorKey::ParseError).msg(msg).loc(v).push();
    }
    r
}

impl Adjacency {
    pub fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        let line = csv[0].loc;

        if csv.len() != 9 {
            let msg = "wrong number of fields for this line, expected 9";
            err(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
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
            start: Coords { x: start_x?, y: start_y? },
            stop: Coords { x: stop_x?, y: stop_y? },
            comment: csv[8].clone(),
        })
    }

    fn validate(&self, provinces: &ImperatorProvinces) {
        for prov in &[self.from, self.to, self.through] {
            if !provinces.provinces.contains_key(prov) {
                let msg = format!("province id {prov} not defined in definitions.csv");
                fatal(ErrorKey::Crash).msg(msg).loc(self.line).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Province {
    key: Token,
    id: ProvId,
    color: Rgb<u8>,
    comment: Token,
}

impl Province {
    fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        if csv.len() < 5 {
            let msg = "too few fields for this line, expected 5";
            err(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
            return None;
        }

        let id = _verify(&csv[0], "expected province id")?;
        let r = _verify(&csv[1], "expected red value")?;
        let g = _verify(&csv[2], "expected green value")?;
        let b = _verify(&csv[3], "expected blue value")?;
        let color = Rgb::from([r, g, b]);
        Some(Province { key: csv[0].clone(), id, color, comment: csv[4].clone() })
    }
}
