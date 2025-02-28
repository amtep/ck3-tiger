use crate::block::{BV, Block};
use crate::ck3::data::legends::LegendChronicle;
use crate::ck3::tables::misc::{LEGEND_QUALITY, OUTBREAK_INTENSITIES, TITLE_HISTORY_TYPES};
use crate::ck3::validate::{
    validate_random_culture, validate_random_faith, validate_random_traits_list,
};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::effect_validation::validate_random_list;
use crate::everything::Everything;
use crate::helpers::TigerHashSet;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, err, warn};
use crate::scopes::Scopes;
use crate::script_value::{validate_non_dynamic_script_value, validate_script_value};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_target_ok_this, validate_trigger};
use crate::validate::{
    ListType, validate_duration, validate_mandatory_duration, validate_optional_duration,
    validate_optional_duration_int,
};
use crate::validator::{Builder, Validator, ValueValidator};

pub fn validate_add_activity_log_entry(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    vd.req_field("key");
    vd.req_field("character");
    if let Some(token) = vd.field_value("key") {
        let loca = format!("{token}_title");
        data.verify_exists_implied(Item::Localization, &loca, token);
    }
    vd.field_script_value("score", sc);
    vd.field_validated_block("tags", |b, data| {
        let mut vd = Validator::new(b, data);
        vd.values(); // TODO
    });
    vd.field_bool("show_in_conclusion");
    vd.field_target("character", sc, Scopes::Character);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_target("location", sc, Scopes::Province);
    vd.field_target("artifact", sc, Scopes::Artifact);
    // effects can be put directly in this block
    validate_effect_internal(&caller, ListType::None, block, data, sc, &mut vd, tooltipped);
}

pub fn validate_add_artifact_history(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("recipient");
    vd.field_item("type", Item::ArtifactHistory);
    vd.field_date("date");
    vd.field_target("actor", sc, Scopes::Character);
    vd.field_target("recipient", sc, Scopes::Character);
    vd.field_target("location", sc, Scopes::Province);
}

pub fn validate_add_artifact_title_history(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("date");
    vd.field_target("target", sc, Scopes::LandedTitle);
    vd.field_date("date");
}

pub fn validate_add_from_contribution(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_script_value("prestige", sc);
    vd.field_script_value("gold", sc);
    vd.field_script_value("piety", sc);
    vd.field_validated_block("opinion", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("modifier", Item::OpinionModifier);
    });
}

