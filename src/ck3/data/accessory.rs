use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{error, old_warn, ErrorKey};
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

#[derive(Clone, Debug)]
pub struct AccessoryVariation {}

impl AccessoryVariation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("variation") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::AccessoryVariation, name.clone(), block, Box::new(Self {}));
            } else {
                error(key, ErrorKey::FieldMissing, "variation without a name");
            }
        } else if key.is("pattern_textures") {
            if let Some(name) = block.get_field_value("name") {
                db.add_exact_dup_ok(
                    Item::AccessoryVariationTextures,
                    name.clone(),
                    block,
                    Box::new(AccessoryVariationTextures {}),
                );
            } else {
                let msg = "pattern_textures without a name";
                error(key, ErrorKey::FieldMissing, msg);
            }
        } else if key.is("pattern_layout") {
            if let Some(name) = block.get_field_value("name") {
                db.add_exact_dup_ok(
                    Item::AccessoryVariationLayout,
                    name.clone(),
                    block,
                    Box::new(AccessoryVariationLayout {}),
                );
            } else {
                error(key, ErrorKey::FieldMissing, "pattern_layout without a name");
            }
        } else {
            old_warn(key, ErrorKey::UnknownField, "unknown variation type");
        }
    }
}

impl DbKind for AccessoryVariation {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_validated_blocks("pattern", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("weight");
            for field in &["r", "g", "b", "a"] {
                vd.field_validated_block(field, |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_item("textures", Item::AccessoryVariationTextures);
                    vd.field_item("layout", Item::AccessoryVariationLayout);
                });
            }
        });
        vd.field_validated_blocks("color_palette", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("weight");
            vd.field_item("texture", Item::File);
        });
    }
}

#[derive(Clone, Debug)]
pub struct AccessoryVariationTextures {}

impl DbKind for AccessoryVariationTextures {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value("name");
        vd.req_field("colormask");
        vd.field_item("colormask", Item::File);
        vd.req_field("normal");
        vd.field_item("normal", Item::File);
        vd.req_field("properties");
        vd.field_item("properties", Item::File);
    }
}

#[derive(Clone, Debug)]
pub struct AccessoryVariationLayout {}

impl DbKind for AccessoryVariationLayout {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value("name");
        vd.req_field("scale");
        vd.field_validated_block("scale", validate_minmax);
        vd.req_field("rotation");
        vd.field_validated_block("rotation", validate_minmax);
        vd.req_field("offset");
        vd.field_validated_block("offset", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("x", validate_minmax);
            vd.field_validated_block("y", validate_minmax);
        });
    }
}

fn validate_minmax(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("min");
    vd.field_numeric("min");
    vd.req_field("max");
    vd.field_numeric("max");
}
