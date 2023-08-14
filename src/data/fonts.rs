use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::pdxfile::PdxEncoding;
use crate::report::{untidy, warn, ErrorKey, Severity};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Font {}

inventory::submit! {
    ItemLoader::Full(GameFlags::all(), Item::Font, PdxEncoding::Utf8OptionalBom, ".font", false, Font::add)
}

impl Font {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("fontfiles") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::Fontfiles, name.clone(), block, Box::new(Fontfiles {}));
            } else {
                let msg = "fontfiles entry without name";
                warn(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else if key.is("font") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::Font, name.clone(), block, Box::new(Self {}));
            } else {
                let msg = "font entry without name";
                warn(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = format!("unknown entry type {key}");
            untidy(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for Font {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);
        vd.field_value("name");
        vd.field_validated_blocks("fontstyle", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_value("style", |_, value, _data| {
                for style in value.split('|') {
                    if !(style.is("regular")
                        || style.is("bold")
                        || style.is("extrabold")
                        || style.is("italic"))
                    {
                        let msg = format!("unknown font style {style}");
                        warn(ErrorKey::Choice).msg(msg).loc(style).push();
                    }
                }
            });
            vd.field_item("fontfiles", Item::Fontfiles);
        });
        vd.field_validated_blocks("underlineformats", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_key, block| {
                // only known key is "default" but there may be others
                let mut vd = Validator::new(block, data);
                vd.field_numeric("thickness");
                vd.field_numeric("offset");
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct Fontfiles {}

impl DbKind for Fontfiles {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);
        vd.field_value("name");
        vd.field_bool("always_load");

        vd.field_validated_blocks("group", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list("languages"); // TODO
            vd.field_list_items("files", Item::File);
        });
    }
}
