use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct RaidIntent {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::RaidIntent, RaidIntent::add)
}

impl RaidIntent {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::RaidIntent, key, block, Box::new(Self {}));
    }
}

impl DbKind for RaidIntent {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_loot");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_flavor");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_effect_builder("on_return_raid_loot", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Army, key);
            sc.define_name("raid_loot", Scopes::Value, key);
            sc.define_name("raider", Scopes::Character, key);
            sc
        });

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_script_value_rooted("ai_will_do", Scopes::Character);
        vd.field_trigger_rooted("is_shown", Tooltipped::No, Scopes::Character);
        vd.field_trigger_rooted("is_valid", Tooltipped::Yes, Scopes::Character);
        vd.field_effect_builder("on_invalidated", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Army, key);
            sc.define_name("raider", Scopes::Character, key);
            sc
        });
    }
}
