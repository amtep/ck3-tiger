//! Loader and validator for the `.mod` files themselves.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::string::ToString;

use anyhow::{Context, Result};

use crate::block::Block;
use crate::fileset::{FileEntry, FileKind};
use crate::pdxfile::PdxFile;
use crate::report::{untidy, warn, ErrorKey};
use crate::token::Token;

/// Representation of a `.mod` file and its contents.
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

/// Validate the [`Block`] form of a `.mod` file and return it as a [`ModFile`].
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
            let msg = "Steam ignores picture= and always uses thumbnail.png.";
            warn(ErrorKey::Packaging).msg(msg).loc(picture).push();
        }
    }

    for path in &modfile.replace_paths {
        if path.is("history") {
            let msg =
                "replace_path only replaces the specific directory, not any directories below it";
            let info =
                "So replace_path = history is not useful, you should replace the paths under it.";
            untidy(ErrorKey::Unneeded).msg(msg).info(info).loc(path).push();
        }
    }

    // TODO: check if supported_version is newer than validator,
    // or is older than known game version.

    modfile
}

impl ModFile {
    /// Take the path to a `.mod` file, validate it, and return its parsed structure.
    pub fn read(pathname: &Path) -> Result<Self> {
        let entry = FileEntry::new(pathname.to_path_buf(), FileKind::Mod);
        let block = PdxFile::read_optional_bom(&entry, pathname)
            .with_context(|| format!("Could not read .mod file {}", pathname.display()))?;
        Ok(validate_modfile(&block))
    }

    /// Return the full path to the mod root.
    #[allow(clippy::missing_panics_doc)] // the panic can't happen
    pub fn modpath(&self) -> PathBuf {
        let modpathname = self.block.loc.pathname();

        // Get the path of the directory the modfile is in.
        let mut dirpath = modpathname.parent().unwrap_or_else(|| Path::new("."));
        if dirpath.components().count() == 0 {
            dirpath = Path::new(".");
        }

        // descriptor.mod is always in the mod's root and does not contain a path field.
        if modpathname.file_name() == Some(OsStr::new("descriptor.mod")) {
            return dirpath.to_path_buf();
        }

        // If the modfile is in a directory called "mod", assume that that's the paradox mod dir and the
        // modpath will be relative to the paradox game dir above it.
        if dirpath.ends_with("mod") {
            // unwrap is safe here because we just checked that dirpath contains a component to strip.
            dirpath = dirpath.parent().unwrap();
        }

        let modpath = if let Some(path) = &self.path {
            dirpath.join(path.as_str())
        } else {
            eprintln!("No mod path found in modfile {}", modpathname.display());
            dirpath.to_path_buf()
        };

        if modpath.exists() {
            modpath
        } else {
            eprintln!("Deduced mod path not found: {}", modpath.display());
            dirpath.to_path_buf()
        }
    }

    /// Return the paths that this mod fully replaces, according to its `.mod` file.
    pub fn replace_paths(&self) -> Vec<PathBuf> {
        self.replace_paths.iter().map(|t| PathBuf::from(t.as_str())).collect()
    }

    /// The mod's name in human-friendly form, if available.
    pub fn display_name(&self) -> Option<String> {
        self.name.as_ref().map(ToString::to_string)
    }
}
