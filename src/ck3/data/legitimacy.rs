use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::Everything;

#[derive(Clone, Debug)]
pub struct LegitimacyType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::LegitimacyType, LegitimacyType::add)
}

impl LegitimacyType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LegitimacyType, key, block, Box::new(Self {}));
    }
}

impl DbKind for LegitimacyType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for block in &block.get_field_blocks("level") {
            for &flag in &block.get_field_values("flag") {
                db.add_flag(Item::LegitimacyFlag, flag.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca = format!("{key}_type_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.field_trigger_rooted("is_valid", Tooltipped::No, Scopes::Character);
        vd.field_script_value_no_breakdown_builder("ai_expected_level", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("liege", Scopes::Character, key);
            sc
        });
        vd.field_script_value_builder("below_expectations_opinion", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_script_value_rooted("max", Scopes::Character);
        vd.multi_field_validated_block("level", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_script_value_rooted("threshold", Scopes::Character);
            vd.field_validated_block("modifier", |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, vd);
            });
            vd.multi_field_value("flag");
        });
    }
}
