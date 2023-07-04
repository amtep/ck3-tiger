use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, Comparator, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_control, validate_normal_effect};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{error, warn, warn_info, ErrorKey};
use crate::scopes::Scopes;
use crate::scriptvalue::{validate_non_dynamic_scriptvalue, validate_scriptvalue};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_target_ok_this, validate_trigger_key_bv};
use crate::validate::{
    validate_duration, validate_optional_duration, validate_optional_duration_int,
    validate_random_culture, validate_random_faith, validate_random_traits_list, ListType,
};
use EvB::*;
use EvBv::*;
use EvV::*;

#[derive(Debug, Copy, Clone)]
pub enum EvB {
    AddActivityLogEntry,
    AddArtifactHistory,
    AddArtifactTitleHistory,
    AddFromContribution,
    AddHook,
    AddOpinion,
    AddRelationFlag,
    AddSchemeCooldown,
    AddSchemeModifier,
    AddSecret,
    AddToVariableList,
    AddToGuestSubset,
    AddTraitXp,
    AddTruce,
    AssignCouncilTask,
    AssignCouncillorType,
    BattleEvent,
    ChangeCulturalAcceptance,
    ChangeVariable,
    ChangeLiege,
    ChangeTitleHolder,
    ChangeTraitRank,
    ClampVariable,
    CopyLocalizedText,
    CreateAccolade,
    CreateArtifact,
    CreateCharacter,
    CreateCharacterMemory,
    CreateDynamicTitle,
    CreateHolyOrder,
    CreateTitleAndVassalChange,
    DelayTravelPlan,
    DivideWarChest,
    Duel,
    FactionStartWar,
    ForceAddToScheme,
    ForceVoteAs,
    Imprison,
    JoinFactionForced,
    MakePregnant,
    MoveBudgetGold,
    OpenInteractionWindow,
    PayGold,
    PayIncome,
    RandomList,
    RemoveFromCurrentPhaseGuestSubset,
    RemoveFromGuestSubset,
    RemoveOpinion,
    ReplaceCourtPosition,
    RevokeCourtPosition,
    RoundVariable,
    SaveOpinionValue,
    SaveScopeValue,
    SchemeFreeze,
    SetCouncilTask,
    SetCultureName,
    SetDeathReason,
    SetGHWTarget,
    SetupCb,
    SpawnArmy,
    StartScheme,
    StartStruggle,
    StartTravelPlan,
    StartWar,
    StressImpact,
    Switch,
    TryCreateImportantAction,
    TryCreateSuggestion,
    VassalContractSetObligationLevel,
}

#[derive(Debug, Copy, Clone)]
pub enum EvBv {
    ActivateStruggleCatalyst,
    AddCharacterFlag,
    BeginCreateHolding,
    ChangeFirstName,
    CloseView,
    CreateAlliance,
    CreateInspiration,
    CreateStory,
    Death,
    OpenView,
    RemoveCourtierOrGuest,
    SetLocation,
    SetOwner,
    SetRelation,
    SetVariable,
    TriggerEvent,
}

#[derive(Debug, Copy, Clone)]
pub enum EvV {
    AddArtifactModifier,
    AddToList,
    GenerateCoa,
    RemoveFromList,
    SaveScope,
    SetCoa,
    SetFocus,
    SetTitleName,
}

