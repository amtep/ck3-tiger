use crate::block::validator::Validator;
use crate::block::Block;
use crate::ck3::data::coa::validate_coa_layout;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{old_warn, ErrorKey};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct CoaDesignerColoredEmblem {}

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

        vd.field_integer_range("colors", 0, 5);
        vd.field_bool("visible");
        if let Some(token) = vd.field_value("category") {
            let loca = format!("COA_DESIGNER_CATEGORY_{token}");
            data.verify_exists_implied(Item::Localization, &loca, token);
        }
    }
}

#[derive(Clone, Debug)]
pub struct CoaDesignerColorPalette {}

impl CoaDesignerColorPalette {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("coa_designer_background_colors") {
            for (token, block) in block.drain_definitions_warn() {
                db.add(Item::CoaDesignerColorPalette, token, block, Box::new(Self {}));
            }
        } else {
            old_warn(key, ErrorKey::UnknownField, "unknown key");
        }
    }
}

impl DbKind for CoaDesignerColorPalette {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut _vd = Validator::new(block, data);

        data.verify_exists(Item::NamedColor, key);
    }
}

#[derive(Clone, Debug)]
pub struct CoaDesignerEmblemLayout {}

impl CoaDesignerEmblemLayout {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CoaDesignerEmblemLayout, key, block, Box::new(Self {}));
    }
}

impl DbKind for CoaDesignerEmblemLayout {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        validate_coa_layout(block, data);
    }
}

#[derive(Clone, Debug)]
pub struct CoaDesignerPattern {}

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

        vd.field_integer_range("colors", 0, 5);
        vd.field_bool("visible");
    }
}
