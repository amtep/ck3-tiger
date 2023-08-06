use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Vic3GameRule {}

impl Vic3GameRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for (key, _) in block.iter_definitions() {
            db.add_flag(Item::GameRuleSetting, key.clone());
        }
        db.add(Item::GameRule, key, block, Box::new(Self {}));
    }
}

/// LAST UPDATED VIC3 VERSION 1.3.6
/// Taken from `common/game_rules/_game_rules.info`
const SIMPLE_GAME_RULE_FLAGS: &[&str] = &[
    "blocks_achievements",
    "lenient_ai",
    "harsh_ai",
    "low_ai_aggression",
    "high_ai_aggression",
    "no_subject_flags",
    "no_subject_map_color",
];

impl DbKind for Vic3GameRule {
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

            vd.field_validated_values("flag", |_, value, _| {
                if SIMPLE_GAME_RULE_FLAGS.contains(&value.as_str()) {
                    return;
                }
                if let Some(pm) = value.as_str().strip_prefix("disable_") {
                    data.verify_exists_implied(Item::ProductionMethod, pm, value);
                }
                if let Some(pm) = value.as_str().strip_prefix("force_") {
                    data.verify_exists_implied(Item::ProductionMethod, pm, value);
                }
            });
        });
    }
}
