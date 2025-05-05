use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PowerBalance {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::PowerBalance, PowerBalance::add)
}

impl PowerBalance {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PowerBalance, key, block, Box::new(Self {}));
    }
}

impl DbKind for PowerBalance {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (key, block) in block.iter_definitions() {
            if key.is("side") {
                if let Some(id) = block.get_field_value("id") {
                    db.add_flag(Item::PowerBalanceSide, id.clone());
                }
            }
        }
        db.set_flag_validator(Item::PowerBalanceSide, |token, data| {
            data.verify_exists(Item::Localization, token);
        });
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_numeric("initial_value");
        vd.field_item("left_side", Item::PowerBalanceSide);
        vd.field_item("right_side", Item::PowerBalanceSide);
        vd.field_item("decision_category", Item::DecisionCategory);

        vd.field_validated_block("range", validate_range);
        vd.multi_field_validated_block("side", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("id");
            vd.field_item("icon", Item::Sprite);
            vd.multi_field_validated_block("range", validate_range);
        });
    }
}

fn validate_range(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_item("id", Item::Localization);
    vd.field_numeric("min");
    vd.field_numeric("max");
    vd.field_validated_block("modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::all(), vd);
    });
    vd.field_effect_rooted("on_activate", Tooltipped::Yes, Scopes::Country);
    vd.field_effect_rooted("on_deactivate", Tooltipped::Yes, Scopes::Country);
}
