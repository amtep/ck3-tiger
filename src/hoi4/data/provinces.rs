use std::borrow::Borrow;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Seek, SeekFrom};
use std::num::NonZero;
use std::path::PathBuf;
use std::str::FromStr;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use image::{DynamicImage, Rgb, RgbImage};
use strum_macros::EnumString;

use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::parse::csv::{parse_csv, read_csv};
use crate::parse::ParserMemory;
use crate::report::{err, report, untidy, warn, ErrorKey, Severity};
use crate::token::Token;

pub type ProvId = u16;

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
pub struct Hoi4Provinces {
    /// Colors in the provinces.bmp
    colors: ColorBitArray,

    /// Provinces defined in definition.csv.
    /// HOI4 requires uninterrupted indices starting at 0, but we want to be able to warn
    /// and continue if they're not.
    provinces: TigerHashSet<Province>,

    /// Kept and used for error reporting.
    definition_csv: Option<FileEntry>,

    adjacencies: Vec<Adjacency>,
}

impl Hoi4Provinces {
    fn parse_definition(&mut self, csv: &[Token]) {
        if let Some(province) = Province::parse(csv) {
            if let Some(old_province) = self.provinces.replace(province) {
                err(ErrorKey::DuplicateItem)
                    .msg("duplicate entry for this province id")
                    .loc(&csv[0])
                    .loc_msg(&old_province.key, "previously defined here")
                    .push();
            }
        }
    }

    pub(crate) fn verify_exists_provid(
        &self,
        provid: ProvId,
        item: &Token,
        max_sev: Severity,
    ) -> bool {
        if self.provinces.contains(&provid) {
            true
        } else {
            let msg = format!("province {provid} not defined in map/definition.csv");
            report(ErrorKey::MissingItem, Item::Province.severity().at_most(max_sev))
                .msg(msg)
                .loc(item)
                .push();
            false
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
            self.provinces.contains(&provid)
        } else {
            false
        }
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.provinces.iter().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in &self.adjacencies {
            item.validate(self, data);
        }
        for item in &self.provinces {
            item.validate(self, data);
        }
    }
}

#[derive(Debug)]
pub enum FileContent {
    Adjacencies(String),
    Definitions(String),
    Provinces(RgbImage),
}

