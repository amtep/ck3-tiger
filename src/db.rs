use std::any::Any;
use std::fmt::Debug;

use as_any::AsAny;
use fnv::FnvHashMap;
use rayon::prelude::*;
use strum::IntoEnumIterator;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::{dup_error, exact_dup_advice, exact_dup_error};
use crate::item::Item;
use crate::token::Token;

/// The main database of game items.
#[derive(Debug)]
pub struct Db {
    /// Items with full DbEntries, meaning a key and a block for each.
    /// The `Vec` is indexed with an `Item` discriminant.
    database: Vec<FnvHashMap<String, DbEntry>>,
    /// Items generated as side effects of the full items in `database`.
    /// The `Vec` is indexed with an `Item` discriminant.
    flags: Vec<FnvHashMap<String, Token>>,
}

impl Default for Db {
    fn default() -> Self {
        let mut db = Self { database: Vec::default(), flags: Vec::default() };
        for _ in Item::iter() {
            db.database.push(FnvHashMap::default());
            db.flags.push(FnvHashMap::default());
        }
        db
    }
}

impl Db {
    pub fn add(&mut self, item: Item, key: Token, block: Block, kind: Box<dyn DbKind>) {
        if let Some(other) = self.database[item as usize].get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                if other.block.equivalent(&block) {
                    exact_dup_error(&key, &other.key, &item.to_string());
                } else {
                    dup_error(&key, &other.key, &item.to_string());
                }
            }
        }
        self.database[item as usize].insert(key.to_string(), DbEntry { key, block, kind });
    }

    pub fn add_exact_dup_ok(
        &mut self,
        item: Item,
        key: Token,
        block: Block,
        kind: Box<dyn DbKind>,
    ) {
        if let Some(other) = self.database[item as usize].get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                if other.block.equivalent(&block) {
                    exact_dup_advice(&key, &other.key, &item.to_string());
                } else {
                    dup_error(&key, &other.key, &item.to_string());
                }
            }
        }
        self.database[item as usize].insert(key.to_string(), DbEntry { key, block, kind });
    }

    pub fn add_flag(&mut self, item: Item, key: Token) {
        self.flags[item as usize].insert(key.to_string(), key);
    }

    pub fn validate(&self, data: &Everything) {
        self.database.par_iter().for_each(|hash| {
            hash.par_iter().for_each(|(_, entry)| {
                entry.kind.validate(&entry.key, &entry.block, data);
            });
        });
    }

    pub fn exists(&self, item: Item, key: &str) -> bool {
        self.database[item as usize].contains_key(key)
            || self.flags[item as usize].contains_key(key)
    }

    #[allow(dead_code)] // not currently used, but was hard to write...
    pub fn get_item<T: DbKind + Any>(&self, item: Item, key: &str) -> Option<(&Token, &Block, &T)> {
        if let Some(entry) = self.database[item as usize].get(key) {
            if let Some(kind) = (*entry.kind).as_any().downcast_ref::<T>() {
                return Some((&entry.key, &entry.block, kind));
            }
        }
        None
    }

    pub fn get_key_block(&self, item: Item, key: &str) -> Option<(&Token, &Block)> {
        self.database[item as usize].get(key).map(|entry| (&entry.key, &entry.block))
    }

    pub fn has_property(&self, item: Item, key: &str, property: &str, data: &Everything) -> bool {
        if let Some(entry) = self.database[item as usize].get(key) {
            entry.kind.has_property(&entry.key, &entry.block, property, data)
        } else {
            false
        }
    }

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn set_property(&mut self, item: Item, key: &str, property: &str) {
        if let Some(entry) = self.database[item as usize].get_mut(key) {
            entry.kind.set_property(&entry.key, &entry.block, property);
        }
    }

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn validate_call(
        &self,
        item: Item,
        key: &Token,
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        if let Some(entry) = self.database[item as usize].get(key.as_str()) {
            entry.kind.validate_call(&entry.key, &entry.block, key, block, data, sc);
        }
    }

    pub fn validate_use(&self, item: Item, key: &Token, block: &Block, data: &Everything) {
        if let Some(entry) = self.database[item as usize].get(key.as_str()) {
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
        if let Some(entry) = self.database[item as usize].get(key.as_str()) {
            entry.kind.validate_property_use(&entry.key, &entry.block, property, caller, data);
        }
    }

    /// TODO: Returns a Vec for now, should become an iterator.
    pub fn iter_itype(&self, itype: Item) -> Vec<(&Token, &Block, &dyn DbKind)> {
        let mut vec = Vec::new();
        for entry in self.database[itype as usize].values() {
            vec.push((&entry.key, &entry.block, &*entry.kind));
        }
        vec
    }

    /// TODO: Returns a Vec for now, should become an iterator.
    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn iter_itype_flags(&self, itype: Item) -> Vec<&Token> {
        let mut vec = Vec::new();
        for token in self.flags[itype as usize].values() {
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
