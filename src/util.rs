//! Miscellaneous helper functions related to filesystem operations.

use std::borrow::Borrow;
use std::path::{Path, PathBuf};

/// A trait to join a string that may contain `.` or `..` to a path, and process those components
/// for their meaning instead of just appending them.
///
/// This is useful when the resulting path has to be compared for equality with previously stored paths.
///
/// Note that this is exclusively a pathname operation. It does not check the filesystem to see if
/// any of the pathname components are symbolic links.
pub trait SmartJoin {
    fn smart_join(&self, with: &str) -> PathBuf;
    fn smart_join_parent(&self, with: &str) -> PathBuf;
}

impl SmartJoin for Path {
    fn smart_join(&self, with: &str) -> PathBuf {
        let mut result = self.to_path_buf();
        for component in with.split('/') {
            if component == "." {
                continue;
            }
            if component == ".." {
                result.pop();
            } else {
                result.push(component);
            }
        }
        result
    }
    fn smart_join_parent(&self, with: &str) -> PathBuf {
        if let Some(parent) = self.parent() {
            parent.smart_join(with)
        } else {
            self.smart_join(with)
        }
    }
}

/// Redo a path so that all the slashes lean the correct way for the target platform.
/// This is mostly for Windows users, to avoid showing them paths with a mix of slashes.
pub fn fix_slashes_for_target_platform<P: Borrow<Path>>(path: P) -> PathBuf {
    path.borrow().components().collect()
}