pub fn validate_effect_block(
    v: EvB,
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let caller = key.as_str().to_lowercase();
    let mut vd = Validator::new(block, data);
    vd.set_case_sensitive(false);
    match v {
        AddActivityLogEntry => {
            vd.req_field("key");
            vd.req_field("character");
            vd.field_item("key", Item::Localization);
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
            validate_effect(&caller, ListType::None, block, data, sc, vd, tooltipped);
        }
        AddArtifactHistory => {
            vd.req_field("type");
            vd.req_field("recipient");
            vd.field_item("type", Item::ArtifactHistory);
            vd.field_date("date");
            vd.field_target("actor", sc, Scopes::Character);
            vd.field_target("recipient", sc, Scopes::Character);
            vd.field_target("location", sc, Scopes::Province);
        }
        AddArtifactTitleHistory => {
            vd.req_field("target");
            vd.req_field("date");
            vd.field_target("target", sc, Scopes::LandedTitle);
            vd.field_date("date");
        }
        AddFromContribution => {
            vd.field_script_value("prestige", sc);
            vd.field_script_value("gold", sc);
            vd.field_script_value("piety", sc);
            vd.field_validated_block("opinion", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("modifier", Item::OpinionModifier);
            });
        }
        AddHook => {
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
        AddOpinion => {
            vd.req_field("modifier");
            vd.req_field("target");
            vd.field_item("modifier", Item::OpinionModifier);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_script_value("opinion", sc); // undocumented
            validate_optional_duration(&mut vd, sc);
        }
        AddRelationFlag => {
            vd.req_field("relation");
            vd.req_field("flag");
            vd.req_field("target");
            vd.field_item("relation", Item::Relation);
            // TODO: check that the flag belongs to the relation
            vd.field_value("flag");
            vd.field_target("target", sc, Scopes::Character);
        }
        AddSchemeCooldown => {
            vd.req_field("target");
            vd.req_field("type");
            vd.field_target("target", sc, Scopes::Character);
            vd.field_item("type", Item::Scheme);
            validate_optional_duration_int(&mut vd);
        }
        AddSchemeModifier => {
            vd.req_field("type");
            if let Some(token) = vd.field_value("type") {
                data.verify_exists(Item::Modifier, token);
                data.database
                    .validate_property_use(Item::Modifier, token, data, key, "");
            }
            vd.field_integer("days");
        }
        AddSecret => {
            vd.req_field("type");
            vd.field_item("type", Item::Secret);
            vd.field_target("target", sc, Scopes::Character);
            if let Some(name) = vd.field_value("save_scope_as") {
                sc.define_name(name.as_str(), Scopes::Secret, name);
            }
        }
        AddToGuestSubset => {
            vd.req_field("name");
            vd.req_field("target");
            vd.field_item("name", Item::GuestSubset);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_item("phase", Item::ActivityPhase);
        }
        AddTraitXp => {
            vd.req_field("trait");
            vd.req_field("value");
            vd.field_item("trait", Item::Trait);
            vd.field_item("track", Item::TraitTrack);
            vd.field_script_value("value", sc);
        }
        AddToVariableList => {
            vd.req_field("name");
            vd.req_field("target");
            vd.field_value("name");
            vd.field_target_ok_this("target", sc, Scopes::all_but_none());
        }
        AddTruce => {
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
                error(block, ErrorKey::Validation, msg);
            }
        }
        AssignCouncilTask => {
            vd.req_field("council_task");
            vd.req_field("target");
            vd.field_target("council_task", sc, Scopes::CouncilTask);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_bool("fire_on_actions");
        }
        AssignCouncillorType => {
            vd.req_field("type");
            vd.req_field("target");
            vd.field_item("type", Item::CouncilPosition);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_bool("fire_on_actions");
            vd.field_bool("remove_existing_councillor");
        }
        BattleEvent => {
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
        ChangeCulturalAcceptance => {
            vd.req_field("target");
            vd.req_field("value");
            vd.field_target("target", sc, Scopes::Culture);
            vd.field_script_value("value", sc);
            vd.field_validated_sc("desc", sc, validate_desc);
        }
        ChangeVariable => {
            vd.req_field("name");
            vd.field_value("name");
            vd.field_script_value("add", sc);
            vd.field_script_value("subtract", sc);
            vd.field_script_value("multiply", sc);
            vd.field_script_value("divide", sc);
            vd.field_script_value("modulo", sc);
            vd.field_script_value("min", sc);
            vd.field_script_value("max", sc);
        }
        ChangeLiege => {
            vd.req_field("liege");
            vd.req_field("change");
            vd.field_target("liege", sc, Scopes::Character);
            vd.field_target("change", sc, Scopes::TitleAndVassalChange);
        }
        ChangeTitleHolder => {
            vd.req_field("holder");
            vd.req_field("change");
            vd.field_target("holder", sc, Scopes::Character);
            vd.field_target("change", sc, Scopes::TitleAndVassalChange);
            vd.field_bool("take_baronies");
            vd.field_target("government_base", sc, Scopes::Character);
        }
        ChangeTraitRank => {
            vd.req_field("trait");
            vd.req_field("rank");
            // TODO: check that it's a rankable trait
            vd.field_item("trait", Item::Trait);
            vd.field_script_value("rank", sc);
            if caller == "change_trait_rank" {
                vd.field_script_value("max", sc);
            }
        }
        ClampVariable => {
            vd.req_field("name");
            vd.field_value("name");
            vd.field_script_value("min", sc);
            vd.field_script_value("max", sc);
        }
        CopyLocalizedText => {
            vd.req_field("key");
            vd.req_field("target");
            vd.field_value("key");
            vd.field_target("target", sc, Scopes::Character);
        }
        CreateAccolade => {
            vd.req_field("knight");
            vd.req_field("primary");
            vd.req_field("secondary");
            vd.field_target("knight", sc, Scopes::Character);
            vd.field_item("primary", Item::AccoladeType);
            vd.field_item("secondary", Item::AccoladeType);
            vd.field_item("name", Item::Localization);
        }
        CreateArtifact => {
            validate_artifact(&caller, block, data, vd, sc, tooltipped);
        }
        CreateCharacter => {
            // docs say save_event_target instead of save_scope
            vd.replaced_field("save_event_target_as", "save_scope_as");
            vd.replaced_field("save_temporary_event_target_as", "save_temporary_scope_as");
            if let Some(name) = vd.field_value("save_scope_as") {
                sc.define_name(name.as_str(), Scopes::Character, name);
            }
            if let Some(name) = vd.field_value("save_temporary_scope_as") {
                sc.define_name(name.as_str(), Scopes::Character, name);
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
            vd.field_items("trait", Item::Trait);
            vd.field_validated_blocks_sc("random_traits_list", sc, validate_random_traits_list);
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
            vd.field_item_or_target(
                "random_faith_in_religion",
                sc,
                Item::Religion,
                Scopes::Faith,
            );
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
            vd.field_script_value("diplomacy", sc);
            vd.field_script_value("intrigue", sc);
            vd.field_script_value("martial", sc);
            vd.field_script_value("learning", sc);
            vd.field_script_value("prowess", sc);
            vd.field_script_value("stewardship", sc);
            vd.field_validated_key_block("after_creation", |key, block, data| {
                sc.open_scope(Scopes::Character, key.clone());
                validate_normal_effect(block, data, sc, tooltipped);
                sc.close();
            });
        }
        CreateCharacterMemory => {
            vd.req_field("type");
            vd.field_item("type", Item::MemoryType);
            // TODO: also check that all participants are specified
            vd.field_validated_block("participants", |b, data| {
                let mut vd = Validator::new(b, data);
                let memtype = block.get_field_value("type");
                for (key, token) in vd.unknown_value_fields() {
                    if let Some(memtype) = memtype {
                        if !data.item_has_property(Item::MemoryType, memtype.as_str(), key.as_str())
                        {
                            let msg = format!(
                                "memory type `{memtype}` does not define participant `{key}`"
                            );
                            warn(key, ErrorKey::Validation, &msg);
                        }
                    }
                    validate_target_ok_this(token, data, sc, Scopes::Character);
                }
            });
            vd.field_validated_block_sc("duration", sc, validate_duration);
            sc.define_name("new_memory", Scopes::CharacterMemory, key);
        }
        CreateDynamicTitle => {
            vd.req_field("tier");
            vd.req_field("name");
            vd.field_choice("tier", &["duchy", "kingdom", "empire"]);
            vd.field_validated_sc("name", sc, validate_desc);
            vd.field_validated_sc("adjective", sc, validate_desc);
            sc.define_name("new_title", Scopes::LandedTitle, key);
        }
        CreateHolyOrder => {
            vd.req_field("leader");
            vd.req_field("capital");
            vd.field_target("leader", sc, Scopes::Character);
            vd.field_target("capital", sc, Scopes::LandedTitle);
            if let Some(name) = vd.field_value("save_scope_as") {
                sc.define_name(name.as_str(), Scopes::HolyOrder, name);
            }
            if let Some(name) = vd.field_value("save_temporary_scope_as") {
                sc.define_name(name.as_str(), Scopes::HolyOrder, name);
            }
        }
        CreateTitleAndVassalChange => {
            vd.req_field("type");
            vd.field_choice(
                "type",
                &[
                    "conquest",
                    "independency",
                    "conquest_claim",
                    "granted",
                    "revoked",
                    "conquest_holy_war",
                    "swear_fealty",
                    "created",
                    "usurped",
                    "returned",
                    "leased_out",
                    "conquest_populist",
                    "faction_demand",
                ],
            );
            if let Some(name) = vd.field_value("save_scope_as") {
                sc.define_name(name.as_str(), Scopes::TitleAndVassalChange, name);
            }
            vd.field_bool("add_claim_on_loss");
        }
        DelayTravelPlan => {
            vd.field_bool("add");
            validate_optional_duration(&mut vd, sc);
        }
        DivideWarChest => {
            vd.field_bool("defenders");
            vd.field_script_value("fraction", sc);
            vd.field_bool("gold");
            vd.field_bool("piety");
            vd.field_bool("prestige");
        }
        Duel => {
            vd.field_item("skill", Item::Skill);
            vd.field_list_items("skills", Item::Skill);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_script_value("value", sc);
            vd.field_item("localization", Item::EffectLocalization);
            sc.define_name("duel_value", Scopes::Value, key);
            validate_random_list("duel", block, data, vd, sc, tooltipped);
        }
        FactionStartWar => {
            vd.field_target("title", sc, Scopes::LandedTitle);
        }
        ForceAddToScheme => {
            vd.field_target("scheme", sc, Scopes::Scheme);
            validate_optional_duration(&mut vd, sc);
        }
        ForceVoteAs => {
            vd.field_target("target", sc, Scopes::Character);
            validate_optional_duration(&mut vd, sc);
        }
        Imprison => {
            vd.field_target("target", sc, Scopes::Character);
            vd.field_item("type", Item::PrisonType);
            // The docs also have a "reason" key, but no indication what it is
        }
        JoinFactionForced => {
            vd.field_target("faction", sc, Scopes::Faction);
            vd.field_target("forced_by", sc, Scopes::Character);
            validate_optional_duration(&mut vd, sc);
        }
        MakePregnant => {
            vd.field_target("father", sc, Scopes::Character);
            vd.field_integer("number_of_children");
            vd.field_bool("known_bastard");
        }
        MoveBudgetGold => {
            vd.field_script_value("gold", sc);
            let choices = &[
                "budget_war_chest",
                "budget_reserved",
                "budget_short_term",
                "budget_long_term",
            ];
            vd.field_choice("from", choices);
            vd.field_choice("to", choices);
        }
        OpenInteractionWindow => {
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
        PayGold => {
            vd.req_field("target");
            vd.field_target("target", sc, Scopes::Character);
            vd.field_script_value("gold", sc);
            // undocumented; it means multiply the gold amount by (whose?) yearly income
            vd.field_bool("yearly_income");
        }
        PayIncome => {
            vd.req_field("target");
            vd.field_target("target", sc, Scopes::Character);
            validate_optional_duration(&mut vd, sc);
        }
        RandomList => {
            validate_random_list("random_list", block, data, vd, sc, tooltipped);
        }
        RemoveFromCurrentPhaseGuestSubset => {
            vd.req_field("name");
            vd.req_field("target");
            vd.field_item("name", Item::GuestSubset);
            vd.field_target("target", sc, Scopes::Character);
        }
        RemoveFromGuestSubset => {
            vd.req_field("name");
            vd.req_field("target");
            vd.field_item("name", Item::GuestSubset);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_item("phase", Item::ActivityPhase);
        }
        RemoveOpinion => {
            vd.req_field("target");
            vd.req_field("modifier");
            vd.field_target("target", sc, Scopes::Character);
            vd.field_item("modifier", Item::OpinionModifier);
            vd.field_bool("single");
        }
        ReplaceCourtPosition => {
            vd.req_field("recipient");
            vd.req_field("court_position");
            vd.field_target("recipient", sc, Scopes::Character);
            vd.field_target("holder", sc, Scopes::Character);
            vd.field_item("court_position", Item::CourtPosition);
        }
        RevokeCourtPosition => {
            vd.req_field("court_position");
            if let Some(token) = vd.field_value("recipient") {
                let msg = "as of 1.9.2 neither `recipient` nor `target` work here";
                let info = "For court positions with multiple holders (such as bodyguard), an arbitrary one will be revoked";
                warn_info(token, ErrorKey::Bugs, msg, info);
            }
            if let Some(token) = vd.field_value("target") {
                let msg = "as of 1.9.2 neither `recipient` nor `target` work here";
                let info = "For court positions with multiple holders (such as bodyguard), an arbitrary one will be revoked";
                warn_info(token, ErrorKey::Bugs, msg, info);
            }
        }
        RoundVariable => {
            vd.req_field("name");
            vd.req_field("nearest");
            vd.field_value("name");
            vd.field_script_value("nearest", sc);
        }
        SaveOpinionValue => {
            vd.req_field("name");
            vd.req_field("target");
            if let Some(name) = vd.field_value("name") {
                sc.define_name(name.as_str(), Scopes::Value, name);
            }
            vd.field_target("target", sc, Scopes::Character);
        }
        SaveScopeValue => {
            vd.req_field("name");
            vd.req_field("value");
            if let Some(name) = vd.field_value("name") {
                // TODO: examine `value` field to check its real scope type
                sc.define_name(name.as_str(), Scopes::primitive(), name);
            }
            vd.field_script_value_or_flag("value", sc);
        }
        SchemeFreeze => {
            vd.field_item("reason", Item::Localization);
            validate_optional_duration(&mut vd, sc);
        }
        SetCouncilTask => {
            vd.req_field("task_type");
            // TODO: figure out for which task types `target` is required
            vd.field_item("task_type", Item::CouncilTask);
            // This has been verified as of 1.9.2, it does require a Province here and not a LandedTitle
            vd.field_target("target", sc, Scopes::Character | Scopes::Province);
        }
        SetCultureName => {
            vd.req_field("noun");
            vd.field_validated_sc("noun", sc, validate_desc);
            vd.field_validated_sc("collective_noun", sc, validate_desc);
            vd.field_validated_sc("prefix", sc, validate_desc);
        }
        SetDeathReason => {
            vd.req_field("death_reason");
            vd.field_item("death_reason", Item::DeathReason);
            vd.field_target("killer", sc, Scopes::Character);
            vd.field_target("artifact", sc, Scopes::Artifact);
        }
        SetGHWTarget => {
            vd.req_field("target_character");
            vd.req_field("target_title");
            vd.field_target("target_character", sc, Scopes::Character);
            vd.field_target("target_title", sc, Scopes::LandedTitle);
            if caller == "start_great_holy_war" {
                vd.field_script_value("delay", sc);
                vd.field_target("war", sc, Scopes::War);
            }
        }
        SetupCb => {
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
            // Undocumented
            } else if caller == "setup_de_jure_cb" {
                vd.field_target("title", sc, Scopes::LandedTitle);
            } else if caller == "setup_invasion_cb" {
                vd.field_value("titles"); // list name
                vd.field_bool("take_occupied");
            }
            sc.define_name("cb_prestige_factor", Scopes::Value, key);
        }
        SpawnArmy => {
            // TODO: either levies or men_at_arms
            vd.req_field("location");
            vd.field_script_value("levies", sc);
            vd.field_validated_blocks("men_at_arms", |b, data| {
                let mut vd = Validator::new(b, data);
                vd.req_field("type");
                vd.field_item("type", Item::MenAtArms);
                vd.field_script_value("men", sc);
                vd.field_script_value("stacks", sc);
            });
            vd.field_target("location", sc, Scopes::Province);
            vd.field_target("origin", sc, Scopes::Province);
            vd.field_target("war", sc, Scopes::War);
            vd.field_bool("war_keep_on_attacker_victory");
            vd.field_bool("inheritable");
            vd.field_bool("uses_supply");
            vd.field_target("army", sc, Scopes::Army);
            if let Some(name) = vd.field_value("save_scope_as") {
                sc.define_name(name.as_str(), Scopes::Army, name);
            }
            if let Some(name) = vd.field_value("save_temporary_scope_as") {
                sc.define_name(name.as_str(), Scopes::Army, name);
            }
            vd.field_validated_sc("name", sc, validate_desc);
        }
        StartScheme => {
            vd.req_field("type");
            vd.req_field("target");
            vd.field_item("type", Item::Scheme);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_target("artifact", sc, Scopes::Artifact);
        }
        StartStruggle => {
            vd.req_field("struggle_type");
            vd.req_field("start_phase");
            vd.field_item("struggle_type", Item::Struggle);
            vd.field_item("start_phase", Item::StrugglePhase);
        }
        StartTravelPlan => {
            vd.req_field("destination");
            for token in vd.field_values("destination") {
                validate_target(token, data, sc, Scopes::Province);
            }
            vd.field_target("travel_leader", sc, Scopes::Character);
            for token in vd.field_values("companion") {
                validate_target(token, data, sc, Scopes::Character);
            }
            vd.field_bool("players_use_planner");
            vd.field_bool("return_trip");
            vd.field_item("on_arrival_event", Item::Event);
            vd.field_item("on_arrival_on_action", Item::OnAction);
            vd.field_item("on_start_event", Item::Event);
            vd.field_item("on_start_on_action", Item::OnAction);
            vd.field_item("on_travel_planner_cancel_event", Item::Event);
            vd.field_item("on_travel_planner_cancel_on_action", Item::OnAction);
            vd.field_choice(
                "on_arrival_destinations",
                &["all", "first", "last", "all_but_last"],
            );
            // Root for these events is travel plan owner
            if let Some(token) = block.get_field_value("on_arrival_event") {
                data.events.check_scope(token, sc);
            }
            if let Some(token) = block.get_field_value("on_start_event") {
                data.events.check_scope(token, sc);
            }
            if let Some(token) = block.get_field_value("on_travel_planner_cancel_event") {
                data.events.check_scope(token, sc);
            }
        }
        StartWar => {
            vd.field_item("casus_belli", Item::CasusBelli);
            vd.field_item("cb", Item::CasusBelli);
            vd.field_target("target", sc, Scopes::Character);
            vd.field_target_ok_this("claimant", sc, Scopes::Character);
            for token in vd.field_values("target_title") {
                validate_target(token, data, sc, Scopes::LandedTitle);
            }
        }
        StressImpact => {
            vd.field_script_value("base", sc);
            for (token, bv) in vd.unknown_fields() {
                data.verify_exists(Item::Trait, token);
                validate_non_dynamic_scriptvalue(bv, data);
            }
        }
        Switch => {
            vd.set_case_sensitive(true);
            vd.req_field("trigger");
            if let Some(target) = vd.field_value("trigger") {
                // clone to avoid calling vd again while target is still borrowed
                let target = target.clone();
                for (key, block) in vd.unknown_block_fields() {
                    if !key.is("fallback") {
                        // Pretend the switch was written as a series of trigger = key lines
                        let synthetic_bv = BV::Value(key.clone());
                        validate_trigger_key_bv(
                            &target,
                            Comparator::Eq,
                            &synthetic_bv,
                            data,
                            sc,
                            tooltipped,
                            false,
                        );
                    }

                    let vd = Validator::new(block, data);
                    validate_effect("", ListType::None, block, data, sc, vd, tooltipped);
                }
            }
        }
        TryCreateImportantAction => {
            vd.req_field("important_action_type");
            vd.field_item("important_action_type", Item::ImportantAction);
            for (_, value) in vd.unknown_value_fields() {
                validate_target_ok_this(value, data, sc, Scopes::all_but_none());
            }
        }
        TryCreateSuggestion => {
            vd.req_field("suggestion_type");
            vd.field_item("suggestion_type", Item::Suggestion);
            vd.field_target_ok_this("actor", sc, Scopes::Character);
            vd.field_target_ok_this("recipient", sc, Scopes::Character);
            vd.field_target_ok_this("secondary_actor", sc, Scopes::Character);
            vd.field_target_ok_this("secondary_recipient", sc, Scopes::Character);
            vd.field_target_ok_this("landed_title", sc, Scopes::LandedTitle);
        }
        VassalContractSetObligationLevel => {
            vd.req_field("type");
            vd.req_field("level");
            if let Some(token) = vd.field_value("type") {
                if !data.item_exists(Item::VassalContract, token.as_str()) {
                    validate_target(token, data, sc, Scopes::VassalContract);
                }
            }
            if let Some(token) = vd.field_value("level") {
                if !token.is_integer()
                    && !data.item_exists(Item::VassalObligationLevel, token.as_str())
                {
                    validate_target(token, data, sc, Scopes::VassalObligationLevel);
                }
            }
        }
    }
}

pub fn validate_effect_value(
    v: EvV,
    _key: &Token,
    value: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match v {
        AddArtifactModifier => {
            data.verify_exists(Item::Modifier, value);
            // TODO: this causes hundreds of warnings. Probably because the tooltip tracking isn't smart enough to figure out
            // things like "scope:newly_created_artifact does not exist yet at tooltipping time, so the body of the if won't
            // be tooltipped here".
            //
            // if tooltipped.is_tooltipped() {
            //     data.verify_exists(Item::Localization, value);
            // }
        }
        AddToList => sc.define_or_expect_list(value),
        GenerateCoa => {
            if !value.is("yes") {
                data.verify_exists(Item::CoaTemplateList, value);
            }
        }
        RemoveFromList => sc.expect_list(value),
        SaveScope => sc.save_current_scope(value.as_str()),
        SetCoa => {
            if !data.item_exists(Item::Coa, value.as_str()) {
                let options = Scopes::LandedTitle | Scopes::Dynasty | Scopes::DynastyHouse;
                validate_target(value, data, sc, options);
            }
        }
        SetFocus => {
            if !value.is("no") {
                data.verify_exists(Item::Focus, value);
            }
        }
        SetTitleName => {
            data.verify_exists(Item::Localization, value);
            let loca = format!("{value}_adj");
            data.item_used(Item::Localization, &loca);
        }
    }
}

pub fn validate_effect_bv(
    v: EvBv,
    key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match v {
        ActivateStruggleCatalyst => match bv {
            BV::Value(token) => data.verify_exists(Item::Catalyst, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("catalyst");
                vd.req_field("character");
                vd.field_item("catalyst", Item::Catalyst);
                vd.field_target("character", sc, Scopes::Character);
            }
        },
        AddCharacterFlag => match bv {
            BV::Value(_token) => (),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("flag");
                vd.field_values("flag");
                validate_optional_duration(&mut vd, sc);
            }
        },
        BeginCreateHolding => match bv {
            BV::Value(token) => data.verify_exists(Item::Holding, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("type");
                vd.field_item("type", Item::Holding);
                vd.field_validated_block("refund_cost", |b, data| {
                    let mut vd = Validator::new(b, data);
                    vd.set_case_sensitive(false);
                    vd.field_script_value("gold", sc);
                    vd.field_script_value("prestige", sc);
                    vd.field_script_value("piety", sc);
                });
            }
        },
        ChangeFirstName => match bv {
            BV::Value(token) => {
                if data.item_exists(Item::Localization, token.as_str()) {
                    data.item_used(Item::Localization, token.as_str());
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
        },
        CloseView => match bv {
            BV::Value(_token) => (), // TODO
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("view");
                vd.field_value("view"); // TODO
                vd.field_target("player", sc, Scopes::Character);
            }
        },
        CreateAlliance => match bv {
            BV::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("target");
                vd.field_target("target", sc, Scopes::Character);
                vd.field_target("allied_through_owner", sc, Scopes::Character);
                vd.field_target("allied_through_target", sc, Scopes::Character);
            }
        },
        CreateInspiration => match bv {
            BV::Value(token) => data.verify_exists(Item::Inspiration, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("type");
                vd.req_field("gold");
                vd.field_item("type", Item::Inspiration);
                vd.field_script_value("gold", sc);
            }
        },
        CreateStory => match bv {
            BV::Value(token) => data.verify_exists(Item::Story, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("type");
                vd.field_item("type", Item::Story);
                if let Some(name) = vd.field_value("save_scope_as") {
                    sc.define_name(name.as_str(), Scopes::StoryCycle, name);
                }
                if let Some(name) = vd.field_value("save_temporary_scope_as") {
                    sc.define_name(name.as_str(), Scopes::StoryCycle, name);
                }
            }
        },
        Death => match bv {
            BV::Value(token) => {
                if !token.is("natural") {
                    let msg = "expected `death = natural`";
                    warn(token, ErrorKey::Validation, msg);
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
        },
        OpenView => match bv {
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
        },
        RemoveCourtierOrGuest => match bv {
            BV::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("character");
                vd.field_target("character", sc, Scopes::Character);
                vd.field_target("new_location", sc, Scopes::Province);
            }
        },
        SetLocation => match bv {
            BV::Value(token) => validate_target(token, data, sc, Scopes::Province),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("location");
                vd.field_target("location", sc, Scopes::Province);
                vd.field_bool("stick_to_location");
            }
        },
        SetOwner => match bv {
            BV::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("target");
                vd.field_target("target", sc, Scopes::Character);
                vd.field_validated_blocks_sc("history", sc, validate_artifact_history);
                vd.field_bool("generate_history");
            }
        },
        SetRelation => match bv {
            BV::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("target");
                vd.req_field("reason");
                vd.field_target("target", sc, Scopes::Character);
                vd.field_item("reason", Item::Localization);
                vd.field_item("copy_reason", Item::Relation);
                vd.field_target("province", sc, Scopes::Province);
                vd.field_target("involved_character", sc, Scopes::Character);
            }
        },
        SetVariable => match bv {
            BV::Value(_token) => (),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.req_field("name");
                vd.field_value("name");
                vd.field_validated("value", |bv, data| match bv {
                    BV::Value(token) => {
                        validate_target_ok_this(token, data, sc, Scopes::all_but_none());
                    }
                    BV::Block(_) => validate_scriptvalue(bv, data, sc),
                });
                validate_optional_duration(&mut vd, sc);
            }
        },
        TriggerEvent => match bv {
            BV::Value(token) => {
                data.verify_exists(Item::Event, token);
                data.events.check_scope(token, sc);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);
                vd.field_item("id", Item::Event);
                vd.field_item("on_action", Item::OnAction);
                vd.field_target("saved_event_id", sc, Scopes::Flag);
                vd.field_date("trigger_on_next_date");
                vd.field_bool("delayed");
                validate_optional_duration(&mut vd, sc);
                if let Some(token) = block.get_field_value("id") {
                    data.events.check_scope(token, sc);
                }
            }
        },
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

