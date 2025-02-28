use std::path::PathBuf;
use std::str::FromStr;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use image::{DynamicImage, Rgb};

use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::game::GameFlags;
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::item::ItemLoader;
use crate::parse::ParserMemory;
use crate::parse::csv::{parse_csv, read_csv};
use crate::pdxfile::{PdxEncoding, PdxFile};
use crate::report::{ErrorKey, Severity, err, fatal, report, untidy, warn};
use crate::token::{Loc, Token};
use crate::validator::Validator;

pub type ProvId = u32;

const COLOUR_COUNT: usize = 256 * 256 * 256;

#[derive(Clone, Debug)]
struct ColorBitArray(BitBox);

impl Default for ColorBitArray {
    fn default() -> Self {
        Self(bitbox![0; COLOUR_COUNT])
    }
}

impl ColorBitArray {
    fn get_index(color: Rgb<u8>) -> usize {
        let Rgb([r, g, b]) = color;
        ((r as usize) << 16) | ((g as usize) << 8) | b as usize
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_color(index: usize) -> Rgb<u8> {
        let r = (index >> 16) as u8;
        let g = (index >> 8) as u8;
        let b = index as u8;
        Rgb([r, g, b])
    }
}

impl std::ops::Deref for ColorBitArray {
    type Target = BitBox;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ColorBitArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct Ck3Provinces {
    /// Colors in the provinces.png
    colors: ColorBitArray,

    /// Provinces defined in definition.csv.
    /// CK3 requires uninterrupted indices starting at 0, but we want to be able to warn
    /// and continue if they're not, so it's a hashmap.
    provinces: TigerHashMap<ProvId, Province>,

    /// Kept and used for error reporting.
    definition_csv: Option<FileEntry>,

    adjacencies: Vec<Adjacency>,

    impassable: TigerHashSet<ProvId>,

    sea_or_river: TigerHashSet<ProvId>,
}

impl Ck3Provinces {
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
                            || key.is("impassable_mountains")
                            || key.is("impassable_seas")
                            || key.is("lakes")
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

    pub(crate) fn verify_exists_provid(&self, provid: ProvId, item: &Token, max_sev: Severity) {
        if !self.provinces.contains_key(&provid) {
            let msg = format!("province {provid} not defined in map_data/definition.csv");
            report(ErrorKey::MissingItem, Item::Province.severity().at_most(max_sev))
                .msg(msg)
                .loc(item)
                .push();
        }
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token, max_sev: Severity) {
        if let Ok(provid) = key.parse::<ProvId>() {
            self.verify_exists_provid(provid, item, max_sev);
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

    pub(crate) fn is_sea_or_river(&self, provid: ProvId) -> bool {
        self.sea_or_river.contains(&provid)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.provinces.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in &self.adjacencies {
            item.validate(self);
        }
        for item in self.provinces.values() {
            item.validate(self, data);
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

impl FileHandler<FileContent> for Ck3Provinces {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<FileContent> {
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
                    return PdxFile::read_optional_bom(entry, parser).map(FileContent::DefaultMap);
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
                    for pixel in img.pixels().copied() {
                        unsafe {
                            // SAFETY: `ColorBitArray::index` is guaranteed to return a valid index
                            self.colors
                                .get_unchecked_mut(ColorBitArray::get_index(pixel))
                                .commit(true);
                        }
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

        let mut seen_colors = TigerHashMap::default();
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
        for color_index in self.colors.iter_ones() {
            let color = ColorBitArray::get_color(color_index);
            if !seen_colors.contains_key(&color) {
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
    /// TODO: check type is sea or `river_large`
    /// sea or `river_large`
    kind: Token,
    through: ProvId,
    /// TODO: check start and stop are map coordinates and have the right color on province.png
    /// They can be -1 -1 though.
    start: Coords,
    stop: Coords,
    comment: Token,
}

fn verify_field<T: FromStr>(v: &Token, msg: &str) -> Option<T> {
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

        let from = verify_field(&csv[0], "expected province id");
        let to = verify_field(&csv[1], "expected province id");
        let through = verify_field(&csv[3], "expected province id");
        let start_x = verify_field(&csv[4], "expected x coordinate");
        let start_y = verify_field(&csv[5], "expected y coordinate");
        let stop_x = verify_field(&csv[6], "expected x coordinate");
        let stop_y = verify_field(&csv[7], "expected y coordinate");

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

    fn validate(&self, provinces: &Ck3Provinces) {
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

        let id = verify_field(&csv[0], "expected province id")?;
        let r = verify_field(&csv[1], "expected red value")?;
        let g = verify_field(&csv[2], "expected green value")?;
        let b = verify_field(&csv[3], "expected blue value")?;
        let color = Rgb::from([r, g, b]);
        Some(Province { key: csv[0].clone(), id, color, comment: csv[4].clone() })
    }

    fn validate(&self, provinces: &Ck3Provinces, data: &Everything) {
        if provinces.sea_or_river.contains(&self.id) {
            // TODO: this really needs an explanation, like "missing .... for sea zone"
            data.verify_exists(Item::Localization, &self.comment);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProvinceMapping {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::ProvinceMapping, PdxEncoding::Utf8Bom, ".txt", true, ProvinceMapping::add)
}

impl ProvinceMapping {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProvinceMapping, key, block, Box::new(Self {}));
    }
}

impl DbKind for ProvinceMapping {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Province, key);
            data.verify_exists(Item::Province, value);
        });
    }
}
