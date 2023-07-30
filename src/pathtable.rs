/// This module provides a global table for the pathnames used in `FileEntry` and `Loc`. This will make the often-cloned Loc
/// faster to copy, since it will just contain an index into the global table. It also makes it faster to compare pathnames,
/// because the table will be created in lexical order by the caller (`Fileset`), with the exception of some stray files
/// (such as the config file) where the order doesn't matter.
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use once_cell::sync::Lazy;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathTableIndex(u32);

static PATHTABLE: Lazy<RwLock<PathTable>> = Lazy::new(|| RwLock::new(PathTable::default()));

#[derive(Debug, Default)]
pub struct PathTable {
    /// This is indexed by a `PathTableIndex`
    paths: Vec<PathBuf>,
}

impl PathTable {
    /// Stores a path in the path table and returns the index for the entry.
    /// It's assumed that the caller has a master list of paths and won't store duplicates.
    pub fn store(pathbuf: PathBuf) -> PathTableIndex {
        PATHTABLE.write().unwrap().store_internal(pathbuf)
    }

    fn store_internal(&mut self, pathbuf: PathBuf) -> PathTableIndex {
        let idx = PathTableIndex(u32::try_from(self.paths.len()).expect("internal error"));
        self.paths.push(pathbuf);
        idx
    }

    /// Return a stored string based on its index.
    /// This will panic if the index is not one provided by `PathTable::store`.
    pub fn lookup(idx: PathTableIndex) -> &'static Path {
        PATHTABLE.read().unwrap().lookup_internal(idx)
    }

    fn lookup_internal(&self, idx: PathTableIndex) -> &'static Path {
        let PathTableIndex(idx) = idx;
        // This will panic if idx is out of range.
        // Should never happen as long as lookups are only done on PathTableIndex provided by this module.
        let s = &self.paths[idx as usize];
        // Go through a raw pointer in order to confuse the borrow checker.
        // The borrow checker complains about returning a str with lifetime 'static, but we promise not
        // to change the paths table except to add to it. So it's safe.
        let ptr: *const Path = &**s;
        unsafe { &*ptr }
    }
}
