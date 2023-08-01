//! Helper functions for loading pdx script files in various character encodings.
//!
//! The main entry point is [`PdxFile`].

#[cfg(feature = "ck3")]
use std::fs::read;
use std::fs::read_to_string;
use std::path::Path;

#[cfg(feature = "ck3")]
use encoding::all::WINDOWS_1252;
#[cfg(feature = "ck3")]
use encoding::{DecoderTrap, Encoding};

use crate::block::Block;
use crate::fileset::FileEntry;
use crate::parse::pdxfile::parse_pdx;
#[cfg(feature = "ck3")]
use crate::report::advice_info;
use crate::report::{error_info, old_warn, ErrorKey};

/// If a windows-1252 file mistakenly starts with a UTF-8 BOM, this is
/// what it will look like after decoding
#[cfg(feature = "ck3")]
const BOM_FROM_1252: &str = "\u{00ef}\u{00bb}\u{00bf}";

pub struct PdxFile;

impl PdxFile {
    /// Internal function to read a file in UTF-8 encoding.
    fn read_utf8(entry: &FileEntry, fullpath: &Path) -> Option<String> {
        match read_to_string(fullpath) {
            Ok(contents) => Some(contents),
            Err(e) => {
                error_info(entry, ErrorKey::ReadError, "could not read file", &format!("{e:#}"));
                None
            }
        }
    }

    /// Internal function to read a file in Windows-1252 (similar to Latin-1) encoding.
    #[cfg(feature = "ck3")]
    fn read_1252(entry: &FileEntry, fullpath: &Path) -> Option<String> {
        let bytes = match read(fullpath) {
            Ok(bytes) => bytes,
            Err(e) => {
                error_info(entry, ErrorKey::ReadError, "could not read file", &format!("{e:#}"));
                return None;
            }
        };
        WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).ok()
    }

    /// Parse a UTF-8 file that has no BOM (Byte Order Marker).
    pub fn read_no_bom(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_utf8(entry, fullpath)?;
        Some(parse_pdx(entry, &contents))
    }

    /// Parse a UTF-8 file that should start with a BOM (Byte Order Marker).
    pub fn read(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_utf8(entry, fullpath)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            Some(parse_pdx(entry, bomless))
        } else {
            old_warn(entry, ErrorKey::Encoding, "file must start with a UTF-8 BOM");
            Some(parse_pdx(entry, &contents))
        }
    }

    /// Parse a UTF-8 file that may optionally start with a BOM (Byte Order Marker).
    pub fn read_optional_bom(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_utf8(entry, fullpath)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            Some(parse_pdx(entry, bomless))
        } else {
            Some(parse_pdx(entry, &contents))
        }
    }

    /// Parse a file in Windows-1251 encoding (similar to Latin-1).
    /// Warn if it starts with a UTF-8 BOM.
    #[cfg(feature = "ck3")]
    pub fn read_cp1252(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_1252(entry, fullpath)?;

        if let Some(bomless) = contents.strip_prefix(BOM_FROM_1252) {
            advice_info(
                entry,
                ErrorKey::Encoding,
                "file should not start with a UTF-8 BOM",
                "This kind of file is expected to be in Windows-1252 encoding",
            );
            Some(parse_pdx(entry, bomless))
        } else {
            Some(parse_pdx(entry, &contents))
        }
    }
}
