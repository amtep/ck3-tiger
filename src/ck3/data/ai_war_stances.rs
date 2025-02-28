use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err};
use crate::token::Token;
use crate::validator::Validator;

// LAST UPDATED CK3 VERSION 1.11.5
// Taken from common/ai_war_stances/_ai_war_stances.info
//
// TODO: find a way to warn if any of these are missing.
// The main problem is we have no Loc to attach the warning to.
const AI_WAR_STANCES: &[&str] = &[
    "attacker_offensive",
    "attacker_defensive",
    "defender_offensive",
    "defender_defensive",
    "defender_desperate",
    "great_holy_war_attacker",
    "great_holy_war_defender",
];

// LAST UPDATED CK3 VERSION 1.11.5
// Taken from common/ai_war_stances/_ai_war_stances.info
const AI_WAR_OBJECTIVES: &[&str] = &[
    "wargoal_province",
    "enemy_unit_province",
    "enemy_capital_province",
    "capital_province",
    "enemy_province",
    "province",
    "defend_wargoal_province",
    // Undocumented
    "enemy_ally_province",
];

// LAST UPDATED CK3 VERSION 1.11.5
// Taken from common/ai_war_stances/_ai_war_stances.info
const AI_WAR_AREAS: &[&str] = &[
    "wargoal",
    "primary_attacker",
    "primary_attacker_ally",
    "primary_defender",
    "primary_defender_ally",
];

#[derive(Clone, Debug)]
pub struct AiWarStance {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::AiWarStance, AiWarStance::add)
}

impl AiWarStance {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiWarStance, key, block, Box::new(Self {}));
    }
}

impl DbKind for AiWarStance {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !AI_WAR_STANCES.contains(&key.as_str()) {
            let msg = format!("unknown war stance `{key}`");
            err(ErrorKey::Validation).msg(msg).loc(key).push();
        }

        vd.field_integer_range("enemy_unit_priority", 1..=1000);

        vd.multi_field_validated_block("objectives", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_validated_block("enemy_unit_province", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer_range("priority", 1..=1000);
                vd.multi_field_choice("area", AI_WAR_AREAS);
            });
            for &objective in AI_WAR_OBJECTIVES {
                if objective != "enemy_unit_province" {
                    vd.field_integer_range(objective, 1..=1000);
                }
            }
        });
    }
}
