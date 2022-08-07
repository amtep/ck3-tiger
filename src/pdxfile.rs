use anyhow::Result;
use std::fs::read_to_string;
use std::path::Path;

use crate::errors::Errors;
use crate::pdxfile::parse::parse_pdx;
use crate::scope::Scope;

mod parse;

pub struct PdxFile;

impl PdxFile {
    pub fn read(pathname: &Path, errors: &mut Errors) -> Result<Scope> {
        let contents = read_to_string(pathname)?;
        parse_pdx(pathname, &contents, errors)
    }
}
