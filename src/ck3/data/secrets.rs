use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Secret {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Secret, Secret::add)
}

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
            data.verify_icon("NGameIcons|SECRET_TYPE_PATH", token, ".dds");
        }

        vd.field_trigger("is_valid", Tooltipped::No, &mut sc.clone());

        for field in &["is_shunned", "is_criminal"] {
            vd.field_trigger_builder(field, Tooltipped::No, |key| {
                let mut sc = sc.clone();
                sc.define_name("target", Scopes::Character, key);
                sc
            });
        }

        let mut sc = ScopeContext::new(Scopes::Secret, key);
        sc.define_name("secret_owner", Scopes::Character, key);
        sc.define_name("secret_target", Scopes::Character, key);

        vd.field_effect_builder("on_discover", Tooltipped::No, |key| {
            let mut sc = sc.clone();
            sc.define_name("discoverer", Scopes::Character, key);
            sc
        });
        vd.field_effect_builder("on_expose", Tooltipped::No, |key| {
            let mut sc = sc.clone();
            sc.define_name("secret_exposer", Scopes::Character, key);
            sc
        });

        vd.field_effect("on_owner_death", Tooltipped::No, &mut sc);
    }
}
