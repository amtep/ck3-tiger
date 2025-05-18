use crate::block::{Block, BV};
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
pub struct Story {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Story, Story::add)
}

impl Story {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Story, key, block, Box::new(Self {}));
    }
}

impl DbKind for Story {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let sc_builder = |key: &Token| {
            let mut sc = ScopeContext::new(Scopes::StoryCycle, key);
            sc.define_name("story", Scopes::StoryCycle, key);
            sc
        };

        vd.field_effect_builder("on_setup", Tooltipped::No, sc_builder);
        vd.field_effect_builder("on_end", Tooltipped::No, sc_builder);
        vd.field_effect_builder("on_owner_death", Tooltipped::No, sc_builder);

        vd.multi_field_validated_key_block("effect_group", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::StoryCycle, key);
            sc.define_name("story", Scopes::StoryCycle, key);
            let mut vd = Validator::new(block, data);

            // TODO: handle case of multiple of these fields being specified
            for field in &["days", "weeks", "months", "years"] {
                vd.field_validated(field, |bv, data| match bv {
                    BV::Value(token) => {
                        token.expect_integer();
                    }
                    BV::Block(block) => {
                        let mut vd = Validator::new(block, data);
                        vd.req_tokens_integers_exactly(2);
                    }
                });
            }
            vd.field_numeric_range("chance", 0.0..=100.0);

            vd.field_trigger("trigger", Tooltipped::No, &mut sc);

            validate_complex_effect(&mut vd, &mut sc);
        });
    }
}

fn validate_complex_effect(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.multi_field_validated_block("first_valid", |block, data| {
        let mut vd = Validator::new(block, data);
        validate_complex_effect(&mut vd, sc);
    });
    vd.multi_field_validated_block("random_valid", |block, data| {
        let mut vd = Validator::new(block, data);
        validate_complex_effect(&mut vd, sc);
    });
    vd.multi_field_validated_block("triggered_effect", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_trigger("trigger", Tooltipped::No, sc);
        vd.field_effect("effect", Tooltipped::No, sc);
    });
    vd.field_effect("fallback", Tooltipped::No, sc);
}
