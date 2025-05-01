use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct LineType {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::LineType, PdxEncoding::Utf8Bom, ".lines", LoadAsFile::No, Recursive::No, LineType::add)
}

impl LineType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("line") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::LineType, name.clone(), block, Box::new(Self {}));
            } else {
                let msg = "line without name";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `line`";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for LineType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");

        vd.field_numeric_range("curvature", 0.0..=1.0);
        vd.field_numeric("max_height"); // TODO: possibly this is an int
        vd.field_numeric("fade_in");
        vd.field_numeric("fade_out");
        vd.field_integer("priority");
        vd.field_numeric("progress_animation_time");
        vd.field_bool("progress_animation_looping");

        vd.multi_field_validated_block("layer", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("diffuse", Item::File);
            vd.field_item("mask", Item::File);
            vd.field_value("shader"); // TODO what are the options here
            vd.field_integer("priority");
            vd.field_validated_block("tintcolor", validate_color);
            vd.field_validated_block("width", validate_zoom_levels);
            vd.field_validated_block("opacity", validate_zoom_levels);
            vd.field_list_numeric_exactly("mask_uv_scale", 2);
            vd.field_list_numeric_exactly("uv_scale", 2);
            vd.field_list_numeric_exactly("animation_speed", 2);
        });
    }
}

fn validate_zoom_levels(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.field_integer("zoom_step");
        vd.field_numeric("value");
    }
}
