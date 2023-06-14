use as_any::AsAny;
use fnv::FnvHashMap;
use std::fmt::Debug;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::helpers::dup_error;
use crate::item::Item;
use crate::token::Token;

#[derive(Debug, Default)]
pub struct Db {
    database: FnvHashMap<(Item, String), DbEntry>,
    flags: FnvHashMap<(Item, String), Token>,
}

impl Db {
    pub fn add(&mut self, item: Item, key: Token, block: Block, kind: Box<dyn DbKind>) {
        let index = (item, key.to_string());
        if let Some(other) = self.database.get(&index) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, &item.to_string());
            }
        }
        self.database.insert(index, DbEntry { key, block, kind });
    }

    pub fn add_flag(&mut self, item: Item, key: Token) {
        let index = (item, key.to_string());
        self.flags.insert(index, key);
    }

    pub fn validate(&self, data: &Everything) {
        // Sort the entries to create a diffable error output
        let mut vec: Vec<&DbEntry> = self.database.values().collect();
        vec.sort_by(|entry_a, entry_b| entry_a.key.loc.cmp(&entry_b.key.loc));
        for entry in vec {
            entry.kind.validate(&entry.key, &entry.block, data);
        }
    }

    pub fn exists(&self, item: Item, key: &str) -> bool {
        // TODO: figure out how to avoid the to_string() here
        let index = (item, key.to_string());
        self.database.contains_key(&index) || self.flags.contains_key(&index)
    }

    /// This doesn't work yet :(
    pub fn get_item<T: DbKind>(&self, item: Item, key: &str) -> Option<(&Token, &Block, &T)> {
        // TODO: figure out how to avoid the to_string() here
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            if let Some(kind) = entry.kind.as_any().downcast_ref::<T>() {
                return Some((&entry.key, &entry.block, kind));
            }
        }
        None
    }

    /// Using this until get_item works
    pub fn get_key_block(&self, item: Item, key: &str) -> Option<(&Token, &Block)> {
        // TODO: figure out how to avoid the to_string() here
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            Some((&entry.key, &entry.block))
        } else {
            None
        }
    }

    pub fn has_property(&self, item: Item, key: &str, property: &str, data: &Everything) -> bool {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .has_property(property, &entry.key, &entry.block, data)
        } else {
            false
        }
    }

    pub fn validate_call(&self, item: Item, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .validate_call(&entry.key, &entry.block, key, data, sc)
        }
    }

    pub fn validate_variant(&self, item: Item, key: &Token, data: &Everything, variant: &Token) {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .validate_variant(&entry.key, &entry.block, data, variant)
        } else {
            warn(key, ErrorKey::MissingItem, &format!("{item} not found"));
        }
    }
}

#[derive(Debug)]
pub struct DbEntry {
    key: Token,
    block: Block,
    kind: Box<dyn DbKind>,
}

pub trait DbKind: Debug + AsAny {
    fn validate(&self, key: &Token, block: &Block, data: &Everything);
    fn has_property(
        &self,
        _property: &str,
        _key: &Token,
        _block: &Block,
        _data: &Everything,
    ) -> bool {
        false
    }
    fn validate_call(
        &self,
        _key: &Token,
        _block: &Block,
        _from: &Token,
        _data: &Everything,
        _sc: &mut ScopeContext,
    ) {
    }
    fn validate_variant(&self, _key: &Token, _block: &Block, _data: &Everything, _variant: &Token) {
    }
}
