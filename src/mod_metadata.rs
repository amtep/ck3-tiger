//! Loader and validator for the `.metadata/metadata.json` files used by Vic3

use std::path::{Path, PathBuf};
use std::string::ToString;

use anyhow::{Context, Result};

use crate::block::Block;
use crate::fileset::{FileEntry, FileKind};
use crate::parse::json::parse_json_file;
use crate::util::fix_slashes_for_target_platform;

/// Representation of a `metadata.json` file and its contents.
#[derive(Clone, Debug)]
pub struct ModMetadata {
    /// Path to the mod itself
    modpath: PathBuf,
    /// Parsed version of the json file
    block: Block,
}

impl ModMetadata {
    /// Read and parse the metadata file for the given mod dir
    pub fn read(mod_dir: &Path) -> Result<Self> {
        let in_mod_path = PathBuf::from(".metadata/metadata.json");
        let pathname = fix_slashes_for_target_platform(mod_dir.join(&in_mod_path));
        let entry = FileEntry::new(in_mod_path, FileKind::Mod, pathname.clone());
        let block = parse_json_file(&entry)
            .with_context(|| format!("Could not read metadata file {}", pathname.display()))?;
        Ok(Self { modpath: mod_dir.to_path_buf(), block })
    }

    /// Return the full path to the mod root.
    pub fn modpath(&self) -> PathBuf {
        self.modpath.clone()
    }

    /// Return the paths that this mod fully replaces
    pub fn replace_paths(&self) -> Vec<PathBuf> {
        if let Some(custom_data) = self.block.get_field_block("game_custom_data") {
            if let Some(replace_paths) = custom_data.get_field_list("replace_paths") {
                return replace_paths.iter().map(|t| PathBuf::from(t.as_str())).collect();
            }
        }
        Vec::new()
    }

    /// The mod's name in human-friendly form, if available.
    pub fn display_name(&self) -> Option<String> {
        self.block.get_field_value("name").map(ToString::to_string)
    }
}
