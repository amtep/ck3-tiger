use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{error, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GameRule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::GameRule, GameRule::add)
}

impl GameRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for (key, _) in block.iter_definitions() {
            if !key.is("categories") {
                db.add_flag(Item::GameRuleSetting, key.clone());
            }
        }
        db.add(Item::GameRule, key, block, Box::new(Self {}));
    }
}

const RULE_FLAGS: &[&str] =
    &["blocks_achievements", "no_end_date", "no_diplomatic_range", "restricted_diplomatic_range"];

impl DbKind for GameRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("rule_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("categories", |block, data| {
            let mut vd = Validator::new(block, data);
            for token in vd.values() {
                let loca = format!("game_rule_category_{token}");
                data.verify_exists_implied(Item::Localization, &loca, token);
            }
        });

        if let Some(token) = vd.field_value("default") {
            if token.is("categories") || block.get_field_block(token.as_str()).is_none() {
                let msg = "this rule does not have that setting";
                error(token, ErrorKey::MissingItem, msg);
            }
        }

        vd.unknown_block_fields(|setting, block| {
            let mut vd = Validator::new(block, data);
            let loca = format!("setting_{setting}");
            data.verify_exists_implied(Item::Localization, &loca, setting);
            let loca = format!("setting_{setting}_desc");
            data.verify_exists_implied(Item::Localization, &loca, setting);
            if let Some(token) = vd.field_value("apply_modifier") {
                if let Some((category, modifier)) = token.split_once(':') {
                    if !category.is("player") && !category.is("ai") && !category.is("all") {
                        let msg = "expected player: ai: or all:";
                        error(category, ErrorKey::Validation, msg);
                    }
                    data.verify_exists(Item::Modifier, &modifier);
                } else {
                    let msg = "expected format category:modifier";
                    error(token, ErrorKey::Validation, msg);
                }
            }

            vd.field_validated_block("defines", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.unknown_block_fields(|group, block| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_fields(|key, _| {
                        let define_key = format!("{group}|{key}");
                        data.verify_exists_implied(Item::Define, &define_key, key);
                    });
                });
            });

            vd.multi_field_choice("flag", RULE_FLAGS);
        });
    }
}
