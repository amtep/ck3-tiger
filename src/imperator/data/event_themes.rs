use crate::block::Block;
use crate::block::BV;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::imperator::data::event_pictures::verify_exists_or_empty;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;
use crate::Severity;

#[derive(Clone, Debug)]
pub struct EventTheme {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::EventTheme, EventTheme::add)
}

impl EventTheme {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventTheme, key, block, Box::new(Self {}));
    }
}

impl DbKind for EventTheme {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(t) => verify_exists_or_empty(data, t, Severity::Error),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_item("texture", Item::File);
                vd.field_validated_block("trigger", |block, data| {
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });
            }
        });
        vd.field_item("soundeffect", Item::Sound);
    }
}