impl FileHandler<FileContent> for Hoi4Provinces {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map")
    }

    fn load_file(&self, entry: &FileEntry, _parser: &ParserMemory) -> Option<FileContent> {
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

                "provinces.bmp" => {
                    let img = match image::open(entry.fullpath()) {
                        Ok(img) => img,
                        Err(e) => {
                            let msg = format!("could not read `{}`: {e:#}", entry.path().display());
                            err(ErrorKey::ReadError).msg(msg).loc(entry).push();
                            return None;
                        }
                    };

                    if let DynamicImage::ImageRgb8(img) = img {
                        {
                            // SAFETY: image file is known to exist and of the bitmap format.
                            let mut file = File::open(entry.fullpath()).unwrap();
                            let mut buf = [0; 1];
                            file.seek(SeekFrom::Start(14)).unwrap(); // file header
                            file.read_exact(&mut buf).unwrap();
                            // DIB header size
                            if buf[0] != 40 {
                                let msg =
                                "bitmap has wrong DIB header format, should be BITMAPINFOHEADER";
                                let info =
                                    "see https://hoi4.paradoxwikis.com/Map_modding#BMP_format";
                                err(ErrorKey::ImageFormat).msg(msg).info(info).loc(entry).push();
                            }
                        }

                        let (width, height) = img.dimensions();
                        let msg = |s, p| -> String {
                            format!("bitmap {s} must be a multiple of 256, it is {p}")
                        };

                        if width % 256 != 0 {
                            err(ErrorKey::ImageSize).msg(msg("width", width)).loc(entry).push();
                        }
                        if height % 256 != 0 {
                            err(ErrorKey::ImageSize).msg(msg("height", height)).loc(entry).push();
                        }
                        let area = u64::from(width) * u64::from(height);
                        if area > 13_107_200 {
                            let msg = format!("total area cannot exceed 13_107_200, it is {area}");
                            err(ErrorKey::ImageSize).msg(msg).loc(entry).push();
                        }

                        return Some(FileContent::Provinces(img));
                    }
                    let msg = format!(
                        "bitmap has wrong color format `{:?}`, should be Rgb8",
                        img.color()
                    );
                    err(ErrorKey::ImageFormat).msg(msg).loc(entry).push();
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
                    let msg = "adjacencies.csv needs a line with all `-1;` at the end of this file";
                    err(ErrorKey::ParseError).msg(msg).loc(entry).push();
                }
            }
            FileContent::Definitions(content) => {
                self.definition_csv = Some(entry.clone());
                // Assume first line has province ID 0.
                for csv in parse_csv(entry, 1, &content) {
                    self.parse_definition(&csv);
                }
            }
            FileContent::Provinces(img) => {
                for pixel in img.pixels().copied() {
                    unsafe {
                        // SAFETY: `ColorBitArray::index` is guaranteed to return a valid index
                        self.colors.get_unchecked_mut(ColorBitArray::get_index(pixel)).commit(true);
                    }
                }
            }
        }
    }

    fn finalize(&mut self) {
        if self.definition_csv.is_none() {
            // Shouldn't happen, it should come from vanilla if not from the mod
            eprintln!("map/definition.csv is missing?!?");
            return;
        }
        let definition_csv = self.definition_csv.as_ref().unwrap();

        let mut seen_colors = TigerHashMap::default();

        let len = self.provinces.len();
        if len > 20_000 {
            // A fail-early error (HOI4 wiki):
            // "No more than 65536 different province borders can be displayed at the same time
            // before an integer overflow causes the in-game engine to stop displaying any
            // additional ones. In-game, this is usually hit at about 21000 provinces."
            let msg = format!("too many ({len}) provinces defined");
            warn(ErrorKey::Validation).msg(msg).loc(definition_csv).push();
        }

        #[allow(clippy::cast_possible_truncation)]
        for i in 1..=len as u16 {
            if let Some(province) = self.provinces.get(&i) {
                if let Some(key) = seen_colors.get(&province.color) {
                    warn(ErrorKey::Colors)
                        .msg("duplicate province color")
                        .loc(&province.key)
                        .loc_msg(key, "previously defined here")
                        .push();
                } else {
                    seen_colors.insert(province.color, province.key.clone());
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
                    "definition.csv lacks entry for color ({}, {}, {})",
                    rgb[0], rgb[1], rgb[2]
                );
                untidy(ErrorKey::Colors).msg(msg).loc(definition_csv).push();
            }
        }
    }
}

fn verify_field<T: FromStr>(v: &Token, msg: &str) -> Option<T> {
    let r = v.as_str().parse().ok();
    if r.is_none() {
        err(ErrorKey::ParseError).msg(msg).loc(v).push();
    }
    r
}

#[derive(Copy, Clone, Debug)]
pub struct Coord(Option<NonZero<u32>>);

