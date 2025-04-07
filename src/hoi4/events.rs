use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::hoi4::data::events::Event;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_ai_chance, validate_modifiers, ListType};
use crate::validator::Validator;

// TODO

const EVENT_TYPES: &[&str] = &["country_event", "news_event", "state_event", "unit_leader_event"];

pub fn get_event_scope(key: &Token, _block: &Block) -> (Scopes, Token) {
    match key.as_str() {
        "country_event" | "news_event" => (Scopes::Country, key.clone()),
        "state_event" => (Scopes::State, key.clone()),
        "unit_leader_event" => (Scopes::Character, key.clone()),
        _ => (Scopes::Country, key.clone()),
    }
}

pub fn validate_event(event: &Event, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(&event.block, data);

    let mut tooltipped_immediate = Tooltipped::Past;
    let mut tooltipped = Tooltipped::Yes;

    if !EVENT_TYPES.contains(&event.key.as_str()) {
        let msg = format!("expected one = {}", EVENT_TYPES.join(", "));
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
            vd.field_trigger_full("trigger", &mut *sc, Tooltipped::No);
        }
    });
    vd.field_item("picture", Item::Sprite);

    vd.field_bool("fire_only_once");
    vd.field_bool("minor_flavor");
    vd.field_bool("major");
    vd.field_bool("is_triggered_only");
    vd.field_bool("hidden");
    let hidden = event.block.field_value_is("hidden", "yes");
    if hidden {
        tooltipped_immediate = Tooltipped::No;
        tooltipped = Tooltipped::No;
    }

    vd.field_item("dlc", Item::Dlc);

    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
    vd.field_validated_block("immediate", |block, data| {
        validate_effect(block, data, sc, tooltipped_immediate);
    });
    vd.field_validated_block("mean_time_to_happen", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_integer("days");
        vd.field_integer("months");
        vd.field_integer("years");
        vd.multi_field_numeric("add");
        vd.multi_field_numeric("factor");
        validate_modifiers(&mut vd, sc);
    });

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

    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });

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
