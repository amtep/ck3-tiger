use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::data::events::Event;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_ai_chance, validate_duration, ListType};
use crate::validator::Validator;

// TODO

const EVENT_TYPES: &[&str] = &["character_event", "country_event", "state_event"];

pub fn get_event_scope(key: &Token, block: &Block) -> (Scopes, Token) {
    if let Some(event_type) = block.get_field_value("type") {
        match event_type.as_str() {
            "character_event" => (Scopes::Character, event_type.clone()),
            "country_event" => (Scopes::Country, event_type.clone()),
            "state_event" => (Scopes::State, event_type.clone()),
            _ => (Scopes::Country, key.clone()),
        }
    } else {
        (Scopes::Country, key.clone())
    }
}

pub fn validate_event(event: &Event, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(&event.block, data);

    let mut tooltipped_immediate = Tooltipped::Past;
    let mut tooltipped = Tooltipped::Yes;

    // TODO: should character_event always be hidden?
    vd.field_choice("type", EVENT_TYPES);

    vd.field_bool("orphan");
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

    vd.multi_field_validated_block("event_image", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_item("texture", Item::File);
        vd.field_item("on_created_soundeffect", Item::Sound);
    });

    vd.field_value("gui_window"); // TODO

    vd.field_item("on_created_soundeffect", Item::Sound);
    vd.field_item("on_opened_soundeffect", Item::Sound);
    vd.field_item("icon", Item::File);

    vd.field_integer("duration");

    vd.field_validated_block("cancellation_trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });

    vd.field_validated_sc("title", sc, validate_desc);
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_validated_sc("flavor", sc, validate_desc);
    vd.field_validated_block_sc("cooldown", sc, validate_duration);

    vd.field_item_or_target("minor_left_icon", sc, Item::File, Scopes::Country | Scopes::Character);
    vd.field_item_or_target(
        "minor_right_icon",
        sc,
        Item::File,
        Scopes::Country | Scopes::Character,
    );
    vd.field_item_or_target("left_icon", sc, Item::File, Scopes::Country | Scopes::Character);
    vd.field_item_or_target("right_icon", sc, Item::File, Scopes::Country | Scopes::Character);
    vd.field_item_or_target("center_icon", sc, Item::File, Scopes::Country | Scopes::Character);

    if !hidden {
        vd.req_field("option");
    }
    let mut has_options = false;
    vd.multi_field_validated_block("option", |block, data| {
        has_options = true;
        validate_event_option(block, data, sc, tooltipped);
    });
    vd.field_validated_key_block("after", |key, block, data| {
        if !has_options {
            let msg = "`after` effect will not run if there are no `option` blocks";
            let info = "you can put it in `immediate` instead";
            err(ErrorKey::Logic).msg(msg).info(info).loc(key).push();
        }
        validate_effect(block, data, sc, tooltipped);
    });
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    // TODO: warn if they use desc, first_valid, random_valid, or triggered_desc directly
    // in the name or tooltip.

    let mut vd = Validator::new(block, data);
    vd.multi_field_validated("name", |bv, data| match bv {
        BV::Value(t) => {
            data.verify_exists(Item::Localization, t);
        }
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);
            vd.req_field("text");
            vd.field_validated_block("trigger", |block, data| {
                validate_trigger(block, data, sc, Tooltipped::No);
            });
            vd.field_validated_sc("text", sc, validate_desc);
        }
    });

    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
    // undocumented
    vd.field_validated_block("show_as_unavailable", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });

    vd.field_bool("default_option");
    vd.field_bool("highlighted_option");
    vd.field_bool("fallback");
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
