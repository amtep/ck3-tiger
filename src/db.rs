use std::any::Any;
use std::fmt::Debug;

use as_any::AsAny;
use fnv::FnvHashMap;
use rayon::prelude::*;
use strum::IntoEnumIterator;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::{dup_advice, dup_error, exact_dup_error};
use crate::item::Item;
use crate::token::Token;

/// The main database of game items.
#[derive(Debug)]
pub struct Db {
    /// Items with full DbEntries, meaning a key and a block for each.
    /// For convenience, the secondary hashmap is pre-made for every Item variant
    database: FnvHashMap<Item, FnvHashMap<String, DbEntry>>,
    /// Items generated as side effects of the full items in `database`.
    /// For convenience, the secondary hashmap is pre-made for every Item variant
    flags: FnvHashMap<Item, FnvHashMap<String, Token>>,
}
// TODO: make the upper level of the Db use a Vec rather than a HashMap

impl Default for Db {
    fn default() -> Self {
        let mut db = Self { database: FnvHashMap::default(), flags: FnvHashMap::default() };
        for itype in Item::iter() {
            db.database.insert(itype, FnvHashMap::default());
            db.flags.insert(itype, FnvHashMap::default());
        }
        db
    }
}

impl Db {
    pub fn add(&mut self, item: Item, key: Token, block: Block, kind: Box<dyn DbKind>) {
        if let Some(other) = self.database.get(&item).unwrap().get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                if other.block.equivalent(&block) {
                    exact_dup_error(&key, &other.key, &item.to_string());
                } else {
                    dup_error(&key, &other.key, &item.to_string());
                }
            }
        }
        self.database.get_mut(&item).unwrap().insert(key.to_string(), DbEntry { key, block, kind });
    }

    pub fn add_exact_dup_ok(
        &mut self,
        item: Item,
        key: Token,
        block: Block,
        kind: Box<dyn DbKind>,
    ) {
        if let Some(other) = self.database.get(&item).unwrap().get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                if other.block.equivalent(&block) {
                    dup_advice(&key, &other.key, &item.to_string());
                } else {
                    dup_error(&key, &other.key, &item.to_string());
                }
            }
        }
        self.database.get_mut(&item).unwrap().insert(key.to_string(), DbEntry { key, block, kind });
    }

    pub fn add_flag(&mut self, item: Item, key: Token) {
        self.flags.get_mut(&item).unwrap().insert(key.to_string(), key);
    }

    pub fn validate(&self, data: &Everything) {
        self.database.par_iter().for_each(|(_, hash)| {
            hash.par_iter().for_each(|(_, entry)| {
                entry.kind.validate(&entry.key, &entry.block, data);
            });
        });
    }

    pub fn exists(&self, item: Item, key: &str) -> bool {
        self.database.get(&item).unwrap().contains_key(key)
            || self.flags.get(&item).unwrap().contains_key(key)
    }

    pub fn get_item<T: DbKind + Any>(&self, item: Item, key: &str) -> Option<(&Token, &Block, &T)> {
        if let Some(entry) = self.database.get(&item).unwrap().get(key) {
            if let Some(kind) = (*entry.kind).as_any().downcast_ref::<T>() {
                return Some((&entry.key, &entry.block, kind));
            }
        }
        None
    }

    pub fn get_key_block(&self, item: Item, key: &str) -> Option<(&Token, &Block)> {
        self.database.get(&item).unwrap().get(key).map(|entry| (&entry.key, &entry.block))
    }

    pub fn has_property(&self, item: Item, key: &str, property: &str, data: &Everything) -> bool {
        if let Some(entry) = self.database.get(&item).unwrap().get(key) {
            entry.kind.has_property(&entry.key, &entry.block, property, data)
        } else {
            false
        }
    }

    pub fn set_property(&mut self, item: Item, key: &str, property: &str) {
        if let Some(entry) = self.database.get_mut(&item).unwrap().get_mut(key) {
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
        if let Some(entry) = self.database.get(&item).unwrap().get(key.as_str()) {
            entry.kind.validate_call(&entry.key, &entry.block, key, block, data, sc);
        }
    }

    pub fn validate_use(&self, item: Item, key: &Token, block: &Block, data: &Everything) {
        if let Some(entry) = self.database.get(&item).unwrap().get(key.as_str()) {
            entry.kind.validate_use(&entry.key, &entry.block, data, key, block);
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
        if let Some(entry) = self.database.get(&item).unwrap().get(key.as_str()) {
            entry.kind.validate_property_use(&entry.key, &entry.block, property, caller, data);
        }
    }

    /// TODO: Returns a Vec for now, should become an iterator.
    pub fn iter_itype(&self, itype: Item) -> Vec<(&Token, &Block, &dyn DbKind)> {
        let mut vec = Vec::new();
        for entry in self.database.get(&itype).unwrap().values() {
            vec.push((&entry.key, &entry.block, &*entry.kind));
        }
        vec
    }

    /// TODO: Returns a Vec for now, should become an iterator.
    pub fn iter_itype_flags(&self, itype: Item) -> Vec<&Token> {
        let mut vec = Vec::new();
        for token in self.flags.get(&itype).unwrap().values() {
            vec.push(token);
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

pub trait DbKind: Debug + AsAny + Sync + Send {
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
