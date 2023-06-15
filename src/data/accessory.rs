use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Accessory {}

impl Accessory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(token) = block.get_field_value("set_tags") {
            for tag in token.split(',') {
                db.add_flag(Item::AccessoryTag, tag);
            }
        }
        db.add(Item::Accessory, key, block, Box::new(Self {}));
    }
}

impl DbKind for Accessory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_blocks("entity", |block, data| {
            let mut vd = Validator::new(block, data);
            if let Some(token) = vd.field_value("required_tags") {
                for tag in token.split(',') {
                    if !tag.is("") {
                        data.verify_exists(Item::AccessoryTag, &tag);
                    }
                }
            }
            vd.field_choice("shared_pose_entity", &["head", "torso"]);
            vd.field_value("node"); // TODO
            vd.field_value("game_entity_override"); // TODO
            vd.field_bool("inherit_rotation");
            vd.field_item("entity", Item::Entity);
        });

        for token in vd.field_values("set_tags") {
            for _tag in token.split(',') {
                // TODO (these tags are mentioned in required_tags in gene settings)
            }
        }
    }
}