pub fn validate_add_hook(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("target");
    vd.field_item("type", Item::Hook);
    vd.field_target("target", sc, Scopes::Character);
    if let Some(token) = vd.field_value("secret") {
        if !data.item_exists(Item::Secret, token.as_str()) {
            validate_target(token, data, sc, Scopes::Secret);
        }
    }
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_add_opinion(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("modifier");
    vd.req_field("target");
    vd.field_item("modifier", Item::OpinionModifier);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_script_value("opinion", sc); // undocumented
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_add_relation_flag(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("relation");
    vd.req_field("flag");
    vd.req_field("target");
    vd.field_item("relation", Item::Relation);
    // TODO: check that the flag belongs to the relation
    vd.field_value("flag");
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_scheme_cooldown(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("type");
    vd.field_target("target", sc, Scopes::Character);
    vd.field_item("type", Item::Scheme);
    validate_optional_duration_int(&mut vd);
}

pub fn validate_scheme_modifier(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    if let Some(token) = vd.field_value("type") {
        data.verify_exists(Item::Modifier, token);
        data.database.validate_call(Item::Modifier, token, block, data, sc);
        data.database.validate_property_use(Item::Modifier, token, data, key, key.as_str());
    }
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_add_secret(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::Secret);
    vd.field_target("target", sc, Scopes::Character);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Secret, name);
    }
}

pub fn validate_guest_subset(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("target");
    vd.field_item("name", Item::GuestSubset);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_item("phase", Item::ActivityPhase);
}

pub fn validate_add_trait_xp(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("trait");
    vd.req_field("value");
    // TODO: if the trait is an Item, verify that the TraitTrack belongs to this trait
    vd.field_item_or_target("trait", sc, Item::Trait, Scopes::Trait);
    vd.field_item("track", Item::TraitTrack);
    vd.field_script_value("value", sc);
}

/// Used by a variety of `add_..._modifier` effects
pub fn validate_add_modifier(
    key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_ascii_lowercase();
    let visible = caller == "add_character_modifier"
        || caller == "add_house_modifier"
        || caller == "add_dynasty_modifier"
        || caller == "add_county_modifier"
        || caller == "add_house_unity_modifier"
        || caller == "add_legend_county_modifier"
        || caller == "add_legend_owner_modifier"
        || caller == "add_legend_province_modifier";
    match bv {
        BV::Value(token) => {
            data.verify_exists(Item::Modifier, token);
            if visible {
                data.verify_exists(Item::Localization, token);
            }
            let block = Block::new(key.loc);
            data.database.validate_call(Item::Modifier, token, &block, data, sc);
            data.database.validate_property_use(Item::Modifier, token, data, key, key.as_str());
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("modifier");
            if let Some(token) = vd.field_value("modifier") {
                data.verify_exists(Item::Modifier, token);
                if visible && !block.has_key("desc") {
                    data.verify_exists(Item::Localization, token);
                }
                data.database.validate_call(Item::Modifier, token, block, data, sc);
                data.database.validate_property_use(Item::Modifier, token, data, key, key.as_str());
            }
            vd.field_validated_sc("desc", sc, validate_desc);
            validate_optional_duration(&mut vd, sc);
        }
    }
}

pub fn validate_add_truce(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("character");
    vd.field_target("character", sc, Scopes::Character);
    vd.field_bool("override");
    vd.field_choice("result", &["white_peace", "victory", "defeat"]);
    vd.field_item("casus_belli", Item::CasusBelli);
    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_target("war", sc, Scopes::War);
    validate_optional_duration(&mut vd, sc);
    if block.has_key("war") && block.has_key("casus_belli") {
        let msg = "cannot use both `war` and `casus_belli`";
        err(ErrorKey::Validation).msg(msg).loc(block).push();
    }
}

pub fn validate_add_unity(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("value");
    vd.req_field("character");
    vd.field_script_value("value", sc);
    vd.field_target("character", sc, Scopes::Character);
    vd.field_validated_sc("desc", sc, validate_desc);
}

pub fn validate_assign_council_task(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("council_task");
    vd.req_field("target");
    vd.field_target("council_task", sc, Scopes::CouncilTask);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_bool("fire_on_actions");
}

pub fn validate_assign_councillor_type(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("target");
    vd.field_item("type", Item::CouncilPosition);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_bool("fire_on_actions");
    vd.field_bool("remove_existing_councillor");
}

pub fn validate_battle_event(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("left_portrait");
    vd.req_field("key");
    if let Some(token) = vd.field_value("key") {
        let loca = format!("{token}_friendly");
        data.verify_exists_implied(Item::Localization, &loca, token);
        let loca = format!("{token}_enemy");
        data.verify_exists_implied(Item::Localization, &loca, token);
    }
    vd.field_target("left_portrait", sc, Scopes::Character);
    vd.field_target("right_portrait", sc, Scopes::Character);
    vd.field_value("type"); // TODO, undocumented
    vd.field_bool("target_right"); // undocumented
}

pub fn validate_change_cultural_acceptance(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("value");
    vd.field_target("target", sc, Scopes::Culture);
    vd.field_script_value("value", sc);
    vd.field_validated_sc("desc", sc, validate_desc);
}

pub fn validate_change_liege(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("liege");
    vd.req_field("change");
    vd.field_target("liege", sc, Scopes::Character);
    vd.field_target("change", sc, Scopes::TitleAndVassalChange);
}

pub fn validate_change_struggle_phase(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            data.verify_exists(Item::StrugglePhase, token);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("struggle_phase");
            vd.req_field("with_transition");
            vd.field_item("struggle_phase", Item::StrugglePhase);
            vd.field_bool("with_transition");
        }
    }
}

pub fn validate_change_struggle_phase_duration(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("duration");
    vd.field_validated_block_sc("duration", sc, |block, data, sc| {
        if let Some(bv) = block.get_field("points") {
            validate_script_value(bv, data, sc);
        } else {
            validate_duration(block, data, sc);
        }
    });
}

pub fn validate_change_title_holder(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("holder");
    vd.req_field("change");
    vd.field_target("holder", sc, Scopes::Character);
    vd.field_target("change", sc, Scopes::TitleAndVassalChange);
    vd.field_bool("take_baronies");
    vd.field_target("government_base", sc, Scopes::Character);
}

pub fn validate_change_trait_rank(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_ascii_lowercase();
    vd.req_field("trait");
    vd.req_field("rank");
    // TODO: check that it's a rankable trait
    vd.field_item("trait", Item::Trait);
    vd.field_script_value("rank", sc);
    if caller == "change_trait_rank" {
        vd.field_script_value("max", sc);
    }
}

