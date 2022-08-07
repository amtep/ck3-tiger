use anyhow::{Context, Result};
use std::path::Path;

use crate::errors::Errors;
use crate::pdxfile::PdxFile;
use crate::scope::Scope;
use crate::verify::Verify;

#[derive(Clone, Debug, Default, Verify)]
pub struct ModFile {
    scope: Scope,
    name: String,
    path: String,
    replace_path: Vec<String>,
    version: String,
    tags: Option<Vec<String>>,
    supported_version: Option<String>,
    remote_file_id: Option<String>,
    picture: Option<String>,
}

impl ModFile {
    pub fn read(pathname: &Path, errors: &mut Errors) -> Result<Self> {
        let scope = PdxFile::read(pathname, errors)
            .with_context(|| format!("Could not read .mod file {}", pathname.display()))?;

        let modfile = ModFile::from_scope(scope, errors);

        Ok(modfile)
    }
}
