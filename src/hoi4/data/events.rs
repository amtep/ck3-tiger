use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use crate::block::{Block, BlockItem, Field};
use crate::context::{Reason, ScopeContext, Signature};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap, TigerHashSet};
use crate::hoi4::events::{get_event_scope, validate_event};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::variables::Variables;

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Hoi4Events {
    events: TigerHashMap<(&'static str, u16), Event>,
    namespaces: TigerHashSet<Token>,
    namespaces_lc: TigerHashSet<Lowercase<'static>>,
}

impl Hoi4Events {
    fn load_event(&mut self, key: Token, block: Block) {
        if let Some(name) = block.get_field_value("id").cloned() {
            if let Some((name_a, name_b)) = name.as_str().split_once('.') {
                if let Ok(id) = u16::from_str(name_b) {
                    if let Some(other) = self.get_event(name.as_str()) {
                        dup_error(&key, &other.key, "event");
                    }
                    self.events.insert((name_a, id), Event::new(key, block, name));
                    return;
                }
            }
            let msg = "Event names should be in the form NAMESPACE.NUMBER";
            let info = "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of up to 4 digits.";
            warn(ErrorKey::EventNamespace).msg(msg).info(info).loc(key).push();
        } else {
            let msg = "event without id field";
            err(ErrorKey::FieldMissing).msg(msg).loc(block).push();
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.events.values() {
            registry.scan(&item.block);
        }
    }

    fn get_event<'a>(&'a self, key: &'a str) -> Option<&'a Event> {
        if let Some((namespace, id)) = key.split_once('.') {
            if let Ok(id) = u16::from_str(id) {
                return self.events.get(&(namespace, id));
            }
        }
        None
    }

    pub fn check_scope(&self, token: &Token, sc: &mut ScopeContext) {
        if let Some(event) = self.get_event(token.as_str()) {
            sc.expect(event.expects_scope, &Reason::Token(token.clone()));
        }
    }

    pub fn namespace_exists(&self, key: &str) -> bool {
        self.namespaces.contains(key)
    }

    pub fn namespace_exists_lc(&self, key: &Lowercase) -> bool {
        self.namespaces_lc.contains(key)
    }

    pub fn iter_namespace_keys(&self) -> impl Iterator<Item = &Token> {
        self.namespaces.iter()
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Some((namespace, id)) = key.split_once('.') {
            if let Ok(id) = u16::from_str(id) {
                if self.events.contains_key(&(namespace, id)) {
                    return true;
                }
            }
        }
        false
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.events.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.events.values() {
            item.validate(data);
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if let Some(event) = self.get_event(key.as_str()) {
            event.validate_call(data, sc);
        }
    }
}

impl FileHandler<Block> for Hoi4Events {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("events")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read_optional_bom(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for item in block.drain() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is("add_namespace") {
                    if let Some(value) = bv.expect_into_value() {
                        self.namespaces_lc.insert(Lowercase::new(value.as_str()));
                        self.namespaces.insert(value);
                    }
                } else if let Some(block) = bv.into_block() {
                    self.load_event(key, block);
                } else {
                    let msg = "unknown setting in event file";
                    err(ErrorKey::UnknownField).msg(msg).loc(key).push();
                }
            } else if let Some(key) = item.expect_value() {
                err(ErrorKey::Validation)
                    .msg("unexpected token")
                    .info("Did you forget an = ?")
                    .loc(key)
                    .push();
            }
        }
    }
}

#[derive(Debug)]
pub struct Event {
    pub key: Token,
    pub block: Block,
    id: Token,
    expects_scope: Scopes,
    expects_from_token: Token,
    visited: Mutex<TigerHashSet<Signature>>,
}

impl Event {
    pub fn new(key: Token, block: Block, id: Token) -> Self {
        let (expects_scope, expects_from_token) = get_event_scope(&key, &block);
        let visited = Mutex::new(TigerHashSet::default());
        Self { key, block, id, expects_scope, expects_from_token, visited }
    }

    pub fn validate(&self, data: &Everything) {
        if let Some((namespace, _)) = self.id.as_str().split_once('.') {
            if !data.item_exists_lc(Item::EventNamespace, &Lowercase::new(namespace)) {
                let msg = format!("event file should start with `add_namespace = {namespace}`");
                let info = "otherwise the event won't be found in-game";
                err(ErrorKey::EventNamespace).msg(msg).info(info).loc(&self.key).push();
            }
        }

        let mut sc = ScopeContext::new(self.expects_scope, &self.expects_from_token);
        sc.set_strict_scopes(false);
        sc.set_source(&self.key);

        validate_event(self, data, &mut sc);
    }

    pub fn validate_call(&self, data: &Everything, sc: &mut ScopeContext) {
        if !self.visited.lock().unwrap().insert(sc.signature()) {
            // The event was already visited with an equivalent sc
            return;
        }
        validate_event(self, data, sc);
    }
}