pub fn validate_copy_localized_text(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("key");
    vd.req_field("target");
    vd.field_value("key");
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_create_accolade(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("knight");
    vd.req_field("primary");
    vd.req_field("secondary");
    vd.field_target("knight", sc, Scopes::Character);
    vd.field_item("primary", Item::AccoladeType);
    vd.field_item("secondary", Item::AccoladeType);
    vd.field_item("name", Item::Localization);
}

pub fn validate_create_artifact(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_ascii_lowercase();
    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_validated_sc("description", sc, validate_desc);
    vd.field_item("rarity", Item::ArtifactRarity);
    vd.field_item("type", Item::ArtifactType);
    vd.multi_field_item("modifier", Item::Modifier);
    vd.field_script_value("durability", sc);
    vd.field_script_value("max_durability", sc);
    vd.field_bool("decaying");
    vd.multi_field_validated_block_sc("history", sc, validate_artifact_history);
    vd.field_item("template", Item::ArtifactTemplate);
    vd.field_item("visuals", Item::ArtifactVisual);
    vd.field_bool("generate_history");
    vd.field_script_value("quality", sc);
    vd.field_script_value("wealth", sc);
    vd.field_target("creator", sc, Scopes::Character);
    vd.field_target(
        "visuals_source",
        sc,
        Scopes::LandedTitle | Scopes::Dynasty | Scopes::DynastyHouse,
    );

    if caller == "create_artifact" {
        if let Some(name) = vd.field_value("save_scope_as") {
            sc.define_name_token(name.as_str(), Scopes::Artifact, name);
        }
        vd.field_target("title_history", sc, Scopes::LandedTitle);
        vd.field_date("title_history_date");
    } else {
        vd.ban_field("save_scope_as", || "`create_artifact`");
        vd.ban_field("title_history", || "`create_artifact`");
        vd.ban_field("title_history_date", || "`create_artifact`");
    }
}

pub fn validate_create_character(
    _key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    // docs say save_event_target instead of save_scope
    vd.replaced_field("save_event_target_as", "save_scope_as");
    vd.replaced_field("save_temporary_event_target_as", "save_temporary_scope_as");
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Character, name);
    }
    if let Some(name) = vd.field_value("save_temporary_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Character, name);
    }

    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_script_value("age", sc);
    if let Some(token) = vd.field_value("gender") {
        if !token.is("male") && !token.is("female") {
            validate_target_ok_this(token, data, sc, Scopes::Character);
        }
    }
    vd.field_script_value("gender_female_chance", sc);
    vd.field_target_ok_this("opposite_gender", sc, Scopes::Character);
    vd.multi_field_item("trait", Item::Trait);
    vd.multi_field_validated_block_sc("random_traits_list", sc, validate_random_traits_list);
    vd.field_bool("random_traits");
    vd.field_script_value("health", sc);
    vd.field_script_value("fertility", sc);
    vd.field_target_ok_this("mother", sc, Scopes::Character);
    vd.field_target_ok_this("father", sc, Scopes::Character);
    vd.field_target_ok_this("real_father", sc, Scopes::Character);
    vd.req_field_one_of(&["location", "employer"]);
    vd.field_target_ok_this("employer", sc, Scopes::Character);
    vd.field_target_ok_this("location", sc, Scopes::Province);
    if let Some(token) = vd.field_value("template") {
        // undocumented
        data.verify_exists(Item::CharacterTemplate, token);
        data.validate_call(Item::CharacterTemplate, token, block, sc);
    }
    vd.field_item("template", Item::CharacterTemplate); // undocumented
    vd.field_target_ok_this("template_character", sc, Scopes::Character);
    vd.field_item_or_target("faith", sc, Item::Faith, Scopes::Faith);
    vd.field_validated_block_sc("random_faith", sc, validate_random_faith);
    vd.field_item_or_target("random_faith_in_religion", sc, Item::Religion, Scopes::Faith);
    vd.field_item_or_target("culture", sc, Item::Culture, Scopes::Culture);
    vd.field_validated_block_sc("random_culture", sc, validate_random_culture);
    // TODO: figure out what a culture group is, and whether this key still works at all
    vd.field_value("random_culture_in_group");
    vd.field_item_or_target("dynasty_house", sc, Item::House, Scopes::DynastyHouse);
    if let Some(token) = vd.field_value("dynasty") {
        if !token.is("generate") && !token.is("inherit") && !token.is("none") {
            validate_target(token, data, sc, Scopes::Dynasty);
        }
    }
    vd.field_validated_value("ethnicity", |_, mut vd| {
        vd.maybe_is("culture");
        vd.maybe_is("mother");
        vd.maybe_is("father");
        vd.maybe_is("parents");
        vd.item(Item::Ethnicity);
    });
    // TODO: Find out the syntax of this. Docs are unclear, no examples in vanilla.
    vd.field_block("ethnicities");
    vd.field_script_value("diplomacy", sc);
    vd.field_script_value("intrigue", sc);
    vd.field_script_value("martial", sc);
    vd.field_script_value("learning", sc);
    vd.field_script_value("prowess", sc);
    vd.field_script_value("stewardship", sc);
    vd.field_validated_key_block("after_creation", |key, block, data| {
        sc.open_scope(Scopes::Character, key.clone());
        validate_effect(block, data, sc, Tooltipped::No); // TODO: verify
        sc.close();
    });
}

pub fn validate_create_character_memory(
    key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::MemoryType);
    // TODO: also check that all participants are specified
    vd.field_validated_block("participants", |b, data| {
        let mut vd = Validator::new(b, data);
        let memtype = block.get_field_value("type");
        vd.unknown_value_fields(|key, token| {
            if let Some(memtype) = memtype {
                if !data.item_has_property(Item::MemoryType, memtype.as_str(), key.as_str()) {
                    let msg =
                        format!("memory type `{memtype}` does not define participant `{key}`");
                    warn(ErrorKey::Validation).msg(msg).loc(key).push();
                }
            }
            validate_target_ok_this(token, data, sc, Scopes::Character);
        });
    });
    vd.field_validated_block_sc("duration", sc, validate_duration);
    sc.define_name("new_memory", Scopes::CharacterMemory, key);
}

