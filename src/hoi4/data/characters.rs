use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Character {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Character, Character::add)
}

impl Character {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("characters") {
            for (item, block) in block.drain_definitions_warn() {
                db.add(Item::Character, item, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "only `characters` is expected here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for Character {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for block in block.get_field_blocks("instance") {
            for block in block.get_field_blocks("advisor") {
                if let Some(token) = block.get_field_value("idea_token") {
                    db.add_flag(Item::CharacterIdeaToken, token.clone());
                }
            }
        }
        for block in block.get_field_blocks("advisor") {
            if let Some(token) = block.get_field_value("idea_token") {
                db.add_flag(Item::CharacterIdeaToken, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // TODO: verify that every character is reqruited exactly once

        if block.has_key("instance") {
            vd.multi_field_validated_block("instance", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_field("allowed");
                vd.field_trigger_rooted("allowed", Tooltipped::No, Scopes::Country);
                validate_character(key, block, data, &mut vd);
            });
        } else {
            validate_character(key, block, data, &mut vd);
        }
    }
}

fn validate_character(key: &Token, _block: &Block, _data: &Everything, vd: &mut Validator) {
    let mut sc = ScopeContext::new(Scopes::Country, key);

    vd.field_item("name", Item::Localization);

    vd.field_choice("gender", &["male", "female", "undefined"]);

    vd.field_trigger_rooted("available", Tooltipped::Yes, Scopes::Country);
    vd.field_trigger_rooted("allowed_civil_war", Tooltipped::No, Scopes::Country);
    vd.field_validated_block("portraits", |block, data| {
        let mut vd = Validator::new(block, data);
        for field in &["civilian", "army", "navy"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("large", Item::Sprite);
                vd.field_item("small", Item::Sprite);
            });
        }
    });

    vd.multi_field_validated_block("country_leader", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("ideology", Item::Ideology);
        vd.field_list_items("traits", Item::CountryLeaderTrait);
        vd.field_date("expire");
        vd.field_integer("id");
    });

    for field in &["field_marshal", "corps_commander", "navy_leader"] {
        vd.field_validated_block(field, |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("traits", Item::UnitLeaderTrait);
            vd.field_integer("skill");
            vd.field_integer("attack_skill");
            vd.field_integer("defense_skill");
            // TODO: not for navy
            vd.field_integer("planning_skill");
            // TODO: not for navy
            vd.field_integer("logistics_skill");
            // TODO: only for navy
            vd.field_integer("coordination_skill");
            // TODO: only for navy
            vd.field_integer("maneuvering_skill");
            vd.field_integer("legacy_id");
            vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::CombinedCountryAndCharacter);
        });
    }

    vd.multi_field_validated_block("advisor", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("slot");
        vd.req_field("idea_token");
        vd.field_item("slot", Item::AdvisorSlot);
        vd.field_value("idea_token");
        vd.field_item("name", Item::Localization);
        // TODO: only require this for theorist and high_command; ban for everyone else
        vd.field_choice(
            "ledger",
            &["army", "navy", "air", "military", "civilian", "all", "hidden"],
        );
        vd.field_trigger_rooted("allowed", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::CombinedCountryAndCharacter);
        vd.field_trigger_rooted("available", Tooltipped::Yes, Scopes::CombinedCountryAndCharacter);
        vd.field_validated_block("research_bonus", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::TechnologyCategory, |_, mut vd| {
                vd.numeric();
            });
        });
        vd.field_list_items("traits", Item::CountryLeaderTrait);
        vd.field_numeric("cost");
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_bool("can_be_fired");
        vd.field_trigger_rooted("do_effect", Tooltipped::Yes, Scopes::Country);
        vd.field_effect_rooted("on_add", Tooltipped::Yes, Scopes::CombinedCountryAndCharacter);
        vd.field_effect_rooted("on_remove", Tooltipped::Yes, Scopes::CombinedCountryAndCharacter);
    });

    vd.field_validated_block("scientist", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_list_items("traits", Item::ScientistTrait);
        // The 'visible' for scientist is different from the others, in that it has character scope
        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::Character);
        vd.field_validated_block("skills", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Specialization, key);
                value.expect_integer();
            });
        });
    });
}
