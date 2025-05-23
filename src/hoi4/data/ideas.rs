use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::validate::validate_equipment_bonus;
use crate::hoi4::validate::validate_rules;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Idea {
    category: Token,
}
#[derive(Clone, Debug)]
pub struct IdeaCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Idea, Idea::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::IdeaCategory, IdeaCategory::add)
}

impl Idea {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("ideas") {
            for (category, mut block) in block.drain_definitions_warn() {
                for (key, block) in block.drain_definitions_warn() {
                    db.add(Item::Idea, key, block, Box::new(Self { category: category.clone() }));
                }
            }
        } else {
            let msg = "unexpected key";
            let info = "only `ideas` is expected here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl IdeaCategory {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("idea_categories") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::IdeaCategory, key, block, Box::new(Self {}));
            }
        } else if key.is("slot_ledgers") {
            // TODO
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for Idea {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        if !data.item_exists(Item::IdeaCategory, self.category.as_str())
            && !data.item_exists(Item::AdvisorSlot, self.category.as_str())
        {
            let msg = "category not found as idea category or advisor slot";
            warn(ErrorKey::MissingItem).msg(msg).loc(&self.category).push();
        }

        vd.field_choice("ledger", &["army", "navy", "air"]);

        if let Some(name) = vd.field_value("name") {
            data.verify_exists(Item::Localization, name);
            let loca = format!("{name}_desc");
            data.localization.suggest(&loca, name);
        } else if !data.item_exists(Item::AdvisorSlot, self.category.as_str()) {
            data.verify_exists(Item::Localization, key);
            let loca = format!("{key}_desc");
            data.localization.suggest(&loca, key);
        }

        if let Some(token) = vd.field_value("picture") {
            let sprite = format!("GFX_idea_{token}");
            data.verify_exists_implied(Item::Sprite, &sprite, token);
        }

        vd.field_trigger_rooted("allowed", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("allowed_civil_war", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("available", Tooltipped::Yes, Scopes::Country);
        vd.field_integer("cost");
        vd.field_integer("removal_cost");

        vd.field_trigger_rooted("cancel", Tooltipped::Yes, Scopes::Country);
        vd.field_trigger_rooted("do_effect", Tooltipped::No, Scopes::Country);
        vd.field_effect_rooted("on_add", Tooltipped::Yes, Scopes::Country);
        vd.field_effect_rooted("on_remove", Tooltipped::Yes, Scopes::Country);

        vd.field_validated_block("research_bonus", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::TechnologyCategory, |_, mut vd| {
                vd.numeric();
            });
        });
        vd.field_list_items("traits", Item::CountryLeaderTrait);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.multi_field_validated_block("targeted_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_value("tag", |_, mut vd| {
                vd.maybe_item(Item::CountryTag);
                vd.variable(&mut sc);
            });
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_validated_block("equipment_bonus", validate_equipment_bonus);

        vd.field_validated_block("rule", validate_rules);

        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}

impl DbKind for IdeaCategory {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for value in block.get_field_values("slot") {
            db.add_flag(Item::IdeaCategory, value.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_bool("hidden");
        vd.field_integer("cost");
        vd.field_integer("removal_cost");
        vd.field_choice(
            "ledger",
            &["army", "air", "navy", "military", "civilian", "all", "hidden", "invalid"],
        );

        // TODO: what are these?
        vd.multi_field_value("slot");
        vd.multi_field_item("character_slot", Item::AdvisorSlot);

        vd.field_choice("type", &["army_spirit", "air_spirit", "navy_spirit", "national_spirit"]);
        vd.field_bool("politics_tab");
    }
}
