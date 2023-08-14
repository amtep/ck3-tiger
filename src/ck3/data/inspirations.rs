use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Inspiration {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Inspiration, Inspiration::add)
}

impl Inspiration {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Inspiration, key, block, Box::new(Self {}));
    }
}

impl DbKind for Inspiration {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Inspiration, key);
        sc.define_name("inspiration", Scopes::Inspiration, key);
        sc.define_name("inspiration_owner", Scopes::Character, key);
        sc.define_name("inspiration_sponsor", Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_script_value("gold", &mut sc);
        vd.field_script_value("progress_chance", &mut sc);

        for field in &[
            "on_creation",
            "on_complete",
            "on_monthly",
            "on_sponsor",
            "on_owner_death",
            "on_invalidated",
            "on_sponsor_invalidated",
            "on_progress_increased", // undocumented
        ] {
            vd.field_validated_block(field, |block, data| {
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        }

        for field in &["is_valid", "is_sponsor_valid", "can_sponsor"] {
            vd.field_validated_block(field, |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
        }
    }
}