pub fn validate_create_dynamic_title(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("tier");
    vd.req_field("name");
    vd.field_choice("tier", &["duchy", "kingdom", "empire"]);
    vd.field_validated_sc("name", sc, validate_desc);
    vd.advice_field("adjective", "changed to adj in 1.13");
    vd.field_validated_sc("adj", sc, validate_desc);
    sc.define_name("new_title", Scopes::LandedTitle, key);
}

pub fn validate_create_holy_order(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("leader");
    vd.req_field("capital");
    vd.field_target("leader", sc, Scopes::Character);
    vd.field_target("capital", sc, Scopes::LandedTitle);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::HolyOrder, name);
    }
    if let Some(name) = vd.field_value("save_temporary_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::HolyOrder, name);
    }
}

pub fn validate_create_title_and_vassal_change(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_choice("type", TITLE_HISTORY_TYPES);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::TitleAndVassalChange, name);
    }
    vd.field_bool("add_claim_on_loss");
}

pub fn validate_delay_travel_plan(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_bool("add");
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_divide_war_chest(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_bool("defenders");
    vd.field_script_value("fraction", sc);
    vd.field_bool("gold");
    vd.field_bool("piety");
    vd.field_bool("prestige");
}

pub fn validate_duel(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    vd.field_item("skill", Item::Skill);
    vd.field_list_items("skills", Item::Skill);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_script_value("value", sc);
    vd.field_item("localization", Item::EffectLocalization);
    sc.define_name("duel_value", Scopes::Value, key);
    validate_random_list(key, block, data, sc, vd, tooltipped);
}

pub fn validate_faction_start_war(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("title", sc, Scopes::LandedTitle);
}

pub fn validate_force_add_to_agent_slot(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("agent_slot", sc, Scopes::AgentSlot);
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_force_vote_as(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("target", sc, Scopes::Character);
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_imprison(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("target", sc, Scopes::Character);
    vd.field_item("type", Item::PrisonType);
    // The docs also have a "reason" key, but no indication what it is
}

pub fn validate_join_faction_forced(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("faction", sc, Scopes::Faction);
    vd.field_target("forced_by", sc, Scopes::Character);
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_make_pregnant(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("father", sc, Scopes::Character);
    vd.field_integer("number_of_children");
    vd.field_bool("known_bastard");
}

pub fn validate_move_budget_gold(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_script_value("gold", sc);
    let choices = &["budget_war_chest", "budget_reserved", "budget_short_term", "budget_long_term"];
    vd.field_choice("from", choices);
    vd.field_choice("to", choices);
}

pub fn validate_open_interaction_window(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_ascii_lowercase();
    vd.req_field("interaction");
    vd.req_field("actor");
    vd.req_field("recipient");
    vd.field_value("interaction"); // TODO
    vd.field_bool("redirect");
    vd.field_target_ok_this("actor", sc, Scopes::Character);
    vd.field_target_ok_this("recipient", sc, Scopes::Character);
    vd.field_target_ok_this("secondary_actor", sc, Scopes::Character);
    vd.field_target_ok_this("secondary_recipient", sc, Scopes::Character);
    if caller == "open_interaction_window" {
        vd.field_target("target_title", sc, Scopes::LandedTitle);
    }
    if caller == "run_interaction" {
        vd.field_choice("execute_threshold", &["accept", "maybe", "decline"]);
        vd.field_choice("send_threshold", &["accept", "maybe", "decline"]);
    }
}

pub fn validate_pay_gold(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.field_target("target", sc, Scopes::Character);
    vd.field_script_value("gold", sc);
    // undocumented; it means multiply the gold amount by (whose?) yearly income
    vd.field_bool("yearly_income");
}

pub fn validate_pay_income(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.field_target("target", sc, Scopes::Character);
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_current_phase_guest_subset(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("target");
    vd.field_item("name", Item::GuestSubset);
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_remove_opinion(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("modifier");
    vd.field_target("target", sc, Scopes::Character);
    vd.field_item("modifier", Item::OpinionModifier);
    vd.field_bool("single");
}

pub fn validate_replace_court_position(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("recipient");
    vd.req_field("court_position");
    vd.field_target("recipient", sc, Scopes::Character);
    vd.field_target("holder", sc, Scopes::Character);
    vd.field_item("court_position", Item::CourtPosition);
}

pub fn validate_revoke_court_position(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("court_position");
    vd.field_item("court_position", Item::CourtPosition);
    vd.field_target("recipient", sc, Scopes::Character);
    vd.field_target("holder", sc, Scopes::Character);
}

pub fn validate_save_opinion_value(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("target");
    if let Some(name) = vd.field_value("name") {
        sc.define_name_token(name.as_str(), Scopes::Value, name);
    }
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_scheme_freeze(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("reason", Item::Localization);
    validate_optional_duration(&mut vd, sc);
}

pub fn validate_set_council_task(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("task_type");
    // TODO: figure out for which task types `target` is required
    vd.field_item("task_type", Item::CouncilTask);
    // This has been verified as of 1.9.2, it does require a Province here and not a LandedTitle
    vd.field_target("target", sc, Scopes::Character | Scopes::Province);
}

pub fn validate_set_culture_name(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("noun");
    vd.field_validated_sc("noun", sc, validate_desc);
    vd.field_validated_sc("collective_noun", sc, validate_desc);
    vd.field_validated_sc("prefix", sc, validate_desc);
}

pub fn validate_set_death_reason(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("death_reason");
    vd.field_item("death_reason", Item::DeathReason);
    vd.field_target("killer", sc, Scopes::Character);
    vd.field_target("artifact", sc, Scopes::Artifact);
}

pub fn validate_set_ghw_target(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_ascii_lowercase();
    vd.req_field("target_character");
    vd.req_field("target_title");
    vd.field_target("target_character", sc, Scopes::Character);
    vd.field_target("target_title", sc, Scopes::LandedTitle);
    if caller == "start_great_holy_war" {
        vd.field_script_value("delay", sc);
        vd.field_target("war", sc, Scopes::War);
    }
}

pub fn validate_set_legend_chapter(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("name", Item::LegendChapter);
    vd.field_item("localization_key", Item::Localization);
}

pub fn validate_set_legend_property(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("name", Item::LegendProperty);
    // TODO: look up possible scope types from the legend properties
    vd.field_target("target", sc, Scopes::all());
}

pub fn validate_setup_cb(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_ascii_lowercase();
    vd.req_field("attacker");
    vd.req_field("defender");
    // vd.req_field("change"); is optional if you just want it to set scope:cb_prestige_factor
    vd.field_target("attacker", sc, Scopes::Character);
    vd.field_target("defender", sc, Scopes::Character);
    vd.field_target("change", sc, Scopes::TitleAndVassalChange);
    vd.field_bool("victory");
    if caller == "setup_claim_cb" {
        vd.req_field("claimant");
        vd.field_target("claimant", sc, Scopes::Character);
        vd.field_bool("take_occupied");
        vd.field_bool("civil_war");
        vd.field_choice("titles", &["target_titles", "faction_titles"]);
    } else if caller == "setup_de_jure_cb" {
        vd.field_target("title", sc, Scopes::LandedTitle);
    } else if caller == "setup_invasion_cb" {
        vd.field_value("titles"); // list name
        vd.field_bool("take_occupied");
        vd.field_target("claimant", sc, Scopes::Character);
    }
    sc.define_name("cb_prestige_factor", Scopes::Value, key);
}

pub fn validate_spawn_army(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    // TODO: either levies or men_at_arms
    vd.req_field("location");
    vd.field_script_value("levies", sc);
    vd.multi_field_validated_block("men_at_arms", |b, data| {
        let mut vd = Validator::new(b, data);
        vd.req_field("type");
        vd.field_item("type", Item::MenAtArms);
        vd.field_script_value("men", sc);
        vd.field_script_value("stacks", sc);
        vd.field_bool("inheritable"); // undocumented
    });
    vd.field_target("location", sc, Scopes::Province);
    vd.field_target("origin", sc, Scopes::Province);
    vd.field_target("war", sc, Scopes::War);
    vd.field_bool("war_keep_on_attacker_victory");
    vd.field_bool("inheritable");
    vd.field_bool("uses_supply");
    vd.field_target("army", sc, Scopes::Army);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Army, name);
    }
    if let Some(name) = vd.field_value("save_temporary_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Army, name);
    }
    vd.field_validated_sc("name", sc, validate_desc);
}

pub fn validate_start_scheme(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field_one_of(&[
        "target_character",
        "target_title",
        "target_culture",
        "target_faith",
        "targets_nothing",
    ]);
    vd.field_item("type", Item::Scheme);
    vd.field_target("contract", sc, Scopes::TaskContract);
    vd.advice_field("target", "replaced with target_character in 1.13");
    vd.field_target("target_character", sc, Scopes::Character);
    vd.field_target("target_title", sc, Scopes::LandedTitle);
    vd.field_target("target_culture", sc, Scopes::Culture);
    vd.field_target("target_faith", sc, Scopes::Faith);
    vd.field_bool("targets_nothing");

    // undocumented

    // TODO: verify if still valid in 1.13
    vd.field_target("artifact", sc, Scopes::Artifact);
}

pub fn validate_start_struggle(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("struggle_type");
    vd.req_field("start_phase");
    vd.field_item("struggle_type", Item::Struggle);
    vd.field_item("start_phase", Item::StrugglePhase);
}

pub fn validate_start_travel_plan(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("destination");
    for token in vd.multi_field_value("destination") {
        validate_target(token, data, sc, Scopes::Province);
    }
    vd.field_target("travel_leader", sc, Scopes::Character);
    for token in vd.multi_field_value("companion") {
        validate_target(token, data, sc, Scopes::Character);
    }
    vd.field_bool("travel_with_domicile");
    vd.field_bool("players_use_planner");
    vd.field_bool("return_trip");
    vd.field_event("on_arrival_event", sc);
    vd.field_action("on_arrival_on_action", sc);
    vd.field_event("on_start_event", sc);
    vd.field_action("on_start_on_action", sc);
    vd.field_event("on_travel_planner_cancel_event", sc);
    vd.field_action("on_travel_planner_cancel_on_action", sc);
    vd.field_choice("on_arrival_destinations", &["all", "first", "last", "all_but_last"]);
}

pub fn validate_start_war(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("casus_belli", Item::CasusBelli);
    vd.field_item("cb", Item::CasusBelli);
    vd.field_target("target", sc, Scopes::Character);
    vd.field_target_ok_this("claimant", sc, Scopes::Character);
    for token in vd.multi_field_value("target_title") {
        validate_target(token, data, sc, Scopes::LandedTitle);
    }
}

pub fn validate_stress_impact(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_script_value("base", sc);
    vd.unknown_fields(|token, bv| {
        data.verify_exists(Item::Trait, token);
        validate_non_dynamic_script_value(bv, data);
    });
}

pub fn validate_try_create_important_action(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("important_action_type");
    vd.field_item("important_action_type", Item::ImportantAction);
    vd.unknown_value_fields(|_, value| {
        validate_target_ok_this(value, data, sc, Scopes::all_but_none());
    });
}

pub fn validate_try_create_suggestion(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("suggestion_type");
    vd.field_item("suggestion_type", Item::Suggestion);
    vd.field_target_ok_this("actor", sc, Scopes::Character);
    vd.field_target_ok_this("recipient", sc, Scopes::Character);
    vd.field_target_ok_this("secondary_actor", sc, Scopes::Character);
    vd.field_target_ok_this("secondary_recipient", sc, Scopes::Character);
    vd.field_target_ok_this("landed_title", sc, Scopes::LandedTitle);
}

pub fn validate_vassal_contract_set_obligation_level(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("level");
    if let Some(token) = vd.field_value("type") {
        if !data.item_exists(Item::VassalContract, token.as_str()) {
            validate_target(token, data, sc, Scopes::VassalContract);
        }
    }
    if let Some(token) = vd.field_value("level") {
        if !token.is_integer() && !data.item_exists(Item::VassalObligationLevel, token.as_str()) {
            validate_target(token, data, sc, Scopes::VassalObligationLevel);
        }
    }
}

pub fn validate_add_artifact_modifier(
    _key: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.item(Item::Modifier);
    // TODO validate `property_use`
    // TODO: this causes hundreds of warnings. Probably because the tooltip tracking isn't smart enough to figure out
    // things like "scope:newly_created_artifact does not exist yet at tooltipping time, so the body of the if won't
    // be tooltipped here".
    //
    // if tooltipped.is_tooltipped() {
    //     data.verify_exists(Item::Localization, value);
    // }
}

pub fn validate_generate_coa(
    _key: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.maybe_is("yes");
    vd.item(Item::CoaTemplateList);
}

pub fn validate_set_coa(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    let options = Scopes::LandedTitle | Scopes::Dynasty | Scopes::DynastyHouse;
    vd.item_or_target(sc, Item::Coa, options);
}

pub fn validate_set_focus(
    _key: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.maybe_is("no");
    vd.item(Item::Focus);
}

pub fn validate_set_title_name(
    _key: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.item(Item::Localization);
    vd.item_used_with_suffix(Item::Localization, "_adj");
}

pub fn validate_activate_struggle_catalyst(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::Catalyst, token),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("catalyst");
            vd.req_field("character");
            vd.field_item("catalyst", Item::Catalyst);
            vd.field_target("character", sc, Scopes::Character);
        }
    }
}

pub fn validate_add_character_flag(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(_token) => (),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("flag");
            vd.multi_field_value("flag");
            validate_optional_duration(&mut vd, sc);
        }
    }
}

