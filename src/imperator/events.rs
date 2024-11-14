use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::events::Event;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_ai_chance, validate_modifiers_with_base, ListType};
use crate::validator::Validator;

const EVENT_TYPES: &[&str] = &[
    "character_event",
    "minor_character_event",
    "country_event",
    "minor_country_event",
    "major_country_event",
    "state_event",
    "province_event",
];

pub fn get_event_scope(key: &Token, block: &Block) -> (Scopes, Token) {
    if let Some(event_type) = block.get_field_value("type") {
        match event_type.as_str() {
            "minor_character_event" | "character_event" => (Scopes::Character, event_type.clone()),
            "country_event" | "minor_country_event" | "major_country_event" => {
                (Scopes::Country, event_type.clone())
            }
            "state_event" => (Scopes::State, event_type.clone()),
            "province_event" => (Scopes::Province, event_type.clone()),
            _ => (Scopes::Country, key.clone()),
        }
    } else {
        (Scopes::Country, key.clone())
    }
}

pub fn validate_event(event: &Event, data: &Everything) {
    let mut vd = Validator::new(&event.block, data);
    let mut sc = event.sc();

    let mut tooltipped_immediate = Tooltipped::Past;
    let mut tooltipped = Tooltipped::Yes;

    vd.field_choice("type", EVENT_TYPES);
    vd.field_bool("hidden");
    vd.field_bool("interface_lock");
    vd.field_bool("fire_only_once");
    vd.field_item_or_target(
        "goto_location",
        &mut sc,
        Item::Province,
        Scopes::Province.union(Scopes::Country),
    );

    vd.field_validated_sc("title", &mut sc, validate_desc);
    vd.field_validated_sc("desc", &mut sc, validate_desc);

    let hidden = event.block.field_value_is("hidden", "yes");
    if hidden {
        tooltipped_immediate = Tooltipped::No;
        tooltipped = Tooltipped::No;
    }

    let mut minor_event = false;
    if event.block.field_value_is("type", "minor_character_event")
        || event.block.field_value_is("type", "minor_country_event")
    {
        minor_event = true;
    }

    if !hidden && !minor_event {
        vd.req_field("picture");
    }
    vd.field_item("picture", Item::EventPicture);

    for field in &["left_portrait", "right_portrait"] {
        let mut count = 0;
        vd.multi_field_validated_value(field, |_, mut vd| {
            count += 1;
            vd.target_ok_this(&mut sc, Scopes::Character);
            if count == 4 {
                let msg = format!("Event has more than 3 {field} attributes.");
                let info = "Events can only have up to 3 portraits displayed at a time.";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(&event.key).push();
            }
        });
    }

    vd.field_validated_block_sc("weight_multiplier", &mut sc, validate_modifiers_with_base);

    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("immediate", |block, data| {
        validate_effect(block, data, &mut sc, tooltipped_immediate);
    });

    if !hidden {
        vd.req_field("option");
    }
    vd.multi_field_validated_block("option", |block, data| {
        validate_event_option(block, data, &mut sc, tooltipped);
    });

    vd.field_validated_block("after", |block, data| {
        validate_effect(block, data, &mut sc, tooltipped_immediate);
    });
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_sc("name", sc, validate_desc);

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });

    vd.field_bool("exclusive");
    vd.field_bool("highlight");
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
