use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::effect::validate_effect_internal;
use crate::everything::Everything;
use crate::hoi4::data::events::Event;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_ai_chance, validate_modifiers, ListType};
use crate::validator::Validator;

// TODO

const EVENT_TYPES: &[&str] =
    &["country_event", "news_event", "state_event", "unit_leader_event", "operative_leader_event"];

pub fn get_event_scope(key: &Token, _block: &Block) -> (Scopes, Token) {
    #[allow(clippy::match_same_arms)]
    match key.as_str() {
        "country_event" | "news_event" => (Scopes::Country, key.clone()),
        "state_event" => (Scopes::CombinedCountryAndState, key.clone()),
        "unit_leader_event" | "operative_leader_event" => {
            (Scopes::CombinedCountryAndCharacter, key.clone())
        }
        _ => (Scopes::Country, key.clone()),
    }
}

pub fn validate_event(event: &Event, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(&event.block, data);

    let mut tooltipped_immediate = Tooltipped::Past;
    let mut tooltipped = Tooltipped::Yes;

    if !EVENT_TYPES.contains(&event.key.as_str()) {
        let msg = format!("expected one of {}", EVENT_TYPES.join(", "));
        err(ErrorKey::Choice).msg(msg).loc(&event.key).push();
    }

    vd.field_value("id"); // checked in add

    vd.field_item("title", Item::Localization);
    // TODO: verify whether a conditional desc after unconditional does anything
    vd.multi_field_validated("desc", |bv, data| match bv {
        BV::Value(value) => {
            data.verify_exists(Item::Localization, value);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.field_item("text", Item::Localization);
            vd.field_trigger("trigger", Tooltipped::No, sc);
        }
    });
    vd.field_item("picture", Item::Sprite);

    vd.field_bool("fire_only_once");
    vd.field_bool("fire_for_sender");
    vd.field_bool("minor_flavor");
    vd.field_bool("major");
    vd.field_bool("is_triggered_only");
    vd.field_bool("hidden");
    let hidden = event.block.get_field_bool("hidden").unwrap_or(false);
    if hidden {
        tooltipped_immediate = Tooltipped::No;
        tooltipped = Tooltipped::No;
    }

    if event.block.get_field_bool("major").unwrap_or(false) {
        vd.field_trigger("show_major", Tooltipped::No, sc);
    } else {
        vd.ban_field("show_major", || "major = yes");
    }

    vd.field_item("dlc", Item::Dlc);

    vd.field_trigger("trigger", Tooltipped::No, sc);
    vd.field_effect("immediate", tooltipped_immediate, sc);
    vd.field_validated_block("mean_time_to_happen", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_integer("days");
        vd.field_integer("months");
        vd.field_integer("years");
        vd.multi_field_numeric("add");
        vd.multi_field_numeric("factor");
        validate_modifiers(&mut vd, sc);
    });
    vd.field_integer("timeout_days");

    if !hidden {
        vd.req_field("option");
    }
    vd.multi_field_validated_block("option", |block, data| {
        validate_event_option(block, data, sc, tooltipped);
    });
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.field_item("name", Item::Localization);

    vd.field_trigger("trigger", Tooltipped::No, sc);

    vd.field_validated_sc("ai_chance", sc, validate_ai_chance);
    validate_effect_internal(
        &Lowercase::new_unchecked("option"),
        ListType::None,
        block,
        data,
        sc,
        &mut vd,
        tooltipped,
    );
}