pub fn validate_add_dead_character_flag(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.set_case_sensitive(false);
    vd.req_field("flag");
    vd.multi_field_value("flag");
    validate_mandatory_duration(block, &mut vd, sc);
}

pub fn validate_begin_create_holding(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::HoldingType, token),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("type");
            vd.field_item("type", Item::HoldingType);
            vd.field_validated_block("refund_cost", |b, data| {
                let mut vd = Validator::new(b, data);
                vd.set_case_sensitive(false);
                vd.field_script_value("gold", sc);
                vd.field_script_value("prestige", sc);
                vd.field_script_value("piety", sc);
            });
        }
    }
}

pub fn validate_change_first_name(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            if data.item_exists(Item::Localization, token.as_str()) {
                data.mark_used(Item::Localization, token.as_str());
            } else {
                validate_target(token, data, sc, Scopes::Flag);
            }
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("template_character");
            vd.field_target("template_character", sc, Scopes::Character);
        }
    }
}

pub fn validate_close_view(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(_token) => (), // TODO
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("view");
            vd.field_value("view"); // TODO
            vd.field_target("player", sc, Scopes::Character);
        }
    }
}

pub fn validate_create_alliance(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_target(token, data, sc, Scopes::Character);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("target");
            vd.field_target("target", sc, Scopes::Character);
            vd.field_target("allied_through_owner", sc, Scopes::Character);
            vd.field_target("allied_through_target", sc, Scopes::Character);
        }
    }
}

