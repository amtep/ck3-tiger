use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::fileset::FileKind;
use crate::pdxfile::PdxFile;
use crate::scope::validator::Validator;
use crate::scope::{Scope, Token};
use crate::validate::{Validate, ValidationError};

#[derive(Clone, Debug)]
pub struct ModFile {
    name: Token,
    path: Option<Token>,
    replace_path: Vec<Token>,
    version: Token,
    tags: Option<Vec<Token>>,
    supported_version: Option<Token>,
    remote_file_id: Option<Token>,
    picture: Option<Token>,
}

impl Validate for ModFile {
    fn from_scope(scope: Scope, id: &str) -> Result<Self, ValidationError> {
        let mut vd = Validator::new(&scope, id);
        // Reference: https://ck3.paradoxwikis.com/Mod_structure#Keys
        let version = vd.require_unique_field_value("version");
        let tags = vd.allow_unique_field_list("tags");
        let name = vd.require_unique_field_value("name");

        let supported_version;
        let path;

        if scope.filename() == "descriptor.mod" {
            supported_version = vd.allow_unique_field_value("supported_version");
            path = vd.allow_unique_field_value("path");
        } else {
            supported_version = Some(vd.require_unique_field_value("supported_version")?);
            path = Some(vd.require_unique_field_value("path")?);
        }

        let remote_file_id = vd.allow_unique_field_value("remote_file_id");
        let picture = vd.allow_unique_field_value("picture");
        let replace_path = vd.allow_field_values("replace_path");
        vd.warn_unused_entries();

        if let Some(err) = vd.err {
            return Err(err);
        }

        let modfile = ModFile {
            name: name?,
            path,
            replace_path,
            version: version?,
            tags,
            supported_version,
            remote_file_id,
            picture,
        };

        if let Some(picture) = &modfile.picture {
            if picture.as_str() != "thumbnail.png" {
                warn(
                    picture,
                    ErrorKey::Packaging,
                    "Steam ignores picture= and always uses thumbnail.png.",
                );
            }
        }

        // TODO: check if supported_version is newer than validator,
        // or is older than known CK3

        Ok(modfile)
    }
}

impl ModFile {
    pub fn read(pathname: &Path) -> Result<Self> {
        let scope = PdxFile::read_no_bom(pathname, FileKind::ModFile, pathname)
            .with_context(|| format!("Could not read .mod file {}", pathname.display()))?;
        let modfile = ModFile::from_scope(scope, "Modfile")?;

        Ok(modfile)
    }

    pub fn modpath(&self) -> PathBuf {
        let mut dirpath = self
            .name
            .loc
            .pathname
            .parent()
            .unwrap_or_else(|| Path::new("."));
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
}
