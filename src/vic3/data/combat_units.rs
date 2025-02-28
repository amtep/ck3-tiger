use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CombatUnit {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CombatUnit, CombatUnit::add)
}

impl CombatUnit {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatUnit, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatUnit {
    // This whole item type is undocumented.
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.req_field("group");
        vd.req_field("combat_unit_image");

        vd.field_item("group", Item::CombatUnitGroup);
        vd.field_validated_block("battle_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Unit, vd);
        });
        vd.field_validated_block("upkeep_modifier", |block, data| {
            let vd = Validator::new(block, data);
            // The upkeep modifier gets applied to the country so it can
            // actually take a variety of `ModifKinds`.
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_validated_block("formation_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::MilitaryFormation, vd);
        });
        vd.field_validated_key_block("can_build_conscript", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_list_items("unlocking_technologies", Item::Technology);
        let mut seen_unconditional = None;
        vd.multi_field_validated_key_block("combat_unit_image", |key, block, data| {
            if let Some(unconditional) = &seen_unconditional {
                let msg = "there was a previous `combat_unit_image` without a trigger, so this one will not be used";
                let info = "try moving that one to the end";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).loc_msg(unconditional, "previous").push();
            }
            let mut vd = Validator::new(block, data);
            vd.field_validated_key_block("trigger", |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Culture, key);
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
            if !block.has_key("trigger") {
                seen_unconditional = Some(key.clone());
            }
            vd.field_item("texture", Item::File);
        });
        if seen_unconditional.is_none() {
            let msg = "there should be a `combat_unit_image` with no trigger as a fallback";
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
        }
        vd.field_list_items("upgrades", Item::CombatUnit);
    }
}

#[derive(Clone, Debug)]
pub struct CombatUnitGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CombatUnitGroup, CombatUnitGroup::add)
}

impl CombatUnitGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatUnitGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatUnitGroup {
    // This whole item type is undocumented.
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::TextIcon, key);
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.field_choice("type", &["army", "navy"]);
        vd.field_integer("manpower_max");
        vd.field_bool("default_group");
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("icon", Item::File);
    }
}

#[derive(Clone, Debug)]
pub struct CombatUnitExperienceLevel {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CombatUnitExperienceLevel, CombatUnitExperienceLevel::add)
}

impl CombatUnitExperienceLevel {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatUnitExperienceLevel, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatUnitExperienceLevel {
    // This whole item type is undocumented.
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        // TODO: is there a requirement for levels to be consecutive?
        vd.field_integer("level");
        vd.field_item("icon", Item::File);
        vd.field_integer("needed_experience");
        vd.field_validated_block("unit_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Unit, vd);
        });
    }
}
