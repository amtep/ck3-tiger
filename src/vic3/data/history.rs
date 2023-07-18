use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

/// The history files in Vic3 are fairly simple. Files under `common/history/` have `keyword = { effect... }` as top-level blocks,
/// where the effects from the same keywords are all added together. The keywords seem to be arbitrary, except for GLOBAL which
/// is documented to go last.

#[derive(Clone, Debug, Default)]
pub struct History {
    history: FnvHashMap<String, HistoryEffect>,
}

impl History {
    fn load_item(&mut self, key: Token, mut block: Block) {
        if let Some(entry) = self.history.get_mut(key.as_str()) {
            entry.block.append(&mut block);
        } else {
            self.history.insert(key.to_string(), HistoryEffect::new(key, block));
        }
    }

    pub fn validate(&self, data: &Everything) {
        // Don't know what order the history effects are executed in, so let's do alphabetic
        // except for `GLOBAL`, which is documented to go last.
        let mut names: Vec<&str> =
            self.history.keys().filter(|name| *name != "GLOBAL").map(|name| &**name).collect();
        names.sort();
        names.push("GLOBAL");

        for name in names {
            if let Some(item) = self.history.get(name) {
                item.validate(data);
            }
        }
    }
}

impl FileHandler<Block> for History {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/history/")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct HistoryEffect {
    key: Token,
    block: Block,
}

impl HistoryEffect {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut sc = ScopeContext::new(Scopes::None, &self.key);
        validate_effect(&self.block, data, &mut sc, Tooltipped::No);
    }
}
