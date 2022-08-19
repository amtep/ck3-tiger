use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::errors::{warn, ErrorKey};
use crate::everything::FileKind;
use crate::pdxfile::PdxFile;
use crate::scope::validator::Validator;
use crate::scope::{Scope, Token};
use crate::validate::{Validate, ValidationError};

#[derive(Clone, Debug)]
pub struct ModFile {
    name: Token,
    path: Token,
    replace_path: Vec<Token>,
    version: Token,
    tags: Option<Vec<Token>>,
    supported_version: Option<Token>,
    remote_file_id: Option<Token>,
    picture: Option<Token>,
}

impl Validate for ModFile {
    fn from_scope(scope: Scope) -> Result<Self, ValidationError> {
        let mut vd = Validator::new(&scope, "Modfile");
        vd.error_limit(3, "Are you sure this is a modfile?");
        // Reference: https://ck3.paradoxwikis.com/Mod_structure#Keys
        vd.require_unique_field_value("version");
        vd.allow_unique_field_list("tags");
        vd.require_unique_field_value("name");

        if scope.filename() == "descriptor.mod" {
            vd.allow_unique_field_value("supported_version");
            vd.allow_unique_field_value("path");
        } else {
            vd.require_unique_field_value("supported_version");
            vd.require_unique_field_value("path");
        }

        vd.allow_unique_field_value("remote_file_id");
        vd.allow_unique_field_value("picture");
        vd.allow_multiple_field_values("replace_path");
        vd.warn_unused_entries();

        if let Some(err) = vd.err {
            return Err(err);
        }

        let modfile = ModFile {
            name: scope.get_field_value("name").unwrap(),
            path: scope.get_field_value("path").unwrap(),
            replace_path: scope.get_field_values("replace_path"),
            version: scope.get_field_value("version").unwrap(),
            tags: scope.get_field_list("tags"),
            supported_version: scope.get_field_value("supported_version"),
            remote_file_id: scope.get_field_value("remote_file_id"),
            picture: scope.get_field_value("picture"),
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
        let modfile = ModFile::from_scope(scope)?;

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
        let modpath = dirpath.join(self.path.as_str());
        if !modpath.exists() && self.name.loc.filename() == "descriptor.mod" {
            dirpath.to_path_buf()
        } else {
            modpath
        }
    }
}
