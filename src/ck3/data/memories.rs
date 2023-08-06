use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_duration;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MemoryType {}

impl MemoryType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(vec) = block.get_field_list("categories") {
            for token in vec {
                db.add_flag(Item::MemoryCategory, token.clone());
            }
        }
        db.add(Item::MemoryType, key, block, Box::new(Self {}));
    }
}

impl DbKind for MemoryType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::CharacterMemory, key);
        sc.define_name("owner", Scopes::Character, key);

        // undocumented
        if let Some(icon) = vd.field_value("icon") {
            let pathname = format!("gfx/interface/icons/memory_types/{icon}");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }

        vd.field_list("categories");
        vd.field_validated_list("participants", |token, _data| {
            sc.define_name(token.as_str(), Scopes::Character, token);
        });

        data.verify_exists(Item::Localization, key);
        vd.field_validated_sc("description", &mut sc, validate_desc);
        vd.field_validated_sc("second_perspective_description", &mut sc, validate_desc);
        vd.field_validated_sc("third_perspective_description", &mut sc, validate_desc);

        if !block.has_key("duration") {
            let msg = "field `duration` missing";
            let info = "without a duration field, the duration is only 1 day";
            err(ErrorKey::FieldMissing).msg(msg).info(info).loc(block).push();
        }
        vd.field_validated_block_rooted("duration", Scopes::Character, validate_duration);
    }

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if let Some(vec) = block.get_field_list("participants") {
            for token in vec {
                if token.is(property) {
                    return true;
                }
            }
        }
        false
    }
}
