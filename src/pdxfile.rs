use anyhow::Result;
use std::fs::read_to_string;
use std::path::Path;
use std::rc::Rc;

use crate::errors::{warn, ErrorKey};
use crate::everything::FileKind;
use crate::pdxfile::parse::parse_pdx;
use crate::scope::{Loc, Scope, Token};

mod parse;

pub struct PdxFile;

impl PdxFile {
    pub fn read_no_bom(pathname: &Path, kind: FileKind, fullpath: &Path) -> Result<Scope> {
        let contents = read_to_string(fullpath)?;
        parse_pdx(pathname, kind, &contents)
    }

    pub fn read(pathname: &Path, kind: FileKind, fullpath: &Path) -> Result<Scope> {
        let contents = read_to_string(fullpath)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            parse_pdx(pathname, kind, bomless)
        } else {
            warn(
                &Token::from(Loc::for_file(Rc::new(pathname.to_path_buf()), kind)),
                ErrorKey::Bom,
                "file must start with a UTF-8 BOM",
            );
            parse_pdx(pathname, kind, &contents)
        }
    }
}
