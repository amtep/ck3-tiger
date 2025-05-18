use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CommanderOrder {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CommanderOrder, CommanderOrder::add)
}

impl CommanderOrder {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CommanderOrder, key, block, Box::new(Self {}));
    }
}

impl DbKind for CommanderOrder {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_gerund");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("texture", Item::File);
        vd.field_choice("military_type", &["army", "navy"]);
        // undocumented
        vd.field_choice(
            "ai_usage",
            &["advance", "defend", "intercept", "raid_convoys", "escort_convoys"],
        );
        vd.field_bool("is_basic_order_type");

        vd.field_validated_key_block("possible", |key, block, data| {
            if block.has_key_recursive("error_check") {
                let msg = "error_check is no longer used to control visibility";
                let info = "there is now a separate `visible` trigger";
                warn(ErrorKey::Removed).msg(msg).info(info).loc(key).push();
            }
            // TODO: `NOT = { has_trait = foo }` in this trigger will be misrepresented in the UI
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            let mk = ModifKinds::Battle | ModifKinds::Character | ModifKinds::Unit;
            validate_modifs(block, data, mk, vd);
        });

        // TODO: this should be a state inside a unit's Item::Entity
        // The connection between the units and the entities is very indirect.
        vd.field_value("entity_animation");

        if block.field_value_is("military_type", "navy") {
            vd.field_item("naval_entity", Item::Entity);
        } else {
            vd.ban_field("naval_entity", || "navy");
        }

        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::Character);

        vd.field_numeric_range("indicator_position_angle", 0.0..360.0);
        vd.field_numeric_range("indicator_position_angle_for_enemy", 0.0..360.0);
        vd.field_item("clicksound", Item::Sound);
        vd.field_numeric("experience");

        // TODO: verify scope type
        let mut sc = ScopeContext::new(Scopes::Character, key);
        // undocumented
        vd.field_script_value("ai_weight", &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct CommanderRank {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CommanderRank, CommanderRank::add)
}

impl CommanderRank {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CommanderRank, key, block, Box::new(Self {}));
    }
}

impl DbKind for CommanderRank {
    // This entire item type is undocumented
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("texture", Item::File);

        // A comment in the script says
        // "If you're adding more ranks that commanders can be promoted to, make sure to change HIGHEST_PROMOTION_RANK in defines"
        // but that doesn't limit the possible `rank_value` values, since they can be set in other
        // ways than promotion.
        vd.field_integer_range("rank_value", 1..);

        for field in
            &["character_modifier", "general_modifier", "admiral_modifier", "country_modifier"]
        {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::all(), vd);
            });
        }
        vd.field_validated_block("interest_group_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::InterestGroup, vd);
        });

        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_validated_sc("title", &mut sc, validate_desc);
    }
}
