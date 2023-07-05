use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::report::{error, ErrorKey};
use crate::scopes::scope_iterator;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug, Default)]
pub struct ScriptedLists {
    lists: FnvHashMap<String, List>,
}

impl ScriptedLists {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.lists.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "scripted list");
            }
        }
        self.lists.insert(key.to_string(), List::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.lists.contains_key(key)
    }

    pub fn base(&self, item: &Token) -> Option<&Token> {
        self.lists
            .get(item.as_str())
            .and_then(|item| item.block.get_field_value("base"))
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.lists.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for ScriptedLists {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_lists")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(mut block) = PdxFile::read(entry, fullpath) else { return; };
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct List {
    pub key: Token,
    block: Block,
}

impl List {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("base");
        vd.req_field("conditions");

        if let Some(token) = vd.field_value("base") {
            if let Some((_, outscope)) = scope_iterator(token, data) {
                let mut sc = ScopeContext::new(outscope, token);
                vd.field_validated_block("conditions", |block, data| {
                    validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
                });
            } else {
                error(token, ErrorKey::MissingItem, "no such base list");
            }
        }
    }
}
