use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Alert {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Alert, Alert::add)
}

impl Alert {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Alert, key, block, Box::new(Self {}));
    }
}

impl DbKind for Alert {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("alert_{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("alert_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("alert_{key}_hint");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("alert_{key}_action");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_setting_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("script_context");
        let mut sc = ScopeContext::new(Scopes::None, key);
        if let Some(script_context) = vd.field_value("script_context") {
            let mut found = false;
            for &(context, s) in SCRIPT_CONTEXTS {
                if script_context.is(context) {
                    sc = ScopeContext::new(s, key);
                    sc.define_name("actor", Scopes::Country, key);
                    found = true;
                    break;
                }
            }
            if !found {
                let msg = "unknown script context";
                err(ErrorKey::Choice).msg(msg).loc(script_context).push();
            }
        }

        vd.field_item("texture", Item::File);
        vd.field_validated_block("valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        // TODO figure out the valid panel and popup values
        vd.req_field_one_of(&["open_panel", "open_popup"]);
        vd.field_value("open_panel");
        vd.field_value("open_popup");
        vd.field_choice("type", &["alert", "important_action", "angry_important_action"]);
        vd.field_item("alert_group", Item::AlertGroup);
        vd.field_validated("color", validate_possibly_named_color);
    }
}

#[derive(Clone, Debug)]
pub struct AlertGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::AlertGroup, AlertGroup::add)
}

impl AlertGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AlertGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for AlertGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        // Just check that the alert group is empty
        Validator::new(block, data);

        let loca = format!("ag_{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("ag_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("ag_{key}_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);
    }
}

// LAST UPDATED VIC3 VERSION 1.5.13
const SCRIPT_CONTEXTS: &[(&str, Scopes)] = &[
    ("player_country", Scopes::Country),
    ("player_diplomatic_play", Scopes::DiplomaticPlay),
    ("player_diplomatic_pact", Scopes::DiplomaticPact),
    ("player_diplomatic_action", Scopes::DiplomaticAction),
    ("player_diplomatic_relations", Scopes::DiplomaticRelations),
    ("player_interest_group", Scopes::InterestGroup),
    ("player_invaded_state", Scopes::State),
    ("player_national_market", Scopes::Market),
    ("player_naval_invasion", Scopes::NavalInvasion),
    ("player_involved_market", Scopes::Market),
    ("player_state", Scopes::State),
    ("player_building", Scopes::Building),
    ("player_market_goods", Scopes::MarketGoods),
    ("player_commander", Scopes::Character),
    ("player_military_formation", Scopes::MilitaryFormation),
    ("player_trade_route", Scopes::TradeRoute),
    // undocumented from here
    ("player_front", Scopes::Front),
    ("player_war", Scopes::War),
    ("player_company", Scopes::Company),
    ("player_state_goods", Scopes::StateGoods),
];
