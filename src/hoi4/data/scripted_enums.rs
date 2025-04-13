use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ScriptedEnum {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::ScriptedEnum, ScriptedEnum::add)
}

impl ScriptedEnum {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("script_enum_operative_mission_type") {
            for value in block.iter_values() {
                db.add_flag(Item::Mission, value.clone());
            }
        } else if key.is("script_enum_advisor_slot_type") {
            for value in block.iter_values() {
                db.add_flag(Item::AdvisorSlot, value.clone());
            }
        } else if key.is("script_enum_equipment_stat") {
            for value in block.iter_values() {
                db.add_flag(Item::EquipmentStat, value.clone());
            }
        } else if key.is("script_enum_production_stat") {
            for value in block.iter_values() {
                db.add_flag(Item::ProductionStat, value.clone());
            }
        } else if key.is("script_enum_equipment_category") {
            for value in block.iter_values() {
                db.add_flag(Item::EquipmentCategory, value.clone());
            }
        } else if key.is("script_enum_equipment_bonus_type") {
            for value in block.iter_values() {
                db.add_flag(Item::EquipmentBonusType, value.clone());
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(&key).push();
        }
        db.add(Item::ScriptedEnum, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedEnum {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let check_loca =
            key.is("script_enum_advisor_slot_type") || key.is("script_enum_equipment_bonus_type");
        let check_desc = key.is("script_enum_equipment_bonus_type");
        for value in vd.values() {
            if check_loca {
                data.verify_exists(Item::Localization, value);
            }
            if check_desc {
                let loca = format!("{value}_desc");
                data.verify_exists_implied(Item::Localization, &loca, value);
            }
        }
    }
}