pub fn validate_create_epidemic_outbreak(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("intensity");
    vd.field_item("type", Item::EpidemicType);
    vd.field_choice("intensity", OUTBREAK_INTENSITIES);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Epidemic, name);
    }
}

pub fn validate_create_inspiration(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::Inspiration, token),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("type");
            vd.field_item("type", Item::Inspiration);
            vd.field_script_value("gold", sc);
        }
    }
}

pub fn validate_create_story(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::Story, token),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("type");
            vd.field_item("type", Item::Story);
            if let Some(name) = vd.field_value("save_scope_as") {
                sc.define_name_token(name.as_str(), Scopes::StoryCycle, name);
            }
            if let Some(name) = vd.field_value("save_temporary_scope_as") {
                sc.define_name_token(name.as_str(), Scopes::StoryCycle, name);
            }
        }
    }
}

pub fn validate_death(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            if !token.is("natural") {
                let msg = "expected `death = natural`";
                warn(ErrorKey::Validation).msg(msg).loc(token).push();
            }
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("death_reason");
            vd.field_item("death_reason", Item::DeathReason);
            vd.field_target("killer", sc, Scopes::Character);
            vd.field_target("artifact", sc, Scopes::Artifact);
        }
    }
}

pub fn validate_open_view(
    key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(_token) => (), // TODO
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("view");
            vd.field_value("view"); // TODO
            vd.field_value("view_message"); // TODO
            vd.field_target("player", sc, Scopes::Character);
            if key.is("open_view_data") {
                vd.field_target("secondary_actor", sc, Scopes::Character); // undocumented
                vd.field_target("data", sc, Scopes::all_but_none()); // undocumented
            }
        }
    }
}

