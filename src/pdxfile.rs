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
    pub fn read(
        pathname: &Path,
        kind: FileKind,
        fullpath: &Path,
        check_bom: bool,
    ) -> Result<Scope> {
        let contents = read_to_string(fullpath)?;
        if check_bom {
            if let Some(contents) = contents.strip_prefix('\u{feff}') {
                return parse_pdx(pathname, kind, contents);
            } else {
                warn(
                    &Token::from(Loc::for_file(Rc::new(pathname.to_path_buf()), kind)),
                    ErrorKey::Bom,
                    "file must start with a UTF-8 BOM",
                );
            }
        }
        parse_pdx(pathname, kind, &contents)
    }
}
