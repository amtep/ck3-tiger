//! Helper functions for loading pdx script files in various character encodings.
//!
//! The main entry point is [`PdxFile`].

#[cfg(feature = "ck3")]
use std::fs::read;
use std::fs::read_to_string;

#[cfg(feature = "ck3")]
use encoding::all::UTF_8;
#[cfg(feature = "ck3")]
use encoding::all::WINDOWS_1252;
#[cfg(feature = "ck3")]
use encoding::{DecoderTrap, Encoding};

use crate::block::Block;
use crate::fileset::FileEntry;
use crate::parse::pdxfile::parse_pdx;
use crate::report::{err, warn, ErrorKey};

#[cfg(feature = "ck3")]
const BOM_AS_BYTES: &[u8] = b"\xef\xbb\xbf";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdxEncoding {
    Utf8Bom,
    Utf8OptionalBom,
    #[cfg(feature = "ck3")]
    Detect,
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
    pub fn read(entry: &FileEntry) -> Option<Block> {
        let contents = Self::read_utf8(entry)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            Some(parse_pdx(entry, bomless))
        } else {
            let msg = "file must start with a UTF-8 BOM";
            warn(ErrorKey::Encoding).msg(msg).loc(entry).push();
            Some(parse_pdx(entry, &contents))
        }
    }

    /// Parse a UTF-8 file that may optionally start with a BOM (Byte Order Marker).
    pub fn read_optional_bom(entry: &FileEntry) -> Option<Block> {
        let contents = Self::read_utf8(entry)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            Some(parse_pdx(entry, bomless))
        } else {
            Some(parse_pdx(entry, &contents))
        }
    }

    /// Parse a file that may be in UTF-8 with BOM encoding, or Windows-1252 encoding.
    #[cfg(feature = "ck3")]
    pub fn read_detect_encoding(entry: &FileEntry) -> Option<Block> {
        let bytes = match read(entry.fullpath()) {
            Ok(bytes) => bytes,
            Err(e) => {
                let msg = "could not read file";
                let info = format!("{e:#}");
                err(ErrorKey::ReadError).msg(msg).info(info).loc(entry).push();
                return None;
            }
        };
        let opt_contents = if bytes.starts_with(BOM_AS_BYTES) {
            match UTF_8.decode(&bytes[3..], DecoderTrap::Strict) {
                Ok(contents) => Some(contents),
                Err(e) => {
                    let msg = "could not decode UTF-8 file";
                    let info = format!("{e:#}");
                    err(ErrorKey::Encoding).msg(msg).info(info).loc(entry).push();
                    None
                }
            }
        } else {
            match WINDOWS_1252.decode(&bytes, DecoderTrap::Strict) {
                Ok(contents) => Some(contents),
                Err(e) => {
                    let msg = "could not decode WINDOWS-1252 file";
                    let info = format!("{e:#}");
                    err(ErrorKey::Encoding).msg(msg).info(info).loc(entry).push();
                    None
                }
            }
        };
        opt_contents.map(|c| parse_pdx(entry, &c))
    }

    pub fn read_encoded(entry: &FileEntry, encoding: PdxEncoding) -> Option<Block> {
        match encoding {
            PdxEncoding::Utf8Bom => Self::read(entry),
            PdxEncoding::Utf8OptionalBom => Self::read_optional_bom(entry),
            #[cfg(feature = "ck3")]
            PdxEncoding::Detect => Self::read_detect_encoding(entry),
        }
    }
}
