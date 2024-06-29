use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_duration;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PoliticalLobby {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PoliticalLobby, PoliticalLobby::add)
}

impl PoliticalLobby {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoliticalLobby, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoliticalLobby {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        fn sc_no_lobby(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::InterestGroup, key);
            sc.define_name("country", Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            sc
        }

        fn sc_with_lobby(key: &Token) -> ScopeContext {
            let mut sc = sc_no_lobby(key);
            sc.define_name("political_lobby", Scopes::PoliticalLobby, key);
            sc
        }

        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("category", &["foreign_pro_country", "foreign_anti_country", "foreign"]);
        vd.field_item("texture", Item::File);

        vd.field_validated_key_block("can_create", |key, block, data| {
            validate_trigger(block, data, &mut sc_no_lobby(key), Tooltipped::No);
        });

        vd.field_validated_key_block("on_created", |key, block, data| {
            validate_effect(block, data, &mut sc_with_lobby(key), Tooltipped::No);
        });

        vd.field_validated_key_block("requirement_to_maintain", |key, block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |block, data| {
                validate_trigger(block, data, &mut sc_with_lobby(key), Tooltipped::No);
            });
            vd.field_validated_block("on_failed", |block, data| {
                validate_effect(block, data, &mut sc_with_lobby(key), Tooltipped::No);
            });
            vd.field_item("swap_type_on_failed", Item::PoliticalLobby);
        });

        // TODO: validate "cannot contain appeasement factors marked as is_always_usable"
        vd.field_list_items("appeasement_factors_pro", Item::PoliticalLobbyAppeasement);
        vd.field_list_items("appeasement_factors_anti", Item::PoliticalLobbyAppeasement);

        vd.field_validated_key_block("available_foR_interest_group", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::InterestGroup, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_script_value("join_weight", &mut sc_with_lobby(key));
    }
}

#[derive(Clone, Debug)]
pub struct PoliticalLobbyAppeasement {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PoliticalLobbyAppeasement, PoliticalLobbyAppeasement::add)
}

impl PoliticalLobbyAppeasement {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoliticalLobbyAppeasement, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoliticalLobbyAppeasement {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // TODO: verify if script values are allowed in these durations at all
        let mut sc = ScopeContext::new(Scopes::PoliticalLobby, key);

        vd.field_validated_block_sc("duration_to_show", &mut sc, validate_duration);
        vd.field_bool("is_always_usable");
    }
}
