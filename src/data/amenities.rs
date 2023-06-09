use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_cost;

#[derive(Clone, Debug)]
pub struct Amenity {}

impl Amenity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Amenity, key, block, Box::new(Self {}));
    }
}

impl DbKind for Amenity {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_value("default", |_, token, _| {
            if !block.has_key(token.as_str()) {
                warn(token, ErrorKey::MissingItem, "default not found in amenity");
            }
        });
        for (key, bv) in vd.unknown_keys() {
            if let Some(block) = bv.expect_block() {
                validate_amenity_setting(key, block, data);
            }
        }
    }
}

fn validate_amenity_setting(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    data.verify_exists(Item::Localization, key);

    vd.field_validated_block_rooted("cost", Scopes::Character, validate_cost);

    vd.field_validated_block_rooted("owner_modifier", Scopes::Character, |block, data, sc| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, sc, vd);
    });
    vd.field_item("owner_modifier_description", Item::Localization);

    vd.field_validated_block_rooted(
        "courtier_guest_modifier",
        Scopes::Character,
        |block, data, sc| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, sc, vd);
        },
    );
    vd.field_item("courtier_guest_modifier_description", Item::Localization);

    vd.field_script_value_rooted("ai_will_do", Scopes::Character);

    vd.field_validated_block_rooted("can_pick", Scopes::Character, |block, data, sc| {
        validate_normal_trigger(block, data, sc, true);
    });
}
