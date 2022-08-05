use anyhow::Result;
use std::fs::read_to_string;
use std::path::Path;

use crate::scope::Scope;
use crate::pdxfile::parse::parse_pdx;

mod parse;

pub struct PdxFile;

impl PdxFile {
    pub fn read(pathname: &Path) -> Result<Scope> {
        let contents = read_to_string(pathname)?;
        Ok(parse_pdx(pathname, &contents))
    }
}
