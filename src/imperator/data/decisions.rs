use fnv::FnvHashMap;
use std::path::PathBuf;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::imperator::data::missions::validate_imperator_highlight;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<&'static str, Decision>,
}

impl Decisions {
    pub fn load_item(&mut self, key: &Token, block: &Block) {
        if key.is("country_decisions") {
            for (key, block) in block.iter_definitions_warn() {
                self.decisions.insert(key.as_str(), Decision::new(key.clone(), block.clone()));
            }
        }
    }
    pub fn exists(&self, key: &str) -> bool {
        self.decisions.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.decisions.values().map(|item| &item.key)
    }
    pub fn validate(&self, data: &Everything) {
        for item in self.decisions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Decisions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("decisions/")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(&key, &block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    key: Token,
    block: Block,
}

impl Decision {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }
    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Country, &self.key);

        data.verify_exists(Item::Localization, &self.key);
        let loca = format!("{}_desc", self.key);
        data.verify_exists_implied(Item::Localization, &loca, &self.key);

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("highlight", |b, data| {
            validate_imperator_highlight(&self.key, b, &mut sc, data);
        });
        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
