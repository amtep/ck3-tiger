use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_optional_duration;

#[derive(Clone, Debug)]
pub struct Story {}

impl Story {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Story, key, block, Box::new(Self {}));
    }
}

impl DbKind for Story {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_key_block("on_setup", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::StoryCycle, key.clone());
            sc.define_name("story", Scopes::StoryCycle, key.clone());
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_end", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::StoryCycle, key.clone());
            sc.define_name("story", Scopes::StoryCycle, key.clone());
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_owner_death", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::StoryCycle, key.clone());
            sc.define_name("story", Scopes::StoryCycle, key.clone());
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_blocks("effect_group", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::StoryCycle, key.clone());
            sc.define_name("story", Scopes::StoryCycle, key.clone());
            let mut vd = Validator::new(block, data);
            validate_optional_duration(&mut vd, &mut sc);

            vd.field_validated("days", |bv, data| match bv {
                BV::Value(token) => {
                    token.expect_integer();
                }
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    vd.req_tokens_integers_exactly(2);
                }
            });
            vd.field_integer("chance");

            vd.field_validated_block_rooted("trigger", Scopes::StoryCycle, |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            });

            validate_complex_effect(&mut vd);
        });
    }
}

fn validate_complex_effect(vd: &mut Validator) {
    vd.field_validated_blocks("first_valid", |block, data| {
        let mut vd = Validator::new(block, data);
        validate_complex_effect(&mut vd);
    });
    vd.field_validated_blocks("random_valid", |block, data| {
        let mut vd = Validator::new(block, data);
        validate_complex_effect(&mut vd);
    });
    vd.field_validated_blocks("triggered_effect", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_rooted("trigger", Scopes::StoryCycle, |block, data, sc| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block_rooted("effect", Scopes::StoryCycle, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
    });
}
