use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

use crate::pdxfile::PdxFile;
use crate::scope::Scope;

#[derive(Clone, Debug)]
pub struct ModFile {
    pathname: PathBuf,
    scope: Scope,
}

impl ModFile {
    pub fn read(pathname: &Path) -> Result<Self> {
        let modfile = ModFile {
            pathname: pathname.to_path_buf(),
            scope: PdxFile::read(pathname)
                .with_context(|| format!("Could not read .mod file {}", pathname.display()))?
        };

        // TODO: verify fields

        Ok(modfile)
    }
}
