use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CourtierGuestManagement {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CourtierGuestManagement, CourtierGuestManagement::add)
}

impl CourtierGuestManagement {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtierGuestManagement, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtierGuestManagement {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if key.is("courtier_management") {
            let sc_builder = |key: &Token| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                sc.define_name("courtier", Scopes::Character, key);
                sc.define_name("liege", Scopes::Character, key);
                sc
            };

            vd.field_validated_key_block("can_leave", |key, block, data| {
                validate_trigger(block, data, &mut sc_builder(key), Tooltipped::No);
            });
            vd.field_validated_key_block("monthly_leave_chance_x10", |key, block, data| {
                validate_modifiers_with_base(block, data, &mut sc_builder(key));
            });
        } else if key.is("guest_management") {
            let sc_builder = |key: &Token| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                sc.define_name("guest", Scopes::Character, key);
                sc.define_name("host", Scopes::Character, key);
                sc
            };
            vd.field_validated_key_block("guest_can_arrive", |key, block, data| {
                validate_trigger(block, data, &mut sc_builder(key), Tooltipped::No);
            });
            vd.field_validated_key_block("guest_score", |key, block, data| {
                validate_modifiers_with_base(block, data, &mut sc_builder(key));
            });
            vd.field_validated_key_block("can_leave", |key, block, data| {
                validate_trigger(block, data, &mut sc_builder(key), Tooltipped::No);
            });
            vd.field_validated_key_block("monthly_leave_chance_x10", |key, block, data| {
                validate_modifiers_with_base(block, data, &mut sc_builder(key));
            });
            vd.field_validated_block("guest_description", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.multi_field_validated_block("description", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("desc");
                    vd.field_validated_key_block("limit", |key, block, data| {
                        validate_trigger(block, data, &mut sc_builder(key), Tooltipped::No);
                    });
                    vd.field_validated_key_block("weight", |key, block, data| {
                        validate_modifiers_with_base(block, data, &mut sc_builder(key));
                    });
                    vd.field_validated_key("desc", |key, bv, data| {
                        validate_desc(bv, data, &mut sc_builder(key));
                    });
                });
            });
        } else {
            let msg = "expected either `courtier_management` or `guest_management`";
            err(ErrorKey::Validation).msg(msg).loc(key).push();
        }
    }
}

#[derive(Clone, Debug)]
pub struct GuestSystem {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::GuestSystem, GuestSystem::add)
}

impl GuestSystem {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GuestSystem, key, block, Box::new(Self {}));
    }
}

impl DbKind for GuestSystem {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        // TODO: the root scope is not documented here
        let mut sc = ScopeContext::new(Scopes::None, key);
        sc.define_name("mover", Scopes::Character, key);

        if key.is("destination_for_guest_entering_pool")
            || key.is("destination_for_courtier_entering_pool")
        {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        } else {
            let msg = "expected either `destination_for_guest_entering_pool` or `destination_for_courtier_entering_pool`";
            err(ErrorKey::Validation).msg(msg).loc(key).push();
        }
    }
}
