use crate::block::Block;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, ErrorKey};
use crate::validator::Validator;

pub fn validate_rules(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_bool("can_create_collaboration_government");
    vd.field_bool("can_send_volunteers");
    vd.field_bool("can_create_factions");
    vd.field_bool("can_join_factions");
    vd.field_bool("can_join_opposite_factions");
    vd.field_bool("can_boost_other_ideologies");
    vd.field_bool("can_guarantee_other_ideologies");
    vd.field_bool("can_not_declare_war");
    vd.field_bool("can_decline_call_to_war");
    vd.field_bool("can_declare_war_on_same_ideology");
    vd.field_bool("can_access_market");
    vd.field_bool("can_force_government");
    vd.field_bool("can_puppet");
    vd.field_bool("can_lower_tension");
    vd.field_bool("can_only_justify_war_on_threat_country");
}

pub fn validate_equipment_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        if !data.item_exists(Item::EquipmentBonusType, key.as_str())
            && !data.item_exists(Item::EquipmentGroup, key.as_str())
        {
            let msg = format!("`{key}` not found as equipment bonus type or equipment group");
            err(ErrorKey::MissingItem).msg(msg).loc(key).push();
        }
        let mut vd = Validator::new(block, data);
        vd.field_bool("instant");
        vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
            vd.numeric();
        });
    });
}