fn validate_artifact(
    caller: &str,
    _block: &Block,
    _data: &Everything,
    mut vd: Validator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_validated_sc("description", sc, validate_desc);
    vd.field_item("rarity", Item::ArtifactRarity);
    vd.field_item("type", Item::ArtifactType);
    vd.field_items("modifier", Item::Modifier);
    vd.field_script_value("durability", sc);
    vd.field_script_value("max_durability", sc);
    vd.field_bool("decaying");
    vd.field_validated_blocks_sc("history", sc, validate_artifact_history);
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
            sc.define_name(name.as_str(), Scopes::Artifact, name);
        }
        vd.field_target("title_history", sc, Scopes::LandedTitle);
        vd.field_date("title_history_date");
    } else {
        vd.ban_field("save_scope_as", || "`create_artifact`");
        vd.ban_field("title_history", || "`create_artifact`");
        vd.ban_field("title_history_date", || "`create_artifact`");
    }
}

fn validate_random_list(
    caller: &str,
    _block: &Block,
    data: &Everything,
    mut vd: Validator,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    vd.field_integer("pick");
    vd.field_bool("unique"); // don't know what this does
    vd.field_validated_sc("desc", sc, validate_desc);
    for (key, block) in vd.unknown_block_fields() {
        if f64::from_str(key.as_str()).is_err() {
            let msg = "expected number";
            error(key, ErrorKey::Validation, msg);
        } else {
            validate_effect_control(caller, block, data, sc, tooltipped);
        }
    }
}
