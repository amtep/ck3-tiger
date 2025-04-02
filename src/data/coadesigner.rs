use crate::block::Block;
#[cfg(feature = "ck3")]
use crate::data::coa::validate_coa_layout;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
#[cfg(feature = "ck3")]
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CoaDesignerColoredEmblem {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::modern(), Item::CoaDesignerColoredEmblem, CoaDesignerColoredEmblem::add)
}

impl CoaDesignerColoredEmblem {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CoaDesignerColoredEmblem, key, block, Box::new(Self {}));
    }
}

impl DbKind for CoaDesignerColoredEmblem {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let pathname = format!("{}{key}", Item::CoaDesignerColoredEmblem.path());
        data.verify_exists_implied(Item::File, &pathname, key);

        vd.field_integer_range("colors", 0..=5);
        vd.field_bool("visible");
        if let Some(token) = vd.field_value("category") {
            let loca = format!("COA_DESIGNER_CATEGORY_{token}");
            data.verify_exists_implied(Item::Localization, &loca, token);
        }
    }
}

#[derive(Clone, Debug)]
#[cfg(feature = "ck3")]
pub struct CoaDesignerColorPalette {}

#[cfg(feature = "ck3")]
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CoaDesignerColorPalette, CoaDesignerColorPalette::add)
}

#[cfg(feature = "ck3")]
impl CoaDesignerColorPalette {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("coa_designer_background_colors") {
            for (token, block) in block.drain_definitions_warn() {
                db.add(Item::CoaDesignerColorPalette, token, block, Box::new(Self {}));
            }
        } else {
            warn(ErrorKey::UnknownField).msg("unknown key").loc(key).push();
        }
    }
}

#[cfg(feature = "ck3")]
impl DbKind for CoaDesignerColorPalette {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut _vd = Validator::new(block, data);

        data.verify_exists(Item::NamedColor, key);
    }
}

#[derive(Clone, Debug)]
#[cfg(feature = "ck3")]
pub struct CoaDesignerEmblemLayout {}

#[cfg(feature = "ck3")]
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CoaDesignerEmblemLayout, CoaDesignerEmblemLayout::add)
}

#[cfg(feature = "ck3")]
impl CoaDesignerEmblemLayout {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CoaDesignerEmblemLayout, key, block, Box::new(Self {}));
    }
}

#[cfg(feature = "ck3")]
impl DbKind for CoaDesignerEmblemLayout {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        validate_coa_layout(block, data);
    }
}

#[derive(Clone, Debug)]
pub struct CoaDesignerPattern {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::CoaDesignerPattern, CoaDesignerPattern::add)
}

impl CoaDesignerPattern {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CoaDesignerPattern, key, block, Box::new(Self {}));
    }
}

impl DbKind for CoaDesignerPattern {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let pathname = format!("{}{key}", Item::CoaDesignerPattern.path());
        data.verify_exists_implied(Item::File, &pathname, key);

        vd.field_integer_range("colors", 0..=5);
        vd.field_bool("visible");
    }
}
