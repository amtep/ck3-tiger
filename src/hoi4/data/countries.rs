use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::lowercase::Lowercase;
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger_internal;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CountryTag {
    file: Token,
}

inventory::submit! {
    ItemLoader::Full(GameFlags::Hoi4, Item::CountryTag, PdxEncoding::Utf8NoBom, ".txt", LoadAsFile::Yes, Recursive::No, CountryTag::add)
}

impl CountryTag {
    pub fn add(db: &mut Db, _file: Token, mut block: Block) {
        for (key, value) in block.drain_assignments_warn() {
            let fake_block = Block::new(key.loc);
            db.add(Item::CountryTag, key, fake_block, Box::new(Self { file: value }));
        }
    }
}

impl DbKind for CountryTag {
    fn validate(&self, _key: &Token, _block: &Block, data: &Everything) {
        let pathname = format!("common/{}", &self.file);
        data.verify_exists_implied(Item::File, &pathname, &self.file);
    }
}

#[derive(Clone, Debug)]
pub struct CountryTagAlias {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::CountryTagAlias, CountryTagAlias::add)
}

impl CountryTagAlias {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CountryTag, key, block, Box::new(Self {}));
    }
}

impl DbKind for CountryTagAlias {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // THIS is country being checked, PREV is country looking up the alias
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.open_scope(Scopes::Country, key.clone());

        let mut count = 0;
        // TODO: correct handling of variable and event target specs
        if vd.field_value("variable").is_some() {
            count += 1;
        }
        if vd.field_value("global_event_target").is_some() {
            count += 1;
        }
        if vd.field_value("event_target").is_some() {
            count += 1;
        }
        if count > 1 {
            let msg =
                "only one of variable, global_event_target, or event_target should be specified";
            warn(ErrorKey::Conflict).msg(msg).loc(block).push();
        }

        let mut has_list = false;
        has_list |= vd.field_item("original_tag", Item::CountryTag);
        has_list |= vd.field_list_items("targets", Item::CountryTag);
        // TODO: validate array names
        has_list |= vd.field_value("target_array").is_some();

        if count > 0 && has_list {
            let msg = "can't combine variable/event targets and listed target";
            warn(ErrorKey::Conflict).msg(msg).loc(block).push();
        }

        if has_list {
            vd.field_validated_block_sc("country_score", &mut sc, validate_modifiers_with_base);
            vd.field_item("fallback", Item::CountryTag);
        } else {
            vd.ban_field("country_score", || "original_tag, targets, or target_array");
            vd.ban_field("fallback", || "original_tag, targets, or target_array");
        }

        validate_trigger_internal(
            &Lowercase::new_unchecked("country_tag_alias"),
            false,
            block,
            data,
            &mut sc,
            vd,
            Tooltipped::No,
            false,
        );

        sc.close();
    }
}
