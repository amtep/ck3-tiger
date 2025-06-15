use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DivisionNamesGroup {}
#[derive(Clone, Debug)]
pub struct RailwayGunNames {}
#[derive(Clone, Debug)]
pub struct ShipNames {}
#[derive(Clone, Debug)]
pub struct UnitNames {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::DivisionNamesGroup, DivisionNamesGroup::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::RailwayGunNames, RailwayGunNames::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::ShipNames, ShipNames::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::UnitNames, UnitNames::add)
}

impl DivisionNamesGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DivisionNamesGroup, key, block, Box::new(Self {}));
    }
}
impl RailwayGunNames {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::RailwayGunNames, key, block, Box::new(Self {}));
    }
}
impl ShipNames {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipNames, key, block, Box::new(Self {}));
    }
}
impl UnitNames {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::UnitNames, key, block, Box::new(Self {}));
    }
}

impl DbKind for DivisionNamesGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_list("division_types"); // TODO: where are these defined?
        vd.field_list_items("link_numbering_with", Item::DivisionNamesGroup);

        validate_names_group(block, data, vd, "division");
    }
}

impl DbKind for RailwayGunNames {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("type"); // always "railway_gun" in vanilla
        vd.field_list_items("link_numbering_with", Item::RailwayGunNames);

        validate_names_group(block, data, vd, "railway gun");
    }
}

impl DbKind for ShipNames {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("type"); // always "ship" in vanilla
        vd.field_validated_list("ship_types", |value, data| {
            if !data.item_exists(Item::SubUnit, value.as_str())
                && !data.item_exists(Item::Equipment, value.as_str())
            {
                let msg = format!("{value} not found as sub unit or equipment");
                warn(ErrorKey::Validation).msg(msg).loc(value).push();
            }
        });
        vd.field_value("prefix");
        vd.field_list("unique");
        vd.field_list_items("link_numbering_with", Item::ShipNames);

        validate_names_group(block, data, vd, "ship");
    }
}

impl DbKind for UnitNames {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !key.is("generic") {
            data.verify_exists(Item::CountryTag, key);
        }

        vd.field_item("air_wing_names_template", Item::Localization);
        vd.field_item("fleet_names_template", Item::Localization);
        vd.field_item("task_force_names_template", Item::Localization);

        vd.unknown_block_fields(|key, block| {
            // TODO: "Air wings can only be named through archetype"
            // The rest should be SubUnit items.
            if !data.item_exists(Item::SubUnit, key.as_str())
                && !data.item_exists(Item::Equipment, key.as_str())
            {
                let msg = format!("{key} not found as sub unit or equipment");
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
            }
            let mut vd = Validator::new(block, data);
            vd.field_value("prefix");
            vd.field_list("generic");
            vd.field_item("generic_pattern", Item::Localization);
            vd.field_list("unique");
        });
    }
}

fn validate_names_group(_block: &Block, _data: &Everything, mut vd: Validator, what: &str) {
    vd.field_value("name");
    vd.field_list_items("for_countries", Item::CountryTag);
    vd.field_trigger_rooted("can_use", Tooltipped::No, Scopes::Country);
    vd.field_value("fallback_name");
    // TODO: verify format of `unordered`. No examples in vanilla.
    // unordered having the same format as ordered is implied by how Kaiserreich uses it.
    for field in &["unordered", "ordered"] {
        vd.field_validated_block(field, |block, data| {
            let mut vd = Validator::new(block, data);
            let mut seen = TigerHashSet::default();
            vd.unknown_block_fields(|key, block| {
                if key.expect_integer().is_some() {
                    if let Some(other) = seen.replace(key.clone()) {
                        let msg = format!("duplicate index in ordered {what} names");
                        let msg2 = "the other one is here";
                        warn(ErrorKey::DuplicateField)
                            .msg(msg)
                            .loc(key)
                            .loc_msg(other, msg2)
                            .push();
                    }
                }
                let mut vd = Validator::new(block, data);
                let values = vd.values();
                if values.is_empty() {
                    let msg = format!("{what} name missing");
                    warn(ErrorKey::Validation).msg(msg).loc(block).push();
                } else if values.len() > 2 {
                    let msg = "too many values";
                    let info = "did you forget to quote the name?";
                    warn(ErrorKey::Validation).msg(msg).info(info).loc(values[2]).push();
                }
                // The first value is an arbitrary name, so nothing to validate.
                // The second value is an optional loca key for the description.
                if let Some(opt_loca) = values.get(1) {
                    data.verify_exists(Item::Localization, opt_loca);
                }
                // The third value is an optional URL.
                // TODO: validate the URL
            });
        });
    }
}
