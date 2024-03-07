use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CharacterInteraction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::CharacterInteraction, CharacterInteraction::add)
}

impl CharacterInteraction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterInteraction, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterInteraction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("actor", Scopes::Country, key);
        sc.define_name("target", Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca1 = format!("{key}_act");
        let loca2 = format!("{key}_past");
        let loca3 = format!("{key}_act_past");
        data.verify_exists_implied(Item::Localization, &loca1, key);
        data.verify_exists_implied(Item::Localization, &loca2, key);
        data.verify_exists_implied(Item::Localization, &loca3, key);

        vd.field_bool("message");
        vd.field_bool("close_ui");
        vd.field_bool("on_other_nation");
        vd.field_bool("on_own_nation");
        vd.field_bool("on_other_nation");
        vd.field_bool("on_own_nation");
        vd.field_bool("needs_senate_approval");

        vd.field_item("sound", Item::Sound);
        // TODO - test if more diplo actions are valid to use here
        vd.field_choice("diplomatic_action", &["ransom"]);

        vd.field_validated_block("potential_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("allowed_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("character_actor_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("character_target_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("country_actor_trigger", |b, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("country_target_trigger", |b, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("province_actor_trigger", |b, data| {
            let mut sc = ScopeContext::new(Scopes::Province, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("province_target_trigger", |b, data| {
            let mut sc = ScopeContext::new(Scopes::Province, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
