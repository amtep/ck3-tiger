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
pub struct Wargoal {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Wargoal, Wargoal::add)
}

impl Wargoal {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Wargoal, key, block, Box::new(Self {}));
    }
}

impl DbKind for Wargoal {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("defender", Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_trigger("allow", Tooltipped::Yes, &mut sc);

        vd.field_bool("uses_civil_war_conquest");
        vd.field_choice(
            "type",
            &[
                "superiority",
                "take_province",
                "naval_superiority",
                "enforce_military_access",
                "independence",
            ],
        );
        vd.field_numeric("ticking_war_score");

        vd.field_validated_block("attacker", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("conquer_cost");
        });
        vd.field_validated_block("defender", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("conquer_cost");
        });
    }
}
