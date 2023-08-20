use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Party {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Party, Party::add)
}

impl Party {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Party, key, block, Box::new(Self {}));
    }
}

impl DbKind for Party {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut country_sc = ScopeContext::new(Scopes::Country, key);
        let mut ig_sc = ScopeContext::new(Scopes::InterestGroup, key);

        vd.field_validated("color", validate_possibly_named_color);
        vd.field_validated_sc("name", &mut country_sc, validate_desc);

        vd.field_validated_block("valid_for_country", |block, data| {
            validate_trigger(block, data, &mut country_sc, Tooltipped::No);
        });
        vd.field_validated_block("available_for_interest_group", |block, data| {
            validate_trigger(block, data, &mut ig_sc, Tooltipped::No);
        });

        ig_sc.define_name("number", Scopes::Value, key);
        vd.field_script_value("join_weight", &mut ig_sc);

        // TODO: what else is allowed here?
        vd.field_validated_block("icon", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("default", Item::File);
        });
        vd.field_list_items("unlocking_technologies", Item::Technology);
    }
}
