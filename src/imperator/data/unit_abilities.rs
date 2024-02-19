use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct UnitAbility {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::UnitAbility, UnitAbility::add)
}

impl UnitAbility {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::UnitAbility, key, block, Box::new(Self {}));
    }
}

impl DbKind for UnitAbility {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Unit, key);
        sc.define_name("province", Scopes::Province, key);

        vd.field_bool("enable");
        vd.field_bool("toggle");
        vd.field_bool("army_only");
        vd.field_bool("navy_only");
        vd.field_bool("map");
        vd.field_bool("is_road_building");
        vd.field_bool("cancel_on_combat_end");
        vd.field_bool("is_desecrate");
        vd.field_bool("confirm");

        vd.field_numeric("duration");

        vd.field_item("soundeffect", Item::Sound);

        vd.field_choice("idle_entity_state", &["recruiting", "army_drill", "raiding"]);
        vd.field_choice("move_entity_state", &["build_road", "force_march"]);

        vd.field_list("available_states");

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("hidden", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("ai_will_revoke", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("finished_when", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("start_effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("start_effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("finish_effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_entering_province", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
