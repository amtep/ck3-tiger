use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum FilesError {
    #[error("Could not read CK3 game files at {ck3path}")]
    VanillaUnreadable {
        ck3path: PathBuf,
        source: walkdir::Error,
    },
    #[error("Could not read mod files at {modpath}")]
    ModUnreadable {
        modpath: PathBuf,
        source: walkdir::Error,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileKind {
    VanillaFile,
    ModFile,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileName {
    /// Pathname components below the mod directory or the vanilla game dir
    path: Vec<String>,
    /// Whether it's a vanilla or mod file
    kind: FileKind,
}

#[derive(Clone, Debug)]
pub struct Everything {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,

    /// The CK3 and mod files in the order the game would load them
    ordered_files: Vec<FileName>,
}

impl Display for FileName {
    #[cfg(target_os = "windows")]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.path.join("\\"))
    }

    #[cfg(not(target_os = "windows"))]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.path.join("/"))
    }
}

impl Everything {
    pub fn new(vanilla_root: PathBuf, mod_root: PathBuf) -> Result<Self, FilesError> {
        let mut files = Vec::new();

        // Abort if whole directories are unreadable, because then we don't have
        // a full map of vanilla's or the mod's contents and might give bad advice.
        Everything::_scan(&vanilla_root, FileKind::VanillaFile, &mut files).map_err(|e| {
            FilesError::VanillaUnreadable {
                ck3path: vanilla_root.clone(),
                source: e,
            }
        })?;
        Everything::_scan(&mod_root, FileKind::ModFile, &mut files).map_err(|e| {
            FilesError::ModUnreadable {
                modpath: mod_root.clone(),
                source: e,
            }
        })?;
        files.sort();

        Ok(Everything {
            vanilla_root,
            mod_root,
            ordered_files: files,
        })
    }

    fn _scan(
        path: &PathBuf,
        kind: FileKind,
        files: &mut Vec<FileName>,
    ) -> Result<(), walkdir::Error> {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.depth() == 0 || !entry.file_type().is_file() {
                continue;
            }
            // unwrap is safe here because WalkDir gives us paths with this prefix.
            let inner_path = entry.path().strip_prefix(path).unwrap();
            let fname = match inner_path
                .components()
                .map(|c| c.as_os_str().to_str().map(str::to_string))
                .collect()
            {
                Some(path) => FileName { path, kind },
                None => {
                    eprintln!("found problem file: {}", inner_path.display());
                    eprintln!("Validator only works on unicode filenames.");
                    continue;
                }
            };
            files.push(fname);
        }
        Ok(())
    }
}
