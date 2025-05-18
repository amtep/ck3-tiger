use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Amenity {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Amenity, Amenity::add)
}

impl Amenity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Amenity, key, block, Box::new(Self {}));
    }
}

impl DbKind for Amenity {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (key, block) in block.iter_definitions() {
            db.add(Item::AmenitySetting, key.clone(), block.clone(), Box::new(AmenitySetting {}));
        }
    }

    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        if let Some(value) = vd.field_value("default") {
            if !block.has_key(value.as_str()) {
                err(ErrorKey::MissingItem).msg("default not found in amenity").loc(value).push();
            }
        }
        // validated in AmenitySetting::validate
        vd.unknown_block_fields(|_, _| {});
    }
}

#[derive(Clone, Debug)]
pub struct AmenitySetting {}

impl DbKind for AmenitySetting {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
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
        vd.field_trigger_rooted("can_pick", Tooltipped::Yes, Scopes::Character);
    }
}
