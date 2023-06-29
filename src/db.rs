use as_any::AsAny;
use fnv::FnvHashMap;
use std::fmt::Debug;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::{dup_advice, dup_error, exact_dup_error};
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
                if other.block.equivalent(&block) {
                    exact_dup_error(&key, &other.key, &item.to_string());
                } else {
                    dup_error(&key, &other.key, &item.to_string());
                }
            }
        }
        self.database.insert(index, DbEntry { key, block, kind });
    }

    pub fn add_exact_dup_ok(
        &mut self,
        item: Item,
        key: Token,
        block: Block,
        kind: Box<dyn DbKind>,
    ) {
        let index = (item, key.to_string());
        if let Some(other) = self.database.get(&index) {
            if other.key.loc.kind >= key.loc.kind {
                if other.block.equivalent(&block) {
                    dup_advice(&key, &other.key, &item.to_string());
                } else {
                    dup_error(&key, &other.key, &item.to_string());
                }
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

    /// Using this until `get_item` works
    pub fn get_key_block(&self, item: Item, key: &str) -> Option<(&Token, &Block)> {
        // TODO: figure out how to avoid the to_string() here
        let index = (item, key.to_string());
        self.database
            .get(&index)
            .map(|entry| (&entry.key, &entry.block))
    }

    pub fn has_property(&self, item: Item, key: &str, property: &str, data: &Everything) -> bool {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .has_property(&entry.key, &entry.block, property, data)
        } else {
            false
        }
    }

    pub fn set_property(&mut self, item: Item, key: &str, property: &str) {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get_mut(&index) {
            entry.kind.set_property(&entry.key, &entry.block, property);
        }
    }

    pub fn validate_call(
        &self,
        item: Item,
        key: &Token,
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .validate_call(&entry.key, &entry.block, key, block, data, sc);
        }
    }

    pub fn validate_use(&self, item: Item, key: &Token, block: &Block, data: &Everything) {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .validate_use(&entry.key, &entry.block, data, key, block);
        }
    }

    pub fn validate_property_use(
        &self,
        item: Item,
        key: &Token,
        data: &Everything,
        property: &Token,
        caller: &str,
    ) {
        let index = (item, key.to_string());
        if let Some(entry) = self.database.get(&index) {
            entry
                .kind
                .validate_property_use(&entry.key, &entry.block, property, caller, data);
        }
    }

    /// TODO: Returns a Vec for now, should become an iterator.
    pub fn iter_itype(&self, itype: Item) -> Vec<(&Token, &Block, &Box<dyn DbKind>)> {
        let mut vec = Vec::new();
        for ((item, _), entry) in &self.database {
            if *item == itype {
                vec.push((&entry.key, &entry.block, &entry.kind));
            }
        }
        vec
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
        _key: &Token,
        _block: &Block,
        _property: &str,
        _data: &Everything,
    ) -> bool {
        false
    }
    fn validate_call(
        &self,
        _key: &Token,
        _block: &Block,
        _from: &Token,
        _from_block: &Block,
        _data: &Everything,
        _sc: &mut ScopeContext,
    ) {
    }

    fn validate_use(
        &self,
        _key: &Token,
        _block: &Block,
        _data: &Everything,
        _call_key: &Token,
        _call_block: &Block,
    ) {
    }

    fn validate_property_use(
        &self,
        _key: &Token,
        _block: &Block,
        _property: &Token,
        _caller: &str,
        _data: &Everything,
    ) {
    }

    fn set_property(&mut self, _key: &Token, _block: &Block, _property: &str) {}
}
