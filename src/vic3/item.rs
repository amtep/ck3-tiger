use strum_macros::{EnumIter, IntoStaticStr};

use crate::report::{Confidence, Severity};

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoStaticStr, Hash, PartialOrd, Ord, EnumIter)]
#[strum(serialize_all = "snake_case")]
#[cfg(feature = "vic3")]
pub enum Item {
    Accessory,
    AccessoryTag,
    AccessoryVariation,
    AccessoryVariationLayout,
    AccessoryVariationTextures,
    AiStrategy,
    Approval,
    Asset,
    Attitude,
    BattleCondition,
    BlendShape,
    BuildingGroup,
    BuildingType,
    CanalType,
    CharacterInteraction,
    CharacterRole,
    Coa,
    CoaColorList,
    CoaColoredEmblemList,
    CoaDesignerColoredEmblem,
    CoaDesignerPattern,
    CoaPatternList,
    CoaTemplate,
    CoaTemplateList,
    CoaTexturedEmblemList,
    CombatUnit,
    Country,
    CountryFormation,
    CountryRank,
    CountryTier,
    CountryType,
    Culture,
    CustomLocalization,
    Decision,
    Decree,
    Define,
    DiplomaticAction,
    DiplomaticPlay,
    Dlc,
    DlcFeature,
    EffectLocalization,
    Entity,
    Ethnicity,
    Event,
    EventNamespace,
    File,
    Font,
    Fontfiles,
    GameConcept,
    GameRule,
    GameRuleSetting,
    GeneAgePreset,
    GeneAttribute,
    GeneCategory,
    Goods,
    GovernmentType,
    GuiLayer,
    GuiTemplate,
    GuiType,
    Ideology,
    InfamyThreshold,
    Institution,
    InterestGroup,
    InterestGroupTrait,
    Journalentry,
    LawGroup,
    LawType,
    Level,
    Localization,
    MapLayer,
    MediaAlias,
    Modifier,
    ModifierType,
    NamedColor,
    Objective,
    ObjectiveSubgoal,
    ObjectiveSubgoalCategory,
    OnAction,
    Pdxmesh,
    PopType,
    PortraitAnimation,
    PortraitCamera,
    PortraitModifierGroup,
    PortraitModifierPack,
    ProductionMethod,
    ProductionMethodGroup,
    Province,
    Religion,
    ScriptedButton,
    ScriptedEffect,
    ScriptedGui,
    ScriptedList,
    ScriptedModifier,
    ScriptedTrigger,
    ScriptValue,
    SecretGoal,
    Shortcut,
    Sound,
    StateRegion,
    StateTrait,
    StrategicRegion,
    SubjectType,
    Technology,
    TechnologyEra,
    Terrain,
    TerrainLabel,
    TerrainManipulator,
    TerrainMask,
    TerrainMaterial,
    TextFormat,
    TextIcon,
    TextureFile,
    TransferOfPower,
    TriggerLocalization,
    TutorialLesson,
    Wargoal,
}

