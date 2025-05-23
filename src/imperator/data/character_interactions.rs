use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
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

        vd.field_trigger("potential_trigger", Tooltipped::No, &mut sc);
        vd.field_trigger("allowed_trigger", Tooltipped::Yes, &mut sc);
        vd.field_trigger("character_actor_trigger", Tooltipped::No, &mut sc);
        vd.field_trigger("character_target_trigger", Tooltipped::No, &mut sc);
        vd.field_trigger_builder("country_actor_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_trigger_builder("country_target_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_trigger_builder("province_actor_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Province, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_trigger_builder("province_target_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Province, key);
            sc.define_name("actor", Scopes::Country, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_effect("effect", Tooltipped::Yes, &mut sc);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
