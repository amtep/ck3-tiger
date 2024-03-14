use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Accessory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::Accessory, Accessory::add)
}

impl Accessory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(token) = block.get_field_value("set_tags") {
            for tag in token.split(',') {
                db.add_flag(Item::AccessoryTag, tag);
            }
        }
        // For some reason I can't get the tags to load from common/genes properly for imperator, so im hacking them in here instead for now.
        #[cfg(feature = "imperator")]
        for tag in &["no_hair", "fat2_normal", "fat2_max", "fat1_normal", "fat1_max", "no_fat"] {
            db.add_flag(Item::AccessoryTag, Token::new(tag, block.loc));
        }
        db.add(Item::Accessory, key, block, Box::new(Self {}));
    }
}

impl DbKind for Accessory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.multi_field_validated_block("entity", |block, data| {
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

        for token in vd.multi_field_value("set_tags") {
            for _tag in token.split(',') {
                // TODO (these tags are mentioned in required_tags in gene settings)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccessoryVariation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::AccessoryVariation, AccessoryVariation::add)
}

impl AccessoryVariation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("variation") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::AccessoryVariation, name.clone(), block, Box::new(Self {}));
            } else {
                err(ErrorKey::FieldMissing).msg("variation without a name").loc(key).push();
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
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
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
                err(ErrorKey::FieldMissing).msg("pattern_layout without a name").loc(key).push();
            }
        } else {
            warn(ErrorKey::UnknownField).msg("unknown variation type").loc(key).push();
        }
    }
}

impl DbKind for AccessoryVariation {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.multi_field_validated_block("pattern", |block, data| {
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
        vd.multi_field_validated_block("color_palette", |block, data| {
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
    // TODO: verify max >= min
}
