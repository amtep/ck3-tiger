//! Helper functions for loading pdx script files in various character encodings.
//!
//! The main entry point is [`PdxFile`].

#[cfg(feature = "ck3")]
use std::fs::read;
use std::fs::read_to_string;
use std::mem::ManuallyDrop;

#[cfg(feature = "ck3")]
use encoding_rs::{UTF_8, WINDOWS_1252};

use crate::block::{Block, Serializer};
use crate::capnp::fileheader_capnp::ParserType;
use crate::fileset::FileEntry;
use crate::game::Game;
use crate::parse::cache::{cache_lookup, cache_put};
use crate::parse::pdxfile::parse_pdx_file;
use crate::report::{err, warn, ErrorKey};

const BOM_UTF8_BYTES: &[u8] = b"\xef\xbb\xbf";
const BOM_UTF8_LEN: usize = BOM_UTF8_BYTES.len();
const BOM_CHAR: char = '\u{feff}';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdxEncoding {
    Utf8Bom,
    Utf8OptionalBom,
    #[cfg(feature = "ck3")]
    Detect,
}

/// Return a modified `String` that does not have the leading UTF-8 BOM.
/// `contents` *must* start with the BOM.
/// The returned `String` *must not* ever be deallocated.
fn strip_bom(contents: String) -> String {
    // leak so does not deallocate memory
    let contents = ManuallyDrop::new(contents);
    let ptr = contents.as_ptr();
    unsafe {
        // Re-using the input `String` is faster than allocating a new version.
        // The tradeoff is that it wastes the three starting bytes and any excess capacity in `contents`, but that is acceptable.
        String::from_raw_parts(
            ptr.cast_mut().add(BOM_UTF8_LEN),
            contents.len() - BOM_UTF8_LEN,
            contents.len() - BOM_UTF8_LEN,
        )
    }
}

pub struct PdxFile {}

impl PdxFile {
    /// Internal function to read a file in UTF-8 encoding.
    fn read_utf8(entry: &FileEntry) -> Option<String> {
        match read_to_string(entry.fullpath()) {
            Ok(contents) => Some(contents),
            Err(e) => {
                let msg = "could not read file";
                let info = &format!("{e:#}");
                err(ErrorKey::ReadError).msg(msg).info(info).loc(entry).push();
                None
            }
        }
    }

    /// Parse a UTF-8 file that should start with a BOM (Byte Order Marker).
    fn _read_mandatory_bom(entry: &FileEntry) -> Option<(Block, bool)> {
        let contents = Self::read_utf8(entry)?;
        if contents.starts_with(BOM_CHAR) {
            Some(parse_pdx_file(entry, strip_bom(contents)))
        } else {
            let msg = "file must start with a UTF-8 BOM";
            warn(ErrorKey::Encoding).msg(msg).loc(entry).push();
            let (block, _can_cache) = parse_pdx_file(entry, contents);
            Some((block, false))
        }
    }

    /// Parse a UTF-8 file that may optionally start with a BOM (Byte Order Marker).
    fn _read_optional_bom(entry: &FileEntry) -> Option<(Block, bool)> {
        let contents = Self::read_utf8(entry)?;
        if contents.starts_with(BOM_CHAR) {
            Some(parse_pdx_file(entry, strip_bom(contents)))
        } else {
            Some(parse_pdx_file(entry, contents))
        }
    }

    /// Parse a file that may be in UTF-8 with BOM encoding, or Windows-1252 encoding.
    #[cfg(feature = "ck3")]
    fn _read_detect_encoding(entry: &FileEntry) -> Option<(Block, bool)> {
        let bytes = match read(entry.fullpath()) {
            Ok(bytes) => bytes,
            Err(e) => {
                let msg = "could not read file";
                let info = format!("{e:#}");
                err(ErrorKey::ReadError).msg(msg).info(info).loc(entry).push();
                return None;
            }
        };
        if bytes.starts_with(BOM_UTF8_BYTES) {
            let (contents, errors) = UTF_8.decode_without_bom_handling(&bytes[BOM_UTF8_LEN..]);
            if errors {
                let msg = "could not decode UTF-8 file";
                err(ErrorKey::Encoding).msg(msg).loc(entry).push();
                None
            } else {
                Some(parse_pdx_file(entry, contents.into_owned()))
            }
        } else {
            let (contents, errors) = WINDOWS_1252.decode_without_bom_handling(&bytes);
            if errors {
                let msg = "could not decode WINDOWS-1252 file";
                err(ErrorKey::Encoding).msg(msg).loc(entry).push();
                None
            } else {
                Some(parse_pdx_file(entry, contents.into_owned()))
            }
        }
    }

    pub fn read_encoded(entry: &FileEntry, encoding: PdxEncoding) -> Option<Block> {
        if let Some(block) = PdxFile::cache_lookup(entry) {
            return Some(block);
        }

        let parse_result = match encoding {
            PdxEncoding::Utf8Bom => Self::_read_mandatory_bom(entry),
            PdxEncoding::Utf8OptionalBom => Self::_read_optional_bom(entry),
            #[cfg(feature = "ck3")]
            PdxEncoding::Detect => Self::_read_detect_encoding(entry),
        };

        if let Some((block, can_cache)) = parse_result {
            if can_cache {
                PdxFile::cache_put(entry, &block);
            }
            return Some(block);
        }

        None
    }

    /// Convenience function for `Utf8Bom`
    pub fn read(entry: &FileEntry) -> Option<Block> {
        Self::read_encoded(entry, PdxEncoding::Utf8Bom)
    }

    /// Convenience function for `Utf8OptionalBom`
    pub fn read_optional_bom(entry: &FileEntry) -> Option<Block> {
        Self::read_encoded(entry, PdxEncoding::Utf8OptionalBom)
    }

    /// Convenience function for detect encoding
    #[cfg(feature = "ck3")]
    pub fn read_detect_encoding(entry: &FileEntry) -> Option<Block> {
        Self::read_encoded(entry, PdxEncoding::Detect)
    }

    fn cache_lookup(entry: &FileEntry) -> Option<Block> {
        // TODO
        _ = cache_lookup(entry, ParserType::pdx_from_game(), 1);
        None
    }

    fn cache_put(entry: &FileEntry, block: &Block) {
        let mut s = Serializer::new();
        cache_put(entry, ParserType::pdx_from_game(), 1, &s.serialize(block));
    }
}

impl ParserType {
    pub fn pdx_from_game() -> Self {
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => ParserType::PdxCk3,
            #[cfg(feature = "vic3")]
            Game::Vic3 => ParserType::PdxVic3,
            #[cfg(feature = "imperator")]
            Game::Imperator => ParserType::Imperator,
        }
    }
}
