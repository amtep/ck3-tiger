use std::cell::RefCell;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct EventTheme {
    validated_scopes: RefCell<Scopes>,
}

impl EventTheme {
    pub fn new() -> Self {
        let validated_scopes = RefCell::new(Scopes::empty());
        Self { validated_scopes }
    }

    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventTheme, key, block, Box::new(Self::new()));
    }
}

impl DbKind for EventTheme {
    fn validate(&self, _key: &Token, _block: &Block, _data: &Everything) {}

    /// Themes are unusual in that they are validated through the events that use them.
    /// This means that unused themes are not validated, which is ok.
    /// The purpose is to allow the triggers to be validated in the context of the scope
    /// of the event that uses them.
    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _caller: &Token,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        // Check if the passed-in scope type has already been validated for
        if self.validated_scopes.borrow().contains(sc.scopes()) {
            return;
        }
        *self.validated_scopes.borrow_mut() |= sc.scopes();

        let mut vd = Validator::new(block, data);

        vd.req_field("background");
        vd.req_field("icon");
        vd.req_field("sound");

        vd.field_validated_blocks("background", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::EventBackground);
        });

        vd.field_validated_blocks("icon", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::File);
        });

        vd.field_validated_blocks("sound", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            // TODO: figure out a way to get a list of all available sounds
            vd.field_value("reference");
        });

        // `transition` is not documented but presumably works the same way
        vd.field_validated_blocks("transition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::EventTransition);
        });
    }
}
