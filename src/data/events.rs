use std::path::PathBuf;
use std::str::FromStr;

use crate::block::{Block, BlockItem, Field};
use crate::context::{Reason, ScopeContext};
use crate::data::scripted_effects::Effect;
use crate::data::scripted_triggers::Trigger;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::game::Game;
use crate::helpers::{dup_error, TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pathtable::PathTableIndex;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Events {
    events: TigerHashMap<(&'static str, u16), Event>,
    namespaces: TigerHashSet<Token>,
    triggers: TigerHashMap<(PathTableIndex, &'static str), Trigger>,
    effects: TigerHashMap<(PathTableIndex, &'static str), Effect>,
}

impl Events {
    fn load_event(&mut self, key: Token, block: Block) {
        if let Some((key_a, key_b)) = key.as_str().split_once('.') {
            if let Ok(id) = u16::from_str(key_b) {
                if let Some(other) = self.get_event(key.as_str()) {
                    dup_error(&key, &other.key, "event");
                }
                self.events.insert((key_a, id), Event::new(key, block));
                return;
            }
        }
        let msg = "Event names should be in the form NAMESPACE.NUMBER";
        let info = "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of up to 4 digits.";
        warn(ErrorKey::EventNamespace).msg(msg).info(info).loc(key).push();
    }

    fn load_scripted_trigger(&mut self, key: Token, block: Block) {
        let index = (key.loc.idx, key.as_str());
        if let Some(other) = self.triggers.get(&index) {
            dup_error(&key, &other.key, "scripted trigger");
        }
        self.triggers.insert(index, Trigger::new(key, block, None));
    }

    fn load_scripted_effect(&mut self, key: Token, block: Block) {
        let index = (key.loc.idx, key.as_str());
        if let Some(other) = self.effects.get(&index) {
            dup_error(&key, &other.key, "scripted effect");
        }
        self.effects.insert(index, Effect::new(key, block, None));
    }

    #[cfg(feature = "ck3")]
    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        let index = (key.loc.idx, key.as_str());
        self.triggers.get(&index)
    }

    #[cfg(feature = "ck3")]
    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        let index = (key.loc.idx, key.as_str());
        self.effects.get(&index)
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
        for item in self.effects.values() {
            item.validate(data);
        }

        for item in self.triggers.values() {
            item.validate(data);
        }

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

impl FileHandler<Block> for Events {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("events")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        #[derive(Copy, Clone)]
        enum Expecting {
            Event,
            ScriptedTrigger,
            ScriptedEffect,
        }

        let mut expecting = Expecting::Event;

        for item in block.drain() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is("namespace") {
                    if let Some(value) = bv.expect_into_value() {
                        self.namespaces.insert(value);
                    }
                } else if key.is("scripted_trigger") || key.is("scripted_effect") {
                    let msg = format!("`{key}` should be used without `=`");
                    err(ErrorKey::ParseError).msg(msg).loc(key).push();
                } else if let Some(block) = bv.into_block() {
                    match expecting {
                        Expecting::ScriptedTrigger => {
                            self.load_scripted_trigger(key, block);
                            expecting = Expecting::Event;
                        }
                        Expecting::ScriptedEffect => {
                            self.load_scripted_effect(key, block);
                            expecting = Expecting::Event;
                        }
                        Expecting::Event => {
                            self.load_event(key, block);
                        }
                    };
                } else {
                    let msg = "unknown setting in event file";
                    err(ErrorKey::UnknownField).msg(msg).loc(key).push();
                }
            } else if let Some(key) = item.expect_value() {
                if matches!(expecting, Expecting::Event) && key.is("scripted_trigger") {
                    if !Game::is_ck3() {
                        let msg = "scripted triggers in event files are only for CK3";
                        err(ErrorKey::WrongGame).msg(msg).loc(key).push();
                    }
                    expecting = Expecting::ScriptedTrigger;
                } else if matches!(expecting, Expecting::Event) && key.is("scripted_effect") {
                    if !Game::is_ck3() {
                        let msg = "scripted effects in event files are only for CK3";
                        err(ErrorKey::WrongGame).msg(msg).loc(key).push();
                    }
                    expecting = Expecting::ScriptedEffect;
                } else {
                    err(ErrorKey::Validation)
                        .msg("unexpected token")
                        .info("Did you forget an = ?")
                        .loc(key)
                        .push();
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub key: Token,
    pub block: Block,
    expects_scope: Scopes,
    expects_from_token: Token,
}

impl Event {
    pub fn new(key: Token, block: Block) -> Self {
        let (expects_scope, expects_from_token) = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::events::get_event_scope(&key, &block),
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::events::get_event_scope(&key, &block),
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::events::get_event_scope(&key, &block),
        };
        Self { key, block, expects_scope, expects_from_token }
    }

    pub fn validate(&self, data: &Everything) {
        if let Some((namespace, _)) = self.key.as_str().split_once('.') {
            if !data.item_exists(Item::EventNamespace, namespace) {
                let msg = format!("event file should start with `namespace = {namespace}`");
                let info = "otherwise the event won't be found in-game";
                err(ErrorKey::EventNamespace).msg(msg).info(info).loc(&self.key).push();
            }
        }

        let mut sc = ScopeContext::new(self.expects_scope, &self.expects_from_token);
        sc.set_strict_scopes(false);
        sc.set_source(&self.key);

        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::events::validate_event(self, data, &mut sc),
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::events::validate_event(self, data, &mut sc),
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::events::validate_event(self, data, &mut sc),
        };
    }

    pub fn validate_call(&self, data: &Everything, sc: &mut ScopeContext) {
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::events::validate_event(self, data, sc),
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::events::validate_event(self, data, sc),
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::events::validate_event(self, data, sc),
        };
    }
}
