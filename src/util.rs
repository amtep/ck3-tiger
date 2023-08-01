//! Miscellaneous helper functions related to filesystem operations.

use std::path::{Path, PathBuf};

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