pub fn validate_remove_courtier_or_guest(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_target(token, data, sc, Scopes::Character);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("character");
            vd.field_target("character", sc, Scopes::Character);
            vd.field_target("new_location", sc, Scopes::Province);
        }
    }
}

pub fn validate_set_dead_character_variable(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_value("name");
    vd.field_validated("value", |bv, data| match bv {
        BV::Value(token) => {
            validate_target_ok_this(token, data, sc, Scopes::all_but_none());
        }
        BV::Block(_) => validate_script_value(bv, data, sc),
    });
    validate_mandatory_duration(block, &mut vd, sc);
}

pub fn validate_set_location(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_target(token, data, sc, Scopes::Province);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("location");
            vd.field_target("location", sc, Scopes::Province);
            vd.field_bool("stick_to_location");
        }
    }
}

pub fn validate_set_owner(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_target(token, data, sc, Scopes::Character);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("target");
            vd.field_target("target", sc, Scopes::Character);
            vd.multi_field_validated_block_sc("history", sc, validate_artifact_history);
            vd.field_bool("generate_history");
        }
    }
}

pub fn validate_set_relation(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_target(token, data, sc, Scopes::Character);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("target");
            // Sometimes both are used and I don't know what that means. TODO: verify
            // vd.req_field_one_of(&["reason", "copy_reason"]);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_item("reason", Item::Localization);
            vd.field_item("copy_reason", Item::Relation);
            vd.field_target("province", sc, Scopes::Province);
            vd.field_target("involved_character", sc, Scopes::Character);
        }
    }
}

fn validate_artifact_history(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.set_case_sensitive(false);
    vd.req_field("type");
    vd.field_item("type", Item::ArtifactHistory);
    vd.field_date("date");
    vd.field_target("actor", sc, Scopes::Character);
    vd.field_target("recipient", sc, Scopes::Character);
    vd.field_target("location", sc, Scopes::Province);
}

pub fn validate_end_struggle(
    _value: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.maybe_is("yes");
    vd.item(Item::Localization); // undocumented
}

pub fn validate_create_legend(
    key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::LegendType);
    vd.req_field("quality");
    vd.field_choice("quality", LEGEND_QUALITY);
    vd.req_field("chronicle");
    vd.req_field("properties");
    vd.field_item("chronicle", Item::LegendChronicle);
    if let Some(chronicle_token) = vd.field_value("chronicle").cloned() {
        data.verify_exists(Item::LegendChronicle, &chronicle_token);

        if let Some((_, _, chronicle)) =
            data.get_item::<LegendChronicle>(Item::LegendChronicle, chronicle_token.as_str())
        {
            vd.field_validated_key_block("properties", |key, block, data| {
                let mut found_properties = TigerHashSet::default();
                let mut vd = Validator::new(block, data);
                vd.unknown_value_fields(|key, value| {
                    if let Some(scopes) = chronicle.properties.get(key).copied() {
                        found_properties.insert(key.clone());
                        validate_target(value, data, sc, scopes);
                    } else {
                        let msg =
                            format!("property {key} not found in {chronicle_token} chronicle");
                        err(ErrorKey::Validation).msg(msg).loc(key).push();
                    }
                });
                for property in chronicle.properties.keys() {
                    if !found_properties.contains(property) {
                        let msg = format!("chronicle property {property} missing from properties");
                        err(ErrorKey::Validation)
                            .msg(msg)
                            .loc(key)
                            .loc_msg(property, "defined here")
                            .push();
                    }
                }
            });
        }
    }
    // This validation function is used for both create_legend and create_legend_seed
    if key.is("create_legend") {
        vd.field_target("protagonist", sc, Scopes::Character);
        if let Some(name) = vd.field_value("save_scope_as") {
            sc.define_name_token(name.as_str(), Scopes::Legend, name);
        }
    }
}

