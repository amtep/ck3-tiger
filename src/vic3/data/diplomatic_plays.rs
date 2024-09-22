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
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DiplomaticPlay {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DiplomaticPlay, DiplomaticPlay::add)
}

impl DiplomaticPlay {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiplomaticPlay, key, block, Box::new(Self {}));
    }
}

impl DbKind for DiplomaticPlay {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("target_country", Scopes::Country, key);
        sc.define_name("initiator", Scopes::Country, key); // undocumented

        let mut diplo_sc = ScopeContext::new(Scopes::DiplomaticPlay, key);
        diplo_sc.define_name("target_country", Scopes::Country, key);
        diplo_sc.define_name("initiator", Scopes::Country, key); // undocumented

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("war_goal", Item::Wargoal);
        vd.field_item("texture", Item::File);

        vd.field_bool("requires_interest_marker");
        vd.field_bool("enable_switch_sides");
        vd.field_bool("mirror_war_goal");
        vd.field_bool("allow_negotiated_peace");
        vd.field_bool("initiator_can_add_war_goals");
        vd.field_bool("target_can_add_war_goals");
        vd.field_bool("add_infamy_for_starting_initiator_wargoals");
        vd.field_bool("add_infamy_for_starting_target_wargoals");
        vd.field_bool("is_epic");
        vd.field_bool("backing_down_always_ends_play"); // undocumented
        vd.field_bool("blocked_by_diplomatic_status"); // undocumented
        vd.field_numeric("ai_acceptance_max");

        vd.field_validated_block("selectable_in_lens", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("additional_involvement_trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_weekly_pulse", |block, data| {
            validate_effect(block, data, &mut diplo_sc, Tooltipped::No);
        });
        vd.field_validated_block("on_war_begins", |block, data| {
            validate_effect(block, data, &mut diplo_sc, Tooltipped::No);
        });
        vd.field_validated_block("on_war_end", |block, data| {
            validate_effect(block, data, &mut diplo_sc, Tooltipped::No);
        });

        // undocumented

        vd.field_item("confirmation_sound", Item::Sound);
    }
}
