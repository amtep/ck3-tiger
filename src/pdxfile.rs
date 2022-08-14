use anyhow::Result;
use std::fs::read_to_string;
use std::path::Path;

use crate::pdxfile::parse::parse_pdx;
use crate::scope::Scope;

mod parse;

pub struct PdxFile;

impl PdxFile {
    pub fn read(pathname: &Path) -> Result<Scope> {
        let contents = read_to_string(pathname)?;
        parse_pdx(pathname, &contents)
    }
}
