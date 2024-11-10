use std::path::PathBuf;

use crate::block::{Block, BlockItem, Field, BV};
use crate::context::ScopeContext;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::game::Game;
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::on_action::on_action_scopecontext;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
#[cfg(feature = "ck3")]
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_duration, validate_modifiers_with_base};
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct OnActions {
    on_actions: TigerHashMap<&'static str, OnAction>,
}

impl OnActions {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.on_actions.get_mut(key.as_str()) {
            on_action_special_append(&mut other.block, block);
        } else {
            self.on_actions.insert(key.as_str(), OnAction::new(key, block));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.on_actions.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.on_actions.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.on_actions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for OnActions {
    fn subpath(&self) -> PathBuf {
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => PathBuf::from("common/on_action"),
            #[cfg(feature = "imperator")]
            Game::Imperator => PathBuf::from("common/on_action"),
            #[cfg(feature = "vic3")]
            Game::Vic3 => PathBuf::from("common/on_actions"),
        }
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

fn on_action_special_append(first: &mut Block, mut second: Block) {
    const SPECIAL_FIELDS: &[&str] = &[
        "events",
        "random_events",
        "first_valid",
        "on_actions",
        "random_on_action",
        "first_valid_on_action",
    ];
    let mut seen: TigerHashSet<String> = TigerHashSet::default();
    for item in second.drain() {
        if let BlockItem::Field(Field(key, cmp, BV::Block(mut block))) = item {
            // For the special fields, append the first one we see to the first block's corresponding field.
            if SPECIAL_FIELDS.contains(&key.as_str()) && !seen.contains(&key.to_string()) {
                seen.insert(key.to_string());
                if first.add_to_field_block(key.as_str(), &mut block) {
                    continue;
                }
            }
            first.add_key_bv(key, cmp, BV::Block(block));
        } else {
            first.add_item(item);
        }
    }
}

#[derive(Clone, Debug)]
pub struct OnAction {
    key: Token,
    block: Block,
}

impl OnAction {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut sc;
        if let Some(sc_builtin) = on_action_scopecontext(&self.key, data) {
            sc = sc_builtin;
        } else {
            sc = ScopeContext::new(Scopes::non_primitive(), &self.key);
            sc.set_strict_scopes(false);
        }

        validate_on_action(&self.block, data, &mut sc);
    }
}

pub fn validate_on_action(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    vd.field_validated_block_sc("weight_multiplier", sc, validate_modifiers_with_base);

    // TODO: multiple random_events blocks in one on_action aren't outright bugged on Vic3,
    // but they might still get merged together into one big event pool. Verify.

    let mut count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("events", |key, b, data| {
        let mut vd = Validator::new(b, data);
        vd.multi_field_validated_block_sc("delay", sc, validate_duration);
        for token in vd.values() {
            data.verify_exists(Item::Event, token);
            data.check_event_scope(token, sc);
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            // TODO: verify
            let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
            let info = "try combining them into one block";
            warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("random_events", |key, b, data| {
        let mut vd = Validator::new(b, data);
        vd.field_numeric("chance_to_happen"); // TODO: 0 - 100
        vd.field_script_value("chance_of_no_event", sc);
        vd.multi_field_validated_block_sc("delay", sc, validate_duration); // undocumented
        for (_key, token) in vd.integer_values() {
            if token.is("0") {
                continue;
            }
            data.verify_exists(Item::Event, token);
            data.check_event_scope(token, sc);
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            let msg = format!("multiple `{key}` blocks in one on_action do not work");
            let info = "try putting each into its own on_action and firing those separately";
            err(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("first_valid", |key, b, data| {
        let mut vd = Validator::new(b, data);
        for token in vd.values() {
            data.verify_exists(Item::Event, token);
            data.check_event_scope(token, sc);
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            // TODO: verify
            let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
            let info = "try putting each into its own on_action and firing those separately";
            warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("on_actions", |key, b, data| {
        let mut vd = Validator::new(b, data);
        vd.multi_field_validated_block_sc("delay", sc, validate_duration);
        for token in vd.values() {
            data.verify_exists(Item::OnAction, token);
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            // TODO: verify
            let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
            let info = "try combining them into one block";
            warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("random_on_action", |key, b, data| {
        let mut vd = Validator::new(b, data);
        vd.field_numeric("chance_to_happen"); // TODO: 0 - 100
        vd.field_script_value("chance_of_no_event", sc);
        for (_key, token) in vd.integer_values() {
            if token.is("0") {
                continue;
            }
            data.verify_exists(Item::OnAction, token);
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            // TODO: verify
            let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
            let info = "try putting each into its own on_action and firing those separately";
            warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("first_valid_on_action", |key, b, data| {
        let mut vd = Validator::new(b, data);
        for token in vd.values() {
            data.verify_exists(Item::OnAction, token);
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            // TODO: verify
            let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
            let info = "try putting each into its own on_action and firing those separately";
            warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    vd.field_validated_block("effect", |b, data| {
        validate_effect(b, data, sc, Tooltipped::No);
    });
    // TODO: check for infinite fallback loops?
    vd.field_item("fallback", Item::OnAction);
}
