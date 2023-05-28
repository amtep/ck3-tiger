use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct Region {
    generates_modifiers: bool,
}

impl Region {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let generates_modifiers = block.get_field_bool("generate_modifiers").unwrap_or(false);
        let region = Self {
            generates_modifiers,
        };
        db.add(Item::Region, key, block, Box::new(region));
    }
}

impl DbKind for Region {
    fn has_property(
        &self,
        property: &str,
        _key: &Token,
        _block: &Block,
        _data: &Everything,
    ) -> bool {
        if property == "generates_modifiers" {
            self.generates_modifiers
        } else {
            false
        }
    }

    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_bool("generate_modifiers");
        vd.field_bool("graphical");
        vd.field_validated_block("color", validate_color);
        vd.field_validated_list("counties", |token, data| {
            if !token.starts_with("c_") {
                let msg = "only counties can be listed in the counties field";
                error(token, ErrorKey::Validation, msg);
            }
            data.verify_exists(Item::Title, token);
        });
        vd.field_validated_list("duchies", |token, data| {
            if !token.starts_with("d_") {
                let msg = "only duchies can be listed in the duchies field";
                error(token, ErrorKey::Validation, msg);
            }
            data.verify_exists(Item::Title, token);
        });
        vd.field_list_items("provinces", Item::Province);
        vd.field_validated_list("regions", |token, data| {
            if !data.item_exists(Item::Region, token.as_str()) {
                let msg = format!(
                    "{} {} not defined in {}",
                    Item::Region,
                    token,
                    Item::Region.path()
                );
                let info = "this will cause a crash";
                error_info(token, ErrorKey::Crash, &msg, info);
            }
        });
    }
}