pub fn validate_change_maa_regiment_size(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(_) => validate_script_value(bv, data, sc),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("size");
            vd.field_script_value("size", sc);
            vd.field_bool("reinforce");
        }
    }
}

pub fn validate_add_to_list_ck3(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(name) => sc.define_or_expect_list(name),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.req_field("value");
            if let Some(target) = vd.field_value("value").cloned() {
                if let Some(name) = vd.field_value("name") {
                    let outscopes =
                        validate_target_ok_this(&target, data, sc, Scopes::all_but_none());
                    sc.open_scope(outscopes, target);
                    sc.define_or_expect_list(name);
                    sc.close();
                }
            }
        }
    }
}

pub fn validate_create_adventurer_title(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("holder");
    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_target("holder", sc, Scopes::Character);
    vd.field_validated_sc("prefix", sc, validate_desc);
    vd.field_validated_sc("adjective", sc, validate_desc);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::LandedTitle, name);
    }

    // undocumented

    vd.field_validated_sc("article", sc, validate_desc);
}

pub fn validate_start_best_war(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_list_items("cb", Item::CasusBelli);
    vd.field_bool("recalculate_cb_targets");
    let sc_builder: &Builder = &|key| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("target_character", Scopes::Character, key);
        sc.define_name("target_title", Scopes::LandedTitle, key);
        sc.define_list("target_titles", Scopes::LandedTitle, key);
        sc.define_name("claimant", Scopes::Character, key);
        sc.define_name("casus_belli_type", Scopes::CasusBelliType, key);
        sc.define_name("has_hostage", Scopes::Bool, key);
        sc.define_name("score", Scopes::Value, key);
        sc
    };
    vd.field_validated_key_block("is_valid", |key, block, data| {
        validate_trigger(block, data, &mut sc_builder(key), Tooltipped::No);
    });
    vd.field_validated_key_block("on_success", |key, block, data| {
        validate_effect(block, data, &mut sc_builder(key), Tooltipped::No);
    });
    vd.field_validated_key_block("on_failure", |key, block, data| {
        validate_effect(block, data, &mut sc_builder(key), Tooltipped::No);
    });
}

pub fn validate_create_maa_regiment(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field_one_of(&["type", "type_of"]);
    vd.field_item("type", Item::MenAtArms);
    vd.field_target("type_of", sc, Scopes::Regiment);
    vd.field_bool("check_can_recruit");
    vd.field_target("title", sc, Scopes::LandedTitle);
    vd.field_script_value("size", sc);
}

pub fn validate_create_task_contract(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("task_contract_type");
    vd.req_field("task_contract_tier");
    vd.req_field("location");

    vd.field_item("task_contract_type", Item::TaskContractType);
    vd.advice_field(
        "task_task_contract_tier",
        "docs say `task_task_contract_tier` but it's `task_contract_tier`",
    );
    vd.field_script_value("task_contract_tier", sc);
    vd.field_target("location", sc, Scopes::Province);

    vd.field_target("task_contract_employer", sc, Scopes::Character);
    vd.field_target("destination", sc, Scopes::Province);
    vd.field_target("target", sc, Scopes::Character);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::TaskContract, name);
    }

    // undocumented

    if let Some(name) = vd.field_value("save_temporary_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::TaskContract, name);
    }
}

pub fn validate_give_noble_family_title(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_validated_sc("article", sc, validate_desc);
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::LandedTitle, name);
    }
}

pub fn validate_contracts_for_area(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("location", sc, Scopes::Province);
    vd.field_script_value("amount", sc);
    vd.field_list_items("group", Item::TaskContractGroup);
}

pub fn validate_change_appointment_investment(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("value");
    vd.field_target("target", sc, Scopes::Character);
    vd.field_target("investor", sc, Scopes::Character);
    vd.field_script_value("value", sc);
}

pub fn validate_set_important_location(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("title", sc, Scopes::LandedTitle);
    // TODO: set scopes for fired events and actions
    // "Events and onactions are fired with this scope:
    // root - title top liege
    // scope:county - important location
    // scope:title - higher tier title that is interested in the county
    // In enter realm:
    // scope:changed_top_liege - former top liege of the important location
    // In leave realm:
    // scope:changed_top_liege - new top liege of the important location"
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("county", Scopes::LandedTitle, key);
    sc.define_name("title", Scopes::LandedTitle, key);
    sc.define_name("changed_top_liege", Scopes::Character, key);

    vd.field_event("enter_realm_event", &mut sc);
    vd.field_action("enter_realm_on_action", &sc);
    vd.field_event("leave_realm_event", &mut sc);
    vd.field_action("leave_realm_on_action", &sc);
}
