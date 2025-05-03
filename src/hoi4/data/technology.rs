use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Technology {}
#[derive(Clone, Debug)]
pub struct TechnologyCategory {}
#[derive(Clone, Debug)]
pub struct TechnologyFolder {}
#[derive(Clone, Debug)]
pub struct TechnologySharing {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Technology, Technology::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::TechnologyCategory, TechnologyCategory::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::TechnologySharing, TechnologySharing::add)
}

impl Technology {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("technologies") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::Technology, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `technologies` here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

/// Loads both technology categories and technology folders
impl TechnologyCategory {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("technology_categories") {
            for value in block.iter_values() {
                db.add_flag(Item::TechnologyCategory, value.clone());
            }
            db.set_flag_validator(Item::TechnologyCategory, |flag, data| {
                data.verify_exists(Item::Localization, flag);
            });
        } else if key.is("technology_folders") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::TechnologyFolder, key, block, Box::new(TechnologyFolder {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `technology_categories` or `technology_folders` here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl TechnologySharing {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("technology_sharing_group") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::TechnologySharing, id.clone(), block, Box::new(Self {}));
            }
        } else {
            let msg = "technology sharing group without id";
            err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
        }
    }
}
impl DbKind for Technology {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("doctrine_name", Item::Localization);
        vd.field_bool("show_equipment_icon");
        vd.field_choice("xp_research_type", &["air", "army", "navy"]);
        vd.field_integer("xp_boost_cost");
        vd.field_integer("xp_unlock_cost");
        vd.field_list_items("xor", Item::Technology);

        vd.field_list_items("enable_equipments", Item::EquipmentBonusType);
        vd.field_list_items("enable_equipment_modules", Item::EquipmentModule);
        vd.field_list_items("enable_subunits", Item::SubUnit);

        vd.field_trigger_rooted("on_research_complete_limit", Scopes::Country, Tooltipped::No);
        vd.field_effect_rooted("on_research_complete", Scopes::Country, Tooltipped::Yes);

        vd.multi_field_validated_block("path", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("leads_to_tech", Item::Technology);
            vd.field_numeric("research_cost_coeff");
        });

        vd.field_bool("doctrine");
        vd.field_bool("show_effect_as_desc");
        vd.field_numeric("research_cost");
        vd.field_integer("start_year");

        vd.field_list_items("special_project_specialization", Item::Specialization);

        vd.field_list_items("categories", Item::TechnologyCategory);
        vd.field_validated_block("folder", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::TechnologyFolder);
            vd.field_validated_block("position", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer("x");
                vd.field_integer("y");
            });
        });

        let mut sc = ScopeContext::new(Scopes::Country, key);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        // TODO: this seems to contain a variety of different items. Not sure what the rules are.
        vd.field_block("ai_research_weights");

        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}

impl DbKind for TechnologyFolder {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_choice("ledger", &["army", "navy", "air", "civilian", "hidden"]);
        vd.field_trigger_rooted("available", Scopes::Country, Tooltipped::No);
        vd.field_bool("doctrine");
    }
}

impl DbKind for TechnologySharing {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("id");
        vd.field_item("name", Item::Localization);
        vd.field_item("desc", Item::Localization);
        vd.field_item("picture", Item::Sprite);

        vd.field_numeric("research_sharing_per_country_bonus");
        vd.field_trigger_rooted("available", Scopes::Country, Tooltipped::No);
    }
}
