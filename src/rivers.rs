//! Special validator for the `rivers.png` file.
//!
//! The `rivers.png` file has detailed requirements for its image format and the layout of every pixel.

use std::fs::File;
use std::ops::{RangeInclusive, RangeToInclusive};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use png::{ColorType, Decoder};

use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::parse::ParserMemory;
use crate::report::{ErrorKey, err, warn, will_maybe_log};

/// The `rivers.png` has an indexed palette where the colors don't matter, only the index values
/// used in the pixels matter. Pixels that are not among the values defined here are ignored when
/// the game processes the `rivers.png`.
struct RiverPixels {}
impl RiverPixels {
    /// Normal rivers of various widths (usually blue through greenish).
    /// They are still all one pixel wide in the `rivers.png`; this just controls how they are painted on the map.
    /// River pixels must be adjacent to each other horizontally or vertically; together they form river segments.
    const NORMAL: RangeInclusive<u8> = (RiverPixels::FIRST_NORMAL..=RiverPixels::LAST_NORMAL);
    const FIRST_NORMAL: u8 = 3;
    const LAST_NORMAL: u8 = 15;
    /// "specials" are the starting and ending pixels of river segments
    const SPECIAL: RangeToInclusive<u8> = (..=RiverPixels::LAST_SPECIAL);
    const LAST_SPECIAL: u8 = 2;
    /// A pixel at the start of a river segment (usually green)
    const SOURCE: u8 = 0;
    /// A pixel that joins one river segment into another (usually red)
    const TRIBUTARY: u8 = 1;
    /// A pixel that is used where a river splits off from another (usually yellow)
    const SPLIT: u8 = 2;
    /// Noncoding pixels
    const FIRST_IGNORE: u8 = 16;
}

#[derive(Clone, Debug, Default)]
pub struct Rivers {
    /// for error reporting
    entry: Option<FileEntry>,
    width: u32,
    height: u32,
    color_type: Option<ColorType>,
    palette: Option<Vec<u8>>,
    pixels: Vec<u8>,
}

impl Rivers {
    pub fn load_png(&mut self, fullpath: &Path) -> Result<()> {
        let decoder = Decoder::new(File::open(fullpath)?);
        let mut reader = decoder.read_info()?;

        let info = reader.info();
        self.width = info.width;
        self.height = info.height;
        self.color_type = Some(info.color_type);
        if let Some(palette) = info.palette.clone() {
            self.palette = Some(palette.into_owned());
        }

        self.pixels = vec![0; reader.output_buffer_size()];
        let frame_info = reader.next_frame(&mut self.pixels)?;

        if frame_info.width != self.width
            || frame_info.height != self.height
            || frame_info.color_type != self.color_type.unwrap()
        {
            bail!("PNG frame did not match image info");
        }

        Ok(())
    }

    fn river_neighbors(&self, x: u32, y: u32, output: &mut Vec<(u32, u32)>) {
        output.clear();
        if x > 0 && RiverPixels::NORMAL.contains(&self.pixel(x - 1, y)) {
            output.push((x - 1, y));
        }
        if y > 0 && RiverPixels::NORMAL.contains(&self.pixel(x, y - 1)) {
            output.push((x, y - 1));
        }
        if x + 1 < self.width && RiverPixels::NORMAL.contains(&self.pixel(x + 1, y)) {
            output.push((x + 1, y));
        }
        if y + 1 < self.height && RiverPixels::NORMAL.contains(&self.pixel(x, y + 1)) {
            output.push((x, y + 1));
        }
    }

    fn special_neighbors(&self, c: (u32, u32)) -> Vec<(u32, u32)> {
        let (x, y) = c;
        let mut vec = Vec::new();
        if x > 0 && RiverPixels::SPECIAL.contains(&self.pixel(x - 1, y)) {
            vec.push((x - 1, y));
        }
        if y > 0 && RiverPixels::SPECIAL.contains(&self.pixel(x, y - 1)) {
            vec.push((x, y - 1));
        }
        if x + 1 < self.width && RiverPixels::SPECIAL.contains(&self.pixel(x + 1, y)) {
            vec.push((x + 1, y));
        }
        if y + 1 < self.height && RiverPixels::SPECIAL.contains(&self.pixel(x, y + 1)) {
            vec.push((x, y + 1));
        }
        vec
    }

