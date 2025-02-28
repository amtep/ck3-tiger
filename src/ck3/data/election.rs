use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Election {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SuccessionElection, Election::add)
}

impl Election {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SuccessionElection, key, block, Box::new(Self {}));
    }
}

impl DbKind for Election {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("title", Scopes::LandedTitle, key);
        sc.define_name("holder", Scopes::Character, key);

        vd.field_validated_block("candidates", |block, data| {
            let mut vd = Validator::new(block, data);
            validate_candidates(&mut vd, &mut sc);
        });

        vd.field_validated_block("electors", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("max");
            vd.field_validated_block_sc("priority", &mut sc, validate_modifiers_with_base);
            validate_candidates(&mut vd, &mut sc);
        });

        vd.field_validated_block_sc("elector_vote_strength", &mut sc, validate_modifiers_with_base);

        sc.define_name("candidate", Scopes::Character, key);
        sc.define_name("holder_candidate", Scopes::Character, key);
        vd.field_validated_block_sc("candidate_score", &mut sc, validate_modifiers_with_base);
    }
}

fn validate_candidates(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.multi_field_validated("add", |bv, data| match bv {
        BV::Value(token) => {
            if !CANDIDATE_TYPES.contains(&token.as_str()) {
                let msg = "unknown candidate category";
                warn(ErrorKey::Choice).msg(msg).loc(token).push();
            }
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.field_choice("type", CANDIDATE_TYPES);
            vd.field_validated_block("limit", |block, data| {
                validate_trigger(block, data, sc, Tooltipped::No);
            });
        }
    });
    vd.field_validated_block("limit", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
}

const CANDIDATE_TYPES: &[&str] = &[
    "title_claimants",
    "title_dejure_vassals",
    "holder",
    "holder_direct_vassals",
    "holder_spouses",
    "holder_close_family",
    "holder_close_or_extended_family",
    "holder_dynasty",
];
