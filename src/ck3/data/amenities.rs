use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{old_warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

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
                old_warn(token, ErrorKey::MissingItem, "default not found in amenity");
            }
        });
        vd.unknown_block_fields(|key, block| {
            validate_amenity_setting(key, block, data);
        });
    }
}

fn validate_amenity_setting(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    data.verify_exists(Item::Localization, key);

    vd.field_validated_block_rooted("cost", Scopes::Character, validate_cost);

    vd.field_validated_block("owner_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_item("owner_modifier_description", Item::Localization);

    vd.field_validated_block("courtier_guest_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_item("courtier_guest_modifier_description", Item::Localization);

    vd.field_script_value_rooted("ai_will_do", Scopes::Character);

    vd.field_validated_block_rooted("can_pick", Scopes::Character, |block, data, sc| {
        validate_trigger(block, data, sc, Tooltipped::Yes);
    });
}