    fn pixel(&self, x: u32, y: u32) -> u8 {
        let idx = (x + self.width * y) as usize;
        self.pixels[idx]
    }

    fn validate_segments(
        &self,
        river_segments: TigerHashMap<(u32, u32), (u32, u32)>,
        mut specials: TigerHashMap<(u32, u32), bool>,
    ) {
        let mut seen = TigerHashSet::default();

        for (start, end) in river_segments {
            if seen.contains(&start) {
                continue;
            }
            seen.insert(end);

            if start == end {
                // Single-pixel segment
                let special_neighbors = self.special_neighbors(start);
                if special_neighbors.len() > 1 {
                    let msg = format!(
                        "({}, {}) river pixel connects two special pixels",
                        start.0, start.1
                    );
                    warn(ErrorKey::Rivers).msg(msg).loc(self.entry.as_ref().unwrap()).push();
                } else if special_neighbors.is_empty() {
                    let msg = format!("({}, {}) orphan river pixel", start.0, start.1);
                    warn(ErrorKey::Rivers).msg(msg).loc(self.entry.as_ref().unwrap()).push();
                } else {
                    let s = special_neighbors[0];
                    if specials[&s] {
                        let msg =
                            format!("({}, {}) pixel terminates multiple river segments", s.0, s.1);
                        warn(ErrorKey::Rivers).msg(msg).loc(self.entry.as_ref().unwrap()).push();
                    } else {
                        specials.insert(s, true);
                    }
                }
            } else {
                let mut special_neighbors = self.special_neighbors(start);
                special_neighbors.append(&mut self.special_neighbors(end));
                if special_neighbors.is_empty() {
                    let msg = format!(
                        "({}, {}) - ({}, {}) orphan river segment",
                        start.0, start.1, end.0, end.1
                    );
                    warn(ErrorKey::Rivers).msg(msg).loc(self.entry.as_ref().unwrap()).push();
                } else if special_neighbors.len() > 1 {
                    let msg = format!(
                        "({}, {}) - ({}, {}) river segment has two terminators",
                        start.0, start.1, end.0, end.1
                    );
                    warn(ErrorKey::Rivers).msg(msg).loc(self.entry.as_ref().unwrap()).push();
                } else {
                    let s = special_neighbors[0];
                    if specials[&s] {
                        let msg =
                            format!("({}, {}) pixel terminates multiple river segments", s.0, s.1);
                        warn(ErrorKey::Rivers).msg(msg).loc(self.entry.as_ref().unwrap()).push();
                    } else {
                        specials.insert(s, true);
                    }
                }
            }
        }
    }

