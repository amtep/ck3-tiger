//! Special validator for the `rivers.png` file.
//!
//! The `rivers.png` file has detailed requirements for its image format and the layout of every pixel.

use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use fnv::FnvHashMap;
use png::{ColorType, Decoder};

use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::report::{error, error_info, will_maybe_log, ErrorKey};

#[derive(Clone, Debug, Default)]
pub struct Rivers {
    /// for error reporting
    entry: Option<FileEntry>,
    width: u32,
    height: u32,
    color_type: Option<ColorType>,
    palette: Option<Vec<u8>>,
    rivers_buf: Vec<u8>,
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

        self.rivers_buf = vec![0; reader.output_buffer_size()];
        let frame_info = reader.next_frame(&mut self.rivers_buf)?;

        if frame_info.width != self.width
            || frame_info.height != self.height
            || frame_info.color_type != self.color_type.unwrap()
        {
            bail!("PNG frame did not match image info");
        }

        Ok(())
    }

    fn river_neighbors(&self, x: u32, y: u32) -> usize {
        let mut n = 0;
        if x > 0 && (3..=11).contains(&self.pixel(x - 1, y)) {
            n += 1;
        }
        if y > 0 && (3..=11).contains(&self.pixel(x, y - 1)) {
            n += 1;
        }
        if x + 1 < self.width && (3..=11).contains(&self.pixel(x + 1, y)) {
            n += 1;
        }
        if y + 1 < self.height && (3..=11).contains(&self.pixel(x, y + 1)) {
            n += 1;
        }
        n
    }

    fn special_neighbors(&self, c: (u32, u32)) -> Vec<(u32, u32)> {
        let (x, y) = c;
        let mut vec = Vec::new();
        if x > 0 && self.pixel(x - 1, y) <= 2 {
            vec.push((x - 1, y));
        }
        if y > 0 && self.pixel(x, y - 1) <= 2 {
            vec.push((x, y - 1));
        }
        if x + 1 < self.width && self.pixel(x + 1, y) <= 2 {
            vec.push((x + 1, y));
        }
        if y + 1 < self.height && self.pixel(x, y + 1) <= 2 {
            vec.push((x, y + 1));
        }
        vec
    }

    fn pixel(&self, x: u32, y: u32) -> u8 {
        let idx = (x + self.width * y) as usize;
        self.rivers_buf[idx]
    }

    fn validate_segments(
        &self,
        river_segments: Vec<RiverSegment>,
        mut specials: FnvHashMap<(u32, u32), bool>,
    ) {
        for segment in river_segments {
            match segment {
                RiverSegment::Single(c) => {
                    let special_neighbors = self.special_neighbors(c);
                    if special_neighbors.len() > 1 {
                        let msg =
                            format!("({}, {}) river pixel connects two special pixels", c.0, c.1);
                        error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                    } else if special_neighbors.is_empty() {
                        let msg = format!("({}, {}) orphan river pixel", c.0, c.1);
                        error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                    } else {
                        let s = special_neighbors[0];
                        if specials[&s] {
                            let msg = format!(
                                "({}, {}) pixel terminates multiple river segments",
                                s.0, s.1
                            );
                            error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                        } else {
                            specials.insert(s, true);
                        }
                    }
                }
                RiverSegment::Stream(c1, c2) => {
                    let mut special_neighbors = self.special_neighbors(c1);
                    special_neighbors.append(&mut self.special_neighbors(c2));
                    if special_neighbors.is_empty() {
                        let msg = format!(
                            "({}, {}) - ({}, {}) orphan river segment",
                            c1.0, c1.1, c2.0, c2.1
                        );
                        error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                    } else if special_neighbors.len() > 1 {
                        let msg = format!(
                            "({}, {}) - ({}, {}) river segment has two terminators",
                            c1.0, c1.1, c2.0, c2.1
                        );
                        error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                    } else {
                        let s = special_neighbors[0];
                        if specials[&s] {
                            let msg = format!(
                                "({}, {}) pixel terminates multiple river segments",
                                s.0, s.1
                            );
                            error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                        } else {
                            specials.insert(s, true);
                        }
                    }
                }
            }
        }
    }

    pub fn validate(&self, _data: &Everything) {
        // TODO: check image width and height against world defines

        if self.color_type != Some(ColorType::Indexed) {
            error(
                self.entry.as_ref().unwrap(),
                ErrorKey::ImageFormat,
                "rivers.png should be in indexed color format (with 8-bit palette)",
            );
            return;
        }

        if self.palette.is_none() {
            error(
                self.entry.as_ref().unwrap(),
                ErrorKey::ImageFormat,
                "rivers.png must have an 8-bit palette",
            );
            return;
        }

        // Early exit before expensive loop, if errors won't be logged anyway
        if !will_maybe_log(self.entry.as_ref().unwrap(), ErrorKey::Rivers) {
            return;
        }

        let mut river_segments = Vec::new();
        let mut specials = FnvHashMap::default();

        let mut bad_problem = false;
        // TODO: multi-thread this
        for x in 0..self.width {
            for y in 0..self.height {
                let river_neighbors = self.river_neighbors(x, y);
                match self.pixel(x, y) {
                    0 => {
                        if river_neighbors == 1 {
                            specials.insert((x, y), false);
                        } else {
                            let msg =
                                format!("({x}, {y}) river source (green) not at source of a river");
                            error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                            bad_problem = true;
                        }
                    }
                    1 => {
                        if river_neighbors >= 2 {
                            specials.insert((x, y), false);
                        } else {
                            let msg = format!(
                                "({x}, {y}) river tributary (red) not joining another river",
                            );
                            error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                            bad_problem = true;
                        }
                    }
                    2 => {
                        if river_neighbors >= 2 {
                            specials.insert((x, y), false);
                        } else {
                            let msg = format!(
                                "({x}, {y}) river split (yellow) not splitting off from a river",
                            );
                            error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                            bad_problem = true;
                        }
                    }
                    3..=15 => {
                        if river_neighbors <= 2 {
                            let mut found = Vec::new();
                            for (i, segment) in river_segments.iter_mut().enumerate() {
                                match *segment {
                                    RiverSegment::Single(coord) => {
                                        if are_neighbors(coord, (x, y)) {
                                            *segment = RiverSegment::Stream(coord, (x, y));
                                            found.push(i);
                                        }
                                    }
                                    RiverSegment::Stream(c1, c2) => {
                                        if are_neighbors(c1, (x, y)) && are_neighbors(c2, (x, y)) {
                                            let msg = format!("({x}, {y}) river forms a loop");
                                            error(
                                                self.entry.as_ref().unwrap(),
                                                ErrorKey::Rivers,
                                                &msg,
                                            );
                                            bad_problem = true;
                                        } else if are_neighbors(c1, (x, y)) {
                                            *segment = RiverSegment::Stream((x, y), c2);
                                            found.push(i);
                                        } else if are_neighbors(c2, (x, y)) {
                                            *segment = RiverSegment::Stream(c1, (x, y));
                                            found.push(i);
                                        }
                                    }
                                }
                            }
                            if found.is_empty() {
                                river_segments.push(RiverSegment::Single((x, y)));
                            } else if found.len() == 2 {
                                let new_segment =
                                    river_segments[found[0]].combine(&river_segments[found[1]]);
                                river_segments[found[0]] = new_segment;
                                river_segments.swap_remove(found[1]);
                            }
                        } else {
                            let msg =
                                format!("({x}, {y}) river pixel has {river_neighbors} neighbors",);
                            error(self.entry.as_ref().unwrap(), ErrorKey::Rivers, &msg);
                            bad_problem = true;
                        }
                    }
                    16.. => (),
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

    fn load_file(&self, _entry: &FileEntry) -> Option<()> {
        Some(())
    }

    fn handle_file(&mut self, entry: &FileEntry, _loaded: ()) {
        self.entry = Some(entry.clone());

        if let Err(e) = self.load_png(entry.fullpath()) {
            error_info(entry, ErrorKey::ReadError, "could not read image", &format!("{e:#}"));
        }
    }
}

// A river segment is either a single pixel or a set of pixels between 2 endpoints.
// Endpoints have 1 neighbor, all pixels in between have 2 neighbors.
// There's no need to keep track of the pixels between the endpoints.
#[derive(Copy, Clone, Debug)]
enum RiverSegment {
    Single((u32, u32)),
    Stream((u32, u32), (u32, u32)),
}

impl RiverSegment {
    fn combine(&self, other: &Self) -> Self {
        // We'll never be asked to combine singles
        if let RiverSegment::Stream(c1, c2) = self {
            if let RiverSegment::Stream(o1, o2) = other {
                if c1 == o1 {
                    return RiverSegment::Stream(*c2, *o2);
                } else if c1 == o2 {
                    return RiverSegment::Stream(*c2, *o1);
                } else if c2 == o1 {
                    return RiverSegment::Stream(*c1, *o2);
                } else if c2 == o2 {
                    return RiverSegment::Stream(*c1, *o1);
                }
                panic!("asked to join non-adjacent river segments");
            }
        }
        panic!("asked to join single-pixel river segments");
    }
}

fn are_neighbors(c1: (u32, u32), c2: (u32, u32)) -> bool {
    let (x1, y1) = c1;
    let (x2, y2) = c2;
    (y1 == y2 && (x1 == x2 + 1 || x2 == x1 + 1)) || (x1 == x2 && (y1 == y2 + 1 || y2 == y1 + 1))
}
