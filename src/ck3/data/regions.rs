use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err, fatal};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Region {
    generates_modifiers: bool,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Region, Region::add)
}

impl Region {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let generates_modifiers = block.get_field_bool("generate_modifiers").unwrap_or(false);
        let region = Self { generates_modifiers };
        db.add(Item::Region, key, block, Box::new(region));
    }
}

impl DbKind for Region {
    fn has_property(
        &self,
        _key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "generates_modifiers" { self.generates_modifiers } else { false }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // TODO: figure out when a region needs to be localized.
        // Probably when it's tooltipped for geographical_region or when the gui code does GetName
        data.mark_used(Item::Localization, key.as_str());

        if block.field_value_is("generate_modifiers", "yes") {
            let modif = format!("{key}_development_growth");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_development_growth_factor");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        }

        vd.field_bool("generate_modifiers");
        vd.field_bool("graphical");
        vd.field_validated_block("color", validate_color);
        vd.field_validated_list("counties", |token, data| {
            if !token.starts_with("c_") {
                let msg = "only counties can be listed in the counties field";
                err(ErrorKey::Validation).msg(msg).loc(token).push();
            }
            data.verify_exists(Item::Title, token);
        });
        vd.field_validated_list("duchies", |token, data| {
            if !token.starts_with("d_") {
                let msg = "only duchies can be listed in the duchies field";
                err(ErrorKey::Validation).msg(msg).loc(token).push();
            }
            data.verify_exists(Item::Title, token);
        });
        vd.field_list_items("provinces", Item::Province);
        vd.field_validated_list("regions", |token, data| {
            if !data.item_exists(Item::Region, token.as_str()) {
                let msg =
                    format!("{} {} not defined in {}", Item::Region, token, Item::Region.path());
                let info = "this will cause a crash";
                fatal(ErrorKey::Crash).strong().msg(msg).info(info).loc(token).push();
            }
        });
    }
}
