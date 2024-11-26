use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;
use crate::vic3::tables::misc::STRATA;

#[derive(Clone, Debug)]
pub struct SocialHierarchy {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::SocialHierarchy, SocialHierarchy::add)
}

impl SocialHierarchy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SocialHierarchy, key, block, Box::new(Self {}));
    }
}

impl DbKind for SocialHierarchy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_bool("is_default");
        vd.field_validated_key_block("pop_criteria", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
    }
}

#[derive(Clone, Debug)]
pub struct SocialClass {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::SocialClass, SocialClass::add)
}

impl SocialClass {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SocialClass, key, block, Box::new(Self {}));
    }
}

impl DbKind for SocialClass {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("social_hierarchy", Item::SocialHierarchy);
        vd.field_choice("strata", STRATA);
        vd.field_list_items("allowed_professions", Item::PopType);
        vd.field_validated_key_block("pop_criteria", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}
