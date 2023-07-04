use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct ProductionMethod {}

impl ProductionMethod {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProductionMethod, key, block, Box::new(Self {}));
    }
}

impl DbKind for ProductionMethod {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("texture", Item::File);
        vd.field_bool("is_default");

        vd.field_validated_block("country_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::Country);
        });
        vd.field_validated_block("state_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::State);
        });
        vd.field_validated_block("building_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::Building);
        });
        vd.field_validated_block("timed_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::Building);
        });

        vd.field_list_items("unlocking_laws", Item::Law);
        vd.field_list_items("disallowing_laws", Item::Law);
        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_list_items("unlocking_production_methods", Item::ProductionMethod);
        vd.field_list_items("unlocking_global_technologies", Item::Technology);

        vd.field_numeric("ai_weight");
        vd.field_numeric("pollution_generation");
    }
}

fn validate_modifier_block(block: &Block, data: &Everything, kinds: ModifKinds) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("workforce_scaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
    vd.field_validated_block("level_scaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
    vd.field_validated_block("unscaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
}
