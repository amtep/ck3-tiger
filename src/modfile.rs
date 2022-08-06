use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::errors::Errors;
use crate::pdxfile::PdxFile;
use crate::scope::Scope;

#[derive(Clone, Debug)]
pub struct ModFile {
    pathname: PathBuf,
    scope: Scope,
}

impl ModFile {
    pub fn read(pathname: &Path, errors: &mut Errors) -> Result<Self> {
        let scope = PdxFile::read(pathname, errors)
            .with_context(|| format!("Could not read .mod file {}", pathname.display()))?;
        let modfile = ModFile {
            pathname: pathname.to_path_buf(),
            scope,
        };

        // TODO: verify fields

        Ok(modfile)
    }
}