    pub fn validate(&self, _data: &Everything) {
        // TODO: check image width and height against world defines

        if self.color_type != Some(ColorType::Indexed) {
            let msg = "rivers.png should be in indexed color format (with 8-bit palette)";
            err(ErrorKey::ImageFormat).msg(msg).loc(self.entry.as_ref().unwrap()).push();
            return;
        }

        if self.palette.is_none() {
            let msg = "rivers.png must have an 8-bit palette";
            err(ErrorKey::ImageFormat).msg(msg).loc(self.entry.as_ref().unwrap()).push();
            return;
        }

        // Early exit before expensive loop, if errors won't be logged anyway
        if !will_maybe_log(self.entry.as_ref().unwrap(), ErrorKey::Rivers) {
            return;
        }

        // Maps each endpoint of a segment to the other endpoint.
        // Single-pixel segments map that coordinate to itself.
        // The river pixels that connect the endpoints are not remembered.
        let mut river_segments: TigerHashMap<(u32, u32), (u32, u32)> = TigerHashMap::default();

        // Maps the coordinates of special pixels (sources, sinks, and splits)
        // to a boolean that says whether the pixel terminates a segment.
        let mut specials = TigerHashMap::default();

        // A working vec, holding the list of river-pixel neighbors of the current pixel.
        // It is declared here to avoid the overhead of creating and destroying the Vec in every
        // iteration.
        let mut river_neighbors = Vec::new();

        let mut bad_problem = false;
        // TODO: multi-thread this
        for x in 0..self.width {
            for y in 0..self.height {
                match self.pixel(x, y) {
                    RiverPixels::SOURCE => {
                        self.river_neighbors(x, y, &mut river_neighbors);
                        if river_neighbors.len() == 1 {
                            specials.insert((x, y), false);
                        } else {
                            let msg =
                                format!("({x}, {y}) river source (green) not at source of a river");
                            warn(ErrorKey::Rivers)
                                .msg(msg)
                                .loc(self.entry.as_ref().unwrap())
                                .push();
                            bad_problem = true;
                        }
                    }
                    RiverPixels::TRIBUTARY => {
                        self.river_neighbors(x, y, &mut river_neighbors);
                        if river_neighbors.len() >= 2 {
                            specials.insert((x, y), false);
                        } else {
                            let msg = format!(
                                "({x}, {y}) river tributary (red) not joining another river",
                            );
                            warn(ErrorKey::Rivers)
                                .msg(msg)
                                .loc(self.entry.as_ref().unwrap())
                                .push();
                            bad_problem = true;
                        }
                    }
                    RiverPixels::SPLIT => {
                        self.river_neighbors(x, y, &mut river_neighbors);
                        if river_neighbors.len() >= 2 {
                            specials.insert((x, y), false);
                        } else {
                            let msg = format!(
                                "({x}, {y}) river split (yellow) not splitting off from a river",
                            );
                            warn(ErrorKey::Rivers)
                                .msg(msg)
                                .loc(self.entry.as_ref().unwrap())
                                .push();
                            bad_problem = true;
                        }
                    }
                    RiverPixels::FIRST_NORMAL..=RiverPixels::LAST_NORMAL => {
                        self.river_neighbors(x, y, &mut river_neighbors);
                        if river_neighbors.len() <= 2 {
                            let mut found = false;
                            for &coords in &river_neighbors {
                                if let Some(&other_end) = river_segments.get(&coords) {
                                    found = true;
                                    if let Some(&third_end) = river_segments.get(&(x, y)) {
                                        // This can only happen if we're on the second iteration.
                                        // It means the pixel borders two segments, and joins them.
                                        // First make sure it's not a single segment in a loop
                                        // though.
                                        if third_end == (x, y) {
                                            let msg = format!("({x}, {y}) river forms a loop");
                                            warn(ErrorKey::Rivers)
                                                .msg(msg)
                                                .loc(self.entry.as_ref().unwrap())
                                                .push();
                                            bad_problem = true;
                                        } else {
                                            river_segments.insert(other_end, third_end);
                                            river_segments.insert(third_end, other_end);
                                            river_segments.remove(&(x, y));
                                            river_segments.remove(&coords);
                                        }
                                    } else {
                                        // Extend the neighboring segment to include this pixel.
                                        river_segments.insert((x, y), other_end);
                                        river_segments.insert(other_end, (x, y));
                                        river_segments.remove(&coords);
                                    }
                                }
                            }
                            if !found {
                                // Start a new single-pixel segment.
                                river_segments.insert((x, y), (x, y));
                            }
                        } else {
                            let msg = format!(
                                "({x}, {y}) river pixel has {} neighbors",
                                river_neighbors.len()
                            );
                            warn(ErrorKey::Rivers)
                                .msg(msg)
                                .loc(self.entry.as_ref().unwrap())
                                .push();
                            bad_problem = true;
                        }
                    }
                    RiverPixels::FIRST_IGNORE.. => (),
                }
            }
        }
        if !bad_problem {
            self.validate_segments(river_segments, specials);
        }
    }
}

impl FileHandler<()> for Rivers {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data/rivers.png")
    }

    fn load_file(&self, _entry: &FileEntry, _parser: &ParserMemory) -> Option<()> {
        Some(())
    }

    fn handle_file(&mut self, entry: &FileEntry, _loaded: ()) {
        self.entry = Some(entry.clone());

        if let Err(e) = self.load_png(entry.fullpath()) {
            err(ErrorKey::ReadError)
                .msg("could not read image")
                .info(format!("{e:#}"))
                .loc(entry)
                .push();
        }
    }
}