impl Item {
    #[cfg(feature = "vic3")]
    pub fn path(self) -> &'static str {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::Accessory => "gfx/portraits/accessories/",
            Item::AccessoryTag => "gfx/portraits/accessories/",
            Item::AccessoryVariation => "gfx/portraits/accessory_variations/",
            Item::AccessoryVariationLayout => "gfx/portraits/accessory_variations/",
            Item::AccessoryVariationTextures => "gfx/portraits/accessory_variations/",
            Item::AiStrategy => "common/ai_strategies/",
            Item::Approval => "",
            Item::Asset => "gfx/models/",
            Item::Attitude => "",
            Item::BattleCondition => "common/battle_conditions/",
            Item::BlendShape => "gfx/models/",
            Item::BuildingGroup => "common/building_groups/",
            Item::BuildingType => "common/buildings/",
            Item::CanalType => "common/canals/",
            Item::CharacterInteraction => "common/character_interactions/",
            Item::CharacterRole => "",
            Item::Coa => "common/coat_of_arms/coat_of_arms/",
            Item::CoaColorList => "common/coat_of_arms/template_lists/",
            Item::CoaColoredEmblemList => "common/coat_of_arms/template_lists/",
            Item::CoaDesignerColoredEmblem => "gfx/coat_of_arms/colored_emblems/",
            Item::CoaDesignerPattern => "gfx/coat_of_arms/patterns/",
            Item::CoaPatternList => "common/coat_of_arms/template_lists/",
            Item::CoaTemplate => "common/coat_of_arms/coat_of_arms/",
            Item::CoaTemplateList => "common/coat_of_arms/template_lists/",
            Item::CoaTexturedEmblemList => "common/coat_of_arms/template_lists/",
            Item::CombatUnit => "common/combat_unit_types",
            Item::Country => "common/country_definitions/",
            Item::CountryFormation => "common/country_formation/",
            Item::CountryRank => "common/country_ranks",
            Item::CountryTier => "",
            Item::CountryType => "common/country_types",
            Item::Culture => "common/cultures/",
            Item::CustomLocalization => "common/customizable_localization/",
            Item::Decision => "common/decisions/",
            Item::Decree => "common/decrees/",
            Item::Define => "common/defines/",
            Item::DiplomaticAction => "common/diplomatic_actions/",
            Item::DiplomaticPlay => "common/diplomatic_plays/",
            Item::Dlc => "",
            Item::DlcFeature => "",
            Item::EffectLocalization => "common/effect_localization/",
            Item::Entity => "gfx/models/",
            Item::Ethnicity => "common/ethnicities/",
            Item::Event => "events/",
            Item::EventNamespace => "events/",
            Item::File => "",
            Item::Font => "fonts/",
            Item::Fontfiles => "fonts/",
            Item::GameConcept => "common/game_concepts/",
            Item::GameRule => "common/game_rules/",
            Item::GameRuleSetting => "common/game_rules/",
            Item::GeneAgePreset => "common/genes/",
            Item::GeneAttribute => "gfx/models/",
            Item::GeneCategory => "common/genes/",
            Item::Goods => "common/goods/",
            Item::GovernmentType => "common/government_types/",
            Item::GuiLayer => "gui/",
            Item::GuiTemplate => "gui/",
            Item::GuiType => "gui/",
            Item::Ideology => "common/ideologies/",
            Item::InfamyThreshold => "",
            Item::Institution => "common/institutions/",
            Item::InterestGroup => "common/interest_groups/",
            Item::InterestGroupTrait => "common/interest_group_traits/",
            Item::Journalentry => "common/journal_entries/",
            Item::LawGroup => "common/law_groups/",
            Item::LawType => "common/laws/",
            Item::Level => "",
            Item::Localization => "localization/",
            Item::MapLayer => "gfx/map/map_object_data/layers.txt",
            Item::MediaAlias => "gfx/media_aliases/",
            Item::Modifier => "common/modifiers/",
            Item::ModifierType => "common/modifier_types/",
            Item::NamedColor => "common/named_colors/",
            Item::Objective => "common/objectives/",
            Item::ObjectiveSubgoal => "common/objective_subgoals/",
            Item::ObjectiveSubgoalCategory => "common/objective_subgoal_categories/",
            Item::OnAction => "common/on_actions/",
            Item::Pdxmesh => "gfx/models/",
            Item::PopType => "common/pop_types/",
            Item::PortraitAnimation => "gfx/portraits/portrait_animations/",
            Item::PortraitCamera => "gfx/portraits/cameras/",
            Item::PortraitModifierGroup => "gfx/portraits/portrait_modifiers/",
            Item::PortraitModifierPack => "gfx/portraits/portrait_animations/",
            Item::ProductionMethod => "common/production_methods/",
            Item::ProductionMethodGroup => "common/production_method_groups/",
            Item::Province => "map_data/provinces.png",
            Item::Religion => "common/religions/",
            Item::ScriptedButton => "common/scripted_buttons/",
            Item::ScriptedEffect => "common/scripted_effects/",
            Item::ScriptedGui => "common/scripted_guis/",
            Item::ScriptedList => "common/scripted_lists/",
            Item::ScriptedModifier => "common/scripted_modifiers/",
            Item::ScriptedTrigger => "common/scripted_triggers/",
            Item::ScriptValue => "common/script_values/",
            Item::SecretGoal => "common/secret_goals/",
            Item::Shortcut => "gui/shortcuts.shortcuts",
            Item::Sound => "",
            Item::StateRegion => "map_data/state_regions/",
            Item::StateTrait => "common/state_traits/",
            Item::StrategicRegion => "common/strategic_regions/",
            Item::SubjectType => "common/subject_types/",
            Item::Technology => "common/technology/technologies/",
            Item::TechnologyEra => "common/technology/eras/",
            Item::Terrain => "common/terrain/",
            Item::TerrainLabel => "common/labels",
            Item::TerrainManipulator => "common/terrain_manipulators/",
            Item::TerrainMask => "gfx/map/masks/",
            Item::TerrainMaterial => "gfx/map/terrain/materials.settings",
            Item::TextFormat => "gui/",
            Item::TextIcon => "gui/",
            Item::TextureFile => "gfx/models/",
            Item::TransferOfPower => "",
            Item::TriggerLocalization => "common/trigger_localization/",
            Item::TutorialLesson => "common/tutorial_lessons/",
            Item::Wargoal => "",
        }
    }

    /// Confidence value to use when reporting that an item is missing.
    /// Should be `Strong` for most, `Weak` for items that aren't defined anywhere but just used (such as gfx flags).
    pub fn confidence(self) -> Confidence {
        match self {
            Item::AccessoryTag
                // GuiType and GuiTemplate are here because referring to templates in other mods is a
            // common compatibility technique.
            | Item::GuiType
            | Item::GuiTemplate
                => Confidence::Weak,
            _ => Confidence::Strong,
        }
    }
    /// Severity value to use when reporting that an item is missing.
    /// * `Error` - most things
    /// * `Warning` - things that only impact visuals or presentation
    /// * `Untidy` - things that don't matter much at all
    /// * `Fatal` - things that cause crashes if they're missing
    /// This is only one piece of the severity puzzle. It can also depend on the caller who's expecting the item to exist.
    /// That part isn't handled yet.
    pub fn severity(self) -> Severity {
        match self {
            // GuiType and GuiTemplate are here because referring to templates in other mods is a
            // common compatibility technique.
            Item::GuiType | Item::GuiTemplate => Severity::Untidy,
            Item::Accessory
            | Item::AccessoryTag
            | Item::AccessoryVariation
            | Item::AccessoryVariationLayout
            | Item::AccessoryVariationTextures
            | Item::Coa
            | Item::CoaColorList
            | Item::CoaColoredEmblemList
            | Item::CoaDesignerColoredEmblem
            | Item::CoaDesignerPattern
            | Item::CoaPatternList
            | Item::CoaTemplate
            | Item::CoaTemplateList
            | Item::CoaTexturedEmblemList
            | Item::CustomLocalization
            | Item::EffectLocalization
            | Item::Ethnicity
            | Item::File
            | Item::GameConcept
            | Item::Localization
            | Item::MapLayer
            | Item::ModifierType
            | Item::NamedColor
            | Item::PortraitAnimation
            | Item::PortraitCamera
            | Item::Sound
            | Item::TerrainManipulator
            | Item::TerrainMask
            | Item::TerrainMaterial
            | Item::TextFormat
            | Item::TextIcon
            | Item::TextureFile
            | Item::TriggerLocalization => Severity::Warning,
            _ => Severity::Error,
        }
    }
}
