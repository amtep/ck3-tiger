use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::lowercase::Lowercase;
use crate::pdxfile::PdxEncoding;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::ListType;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CountryHistory {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Hoi4, Item::CountryHistory, PdxEncoding::Utf8NoBom, ".txt", LoadAsFile::Yes, Recursive::No, CountryHistory::add)
}

impl CountryHistory {
    pub fn add(db: &mut Db, file: Token, block: Block) {
        db.add(Item::CountryHistory, file, block, Box::new(Self {}));
    }
}

impl DbKind for CountryHistory {
    fn validate(&self, file: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_case_sensitive(false);

        vd.validate_history_blocks(|_, key, block, data| {
            if key.is_integer() {
                // This is actually a state id, not a date
                let mut sc = ScopeContext::new(Scopes::State, key);
                validate_effect(block, data, &mut sc, Tooltipped::No);
            } else {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                validate_history(file, block, data, vd);
            }
        });
        validate_history(file, block, data, vd);
    }
}

fn validate_history(key: &Token, block: &Block, data: &Everything, mut vd: Validator) {
    // TODO: verify that the capital is in the country
    vd.field_item("capital", Item::State);
    vd.field_integer("set_convoys");
    vd.field_numeric("starting_train_buffer");
    vd.field_item("oob", Item::UnitHistory);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    validate_effect_internal(
        Lowercase::empty(),
        ListType::None,
        block,
        data,
        &mut sc,
        &mut vd,
        Tooltipped::No,
    );
}
