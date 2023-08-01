//! Loader and validator for the `.mod` files themselves.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::block::Block;
use crate::fileset::{FileEntry, FileKind};
use crate::pdxfile::PdxFile;
use crate::report::{advice_info, old_warn, ErrorKey};
use crate::token::Token;

#[allow(dead_code)] // TODO, see below
#[derive(Clone, Debug)]
pub struct ModFile {
    block: Block,
    name: Option<Token>,
    path: Option<Token>,
    replace_paths: Vec<Token>,
    version: Option<Token>,
    // TODO: check that these are tags accepted by steam ?
    tags: Option<Vec<Token>>,
    // TODO: check if the version is compatible with the validator.
    // (Newer means the validator is too old, older means it's not up to date
    // with current CK3)
    supported_version: Option<Token>,
    picture: Option<Token>,
}

fn validate_modfile(block: &Block) -> ModFile {
    let modfile = ModFile {
        block: block.clone(),
        name: block.get_field_value("name").cloned(),
        path: block.get_field_value("path").cloned(),
        replace_paths: block.get_field_values("replace_path").into_iter().cloned().collect(),
        version: block.get_field_value("version").cloned(),
        tags: block.get_field_list("tags"),
        supported_version: block.get_field_value("supported_version").cloned(),
        picture: block.get_field_value("picture").cloned(),
    };

    if let Some(picture) = &modfile.picture {
        if !picture.is("thumbnail.png") {
            old_warn(
                picture,
                ErrorKey::Packaging,
                "Steam ignores picture= and always uses thumbnail.png.",
            );
        }
    }

    for path in &modfile.replace_paths {
        if path.is("history") {
            advice_info(
                path,
                ErrorKey::Unneeded,
                "replace_path only replaces the specific directory, not any directories below it",
                "So replace_path = history is not useful, you should replace the paths under it.",
            );
        }
    }

    // TODO: check if supported_version is newer than validator,
    // or is older than known CK3

    modfile
}

impl ModFile {
    pub fn read(pathname: &Path) -> Result<Self> {
        let entry = FileEntry::new(pathname.to_path_buf(), FileKind::Mod);
        let block = PdxFile::read_optional_bom(&entry, pathname)
            .with_context(|| format!("Could not read .mod file {}", pathname.display()))?;
        Ok(validate_modfile(&block))
    }

    pub fn modpath(&self) -> PathBuf {
        let mut dirpath = self.block.loc.pathname().parent().unwrap_or_else(|| Path::new("."));
        if dirpath.components().count() == 0 {
            dirpath = Path::new(".");
        }

        let modpath = if let Some(path) = &self.path {
            dirpath.join(path.as_str())
        } else {
            dirpath.to_path_buf()
        };

        if modpath.exists() {
            modpath
        } else {
            dirpath.to_path_buf()
        }
    }

    pub fn replace_paths(&self) -> Vec<PathBuf> {
        self.replace_paths.iter().map(|t| PathBuf::from(t.as_str())).collect()
    }

    pub fn display_name_ext(&self) -> String {
        if let Some(name) = &self.name {
            format!(" \"{name}\"")
        } else {
            String::new()
        }
    }
}
