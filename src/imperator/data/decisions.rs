use std::path::PathBuf;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: TigerHashMap<&'static str, Decision>,
}

impl Decisions {
    pub fn load_item(&mut self, key: &Token, block: &Block) {
        if key.is("country_decisions") {
            for (key, block) in block.iter_definitions_warn() {
                self.decisions.insert(key.as_str(), Decision::new(key.clone(), block.clone()));
            }
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.decisions.values() {
            registry.scan(&item.block);
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

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
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

        vd.field_trigger("potential", Tooltipped::No, &mut sc);
        vd.field_trigger_builder("highlight", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("province", Scopes::Province, key);
            sc
        });
        vd.field_trigger("allow", Tooltipped::Yes, &mut sc);
        vd.field_effect("effect", Tooltipped::Yes, &mut sc);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
