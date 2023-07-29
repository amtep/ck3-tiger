use strum_macros::{EnumIter, IntoStaticStr};

use crate::report::{Confidence, Severity};

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoStaticStr, Hash, PartialOrd, Ord, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum Item {
    Ambition,
    Area,
    Building,
    Culture,
    CultureGroup,
    CustomLocalization,
    DeathReason,
    Decision,
    Define,
    Deity,
    DeityCategory,
    DiplomaticStance,
    EconomicPolicy,
    Ethnicity,
    Event,
    EventNamespace,
    EventPicture,
    EventTheme,
    File,
    GameConcept,
    Government,
    GovernorPolicy,
    GraphicalCultureType,
    GreatWorkEffect,
    GreatWorkCategory,
    GreatWorkMaterial,
    Heritage,
    Idea,
    Invention,
    Law,
    LegionDistinction,
    LevyTemplate,
    Localization,
    Loyalty,
    MilitaryTraditionTree,
    MilitaryTradition,
    Mission,
    MissionTask,
    Modifier,
    NamedColor,
    Office,
    OnAction,
    Opinion,
    PartyType,
    PopType,
    Price,
    Province,
    ProvinceRank,
    Region,
    Religion,
    ScriptedEffect,
    ScriptedGui,
    ScriptedList,
    ScriptedModifier,
    ScriptedTrigger,
    ScriptValue,
    Sound,
    SubjectType,
    TechnologyTable,
    TerrainType,
    TextureFile,
    TradeGood,
    Treasure,
    CharacterTrait,
    Unit,
    UnitAbility,
    WarGoal,
}

impl Item {
    pub fn path(self) -> &'static str {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::Ambition => "common/ambitions/",
            Item::Area => "map_data/areas.txt",
            Item::Building => "common/buildings/",
            Item::Culture => "common/cultures/",
            Item::CultureGroup => "common/cultures/",
            Item::CustomLocalization => "common/customizable_localization/",
            Item::DeathReason => "common/deathreasons/",
            Item::Decision => "decisions/",
            Item::Define => "common/defines/",
            Item::Deity => "common/deities/",
            Item::DeityCategory => "common/deity_categories/",
            Item::DiplomaticStance => "common/diplomatic_stance/",
            Item::EconomicPolicy => "common/economic_policy/",
            Item::Ethnicity => "common/ethnicities/",
            Item::Event => "events/",
            Item::EventNamespace => "events/",
            Item::EventPicture => "common/event_pictures/",
            Item::EventTheme => "common/event_themes/",
            Item::File => "",
            Item::GameConcept => "common/game_concepts/",
            Item::Government => "common/governments/",
            Item::GovernorPolicy => "common/governor_policies/",
            Item::GraphicalCultureType => "common/graphical_culture_types/",
            Item::GreatWorkEffect => "common/great_work_effects/",
            Item::GreatWorkCategory => "common/great_work_categories/",
            Item::GreatWorkMaterial => "common/great_work_materials/",
            Item::Heritage => "common/heritage/",
            Item::Idea => "common/ideas/",
            Item::Invention => "common/inventions/",
            Item::Law => "common/laws/",
            Item::LegionDistinction => "common/legion_distinctions/",
            Item::LevyTemplate => "common/levy_templates/",
            Item::Localization => "localization/",
            Item::Loyalty => "common/loyalty/",
            Item::MilitaryTraditionTree => "common/military_traditions/",
            Item::MilitaryTradition => "common/military_traditions/",
            Item::Mission => "common/missions/",
            Item::MissionTask => "common/missions/",
            Item::Modifier => "common/modifiers/",
            Item::NamedColor => "common/named_colors/",
            Item::Office => "common/offices/",
            Item::OnAction => "common/on_action/",
            Item::Opinion => "common/opinions/",
            Item::PartyType => "common/party_types/",
            Item::PopType => "common/pop_types/",
            Item::Price => "common/prices/",
            Item::Province => "map_data/provinces.png",
            Item::ProvinceRank => "common/province_ranks/",
            Item::Region => "map_data/regions.txt",
            Item::Religion => "common/religions/",
            Item::ScriptedEffect => "common/scripted_effects/",
            Item::ScriptedGui => "common/scripted_guis/",
            Item::ScriptedList => "common/scripted_lists/",
            Item::ScriptedModifier => "common/scripted_modifiers/",
            Item::ScriptedTrigger => "common/scripted_triggers/",
            Item::ScriptValue => "common/script_values/",
            Item::Sound => "",
            Item::SubjectType => "common/subject_types/",
            Item::TechnologyTable => "common/technology_tables/",
            Item::TerrainType => "common/terrain/",
            Item::TextureFile => "gfx/models/",
            Item::TradeGood => "common/trade_goods/",
            Item::Treasure => "setup/main/",
            Item::CharacterTrait => "common/traits/",
            Item::Unit => "common/units/",
            Item::UnitAbility => "common/unit_abilities/",
            Item::WarGoal => "common/wargoals/",
        }
    }

    /// Confidence value to use when reporting that an item is missing.
    /// Should be `Strong` for most, `Weak` for items that aren't defined anywhere but just used (such as gfx flags).
    pub fn confidence(self) -> Confidence {
        match self {
            // TODO
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
           // TODO
        }
    }
}
