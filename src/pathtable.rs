//! A global table for the pathnames used in `FileEntry` and `Loc`.
//!
//! Using this will make the often-cloned Loc faster to copy, since it will just contain an index into the global table.
//! It also makes it faster to compare pathnames, because the table will be created in lexical order by the caller
//! ([`Fileset`](crate::fileset::Fileset)), with the exception of some stray files (such as the config file)
//! where the order doesn't matter.
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathTableIndex(u32);

static PATHTABLE: LazyLock<RwLock<PathTable>> = LazyLock::new(|| RwLock::new(PathTable::default()));

/// A global table for the pathnames used in `FileEntry` and `Loc`.
///
/// See the [`self`](module-level documentation) for details.
#[derive(Debug, Default)]
pub struct PathTable {
    /// This is indexed by a `PathTableIndex`. It contains two paths per entry: a path relative to
    /// a `FileKind` root, and a full filesystem path.
    ///
    /// The paths must never be moved. This works even though the `Vec` can reallocate, because the
    /// underlying paths are constructed from leaked `Pathbuf`s.
    paths: Vec<(&'static Path, &'static Path)>,
}

impl PathTable {
    /// Stores a path in the path table and returns the index for the entry.
    /// It's assumed that the caller has a master list of paths and won't store duplicates.
    ///
    /// The indexes are guaranteed to be in ascending order, so that if the caller stores a sorted
    /// list of paths then the indexes will also be sorted.
    pub fn store(local: PathBuf, fullpath: PathBuf) -> PathTableIndex {
        PATHTABLE.write().unwrap().store_internal(local, fullpath)
    }

    fn store_internal(&mut self, local: PathBuf, fullpath: PathBuf) -> PathTableIndex {
        let idx = PathTableIndex(u32::try_from(self.paths.len()).expect("internal error"));
        let local = Box::leak(local.into_boxed_path());
        let fullpath = Box::leak(fullpath.into_boxed_path());
        self.paths.push((local, fullpath));
        idx
    }

    /// Return the local path based on its index.
    /// This can panic if the index is not one provided by `PathTable::store`.
    pub fn lookup_path(idx: PathTableIndex) -> &'static Path {
        PATHTABLE.read().unwrap().lookup_paths_inner(idx).0
    }

    /// Return the full path based on its index.
    /// This can panic if the index is not one provided by `PathTable::store`.
    pub fn lookup_fullpath(idx: PathTableIndex) -> &'static Path {
        PATHTABLE.read().unwrap().lookup_paths_inner(idx).1
    }

    #[inline]
    fn lookup_paths_inner(&self, idx: PathTableIndex) -> (&'static Path, &'static Path) {
        let PathTableIndex(idx) = idx;
        // This will panic if idx is out of range.
        // Should never happen as long as lookups are only done on PathTableIndex provided by this module.
        self.paths[idx as usize]
    }
}
