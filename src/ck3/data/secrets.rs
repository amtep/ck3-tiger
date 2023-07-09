use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct Secret {}

impl Secret {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Secret, key, block, Box::new(Self {}));
    }
}

impl DbKind for Secret {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("secret_owner", Scopes::Character, key);
        sc.define_name("secret_target", Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_tooltip_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_type_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        if let Some(token) = vd.field_value("category") {
            let loca = format!("secret_category_{token}");
            data.verify_exists_implied(Item::Localization, &loca, token);
            let pathname = format!("gfx/interface/icons/secret_categories/{token}.dds");
            data.verify_exists_implied(Item::File, &pathname, token);
        }

        vd.field_validated_block("is_valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_block("is_shunned", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("target", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("is_criminal", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("target", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        let mut sc = ScopeContext::new(Scopes::Secret, key);
        sc.define_name("secret_owner", Scopes::Character, key);
        sc.define_name("secret_target", Scopes::Character, key);
        vd.field_validated_key_block("on_discover", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("discoverer", Scopes::Character, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_expose", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("secret_exposer", Scopes::Character, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("on_owner_death", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}
