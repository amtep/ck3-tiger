use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PrincipleGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PrincipleGroup, PrincipleGroup::add)
}

impl PrincipleGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PrincipleGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for PrincipleGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.multi_field_item("blocking_identity", Item::PowerBlocIdentity);
        vd.multi_field_item("unlocking_identity", Item::PowerBlocIdentity);
        vd.multi_field_item("primary_for_identity", Item::PowerBlocIdentity);

        vd.req_field("levels");
        vd.field_list_items("levels", Item::Principle);
    }
}

#[derive(Clone, Debug)]
pub struct Principle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Principle, Principle::add)
}

impl Principle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Principle, key, block, Box::new(Self {}));
    }
}

impl DbKind for Principle {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_validated_block("visible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.multi_field_item("incompatible_with", Item::Principle);
        vd.field_item("icon", Item::File);
        vd.field_item("background", Item::File);

        vd.field_validated_block("power_bloc_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::PowerBloc, vd);
        });

        vd.advice_field(
            "participant_modifier",
            "docs say participant_modifier but it's member_modifier",
        );
        for field in &["member_modifier", "leader_modifier", "non_leader_modifier"] {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Country, vd);
            });
        }

        vd.field_item("institution", Item::Institution);
        vd.field_validated_block("institution_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        // undocumented

        vd.field_script_value_full("ai_weight", Scopes::Country, false);
    }
}