impl FromStr for Coord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-1" => Ok(Self(None)),
            "0" => Ok(Self(Some(unsafe { NonZero::new_unchecked(1) }))), // hack to allow 0 edge-case
            _ => s.parse::<NonZero<u32>>().map(|c| Self(Some(c))).map_err(|_| ()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AdjacencyKind {
    Sea,
    Impassable,
}

impl FromStr for AdjacencyKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sea" | "" => Ok(AdjacencyKind::Sea),
            "impassable" => Ok(AdjacencyKind::Impassable),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Adjacency {
    key: Token,
    from: ProvId,
    to: ProvId,
    kind: AdjacencyKind,
    through: Option<ProvId>,
    // TODO: check start and stop are map coordinates and have the right color on province.bmp
    start_x: Coord,
    start_y: Coord,
    stop_x: Coord,
    stop_y: Coord,
    rule: Option<&'static str>,
}

impl Adjacency {
    pub fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        if csv.len() != 10 {
            let msg = "wrong number of fields for this line, expected 10";
            err(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
            return None;
        }

        let from = verify_field(&csv[0], "expected province id")?;
        let to = verify_field(&csv[1], "expected province id")?;
        let kind = verify_field(&csv[2], "expected adjacency type: sea | impassable")?;
        let through = if csv[3].is("-1") {
            None
        } else {
            Some(verify_field(&csv[3], "expected province id | -1")?)
        };
        let start_x = verify_field(&csv[4], "expected x coordinate | -1")?;
        let start_y = verify_field(&csv[5], "expected y coordinate | -1")?;
        let stop_x = verify_field(&csv[6], "expected x coordinate | -1")?;
        let stop_y = verify_field(&csv[7], "expected y coordinate | -1")?;
        let rule = if csv[8].is("") { None } else { Some(csv[8].as_str()) };

        Some(Adjacency {
            key: csv[0].clone(),
            from,
            to,
            kind,
            through,
            start_x,
            start_y,
            stop_x,
            stop_y,
            rule,
        })
    }

    fn validate(&self, provinces: &Hoi4Provinces, data: &Everything) {
        if !provinces.verify_exists_provid(self.from, &self.key, Severity::Error)
            || !provinces.verify_exists_provid(self.to, &self.key, Severity::Error)
        {
            return;
        }

        match self.kind {
            AdjacencyKind::Sea => {
                // TODO: Verify two land provinces are not touching
                let from_kind = provinces.provinces.get(&self.from).unwrap().kind;
                let to_kind = provinces.provinces.get(&self.to).unwrap().kind;

                if from_kind != to_kind {
                    let msg = "from and to provinces must have the same type";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }
                // TODO: Verify non-touching provinces have a through province
            }
            AdjacencyKind::Impassable => {
                // TODO: Verify provinces are touching
                for (s, coord) in [
                    ("start_x", self.start_x),
                    ("start_y", self.start_y),
                    ("stop_x", self.stop_x),
                    ("stop_y", self.stop_y),
                ] {
                    if coord.0.is_some() {
                        let msg = format!("{s} coordinate must be `-1` for impassable adjacency");
                        err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                    }
                }

                if self.through.is_some() {
                    let msg = "through province must be `-1` for impassable adjacency";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }

                if self.rule.is_some() {
                    let msg = "adjacency rule must be left empty for impassable adjacency";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }
            }
        }

        if let Some(rule) = self.rule {
            data.verify_exists_implied(Item::AdjacencyRule, rule, &self.key);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ProvinceKind {
    Land,
    Sea,
    Lake,
}

#[derive(Clone, Debug)]
pub struct Province {
    key: Token,
    id: ProvId,
    color: Rgb<u8>,
    kind: ProvinceKind,
    terrain: &'static str,
    continent: u16,
}

impl PartialEq for Province {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Province {}

impl Hash for Province {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Borrow<ProvId> for Province {
    fn borrow(&self) -> &ProvId {
        &self.id
    }
}

impl Province {
    fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        if csv.len() != 8 {
            let msg = "incorrect number of fields, expected 8";
            err(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
            return None;
        }

        let id = verify_field(&csv[0], "expected province id")?;
        let r = verify_field(&csv[1], "expected red value: 0-255")?;
        let g = verify_field(&csv[2], "expected green value: 0-255")?;
        let b = verify_field(&csv[3], "expected blue value: 0-255")?;
        let color = Rgb::from([r, g, b]);
        let kind = verify_field(&csv[4], "expected province type: land | sea | lake")?;
        // Legacy coastal status; bitmap adjacency with a sea province takes precedence.
        // TODO: Store status and compare with bitmap computed result, warn if different.
        verify_field::<bool>(&csv[5], "expected boolean: true | false")?;
        let continent = verify_field(&csv[7], "expected continent id")?;

        Some(Province { key: csv[0].clone(), id, color, kind, terrain: csv[6].as_str(), continent })
    }

    fn validate(&self, _provinces: &Hoi4Provinces, data: &Everything) {
        #[allow(clippy::cast_possible_truncation)]
        let continent_count = data.iter_keys(Item::Continent).count() as u16;
        match self.kind {
            ProvinceKind::Land => {
                if self.continent == 0 {
                    let msg = "land province must have a non-zero continent ID";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                } else if self.continent > continent_count {
                    let msg = format!(
                        "continent ID greater than total number of continents ({continent_count})"
                    );
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }

                if self.terrain == "unknown" {
                    let msg = "default unknown land province";
                    untidy(ErrorKey::UseOfThis).msg(msg).loc(&self.key).push();
                }

                data.verify_exists_implied(Item::Terrain, self.terrain, &self.key);
                if data.item_has_property(Item::Terrain, self.terrain, "is_water") {
                    let msg = "land province must have a land terrain";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }
            }
            ProvinceKind::Sea => {
                if self.continent != 0 {
                    let msg = "sea province must have a zero continent ID";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }

                if self.terrain != "ocean" {
                    let msg = "sea province must have `ocean` terrain";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }
            }
            ProvinceKind::Lake => {
                if self.terrain != "lakes" {
                    let msg = "lake province must have `lakes` terrain";
                    err(ErrorKey::Validation).msg(msg).loc(&self.key).push();
                }
            }
        }
    }
}
