use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_duration;
use crate::validator::{Builder, Validator};

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
        let sc_no_lobby: &Builder = &|key: &Token| {
            let mut sc = ScopeContext::new(Scopes::InterestGroup, key);
            sc.define_name("country", Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            sc
        };

        let sc_with_lobby: &Builder = &|key: &Token| {
            let mut sc = sc_no_lobby(key);
            sc.define_name("political_lobby", Scopes::PoliticalLobby, key);
            sc
        };

        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("category", &["foreign_pro_country", "foreign_anti_country", "foreign"]);
        vd.field_item("texture", Item::File);

        vd.field_trigger_full("can_create", sc_no_lobby, Tooltipped::No);
        vd.field_effect_full("on_created", sc_with_lobby, Tooltipped::No);

        vd.field_validated_block("requirement_to_maintain", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger_full("trigger", sc_with_lobby, Tooltipped::No);
            vd.field_effect_full("on_failed", sc_with_lobby, Tooltipped::No);
            vd.field_item("swap_type_on_failed", Item::PoliticalLobby);
        });

        let appeasement_factors_validation = |value: &Token, data: &Everything| {
            data.verify_exists(Item::PoliticalLobbyAppeasement, value);
            if data.item_has_property(
                Item::PoliticalLobbyAppeasement,
                value.as_str(),
                "is_always_usable",
            ) {
                let msg = "cannot contain appeasement factors marked as `is_always_usable`";
                warn(ErrorKey::Validation).msg(msg).loc(value).push();
            }
        };

        vd.field_validated_list("appeasement_factors_pro", appeasement_factors_validation);
        vd.field_validated_list("appeasement_factors_anti", appeasement_factors_validation);

        vd.field_trigger_full(
            "available_for_interest_group",
            Scopes::InterestGroup,
            Tooltipped::No,
        );

        vd.field_script_value_full("join_weight", sc_with_lobby, false);
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

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        block.get_field_bool(property).unwrap_or_default()
    }
}
