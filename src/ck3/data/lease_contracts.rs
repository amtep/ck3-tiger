use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct LeaseContract {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::LeaseContract, LeaseContract::add)
}

// TODO: verify somewhere that `theocracy_lease` is a defined item.

impl LeaseContract {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LeaseContract, key, block, Box::new(Self {}));
    }
}

impl DbKind for LeaseContract {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        if key.is("theocracy_lease") {
            vd.req_field("hierarchy");
            vd.field_validated_block("hierarchy", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_trigger_rooted("ruler_valid", Tooltipped::No, Scopes::Character);
                vd.field_trigger_builder("liege_or_vassal_valid", Tooltipped::No, |key| {
                    let mut sc = ScopeContext::new(Scopes::Character, key);
                    sc.define_name("target", Scopes::Character, key);
                    sc
                });
                vd.field_trigger_rooted("barony_valid", Tooltipped::No, Scopes::LandedTitle);
                let mut sc = ScopeContext::new(Scopes::Character, key);
                vd.field_target("lessee", &mut sc, Scopes::Character);
            });
        } else {
            vd.ban_field("hierarchy", || "theocracy_lease");
        }

        vd.field_item("government", Item::GovernmentType); // undocumented
        vd.field_list_items("valid_holdings", Item::HoldingType);
        vd.field_integer("ruler_share_min_opinion_from_lessee");
        vd.field_choice("hook_strength_max_opinion", &["none", "any", "strong"]);

        for field in &["tax", "levy"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                if key.is("theocracy_lease") {
                    vd.field_integer_range("lease_liege", 0..=100);
                } else {
                    // Technically it just requires a hierarchy definition,
                    // but hierarchy is only valid for theocracy_lease.
                    vd.ban_field("lease_liege", || "theocracy_lease");
                }
                vd.field_validated_key("rest", |key, bv, data| match bv {
                    BV::Value(token) => {
                        let mut vd = ValueValidator::new(token, data);
                        vd.choice(&["ruler", "lessee"]);
                    }
                    BV::Block(block) => {
                        let mut vd = Validator::new(block, data);
                        vd.field_integer_range("max", 0..=100);
                        let mut sc = ScopeContext::new(Scopes::None, key);
                        sc.define_name("ruler", Scopes::Character, key);
                        sc.define_name("lessee", Scopes::Character, key);
                        vd.field_validated_block_sc(
                            "weight",
                            &mut sc,
                            validate_modifiers_with_base,
                        );
                        vd.field_choice("beneficiary", &["ruler", "lessee"]);
                        vd.field_choice("rest", &["ruler", "lessee"]);
                    }
                });
            });
        }
    }
}
