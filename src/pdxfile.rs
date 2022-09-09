use anyhow::Result;
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use std::fs::{read, read_to_string};
use std::path::Path;

use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, warn};
use crate::fileset::FileKind;
use crate::parse::pdxfile::parse_pdx;

/// If a windows-1252 file mistakenly starts with a UTF-8 BOM, this is
/// what it will look like after decoding
const BOM_FROM_1252: &str = "\u{00ef}\u{00bb}\u{00bf}";

pub struct PdxFile;

impl PdxFile {
    pub fn read_no_bom(pathname: &Path, kind: FileKind, fullpath: &Path) -> Result<Block> {
        let contents = read_to_string(fullpath)?;
        parse_pdx(pathname, kind, &contents)
    }

    pub fn read(pathname: &Path, kind: FileKind, fullpath: &Path) -> Result<Block> {
        let contents = read_to_string(fullpath)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            parse_pdx(pathname, kind, bomless)
        } else {
            warn(
                (pathname, kind),
                ErrorKey::Encoding,
                "file must start with a UTF-8 BOM",
            );
            parse_pdx(pathname, kind, &contents)
        }
    }

    pub fn read_cp1252(pathname: &Path, kind: FileKind, fullpath: &Path) -> Result<Block> {
        let bytes = read(fullpath)?;
        let contents = WINDOWS_1252
            .decode(&bytes, DecoderTrap::Strict)
            .map_err(anyhow::Error::msg)?;

        if let Some(bomless) = contents.strip_prefix(BOM_FROM_1252) {
            advice_info(
                (pathname, kind),
                ErrorKey::Encoding,
                "file should not start with a UTF-8 BOM",
                "This kind of file is expected to be in Windows-1252 encoding",
            );
            parse_pdx(pathname, kind, bomless)
        } else {
            parse_pdx(pathname, kind, &contents)
        }
    }
}
