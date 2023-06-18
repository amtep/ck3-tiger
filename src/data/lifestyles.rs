use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct Lifestyle {}

impl Lifestyle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Lifestyle, key, block, Box::new(Self {}));
    }
}

impl DbKind for Lifestyle {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca = format!("{key}_name");
        data.localization.verify_exists_implied(&loca, key);
        let loca = format!("{key}_desc");
        data.localization.verify_exists_implied(&loca, key);
        let loca = format!("{key}_highlight_desc");
        data.localization.verify_exists_implied(&loca, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());

        vd.field_validated_block("is_highlighted", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("is_valid_showing_failures_only", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        let icon = vd.field_value("icon").unwrap_or(key);
        if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|LIFESTYLE_ICON_PATH")
        {
            let pathname = format!("{icon_path}/{icon}.dds");
            data.fileset.verify_exists_implied(&pathname, icon);
        }

        vd.field_numeric("xp_per_level");
        vd.field_numeric("base_xp_gain");
    }
}
