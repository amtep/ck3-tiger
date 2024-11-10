//! Helper functions for loading pdx script files in various character encodings.
//!
//! The main entry point is [`PdxFile`].

#[cfg(feature = "ck3")]
use std::fs::read;
use std::fs::read_to_string;

#[cfg(feature = "ck3")]
use encoding_rs::{UTF_8, WINDOWS_1252};

use crate::block::Block;
use crate::fileset::FileEntry;
use crate::parse::pdxfile::parse_pdx_file;
#[cfg(feature = "ck3")]
use crate::parse::pdxfile::{parse_reader_export, PdxfileMemory};
use crate::parse::ParserMemory;
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
    pub fn read(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        let contents = Self::read_utf8(entry)?;
        if contents.starts_with(BOM_CHAR) {
            Some(parse_pdx_file(entry, contents, BOM_UTF8_LEN, parser))
        } else {
            let msg = "file must start with a UTF-8 BOM";
            warn(ErrorKey::Encoding).msg(msg).loc(entry).push();
            Some(parse_pdx_file(entry, contents, 0, parser))
        }
    }

    /// Parse a UTF-8 file that may optionally start with a BOM (Byte Order Marker).
    pub fn read_optional_bom(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        let contents = Self::read_utf8(entry)?;
        if contents.starts_with(BOM_CHAR) {
            Some(parse_pdx_file(entry, contents, BOM_UTF8_LEN, parser))
        } else {
            Some(parse_pdx_file(entry, contents, 0, parser))
        }
    }

    /// Parse a file that may be in UTF-8 with BOM encoding, or Windows-1252 encoding.
    #[cfg(feature = "ck3")]
    pub fn read_detect_encoding(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
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
                Some(parse_pdx_file(entry, contents.into_owned(), 0, parser))
            }
        } else {
            let (contents, errors) = WINDOWS_1252.decode_without_bom_handling(&bytes);
            if errors {
                let msg = "could not decode WINDOWS-1252 file";
                err(ErrorKey::Encoding).msg(msg).loc(entry).push();
                None
            } else {
                Some(parse_pdx_file(entry, contents.into_owned(), 0, parser))
            }
        }
    }

    pub fn read_encoded(
        entry: &FileEntry,
        encoding: PdxEncoding,
        parser: &ParserMemory,
    ) -> Option<Block> {
        match encoding {
            PdxEncoding::Utf8Bom => Self::read(entry, parser),
            PdxEncoding::Utf8OptionalBom => Self::read_optional_bom(entry, parser),
            #[cfg(feature = "ck3")]
            PdxEncoding::Detect => Self::read_detect_encoding(entry, parser),
        }
    }

    #[cfg(feature = "ck3")]
    pub fn reader_export(entry: &FileEntry, memory: &mut PdxfileMemory) {
        if let Some(contents) = Self::read_utf8(entry) {
            if contents.starts_with(BOM_CHAR) {
                parse_reader_export(entry, contents, BOM_UTF8_LEN, memory);
            } else {
                let msg = "file must start with a UTF-8 BOM";
                warn(ErrorKey::Encoding).msg(msg).loc(entry).push();
                parse_reader_export(entry, contents, 0, memory);
            }
        }
    }
}
