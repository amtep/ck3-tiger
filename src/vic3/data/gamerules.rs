use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GameRule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::GameRule, GameRule::add)
}

impl GameRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GameRule, key, block, Box::new(Self {}));
    }
}

/// LAST UPDATED VIC3 VERSION 1.6.0
/// Taken from `common/game_rules/_game_rules.info`
const SIMPLE_GAME_RULE_FLAGS: &[&str] = &[
    "blocks_achievements",
    "lenient_ai",
    "harsh_ai",
    "low_ai_aggression",
    "high_ai_aggression",
    "no_subject_flags",
    "no_subject_map_color",
    // undocumented flags follow
    "autonomous_investment",
    "no_pop_consolidation",
    "minor_pop_consolidation",
    "moderate_pop_consolidation",
    "aggressive_pop_consolidation",
    "directly_controlled_investment",
    "loyalties_grace_period_none",
    "loyalties_grace_period_short",
    "loyalties_grace_period_long",
    "loyalties_grace_period_extra_long",
    "no_fantastical_content",
    "use_custom_rng_seed",
    "no_dynamic_naming",
];

impl DbKind for GameRule {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (key, _) in block.iter_definitions() {
            db.add_flag(Item::GameRuleSetting, key.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("rule_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("default");
        if let Some(token) = vd.field_value("default") {
            if block.get_field_block(token.as_str()).is_none() {
                let msg = "default value not found among the settings";
                warn(ErrorKey::MissingItem).strong().msg(msg).loc(token).push();
            }
        }

        vd.unknown_block_fields(|key, block| {
            let mut vd = Validator::new(block, data);
            let loca = format!("setting_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("setting_{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);

            vd.multi_field_validated_value("flag", |_, mut vd| {
                vd.maybe_prefix_item("disable_", Item::ProductionMethod);
                vd.maybe_prefix_item("force_", Item::ProductionMethod);
                vd.choice(SIMPLE_GAME_RULE_FLAGS);
            });
        });
    }
}
