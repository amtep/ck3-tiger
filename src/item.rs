//! Giant enum for all the [`Item`] types in the game.

use std::fmt::{Display, Formatter};

use strum_macros::{EnumCount, EnumIter, IntoStaticStr};

use crate::block::Block;
use crate::db::Db;
#[cfg(doc)]
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::pdxfile::PdxEncoding;
use crate::report::{Confidence, Severity};
use crate::token::Token;

/// "items" are all the things that can be looked up in the game databases.
/// Anything that can be looked up in script with a literal string key, or that's loaded into
/// tiger's database and needs a unique key, is an `Item`.
///
/// There is some overlap with scopes, for example "culture" is both an `Item` and a scope type,
/// but the difference is that scopes are runtime values while items are always strings.
///
/// For example if a trigger takes a culture *scope*, you could supply either `culture:german` or
/// `scope:target_culture`, while if a trigger takes a culture *item*, you would have to supply just
/// `german` and don't have the option of supplying something determined at runtime.
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoStaticStr, Hash, PartialOrd, Ord, EnumCount, EnumIter)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
// The item table is in several alphabetized sections. First the generic items, then items used by
// multiple games, then a section for each game.
//
// Each item is marked with a cfg clause for its game. This is not strictly necessary, but it helps
// prevent "leakage" between the games, where they accidentally use each other's item types.
#[rustfmt::skip] // having cfg and the variant on the same line is much more readable
pub enum Item {
    // Generic items used by all games and assumed to be there by the validators in
    // non-game-specific `data` modules.
    #[cfg(feature = "jomini")]
    Accessory,
    #[cfg(feature = "jomini")]
    AccessoryTag,
    #[cfg(feature = "jomini")]
    AccessoryVariation,
    #[cfg(feature = "jomini")]
    AccessoryVariationLayout,
    #[cfg(feature = "jomini")]
    AccessoryVariationTextures,
    Achievement,
    #[cfg(feature = "jomini")]
    AchievementGroup,
    Asset,
    BlendShape,
    #[cfg(feature = "jomini")]
    CharacterInteraction,
    #[cfg(feature = "jomini")]
    Coa,
    #[cfg(feature = "jomini")]
    CoaColorList,
    #[cfg(feature = "jomini")]
    CoaColoredEmblemList,
    #[cfg(feature = "ck3")]
    CoaDesignerColorPalette,
    #[cfg(feature = "jomini")]
    CoaDesignerColoredEmblem,
    #[cfg(feature = "ck3")]
    CoaDesignerEmblemLayout,
    #[cfg(feature = "jomini")]
    CoaDesignerPattern,
    #[cfg(feature = "jomini")]
    CoaPatternList,
    #[cfg(feature = "jomini")]
    CoaTemplate,
    #[cfg(feature = "jomini")]
    CoaTemplateList,
    #[cfg(feature = "jomini")]
    CoaTexturedEmblemList,
    #[cfg(feature = "jomini")]
    Culture,
    #[cfg(feature = "jomini")]
    CustomLocalization,
    Decision,
    Define,
    Directory,
    Dlc,
    DlcFeature,
    DlcName,
    #[cfg(feature = "jomini")]
    EffectLocalization,
    Entity,
    Entry,
    #[cfg(feature = "jomini")]
    Ethnicity,
    Event,
    EventNamespace,
    File,
    Font,
    Fontfiles,
    #[cfg(feature = "jomini")]
    GameConcept,
    GameRule,
    GameRuleSetting,
    #[cfg(feature = "jomini")]
    GeneAgePreset,
    #[cfg(feature = "jomini")]
    GeneAttribute,
    #[cfg(feature = "jomini")]
    GeneCategory,
    #[cfg(feature = "jomini")]
    GovernmentType,
    GuiLayer,
    GuiTemplate,
    GuiType,
    #[cfg(feature = "jomini")]
    LawGroup,
    Localization,
    MapEnvironment,
    MapMode,
    Modifier,
    Music,
    MusicPlayerCategory,
    #[cfg(feature = "jomini")]
    NamedColor,
    OnAction,
    Pdxmesh,
    #[cfg(feature = "jomini")]
    PortraitAnimation,
    #[cfg(feature = "jomini")]
    PortraitCamera,
    #[cfg(feature = "jomini")]
    PortraitEnvironment,
    #[cfg(feature = "jomini")]
    PortraitModifierGroup,
    #[cfg(feature = "jomini")]
    PortraitModifierPack,
    Province,
    #[cfg(feature = "jomini")]
    Religion,
    ScriptedEffect,
    ScriptedGui,
    #[cfg(feature = "jomini")]
    ScriptedList,
    #[cfg(feature = "jomini")]
    ScriptedModifier,
    #[cfg(feature = "jomini")]
    ScriptedRule,
    ScriptedTrigger,
    #[cfg(feature = "jomini")]
    ScriptValue,
    Shortcut,
    Sound,
    Terrain,
    TextFormat,
    TextIcon,
    TextureFile,
    #[cfg(feature = "jomini")]
    TriggerLocalization,
    WidgetName,

    // Items shared by more than one game
    #[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
    Building,
    #[cfg(any(feature = "ck3", feature = "hoi4"))]
    Character,
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    CharacterTemplate,
    #[cfg(any(feature = "vic3", feature = "imperator", feature = "hoi4"))]
    CharacterTrait,
    #[cfg(any(feature = "imperator", feature = "hoi4"))]
    CombatTactic,
    #[cfg(any(feature = "vic3", feature = "imperator", feature = "hoi4"))]
    Country,
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    DeathReason,
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    Dna,
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    EventTheme,
    #[cfg(any(feature = "imperator", feature = "hoi4"))]
    Idea,
    #[cfg(any(feature = "vic3", feature = "hoi4"))]
    Ideology,
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    Law,
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    Message,
    #[cfg(any(feature = "imperator", feature = "hoi4"))]
    Mission,
    #[cfg(any(feature = "vic3", feature = "imperator"))]
    PopType,
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    Region,
    #[cfg(any(feature = "vic3", feature = "imperator"))]
    SubjectType,
    #[cfg(any(feature = "vic3", feature = "hoi4"))]
    Technology,
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    TutorialLesson,
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    TutorialLessonChain,
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    TutorialLessonStep,
    #[cfg(any(feature = "imperator", feature = "hoi4"))]
    Unit,
    #[cfg(any(feature = "vic3", feature = "imperator"))]
    Wargoal,

    // Items for ck3
    #[cfg(feature = "ck3")] AccoladeCategory,
    #[cfg(feature = "ck3")] AccoladeIcon,
    #[cfg(feature = "ck3")] AccoladeName,
    #[cfg(feature = "ck3")] AccoladeParameter,
    #[cfg(feature = "ck3")] AccoladeType,
    #[cfg(feature = "ck3")] ActivityGroupType,
    #[cfg(feature = "ck3")] ActivityIntent,
    #[cfg(feature = "ck3")] ActivityLocale,
    #[cfg(feature = "ck3")] ActivityOption,
    #[cfg(feature = "ck3")] ActivityOptionCategory,
    #[cfg(feature = "ck3")] ActivityPhase,
    #[cfg(feature = "ck3")] ActivityPulseAction,
    #[cfg(feature = "ck3")] ActivityState,
    #[cfg(feature = "ck3")] ActivityType,
    #[cfg(feature = "ck3")] AiWarStance,
    #[cfg(feature = "ck3")] AgentType,
    #[cfg(feature = "ck3")] Amenity,
    #[cfg(feature = "ck3")] AmenitySetting,
    #[cfg(feature = "ck3")] ArtifactBlueprint,
    #[cfg(feature = "ck3")] ArtifactFeature,
    #[cfg(feature = "ck3")] ArtifactFeatureGroup,
    #[cfg(feature = "ck3")] ArtifactHistory,
    #[cfg(feature = "ck3")] ArtifactRarity,
    #[cfg(feature = "ck3")] ArtifactSlot,
    #[cfg(feature = "ck3")] ArtifactSlotType,
    #[cfg(feature = "ck3")] ArtifactTemplate,
    #[cfg(feature = "ck3")] ArtifactType,
    #[cfg(feature = "ck3")] ArtifactVisual,
    #[cfg(feature = "ck3")] Bookmark,
    #[cfg(feature = "ck3")] BookmarkGroup,
    #[cfg(feature = "ck3")] BookmarkPortrait,
    #[cfg(feature = "ck3")] BuildingFlag,
    #[cfg(feature = "ck3")] BuildingGfx,
    #[cfg(feature = "ck3")] CasusBelli,
    #[cfg(feature = "ck3")] CasusBelliGroup,
    #[cfg(feature = "ck3")] Catalyst,
    #[cfg(feature = "ck3")] ChallengeCharacter,
    #[cfg(feature = "ck3")] CharacterBackground,
    #[cfg(feature = "ck3")] CharacterInteractionCategory,
    #[cfg(feature = "ck3")] Climate,
    #[cfg(feature = "ck3")] ClothingGfx,
    #[cfg(feature = "ck3")] CoaGfx,
    #[cfg(feature = "ck3")] CoaDynamicDefinition,
    #[cfg(feature = "ck3")] CombatEffect,
    #[cfg(feature = "ck3")] CombatPhaseEvent,
    #[cfg(feature = "ck3")] CouncilPosition,
    #[cfg(feature = "ck3")] CouncilTask,
    #[cfg(feature = "ck3")] Countermeasure,
    #[cfg(feature = "ck3")] CountermeasureParameter,
    #[cfg(feature = "ck3")] CourtPosition,
    #[cfg(feature = "ck3")] CourtPositionTask,
    #[cfg(feature = "ck3")] CourtSceneCulture,
    #[cfg(feature = "ck3")] CourtSceneGroup,
    #[cfg(feature = "ck3")] CourtSceneRole,
    #[cfg(feature = "ck3")] CourtSceneSetting,
    #[cfg(feature = "ck3")] CourtType,
    #[cfg(feature = "ck3")] CourtierGuestManagement,
    #[cfg(feature = "ck3")] CultureAesthetic,
    #[cfg(feature = "ck3")] CultureCreationName,
    #[cfg(feature = "ck3")] CultureEra,
    #[cfg(feature = "ck3")] CultureEthos,
    #[cfg(feature = "ck3")] CultureHeritage,
    #[cfg(feature = "ck3")] CultureHistory,
    #[cfg(feature = "ck3")] CultureParameter,
    #[cfg(feature = "ck3")] CulturePillar,
    #[cfg(feature = "ck3")] CultureTradition,
    #[cfg(feature = "ck3")] CultureTraditionCategory,
    #[cfg(feature = "ck3")] DangerType,
    #[cfg(feature = "ck3")] DecisionGroup,
    #[cfg(feature = "ck3")] DiarchyMandate,
    #[cfg(feature = "ck3")] DiarchyParameter,
    #[cfg(feature = "ck3")] DiarchyType,
    #[cfg(feature = "ck3")] Doctrine,
    #[cfg(feature = "ck3")] DoctrineCategory,
    #[cfg(feature = "ck3")] DoctrineParameter,
    #[cfg(feature = "ck3")] DomicileBuilding,
    #[cfg(feature = "ck3")] DomicileParameter,
    #[cfg(feature = "ck3")] DomicileType,
    #[cfg(feature = "ck3")] Dynasty,
    #[cfg(feature = "ck3")] DynastyLegacy,
    #[cfg(feature = "ck3")] DynastyPerk,
    #[cfg(feature = "ck3")] EpidemicType,
    #[cfg(feature = "ck3")] EpidemicDeathReason,
    #[cfg(feature = "ck3")] EventBackground,
    #[cfg(feature = "ck3")] EventEffect2d,
    #[cfg(feature = "ck3")] EventTransition,
    #[cfg(feature = "ck3")] Faction,
    #[cfg(feature = "ck3")] Faith,
    #[cfg(feature = "ck3")] FaithIcon,
    #[cfg(feature = "ck3")] FervorModifier,
    #[cfg(feature = "ck3")] Flavorization,
    #[cfg(feature = "ck3")] Focus,
    #[cfg(feature = "ck3")] GeneticConstraint,
    #[cfg(feature = "ck3")] GovernmentFlag,
    #[cfg(feature = "ck3")] GraphicalFaith,
    #[cfg(feature = "ck3")] GuestInviteRule,
    #[cfg(feature = "ck3")] GuestSubset,
    #[cfg(feature = "ck3")] GuestSystem,
    #[cfg(feature = "ck3")] HoldingFlag,
    #[cfg(feature = "ck3")] HoldingType,
    #[cfg(feature = "ck3")] HolySite,
    #[cfg(feature = "ck3")] HolySiteFlag,
    #[cfg(feature = "ck3")] Hook,
    #[cfg(feature = "ck3")] House,
    #[cfg(feature = "ck3")] HousePowerBonus,
    #[cfg(feature = "ck3")] HouseUnity,
    #[cfg(feature = "ck3")] HouseUnityParameter,
    #[cfg(feature = "ck3")] HouseUnityStage,
    #[cfg(feature = "ck3")] ImportantAction,
    #[cfg(feature = "ck3")] Innovation,
    #[cfg(feature = "ck3")] InnovationFlag,
    #[cfg(feature = "ck3")] Inspiration,
    #[cfg(feature = "ck3")] Language,
    #[cfg(feature = "ck3")] LawFlag,
    #[cfg(feature = "ck3")] LeaseContract,
    #[cfg(feature = "ck3")] LegendChapter,
    #[cfg(feature = "ck3")] LegendChronicle,
    #[cfg(feature = "ck3")] LegendProperty,
    #[cfg(feature = "ck3")] LegendSeed,
    #[cfg(feature = "ck3")] LegendType,
    #[cfg(feature = "ck3")] LegitimacyFlag,
    #[cfg(feature = "ck3")] LegitimacyType,
    #[cfg(feature = "ck3")] Lifestyle,
    #[cfg(feature = "ck3")] MartialCustom,
    #[cfg(feature = "ck3")] MemoryCategory,
    #[cfg(feature = "ck3")] MemoryType,
    #[cfg(feature = "ck3")] MenAtArms,
    #[cfg(feature = "ck3")] MenAtArmsBase,
    #[cfg(feature = "ck3")] MessageFilterType,
    #[cfg(feature = "ck3")] MessageGroupType,
    #[cfg(feature = "ck3")] ModifierFormat,
    #[cfg(feature = "ck3")] MottoInsert,
    #[cfg(feature = "ck3")] Motto,
    #[cfg(feature = "ck3")] NameEquivalency,
    #[cfg(feature = "ck3")] NameList,
    #[cfg(feature = "ck3")] Nickname,
    #[cfg(feature = "ck3")] OpinionModifier,
    #[cfg(feature = "ck3")] Perk,
    #[cfg(feature = "ck3")] PerkTree,
    #[cfg(feature = "ck3")] PlayableDifficultyInfo,
    #[cfg(feature = "ck3")] PointOfInterest,
    #[cfg(feature = "ck3")] PoolSelector,
    #[cfg(feature = "ck3")] PortraitType,
    #[cfg(feature = "ck3")] PrisonType,
    #[cfg(feature = "ck3")] ProvinceMapping,
    #[cfg(feature = "ck3")] Relation,
    #[cfg(feature = "ck3")] RelationFlag,
    #[cfg(feature = "ck3")] ReligionFamily,
    #[cfg(feature = "ck3")] RewardItem,
    #[cfg(feature = "ck3")] Scheme,
    #[cfg(feature = "ck3")] SchemePulseAction,
    #[cfg(feature = "ck3")] ScriptedAnimation,
    #[cfg(feature = "ck3")] ScriptedCost,
    #[cfg(feature = "ck3")] ScriptedIllustration,
    #[cfg(feature = "ck3")] Secret,
    #[cfg(feature = "ck3")] Sexuality,
    #[cfg(feature = "ck3")] Skill,
    #[cfg(feature = "ck3")] SpecialBuilding,
    #[cfg(feature = "ck3")] SpecialGuest,
    #[cfg(feature = "ck3")] Story,
    #[cfg(feature = "ck3")] Struggle,
    #[cfg(feature = "ck3")] StruggleHistory,
    #[cfg(feature = "ck3")] StrugglePhase,
    #[cfg(feature = "ck3")] StrugglePhaseParameter,
    #[cfg(feature = "ck3")] SuccessionAppointment,
    #[cfg(feature = "ck3")] SuccessionElection,
    #[cfg(feature = "ck3")] Suggestion,
    #[cfg(feature = "ck3")] TaskContractGroup,
    #[cfg(feature = "ck3")] TaskContractReward,
    #[cfg(feature = "ck3")] TaskContractType,
    #[cfg(feature = "ck3")] TaxSlotFlag,
    #[cfg(feature = "ck3")] TaxSlotObligation,
    #[cfg(feature = "ck3")] TaxSlotType,
    #[cfg(feature = "ck3")] Title,
    #[cfg(feature = "ck3")] TitleHistory,
    #[cfg(feature = "ck3")] TitleHistoryType,
    #[cfg(feature = "ck3")] Trait,
    #[cfg(feature = "ck3")] TraitCategory,
    #[cfg(feature = "ck3")] TraitFlag,
    #[cfg(feature = "ck3")] TraitPortraitModifier,
    #[cfg(feature = "ck3")] TraitTrack,
    #[cfg(feature = "ck3")] TravelOption,
    #[cfg(feature = "ck3")] UnitGfx,
    #[cfg(feature = "ck3")] VassalContract,
    #[cfg(feature = "ck3")] VassalContractFlag,
    #[cfg(feature = "ck3")] VassalObligationLevel,
    #[cfg(feature = "ck3")] VassalStance,

    // Items specific to vic3
    #[cfg(feature = "vic3")] AcceptanceStatus,
    #[cfg(feature = "vic3")] AiStrategy,
    #[cfg(feature = "vic3")] Alert,
    #[cfg(feature = "vic3")] AlertGroup,
    #[cfg(feature = "vic3")] Approval,
    #[cfg(feature = "vic3")] Attitude,
    #[cfg(feature = "vic3")] BattleCondition,
    #[cfg(feature = "vic3")] BuildingGroup,
    #[cfg(feature = "vic3")] BuildingType,
    #[cfg(feature = "vic3")] BuyPackage,
    #[cfg(feature = "vic3")] CanalType,
    #[cfg(feature = "vic3")] CharacterRole,
    #[cfg(feature = "vic3")] CombatUnit,
    #[cfg(feature = "vic3")] CombatUnitExperienceLevel,
    #[cfg(feature = "vic3")] CombatUnitGroup,
    #[cfg(feature = "vic3")] CommanderOrder,
    #[cfg(feature = "vic3")] CommanderRank,
    #[cfg(feature = "vic3")] CompanyType,
    #[cfg(feature = "vic3")] CohesionLevel,
    #[cfg(feature = "vic3")] CountryCreation,
    #[cfg(feature = "vic3")] CountryFormation,
    #[cfg(feature = "vic3")] CountryRank,
    #[cfg(feature = "vic3")] CountryTier,
    #[cfg(feature = "vic3")] CountryType,
    #[cfg(feature = "vic3")] CultureGraphics,
    #[cfg(feature = "vic3")] Decree,
    #[cfg(feature = "vic3")] DiplomaticAction,
    #[cfg(feature = "vic3")] DiplomaticCatalyst,
    #[cfg(feature = "vic3")] DiplomaticCatalystCategory,
    #[cfg(feature = "vic3")] DiplomaticPlay,
    #[cfg(feature = "vic3")] DiscriminationTrait,
    #[cfg(feature = "vic3")] DynamicCompanyName,
    #[cfg(feature = "vic3")] DynamicCountryMapColor,
    #[cfg(feature = "vic3")] DynamicCountryName,
    #[cfg(feature = "vic3")] EventCategory,
    #[cfg(feature = "vic3")] FlagDefinition,
    #[cfg(feature = "vic3")] Goods,
    #[cfg(feature = "vic3")] GradientBorderSettings,
    #[cfg(feature = "vic3")] HarvestConditionType,
    #[cfg(feature = "vic3")] InfamyThreshold,
    #[cfg(feature = "vic3")] Institution,
    #[cfg(feature = "vic3")] InterestGroup,
    #[cfg(feature = "vic3")] InterestGroupTrait,
    #[cfg(feature = "vic3")] JournalEntry,
    #[cfg(feature = "vic3")] JournalEntryGroup,
    #[cfg(feature = "vic3")] LawType,
    #[cfg(feature = "vic3")] LegitimacyLevel,
    #[cfg(feature = "vic3")] Level,
    #[cfg(feature = "vic3")] LibertyDesireLevel,
    #[cfg(feature = "vic3")] MapLayer,
    #[cfg(feature = "vic3")] MapInteractionType,
    #[cfg(feature = "vic3")] MapNotificationType,
    #[cfg(feature = "vic3")] MediaAlias,
    #[cfg(feature = "vic3")] MilitaryFormationFlag,
    #[cfg(feature = "vic3")] MobilizationOption,
    #[cfg(feature = "vic3")] MobilizationOptionGroup,
    #[cfg(feature = "vic3")] ModifierTypeDefinition,
    #[cfg(feature = "vic3")] Objective,
    #[cfg(feature = "vic3")] ObjectiveSubgoal,
    #[cfg(feature = "vic3")] ObjectiveSubgoalCategory,
    #[cfg(feature = "vic3")] Party,
    #[cfg(feature = "vic3")] PoliticalLobby,
    #[cfg(feature = "vic3")] PoliticalLobbyAppeasement,
    #[cfg(feature = "vic3")] PoliticalMovement,
    #[cfg(feature = "vic3")] PoliticalMovementCategory,
    #[cfg(feature = "vic3")] PoliticalMovementPopSupport,
    #[cfg(feature = "vic3")] PopNeed,
    #[cfg(feature = "vic3")] PowerBlocCoaPiece,
    #[cfg(feature = "vic3")] PowerBlocIdentity,
    #[cfg(feature = "vic3")] PowerBlocMapTexture,
    #[cfg(feature = "vic3")] PowerBlocName,
    #[cfg(feature = "vic3")] Principle,
    #[cfg(feature = "vic3")] PrincipleGroup,
    #[cfg(feature = "vic3")] ProductionMethod,
    #[cfg(feature = "vic3")] ProductionMethodGroup,
    #[cfg(feature = "vic3")] ProposalType,
    #[cfg(feature = "vic3")] RelationsThreshold,
    #[cfg(feature = "vic3")] ScriptedButton,
    #[cfg(feature = "vic3")] ScriptedProgressBar,
    #[cfg(feature = "vic3")] ScriptedTest,
    #[cfg(feature = "vic3")] SecretGoal,
    #[cfg(feature = "vic3")] Skin,
    #[cfg(feature = "vic3")] SocialClass,
    #[cfg(feature = "vic3")] SocialHierarchy,
    #[cfg(feature = "vic3")] StateRegion,
    #[cfg(feature = "vic3")] StateTrait,
    #[cfg(feature = "vic3")] Strata,
    #[cfg(feature = "vic3")] StrategicRegion,
    #[cfg(feature = "vic3")] TechnologyEra,
    #[cfg(feature = "vic3")] TerrainKey,
    #[cfg(feature = "vic3")] TerrainLabel,
    #[cfg(feature = "vic3")] TerrainManipulator,
    #[cfg(feature = "vic3")] TerrainMask,
    #[cfg(feature = "vic3")] TerrainMaterial,
    #[cfg(feature = "vic3")] Theme,
    #[cfg(feature = "vic3")] TransferOfPower,

    // Items specific to imperator
    #[cfg(feature = "imperator")] Ambition,
    #[cfg(feature = "imperator")] AiPlanGoals,
    #[cfg(feature = "imperator")] Area,
    #[cfg(feature = "imperator")] CultureGroup,
    #[cfg(feature = "imperator")] Deity,
    #[cfg(feature = "imperator")] DeityCategory,
    #[cfg(feature = "imperator")] DiplomaticStance,
    #[cfg(feature = "imperator")] EconomicPolicy,
    #[cfg(feature = "imperator")] EventPicture,
    #[cfg(feature = "imperator")] GovernorPolicy,
    #[cfg(feature = "imperator")] GraphicalCultureType,
    #[cfg(feature = "imperator")] GreatWorkEffectTier,
    #[cfg(feature = "imperator")] GreatWorkEffect,
    #[cfg(feature = "imperator")] GreatWorkCategory,
    #[cfg(feature = "imperator")] GreatWorkMaterial,
    #[cfg(feature = "imperator")] GreatWorkModule,
    #[cfg(feature = "imperator")] GreatWorkTemplate,
    #[cfg(feature = "imperator")] Heritage,
    #[cfg(feature = "imperator")] Invention,
    #[cfg(feature = "imperator")] InventionGroup,
    #[cfg(feature = "imperator")] LegionDistinction,
    #[cfg(feature = "imperator")] LevyTemplate,
    #[cfg(feature = "imperator")] Loyalty,
    #[cfg(feature = "imperator")] MilitaryTraditionTree,
    #[cfg(feature = "imperator")] MilitaryTradition,
    #[cfg(feature = "imperator")] MissionTask,
    #[cfg(feature = "imperator")] Office,
    #[cfg(feature = "imperator")] Opinion,
    #[cfg(feature = "imperator")] PartyAgenda,
    #[cfg(feature = "imperator")] PartyType,
    #[cfg(feature = "imperator")] PostSetupCharacters,
    #[cfg(feature = "imperator")] Price,
    #[cfg(feature = "imperator")] ProvinceRank,
    #[cfg(feature = "imperator")] SetupCharacters,
    #[cfg(feature = "imperator")] SetupProvinces,
    #[cfg(feature = "imperator")] TechnologyTable,
    #[cfg(feature = "imperator")] TradeGood,
    #[cfg(feature = "imperator")] Treasure,
    #[cfg(feature = "imperator")] UnitAbility,

    #[cfg(feature = "hoi4")] Ability,
    #[cfg(feature = "hoi4")] Acclimatation,
    #[cfg(feature = "hoi4")] AceModifier,
    #[cfg(feature = "hoi4")] AdjacencyRule,
    #[cfg(feature = "hoi4")] AdvisorSlot,
    #[cfg(feature = "hoi4")] AiStrategyType,
    #[cfg(feature = "hoi4")] Continent,
    #[cfg(feature = "hoi4")] CountryLeaderTrait,
    #[cfg(feature = "hoi4")] CountryTag,
    #[cfg(feature = "hoi4")] CountryTagAlias,
    #[cfg(feature = "hoi4")] DecisionCategory,
    #[cfg(feature = "hoi4")] DynamicModifier,
    #[cfg(feature = "hoi4")] Equipment,
    #[cfg(feature = "hoi4")] EquipmentBonusType,
    #[cfg(feature = "hoi4")] EquipmentCategory,
    #[cfg(feature = "hoi4")] EquipmentStat,
    #[cfg(feature = "hoi4")] EquipmentModule,
    #[cfg(feature = "hoi4")] GraphicalTerrain,
    #[cfg(feature = "hoi4")] IdeaCategory,
    #[cfg(feature = "hoi4")] IdeaGroup,
    #[cfg(feature = "hoi4")] IdeologyGroup,
    #[cfg(feature = "hoi4")] IndustrialOrg,
    #[cfg(feature = "hoi4")] IndustrialOrgBonusWeight,
    #[cfg(feature = "hoi4")] IndustrialOrgPolicy,
    #[cfg(feature = "hoi4")] IndustrialOrgTrait,
    #[cfg(feature = "hoi4")] NationalFocus,
    #[cfg(feature = "hoi4")] Operation,
    #[cfg(feature = "hoi4")] ProductionStat,
    #[cfg(feature = "hoi4")] ProjectTag,
    #[cfg(feature = "hoi4")] PrototypeReward,
    #[cfg(feature = "hoi4")] Resource,
    #[cfg(feature = "hoi4")] ScriptedConstant,
    #[cfg(feature = "hoi4")] ScriptedEnum,
    #[cfg(feature = "hoi4")] ScriptedLocalisation,
    #[cfg(feature = "hoi4")] SoundEffect,
    #[cfg(feature = "hoi4")] SoundFalloff,
    #[cfg(feature = "hoi4")] SpawnPoint,
    #[cfg(feature = "hoi4")] Specialization,
    #[cfg(feature = "hoi4")] SpecialProject,
    #[cfg(feature = "hoi4")] ScientistTrait,
    #[cfg(feature = "hoi4")] Sprite,
    #[cfg(feature = "hoi4")] State,
    #[cfg(feature = "hoi4")] StateCategory,
    #[cfg(feature = "hoi4")] SubUnit,
    #[cfg(feature = "hoi4")] UnitLeaderSkill,
    #[cfg(feature = "hoi4")] UnitLeaderTrait,
}

/// Display items in `separated word case` for maximum friendliness.
///
/// Unfortunately there's no option for this in `strum` so we have to roll our own
/// by using `snake_case` and changing the `_` to a space.
impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let s: &'static str = self.into();
        write!(f, "{}", s.replace('_', " "))
    }
}

impl Item {
    /// Returns a path where items of this type are kept in the script files. Can be `""` for items
    /// that are built in.
    ///
    /// These paths are used both for the user in error reports, and to find the items when loading them.
    pub fn path(self) -> &'static str {
        #[allow(clippy::match_same_arms)]
        // These variants are in the same order as the Item enum declaration
        match self {
            #[cfg(feature = "jomini")]
            Item::Accessory => "gfx/portraits/accessories/",
            #[cfg(feature = "jomini")]
            Item::AccessoryTag => "gfx/portraits/accessories/",
            #[cfg(feature = "jomini")]
            Item::AccessoryVariation => "gfx/portraits/accessory_variations/",
            #[cfg(feature = "jomini")]
            Item::AccessoryVariationLayout => "gfx/portraits/accessory_variations/",
            #[cfg(feature = "jomini")]
            Item::AccessoryVariationTextures => "gfx/portraits/accessory_variations/",
            Item::Achievement => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/achievements/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/achievements/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/achievements/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/achievements.txt",
            },
            #[cfg(feature = "jomini")]
            Item::AchievementGroup => "common/achievement_groups.txt",
            Item::Asset => "gfx/models/",
            Item::BlendShape => "gfx/models/",
            #[cfg(feature = "jomini")]
            Item::CharacterInteraction => "common/character_interactions/",
            #[cfg(feature = "jomini")]
            Item::Coa => "common/coat_of_arms/coat_of_arms/",
            #[cfg(feature = "jomini")]
            Item::CoaColorList => "common/coat_of_arms/template_lists/",
            #[cfg(feature = "jomini")]
            Item::CoaColoredEmblemList => "common/coat_of_arms/template_lists/",
            #[cfg(feature = "ck3")]
            Item::CoaDesignerColorPalette => "gfx/coat_of_arms/color_palettes/",
            #[cfg(feature = "jomini")]
            Item::CoaDesignerColoredEmblem => "gfx/coat_of_arms/colored_emblems/",
            #[cfg(feature = "ck3")]
            Item::CoaDesignerEmblemLayout => "gfx/coat_of_arms/emblem_layouts/",
            #[cfg(feature = "jomini")]
            Item::CoaDesignerPattern => "gfx/coat_of_arms/patterns/",
            #[cfg(feature = "jomini")]
            Item::CoaPatternList => "common/coat_of_arms/template_lists/",
            #[cfg(feature = "jomini")]
            Item::CoaTemplate => "common/coat_of_arms/coat_of_arms/",
            #[cfg(feature = "jomini")]
            Item::CoaTemplateList => "common/coat_of_arms/template_lists/",
            #[cfg(feature = "jomini")]
            Item::CoaTexturedEmblemList => "common/coat_of_arms/template_lists/",
            #[cfg(feature = "jomini")]
            Item::Culture => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/culture/cultures/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/cultures/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/cultures/",
            },
            #[cfg(feature = "jomini")]
            Item::CustomLocalization => "common/customizable_localization/",
            Item::Decision => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/decisions/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/decisions/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "decisions/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/decisions/",
            },
            Item::Define => "common/defines/",
            Item::Dlc => "dlc_metadata/",
            Item::DlcFeature => "",
            Item::DlcName => "dlc_metadata/",
            Item::Directory => "",
            #[cfg(feature = "jomini")]
            Item::EffectLocalization => "common/effect_localization/",
            Item::Entity => "gfx/models/",
            Item::Entry => "",
            #[cfg(feature = "jomini")]
            Item::Ethnicity => "common/ethnicities/",
            Item::Event => "events/",
            Item::EventNamespace => "events/",
            Item::File => "",
            Item::Font => "fonts/",
            Item::Fontfiles => "fonts/",
            #[cfg(feature = "jomini")]
            Item::GameConcept => "common/game_concepts/",
            Item::GameRule => "common/game_rules/",
            Item::GameRuleSetting => "common/game_rules/",
            #[cfg(feature = "jomini")]
            Item::GeneAgePreset => "common/genes/",
            #[cfg(feature = "jomini")]
            Item::GeneAttribute => "gfx/models/",
            #[cfg(feature = "jomini")]
            Item::GeneCategory => "common/genes/",
            #[cfg(feature = "jomini")]
            Item::GovernmentType => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/governments/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/government_types/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/governments/",
            },
            Item::GuiLayer => "gui/",
            Item::GuiTemplate => "gui/",
            Item::GuiType => "gui/",
            Item::Localization => "localization/",
            Item::MapEnvironment => "gfx/map/environment/",
            Item::MapMode => "gfx/map/map_modes/",
            Item::Modifier => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/modifiers/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/static_modifiers/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/modifiers/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/modifiers/", // TODO HOI4
            },
            Item::Music => "music/",
            Item::MusicPlayerCategory => "music/music_player_categories/",
            #[cfg(feature = "jomini")]
            Item::NamedColor => "common/named_colors/",
            Item::OnAction => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/on_action/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/on_actions/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/on_action/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/on_actions/", // TODO HOI4
            },
            Item::Pdxmesh => "gfx/models/",
            #[cfg(feature = "jomini")]
            Item::PortraitAnimation => "gfx/portraits/portrait_animations/",
            #[cfg(feature = "jomini")]
            Item::PortraitCamera => "gfx/portraits/cameras/",
            #[cfg(feature = "jomini")]
            Item::PortraitEnvironment => "gfx/portraits/environments/",
            #[cfg(feature = "jomini")]
            Item::PortraitModifierGroup => "gfx/portraits/portrait_modifiers/",
            #[cfg(feature = "jomini")]
            Item::PortraitModifierPack => "gfx/portraits/portrait_animations/",
            Item::Province => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "map_data/definition.csv",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "map_data/provinces.png",
                #[cfg(feature = "imperator")]
                Game::Imperator => "map_data/provinces.png",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "map/definition.csv", // TODO HOI4
            },
            #[cfg(feature = "jomini")]
            Item::Religion => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/religion/religions/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/religions/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/religions/",
            },
            Item::ScriptedEffect => "common/scripted_effects/",
            Item::ScriptedGui => "common/scripted_guis/",
            #[cfg(feature = "jomini")]
            Item::ScriptedList => "common/scripted_lists/",
            #[cfg(feature = "jomini")]
            Item::ScriptedModifier => "common/scripted_modifiers/",
            #[cfg(feature = "jomini")]
            Item::ScriptedRule => "common/scripted_rules/",
            Item::ScriptedTrigger => "common/scripted_triggers/",
            #[cfg(feature = "jomini")]
            Item::ScriptValue => "common/script_values/",
            Item::Shortcut => "gui/shortcuts.shortcuts",
            Item::Sound => match Game::game() {
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "sound/",
                #[cfg(feature = "jomini")]
                _ => "",
            },
            Item::Terrain => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/terrain_types/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/terrain/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/terrain_types/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/terrain/",
            },
            Item::TextFormat => "gui/",
            Item::TextIcon => "gui/",
            Item::TextureFile => "gfx/models/",
            #[cfg(feature = "jomini")]
            Item::TriggerLocalization => "common/trigger_localization/",
            Item::WidgetName => "gui/",

            #[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
            Item::Building => "common/buildings/",
            #[cfg(any(feature = "ck3", feature = "hoi4"))]
            Item::Character => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "history/characters/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/characters/",
            },
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::CharacterTemplate => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/scripted_character_templates/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/character_templates/",
            },
            #[cfg(any(feature = "vic3", feature = "imperator", feature = "hoi4"))]
            Item::CharacterTrait => match Game::game() {
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/character_traits/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/traits/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/unit_leader/", // TODO HOI4
            },
            #[cfg(any(feature = "imperator", feature = "hoi4"))]
            Item::CombatTactic => match Game::game() {
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/combat_tactics/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/combat_tactics.txt", // TODO HOI4
            },
            #[cfg(any(feature = "vic3", feature = "imperator", feature = "hoi4"))]
            Item::Country => match Game::game() {
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/country_definitions/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "setup/countries/countries.txt",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/countries/", // TODO HOI4
            },
            #[cfg(any(feature = "ck3", feature = "imperator"))]
            Item::DeathReason => "common/deathreasons/",
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::Dna => "common/dna_data/",
            #[cfg(any(feature = "ck3", feature = "imperator"))]
            Item::EventTheme => "common/event_themes/",
            #[cfg(any(feature = "imperator", feature = "hoi4"))]
            Item::Idea => "common/ideas/", // TODO HOI4
            #[cfg(any(feature = "vic3", feature = "hoi4"))]
            Item::Ideology => "common/ideologies/",
            #[cfg(any(feature = "ck3", feature = "imperator"))]
            Item::Law => "common/laws/",
            #[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator"))]
            Item::LawGroup => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "common/laws/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/laws/",
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/law_groups/",
            },
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::Message => "common/messages",
            #[cfg(any(feature = "imperator", feature = "hoi4"))]
            Item::Mission => match Game::game() {
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/missions/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/script_enums.txt",
            },
            #[cfg(any(feature = "vic3", feature = "imperator"))]
            Item::PopType => "common/pop_types/",
            #[cfg(any(feature = "ck3", feature = "imperator"))]
            Item::Region => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => "map_data/geographical_regions/",
                #[cfg(feature = "imperator")]
                Game::Imperator => "map_data/regions.txt",
            },
            #[cfg(any(feature = "vic3", feature = "imperator"))]
            Item::SubjectType => "common/subject_types/",
            #[cfg(any(feature = "vic3", feature = "hoi4"))]
            Item::Technology => match Game::game() {
                #[cfg(feature = "vic3")]
                Game::Vic3 => "common/technology/technologies/",
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => "common/technologies/", // TODO HOI4
            },
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::TutorialLesson => "common/tutorial_lessons",
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::TutorialLessonChain => "common/tutorial_lesson_chains",
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::TutorialLessonStep => "common/tutorial_lessons",
            #[cfg(any(feature = "imperator", feature = "hoi4"))]
            Item::Unit => "common/units/", // TODO HOI4
            #[cfg(any(feature = "vic3", feature = "imperator"))]
            Item::Wargoal => match Game::game() {
                #[cfg(feature = "vic3")]
                Game::Vic3 => "",
                #[cfg(feature = "imperator")]
                Game::Imperator => "common/wargoals",
            },

            #[cfg(feature = "ck3")]
            Item::AccoladeCategory => "common/accolade_types/",
            #[cfg(feature = "ck3")]
            Item::AccoladeIcon => "common/accolade_icons/",
            #[cfg(feature = "ck3")]
            Item::AccoladeName => "common/accolade_names/",
            #[cfg(feature = "ck3")]
            Item::AccoladeParameter => "common/accolade_types/",
            #[cfg(feature = "ck3")]
            Item::AccoladeType => "common/accolade_types/",
            #[cfg(feature = "ck3")]
            Item::ActivityGroupType => "common/activities/activity_group_types/",
            #[cfg(feature = "ck3")]
            Item::ActivityIntent => "common/activities/intents/",
            #[cfg(feature = "ck3")]
            Item::ActivityLocale => "common/activities/activity_locales/",
            #[cfg(feature = "ck3")]
            Item::ActivityOption => "common/activities/activity_types/",
            #[cfg(feature = "ck3")]
            Item::ActivityOptionCategory => "common/activities/activity_types/",
            #[cfg(feature = "ck3")]
            Item::ActivityPhase => "common/activities/activity_types/",
            #[cfg(feature = "ck3")]
            Item::ActivityPulseAction => "common/activities/pulse_actions/",
            #[cfg(feature = "ck3")]
            Item::ActivityState => "",
            #[cfg(feature = "ck3")]
            Item::ActivityType => "common/activities/activity_types/",
            #[cfg(feature = "ck3")]
            Item::AiWarStance => "common/ai_war_stances/",
            #[cfg(feature = "ck3")]
            Item::AgentType => "common/schemes/agent_types/",
            #[cfg(feature = "ck3")]
            Item::Amenity => "common/court_amenities/",
            #[cfg(feature = "ck3")]
            Item::AmenitySetting => "common/court_amenities/",
            #[cfg(feature = "ck3")]
            Item::ArtifactBlueprint => "common/artifacts/blueprints/",
            #[cfg(feature = "ck3")]
            Item::ArtifactFeature => "common/artifacts/features/",
            #[cfg(feature = "ck3")]
            Item::ArtifactFeatureGroup => "common/artifacts/feature_groups/",
            #[cfg(feature = "ck3")]
            Item::ArtifactHistory => "",
            #[cfg(feature = "ck3")]
            Item::ArtifactRarity => "",
            #[cfg(feature = "ck3")]
            Item::ArtifactSlot => "common/artifacts/slots/",
            #[cfg(feature = "ck3")]
            Item::ArtifactSlotType => "common/artifacts/slots/",
            #[cfg(feature = "ck3")]
            Item::ArtifactTemplate => "common/artifacts/templates/",
            #[cfg(feature = "ck3")]
            Item::ArtifactType => "common/artifacts/types/",
            #[cfg(feature = "ck3")]
            Item::ArtifactVisual => "common/artifacts/visuals/",
            #[cfg(feature = "ck3")]
            Item::Bookmark => "common/bookmarks/bookmarks/",
            #[cfg(feature = "ck3")]
            Item::BookmarkGroup => "common/bookmarks/groups/",
            #[cfg(feature = "ck3")]
            Item::BookmarkPortrait => "common/bookmark_portraits/",
            #[cfg(feature = "ck3")]
            Item::BuildingFlag => "common/buildings/",
            #[cfg(feature = "ck3")]
            Item::BuildingGfx => "common/culture/cultures/",
            #[cfg(feature = "ck3")]
            Item::CasusBelli => "common/casus_belli_types/",
            #[cfg(feature = "ck3")]
            Item::CasusBelliGroup => "common/casus_belli_groups/",
            #[cfg(feature = "ck3")]
            Item::Catalyst => "common/struggle/catalysts/",
            #[cfg(feature = "ck3")]
            Item::ChallengeCharacter => "common/bookmarks/challenge_characters/",
            #[cfg(feature = "ck3")]
            Item::CharacterBackground => "common/character_backgrounds/",
            #[cfg(feature = "ck3")]
            Item::CharacterInteractionCategory => "common/character_interaction_categories/",
            #[cfg(feature = "ck3")]
            Item::Climate => "map_data/climate.txt",
            #[cfg(feature = "ck3")]
            Item::ClothingGfx => "common/culture/cultures/",
            #[cfg(feature = "ck3")]
            Item::CoaGfx => "common/culture/cultures/",
            #[cfg(feature = "ck3")]
            Item::CoaDynamicDefinition => "common/coat_of_arms/dynamic_definitions/",
            #[cfg(feature = "ck3")]
            Item::CombatEffect => "common/combat_effects/",
            #[cfg(feature = "ck3")]
            Item::CombatPhaseEvent => "common/combat_phase_events/",
            #[cfg(feature = "ck3")]
            Item::CouncilPosition => "common/council_positions/",
            #[cfg(feature = "ck3")]
            Item::CouncilTask => "common/council_tasks/",
            #[cfg(feature = "ck3")]
            Item::Countermeasure => "common/schemes/scheme_countermeasures/",
            #[cfg(feature = "ck3")]
            Item::CountermeasureParameter => "common/schemes/scheme_countermeasures/",
            #[cfg(feature = "ck3")]
            Item::CourtPosition => "common/court_positions/types/",
            #[cfg(feature = "ck3")]
            Item::CourtPositionTask => "common/court_positions/tasks/",
            #[cfg(feature = "ck3")]
            Item::CourtSceneCulture => "gfx/court_scene/scene_cultures/",
            #[cfg(feature = "ck3")]
            Item::CourtSceneGroup => "gfx/court_scene/character_groups/",
            #[cfg(feature = "ck3")]
            Item::CourtSceneRole => "gfx/court_scene/character_roles/",
            #[cfg(feature = "ck3")]
            Item::CourtSceneSetting => "gfx/court_scene/scene_settings/",
            #[cfg(feature = "ck3")]
            Item::CourtType => "common/court_types/",
            #[cfg(feature = "ck3")]
            Item::CourtierGuestManagement => "common/courtier_guest_management/",
            #[cfg(feature = "ck3")]
            Item::CultureAesthetic => "common/culture/aesthetics_bundles/",
            #[cfg(feature = "ck3")]
            Item::CultureCreationName => "common/culture/creation_names/",
            #[cfg(feature = "ck3")]
            Item::CultureEra => "common/culture/eras/",
            #[cfg(feature = "ck3")]
            Item::CultureEthos => "common/culture/pillars/",
            #[cfg(feature = "ck3")]
            Item::CultureHeritage => "common/culture/pillars/",
            #[cfg(feature = "ck3")]
            Item::CultureHistory => "history/cultures/",
            #[cfg(feature = "ck3")]
            Item::CultureParameter => "common/culture/",
            #[cfg(feature = "ck3")]
            Item::CulturePillar => "common/culture/pillars/",
            #[cfg(feature = "ck3")]
            Item::CultureTradition => "common/culture/traditions/",
            #[cfg(feature = "ck3")]
            Item::CultureTraditionCategory => "common/culture/traditions/",
            #[cfg(feature = "ck3")]
            Item::DangerType => "",
            #[cfg(feature = "ck3")]
            Item::DecisionGroup => "common/decision_group_types/",
            #[cfg(feature = "ck3")]
            Item::DiarchyMandate => "common/diarchies/diarchy_mandates/",
            #[cfg(feature = "ck3")]
            Item::DiarchyParameter => "common/diarchies/diarchy_types/",
            #[cfg(feature = "ck3")]
            Item::DiarchyType => "common/diarchies/diarchy_types/",
            #[cfg(feature = "ck3")]
            Item::Doctrine => "common/religion/doctrines/",
            #[cfg(feature = "ck3")]
            Item::DoctrineCategory => "common/religion/doctrines/",
            #[cfg(feature = "ck3")]
            Item::DoctrineParameter => "common/religion/doctrines/",
            #[cfg(feature = "ck3")]
            Item::DomicileBuilding => "common/domiciles/buildings/",
            #[cfg(feature = "ck3")]
            Item::DomicileParameter => "common/domiciles/buildings/",
            #[cfg(feature = "ck3")]
            Item::DomicileType => "common/domiciles/types/",
            #[cfg(feature = "ck3")]
            Item::Dynasty => "common/dynasties/",
            #[cfg(feature = "ck3")]
            Item::DynastyLegacy => "common/dynasty_legacies/",
            #[cfg(feature = "ck3")]
            Item::DynastyPerk => "common/dynasty_perks/",
            #[cfg(feature = "ck3")]
            Item::EpidemicType => "common/epidemics/",
            #[cfg(feature = "ck3")]
            Item::EpidemicDeathReason => "common/deathreasons/",
            #[cfg(feature = "ck3")]
            Item::EventBackground => "common/event_backgrounds/",
            #[cfg(feature = "ck3")]
            Item::EventEffect2d => "common/event_2d_effects/",
            #[cfg(feature = "ck3")]
            Item::EventTransition => "common/event_transitions/",
            #[cfg(feature = "ck3")]
            Item::Faith => "common/religion/religions/",
            #[cfg(feature = "ck3")]
            Item::FaithIcon => "common/religion/religions/",
            #[cfg(feature = "ck3")]
            Item::FervorModifier => "common/religion/fervor_modifiers/",
            #[cfg(feature = "ck3")]
            Item::Faction => "common/factions/",
            #[cfg(feature = "ck3")]
            Item::Flavorization => "common/flavorization/",
            #[cfg(feature = "ck3")]
            Item::Focus => "common/focuses/",
            #[cfg(feature = "ck3")]
            Item::GeneticConstraint => "common/traits/",
            #[cfg(feature = "ck3")]
            Item::GovernmentFlag => "common/governments/",
            #[cfg(feature = "ck3")]
            Item::GraphicalFaith => "common/religion/religions/",
            #[cfg(feature = "ck3")]
            Item::GuestInviteRule => "common/activities/guest_invite_rules/",
            #[cfg(feature = "ck3")]
            Item::GuestSubset => "common/activities/activity_types/",
            #[cfg(feature = "ck3")]
            Item::GuestSystem => "common/guest_system/",
            #[cfg(feature = "ck3")]
            Item::HoldingFlag => "common/holdings/",
            #[cfg(feature = "ck3")]
            Item::HoldingType => "common/holdings/",
            #[cfg(feature = "ck3")]
            Item::HolySite => "common/religion/holy_sites/",
            #[cfg(feature = "ck3")]
            Item::HolySiteFlag => "common/religion/holy_sites/",
            #[cfg(feature = "ck3")]
            Item::Hook => "common/hook_types/",
            #[cfg(feature = "ck3")]
            Item::House => "common/dynasty_houses/",
            #[cfg(feature = "ck3")]
            Item::HousePowerBonus => "common/house_power_bonus/",
            #[cfg(feature = "ck3")]
            Item::HouseUnity => "common/house_unities/",
            #[cfg(feature = "ck3")]
            Item::HouseUnityParameter => "common/house_unities",
            #[cfg(feature = "ck3")]
            Item::HouseUnityStage => "common/house_unities/",
            #[cfg(feature = "ck3")]
            Item::ImportantAction => "common/important_actions/",
            #[cfg(feature = "ck3")]
            Item::Innovation => "common/culture/innovations/",
            #[cfg(feature = "ck3")]
            Item::InnovationFlag => "common/culture/innovations/",
            #[cfg(feature = "ck3")]
            Item::Inspiration => "common/inspirations/",
            #[cfg(feature = "ck3")]
            Item::Language => "common/culture/pillars/",
            #[cfg(feature = "ck3")]
            Item::LawFlag => "common/laws/",
            #[cfg(feature = "ck3")]
            Item::LeaseContract => "common/lease_contracts/",
            #[cfg(feature = "ck3")]
            Item::LegendChapter => "common/legends/chronicles/",
            #[cfg(feature = "ck3")]
            Item::LegendChronicle => "common/legends/chronicles/",
            #[cfg(feature = "ck3")]
            Item::LegendProperty => "common/legends/chronicles/",
            #[cfg(feature = "ck3")]
            Item::LegendSeed => "common/legends/legend_seeds/",
            #[cfg(feature = "ck3")]
            Item::LegendType => "common/legends/legend_types/",
            #[cfg(feature = "ck3")]
            Item::LegitimacyFlag => "common/legitimacy/",
            #[cfg(feature = "ck3")]
            Item::LegitimacyType => "common/legitimacy/",
            #[cfg(feature = "ck3")]
            Item::Lifestyle => "common/lifestyles/",
            #[cfg(feature = "ck3")]
            Item::MartialCustom => "common/culture/pillars/",
            #[cfg(feature = "ck3")]
            Item::MemoryCategory => "common/character_memory_types/",
            #[cfg(feature = "ck3")]
            Item::MemoryType => "common/character_memory_types/",
            #[cfg(feature = "ck3")]
            Item::MenAtArms => "common/men_at_arms_types/",
            #[cfg(feature = "ck3")]
            Item::MenAtArmsBase => "common/men_at_arms_types/",
            #[cfg(feature = "ck3")]
            Item::MessageFilterType => "common/message_filter_types/",
            #[cfg(feature = "ck3")]
            Item::MessageGroupType => "common/message_group_types/",
            #[cfg(feature = "ck3")]
            Item::ModifierFormat => "common/modifier_definition_formats/",
            #[cfg(feature = "ck3")]
            Item::MottoInsert => "common/dynasty_house_motto_inserts/",
            #[cfg(feature = "ck3")]
            Item::Motto => "common/dynasty_house_mottos/",
            #[cfg(feature = "ck3")]
            Item::NameEquivalency => "common/culture/name_equivalency/",
            #[cfg(feature = "ck3")]
            Item::NameList => "common/culture/name_lists/",
            #[cfg(feature = "ck3")]
            Item::Nickname => "common/nicknames/",
            #[cfg(feature = "ck3")]
            Item::OpinionModifier => "common/opinion_modifiers/",
            #[cfg(feature = "ck3")]
            Item::Perk => "common/lifestyle_perks/",
            #[cfg(feature = "ck3")]
            Item::PerkTree => "common/lifestyle_perks/",
            #[cfg(feature = "ck3")]
            Item::PlayableDifficultyInfo => "common/playable_difficulty_infos/",
            #[cfg(feature = "ck3")]
            Item::PointOfInterest => "common/travel/point_of_interest_types/",
            #[cfg(feature = "ck3")]
            Item::PoolSelector => "common/pool_character_selectors/",
            #[cfg(feature = "ck3")]
            Item::PortraitType => "common/portrait_types/",
            #[cfg(feature = "ck3")]
            Item::ProvinceMapping => "history/province_mapping/",
            #[cfg(feature = "ck3")]
            Item::PrisonType => "",
            #[cfg(feature = "ck3")]
            Item::Relation => "common/scripted_relations/",
            #[cfg(feature = "ck3")]
            Item::RelationFlag => "common/scripted_relations/",
            #[cfg(feature = "ck3")]
            Item::ReligionFamily => "common/religion/religion_families/",
            #[cfg(feature = "ck3")]
            Item::RewardItem => "",
            #[cfg(feature = "ck3")]
            Item::Scheme => "common/schemes/scheme_types",
            #[cfg(feature = "ck3")]
            Item::SchemePulseAction => "common/schemes/pulse_actions",
            #[cfg(feature = "ck3")]
            Item::ScriptedAnimation => "common/scripted_animations/",
            #[cfg(feature = "ck3")]
            Item::ScriptedCost => "common/scripted_costs/",
            #[cfg(feature = "ck3")]
            Item::ScriptedIllustration => "gfx/interface/illustrations/scripted_illustrations/",
            #[cfg(feature = "ck3")]
            Item::Secret => "common/secret_types/",
            #[cfg(feature = "ck3")]
            Item::Sexuality => "",
            #[cfg(feature = "ck3")]
            Item::Skill => "",
            #[cfg(feature = "ck3")]
            Item::SpecialBuilding => "common/buildings/",
            #[cfg(feature = "ck3")]
            Item::SpecialGuest => "common/activities/activity_types/",
            #[cfg(feature = "ck3")]
            Item::Story => "common/story_cycles/",
            #[cfg(feature = "ck3")]
            Item::Struggle => "common/struggle/struggles/",
            #[cfg(feature = "ck3")]
            Item::StruggleHistory => "history/struggles/",
            #[cfg(feature = "ck3")]
            Item::StrugglePhase => "common/struggle/struggles/",
            #[cfg(feature = "ck3")]
            Item::StrugglePhaseParameter => "common/struggle/struggles/",
            #[cfg(feature = "ck3")]
            Item::SuccessionAppointment => "common/succession_appointment/",
            #[cfg(feature = "ck3")]
            Item::SuccessionElection => "common/succession_election/",
            #[cfg(feature = "ck3")]
            Item::Suggestion => "common/suggestions/",
            #[cfg(feature = "ck3")]
            Item::TaskContractGroup => "common/task_contracts/",
            #[cfg(feature = "ck3")]
            Item::TaskContractReward => "common/task_contracts/",
            #[cfg(feature = "ck3")]
            Item::TaskContractType => "common/task_contracts/",
            #[cfg(feature = "ck3")]
            Item::TaxSlotFlag => "common/tax_slots/obligations",
            #[cfg(feature = "ck3")]
            Item::TaxSlotObligation => "common/tax_slots/obligations",
            #[cfg(feature = "ck3")]
            Item::TaxSlotType => "common/tax_slots/types",
            #[cfg(feature = "ck3")]
            Item::Title => "common/landed_titles/",
            #[cfg(feature = "ck3")]
            Item::TitleHistory => "history/titles/",
            #[cfg(feature = "ck3")]
            Item::TitleHistoryType => "",
            #[cfg(feature = "ck3")]
            Item::Trait => "common/traits/",
            #[cfg(feature = "ck3")]
            Item::TraitCategory => "",
            #[cfg(feature = "ck3")]
            Item::TraitFlag => "common/traits/",
            #[cfg(feature = "ck3")]
            Item::TraitPortraitModifier => "gfx/portraits/trait_portrait_modifiers",
            #[cfg(feature = "ck3")]
            Item::TraitTrack => "common/traits/",
            #[cfg(feature = "ck3")]
            Item::TravelOption => "common/travel/travel_options/",
            #[cfg(feature = "ck3")]
            Item::UnitGfx => "common/culture/cultures/",
            #[cfg(feature = "ck3")]
            Item::VassalContract => "common/vassal_contracts/",
            #[cfg(feature = "ck3")]
            Item::VassalContractFlag => "common/vassal_contracts/",
            #[cfg(feature = "ck3")]
            Item::VassalObligationLevel => "common/vassal_contracts/",
            #[cfg(feature = "ck3")]
            Item::VassalStance => "common/vassal_stances/",

            #[cfg(feature = "vic3")]
            Item::AcceptanceStatus => "common/acceptance_statuses/",
            #[cfg(feature = "vic3")]
            Item::AiStrategy => "common/ai_strategies/",
            #[cfg(feature = "vic3")]
            Item::Alert => "common/alert_types",
            #[cfg(feature = "vic3")]
            Item::AlertGroup => "common/alert_groups",
            #[cfg(feature = "vic3")]
            Item::Approval => "",
            #[cfg(feature = "vic3")]
            Item::Attitude => "",
            #[cfg(feature = "vic3")]
            Item::BattleCondition => "common/battle_conditions/",
            #[cfg(feature = "vic3")]
            Item::BuildingGroup => "common/building_groups/",
            #[cfg(feature = "vic3")]
            Item::BuildingType => "common/buildings/",
            #[cfg(feature = "vic3")]
            Item::BuyPackage => "common/buy_packages/",
            #[cfg(feature = "vic3")]
            Item::CanalType => "common/canals/",
            #[cfg(feature = "vic3")]
            Item::CharacterRole => "",
            #[cfg(feature = "vic3")]
            Item::CombatUnit => "common/combat_unit_types/",
            #[cfg(feature = "vic3")]
            Item::CombatUnitExperienceLevel => "common/combat_unit_experience_levels/",
            #[cfg(feature = "vic3")]
            Item::CombatUnitGroup => "common/combat_unit_groups/",
            #[cfg(feature = "vic3")]
            Item::CommanderOrder => "common/commander_orders/",
            #[cfg(feature = "vic3")]
            Item::CommanderRank => "common/commander_ranks/",
            #[cfg(feature = "vic3")]
            Item::CompanyType => "common/company_types/",
            #[cfg(feature = "vic3")]
            Item::CohesionLevel => "common/cohesion_levels/",
            #[cfg(feature = "vic3")]
            Item::CountryCreation => "common/country_creation/",
            #[cfg(feature = "vic3")]
            Item::CountryFormation => "common/country_formation/",
            #[cfg(feature = "vic3")]
            Item::CountryRank => "common/country_ranks/",
            #[cfg(feature = "vic3")]
            Item::CountryTier => "",
            #[cfg(feature = "vic3")]
            Item::CountryType => "common/country_types/",
            #[cfg(feature = "vic3")]
            Item::CultureGraphics => "common/culture_graphics/",
            #[cfg(feature = "vic3")]
            Item::Decree => "common/decrees/",
            #[cfg(feature = "vic3")]
            Item::DiplomaticAction => "common/diplomatic_actions/",
            #[cfg(feature = "vic3")]
            Item::DiplomaticCatalyst => "common/diplomatic_catalysts/",
            #[cfg(feature = "vic3")]
            Item::DiplomaticCatalystCategory => "common/diplomatic_catalyst_categories/",
            #[cfg(feature = "vic3")]
            Item::DiplomaticPlay => "common/diplomatic_plays/",
            #[cfg(feature = "vic3")]
            Item::DiscriminationTrait => "common/discrimination_traits/",
            #[cfg(feature = "vic3")]
            Item::DynamicCompanyName => "common/dynamic_company_names/",
            #[cfg(feature = "vic3")]
            Item::DynamicCountryMapColor => "common/dynamic_country_map_colors/",
            #[cfg(feature = "vic3")]
            Item::DynamicCountryName => "common/dynamic_country_names/",
            #[cfg(feature = "vic3")]
            Item::EventCategory => "",
            #[cfg(feature = "vic3")]
            Item::FlagDefinition => "common/flag_definitions/",
            #[cfg(feature = "vic3")]
            Item::Goods => "common/goods/",
            #[cfg(feature = "vic3")]
            // TODO: find out if different filenames are acceptable in this dir
            Item::GradientBorderSettings => "gfx/map/gradient_border_settings/",
            #[cfg(feature = "vic3")]
            Item::HarvestConditionType => "common/harvest_condition_types/",
            #[cfg(feature = "vic3")]
            Item::InfamyThreshold => "",
            #[cfg(feature = "vic3")]
            Item::Institution => "common/institutions/",
            #[cfg(feature = "vic3")]
            Item::InterestGroup => "common/interest_groups/",
            #[cfg(feature = "vic3")]
            Item::InterestGroupTrait => "common/interest_group_traits/",
            #[cfg(feature = "vic3")]
            Item::JournalEntry => "common/journal_entries/",
            #[cfg(feature = "vic3")]
            Item::JournalEntryGroup => "common/journal_entry_groups/",
            #[cfg(feature = "vic3")]
            Item::LawType => "common/laws/",
            #[cfg(feature = "vic3")]
            Item::LegitimacyLevel => "common/legitimacy_levels/",
            #[cfg(feature = "vic3")]
            Item::Level => "",
            #[cfg(feature = "vic3")]
            Item::LibertyDesireLevel => "common/liberty_desire_levels/",
            #[cfg(feature = "vic3")]
            Item::MapLayer => "gfx/map/map_object_data/layers.txt",
            #[cfg(feature = "vic3")]
            Item::MapInteractionType => "common/map_interaction_types/",
            #[cfg(feature = "vic3")]
            Item::MapNotificationType => "common/map_notification_types/",
            #[cfg(feature = "vic3")]
            Item::MediaAlias => "gfx/media_aliases/",
            #[cfg(feature = "vic3")]
            Item::MilitaryFormationFlag => "common/military_formation_flags/",
            #[cfg(feature = "vic3")]
            Item::MobilizationOption => "common/mobilization_options/",
            #[cfg(feature = "vic3")]
            Item::MobilizationOptionGroup => "common/mobilization_option_groups/",
            #[cfg(feature = "vic3")]
            Item::ModifierTypeDefinition => "common/modifier_type_definitions/",
            #[cfg(feature = "vic3")]
            Item::Objective => "common/objectives/",
            #[cfg(feature = "vic3")]
            Item::ObjectiveSubgoal => "common/objective_subgoals/",
            #[cfg(feature = "vic3")]
            Item::ObjectiveSubgoalCategory => "common/objective_subgoal_categories/",
            #[cfg(feature = "vic3")]
            Item::Party => "common/parties/",
            #[cfg(feature = "vic3")]
            Item::PoliticalLobby => "common/political_lobbies/",
            #[cfg(feature = "vic3")]
            Item::PoliticalLobbyAppeasement => "common/political_lobby_appeasement/",
            #[cfg(feature = "vic3")]
            Item::PoliticalMovement => "common/political_movements",
            #[cfg(feature = "vic3")]
            Item::PoliticalMovementCategory => "common/political_movement_categories",
            #[cfg(feature = "vic3")]
            Item::PoliticalMovementPopSupport => "common/political_movement_pop_support",
            #[cfg(feature = "vic3")]
            Item::PopNeed => "common/pop_needs/",
            #[cfg(feature = "vic3")]
            Item::PowerBlocCoaPiece => "common/power_bloc_coa_pieces/",
            #[cfg(feature = "vic3")]
            Item::PowerBlocIdentity => "common/power_bloc_identities/",
            #[cfg(feature = "vic3")]
            Item::PowerBlocMapTexture => "common/power_bloc_map_textures/",
            #[cfg(feature = "vic3")]
            Item::PowerBlocName => "common/power_bloc_names/",
            #[cfg(feature = "vic3")]
            Item::Principle => "common/power_bloc_principles/",
            #[cfg(feature = "vic3")]
            Item::PrincipleGroup => "common/power_bloc_principle_groups/",
            #[cfg(feature = "vic3")]
            Item::ProductionMethod => "common/production_methods/",
            #[cfg(feature = "vic3")]
            Item::ProductionMethodGroup => "common/production_method_groups/",
            #[cfg(feature = "vic3")]
            Item::ProposalType => "common/proposal_types/",
            #[cfg(feature = "vic3")]
            Item::RelationsThreshold => "",
            #[cfg(feature = "vic3")]
            Item::ScriptedButton => "common/scripted_buttons/",
            #[cfg(feature = "vic3")]
            Item::ScriptedProgressBar => "common/scripted_progress_bars/",
            #[cfg(feature = "vic3")]
            Item::ScriptedTest => "common/scripted_tests/",
            #[cfg(feature = "vic3")]
            Item::SecretGoal => "",
            #[cfg(feature = "vic3")]
            Item::Skin => "gfx/skins/",
            #[cfg(feature = "vic3")]
            Item::SocialClass => "common/social_classes/",
            #[cfg(feature = "vic3")]
            Item::SocialHierarchy => "common/social_hierarchies/",
            #[cfg(feature = "vic3")]
            Item::StateRegion => "map_data/state_regions/",
            #[cfg(feature = "vic3")]
            Item::StateTrait => "common/state_traits/",
            #[cfg(feature = "vic3")]
            Item::Strata => "",
            #[cfg(feature = "vic3")]
            Item::StrategicRegion => "common/strategic_regions/",
            #[cfg(feature = "vic3")]
            Item::TechnologyEra => "common/technology/eras/",
            #[cfg(feature = "vic3")]
            Item::TerrainKey => "common/labels/",
            #[cfg(feature = "vic3")]
            Item::TerrainLabel => "common/labels/",
            #[cfg(feature = "vic3")]
            Item::TerrainManipulator => "common/terrain_manipulators/",
            #[cfg(feature = "vic3")]
            Item::TerrainMask => "gfx/map/masks/",
            #[cfg(feature = "vic3")]
            Item::TerrainMaterial => "gfx/map/terrain/materials.settings",
            #[cfg(feature = "vic3")]
            Item::Theme => "common/themes/",
            #[cfg(feature = "vic3")]
            Item::TransferOfPower => "",

            #[cfg(feature = "imperator")]
            Item::AiPlanGoals => "common/ai_plan_goals/",
            #[cfg(feature = "imperator")]
            Item::Ambition => "common/ambitions/",
            #[cfg(feature = "imperator")]
            Item::Area => "map_data/areas.txt",
            #[cfg(feature = "imperator")]
            Item::CultureGroup => "common/cultures/",
            #[cfg(feature = "imperator")]
            Item::Deity => "common/deities/",
            #[cfg(feature = "imperator")]
            Item::DeityCategory => "common/deity_categories/",
            #[cfg(feature = "imperator")]
            Item::DiplomaticStance => "common/diplomatic_stances/",
            #[cfg(feature = "imperator")]
            Item::EconomicPolicy => "common/economic_policies/",
            #[cfg(feature = "imperator")]
            Item::EventPicture => "common/event_pictures/",
            #[cfg(feature = "imperator")]
            Item::GovernorPolicy => "common/governor_policies/",
            #[cfg(feature = "imperator")]
            Item::GraphicalCultureType => "common/graphical_culture_types/",
            #[cfg(feature = "imperator")]
            Item::GreatWorkEffect => "common/great_work_effects/",
            #[cfg(feature = "imperator")]
            Item::GreatWorkEffectTier => "common/great_work_effect_tiers/",
            #[cfg(feature = "imperator")]
            Item::GreatWorkCategory => "common/great_work_categories/",
            #[cfg(feature = "imperator")]
            Item::GreatWorkMaterial => "common/great_work_materials/",
            #[cfg(feature = "imperator")]
            Item::GreatWorkModule => "common/great_work_modules/",
            #[cfg(feature = "imperator")]
            Item::GreatWorkTemplate => "common/great_work_templates/",
            #[cfg(feature = "imperator")]
            Item::Heritage => "common/heritage/",
            #[cfg(feature = "imperator")]
            Item::Invention => "common/inventions/",
            #[cfg(feature = "imperator")]
            Item::InventionGroup => "common/inventions/",
            #[cfg(feature = "imperator")]
            Item::LegionDistinction => "common/legion_distinctions/",
            #[cfg(feature = "imperator")]
            Item::LevyTemplate => "common/levy_templates/",
            #[cfg(feature = "imperator")]
            Item::Loyalty => "common/loyalty/",
            #[cfg(feature = "imperator")]
            Item::MilitaryTraditionTree => "common/military_traditions/",
            #[cfg(feature = "imperator")]
            Item::MilitaryTradition => "common/military_traditions/",
            #[cfg(feature = "imperator")]
            Item::MissionTask => "common/missions/",
            #[cfg(feature = "imperator")]
            Item::Office => "common/offices/",
            #[cfg(feature = "imperator")]
            Item::Opinion => "common/opinions/",
            #[cfg(feature = "imperator")]
            Item::PartyAgenda => "common/party_agendas",
            #[cfg(feature = "imperator")]
            Item::PartyType => "common/party_types/",
            #[cfg(feature = "imperator")]
            Item::PostSetupCharacters => "setup/post_character/",
            #[cfg(feature = "imperator")]
            Item::Price => "common/prices/",
            #[cfg(feature = "imperator")]
            Item::ProvinceRank => "common/province_ranks/",
            #[cfg(feature = "imperator")]
            Item::TechnologyTable => "common/technology_tables/",
            #[cfg(feature = "imperator")]
            Item::SetupCharacters => "setup/characters/",
            #[cfg(feature = "imperator")]
            Item::SetupProvinces => "setup/provinces/",
            #[cfg(feature = "imperator")]
            Item::TradeGood => "common/trade_goods/",
            #[cfg(feature = "imperator")]
            Item::Treasure => "setup/main/",
            #[cfg(feature = "imperator")]
            Item::UnitAbility => "common/unit_abilities/",

            #[cfg(feature = "hoi4")]
            Item::Ability => "common/abilities/",
            #[cfg(feature = "hoi4")]
            Item::Acclimatation => "common/acclimatation.txt",
            #[cfg(feature = "hoi4")]
            Item::AdjacencyRule => "map/adjacency_rules.txt",
            #[cfg(feature = "hoi4")]
            Item::AceModifier => "common/aces",
            #[cfg(feature = "hoi4")]
            Item::AdvisorSlot => "common/script_enums.txt",
            #[cfg(feature = "hoi4")]
            Item::AiStrategyType => "",
            #[cfg(feature = "hoi4")]
            Item::Continent => "map/continent.txt",
            #[cfg(feature = "hoi4")]
            Item::CountryLeaderTrait => "common/country_leader/",
            #[cfg(feature = "hoi4")]
            Item::CountryTag => "common/country_tags/",
            #[cfg(feature = "hoi4")]
            Item::CountryTagAlias => "common/country_tag_aliases/",
            #[cfg(feature = "hoi4")]
            Item::DecisionCategory => "common/decisions/categories/",
            #[cfg(feature = "hoi4")]
            Item::DynamicModifier => "common/dynamic_modifiers/",
            #[cfg(feature = "hoi4")]
            Item::Equipment => "common/units/equipment/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::EquipmentBonusType => "common/script_enums.txt",
            #[cfg(feature = "hoi4")]
            Item::EquipmentCategory => "common/script_enums.txt",
            #[cfg(feature = "hoi4")]
            Item::EquipmentStat => "common/script_enums.txt",
            #[cfg(feature = "hoi4")]
            Item::EquipmentModule => "common/units/equipment/modules/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::GraphicalTerrain => "common/terrain/",
            #[cfg(feature = "hoi4")]
            Item::IdeaCategory => "", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::IdeaGroup => "common/ideas/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::IdeologyGroup => "common/ideologies/",
            #[cfg(feature = "hoi4")]
            Item::IndustrialOrg => "common/military_industrial_organization/organizations/",
            #[cfg(feature = "hoi4")]
            Item::IndustrialOrgBonusWeight => {
                "common/military_industrial_organization/ai_bonus_weights/"
            }
            #[cfg(feature = "hoi4")]
            Item::IndustrialOrgPolicy => "common/military_industrial_organization/policies/",
            #[cfg(feature = "hoi4")]
            Item::IndustrialOrgTrait => "common/military_industrial_organization/organizations/",
            #[cfg(feature = "hoi4")]
            Item::NationalFocus => "common/national_focus/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::Operation => "common/operations/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::ProductionStat => "common/script_enum.txt",
            #[cfg(feature = "hoi4")]
            Item::ProjectTag => "common/special_projects/project_tags/",
            #[cfg(feature = "hoi4")]
            Item::PrototypeReward => "common/special_projects/prototype_rewards/",
            #[cfg(feature = "hoi4")]
            Item::Resource => "common/resources/",
            #[cfg(feature = "hoi4")]
            Item::ScriptedConstant => "common/scripted_constants/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::ScriptedEnum => "common/script_enums.txt",
            #[cfg(feature = "hoi4")]
            Item::ScriptedLocalisation => "common/scripted_localisation/",
            #[cfg(feature = "hoi4")]
            Item::SoundEffect | Item::SoundFalloff => "sound/",
            #[cfg(feature = "hoi4")]
            Item::SpawnPoint => "common/buildings/",
            #[cfg(feature = "hoi4")]
            Item::Specialization => "common/special_projects/specialization/",
            #[cfg(feature = "hoi4")]
            Item::SpecialProject => "common/special_projects/projects/",
            #[cfg(feature = "hoi4")]
            Item::ScientistTrait => "common/scientist_traits/",
            #[cfg(feature = "hoi4")]
            Item::Sprite => "gfx/",
            #[cfg(feature = "hoi4")]
            Item::State => "history/states/",
            #[cfg(feature = "hoi4")]
            Item::StateCategory => "common/state_category/",
            #[cfg(feature = "hoi4")]
            Item::SubUnit => "common/units/", // TODO HOI4
            #[cfg(feature = "hoi4")]
            Item::UnitLeaderSkill => "common/unit_leader/",
            #[cfg(feature = "hoi4")]
            Item::UnitLeaderTrait => "common/unit_leader/",
        }
    }

    /// Confidence value to use when reporting that an item is missing.
    /// Should be `Strong` for most, `Weak` for items that aren't defined anywhere but just used (such as gfx flags).
    pub fn confidence(self) -> Confidence {
        match self {
            #[cfg(feature = "jomini")]
            Item::AccessoryTag => Confidence::Weak,

            // GuiType and GuiTemplate are here because referring to templates in other mods is a
            // common compatibility technique.
            Item::GuiType | Item::GuiTemplate | Item::Sound => Confidence::Weak,

            #[cfg(feature = "ck3")]
            Item::AccoladeCategory
            | Item::BuildingGfx
            | Item::ClothingGfx
            | Item::CoaGfx
            | Item::MemoryCategory
            | Item::UnitGfx => Confidence::Weak,

            #[cfg(feature = "ck3")]
            Item::SpecialBuilding => Confidence::Reasonable,

            _ => Confidence::Strong,
        }
    }

    /// Severity value to use when reporting that an item is missing.
    /// * `Error` - most things
    /// * `Warning` - things that only impact visuals or presentation
    /// * `Untidy` - things that don't matter much at all
    /// * `Fatal` - things that cause crashes if they're missing
    ///
    /// This is only one piece of the severity puzzle. It can also depend on the caller who's expecting the item to exist.
    /// That part isn't handled yet.
    pub fn severity(self) -> Severity {
        match self {
            // GuiType and GuiTemplate are here because referring to templates in other mods is a
            // common compatibility technique.
            Item::GuiType | Item::GuiTemplate => Severity::Untidy,

            Item::File | Item::Localization | Item::MapEnvironment => Severity::Warning,

            #[cfg(feature = "jomini")]
            Item::Accessory
            | Item::AccessoryTag
            | Item::AccessoryVariation
            | Item::AccessoryVariationLayout
            | Item::AccessoryVariationTextures
            | Item::Coa
            | Item::CoaColorList
            | Item::CoaColoredEmblemList
            | Item::CoaPatternList
            | Item::CoaTemplate
            | Item::CoaTemplateList
            | Item::CoaTexturedEmblemList
            | Item::CustomLocalization
            | Item::EffectLocalization
            | Item::Ethnicity
            | Item::GameConcept
            | Item::NamedColor
            | Item::PortraitAnimation
            | Item::PortraitCamera
            | Item::PortraitEnvironment
            | Item::Sound
            | Item::TextFormat
            | Item::TextIcon
            | Item::TextureFile
            | Item::TriggerLocalization => Severity::Warning,

            #[cfg(feature = "ck3")]
            Item::AccoladeIcon
            | Item::ArtifactVisual
            | Item::BuildingGfx
            | Item::ClothingGfx
            | Item::CoaDynamicDefinition
            | Item::CoaGfx
            | Item::CultureAesthetic
            | Item::CultureCreationName
            | Item::EventBackground
            | Item::EventTheme
            | Item::EventTransition
            | Item::Flavorization
            | Item::GraphicalFaith
            | Item::ModifierFormat
            | Item::MottoInsert
            | Item::Motto
            | Item::Music
            | Item::Nickname
            | Item::ScriptedIllustration
            | Item::UnitGfx => Severity::Warning,

            #[cfg(feature = "vic3")]
            Item::MapLayer
            | Item::ModifierTypeDefinition
            | Item::TerrainManipulator
            | Item::TerrainMask
            | Item::TerrainMaterial => Severity::Warning,

            #[cfg(feature = "hoi4")]
            Item::Sprite => Severity::Warning,

            _ => Severity::Error,
        }
    }
}

/// The callback type for adding one item instance to the database.
pub(crate) type ItemAdder = fn(&mut Db, Token, Block);

/// The specification for loading an [`Item`] type into the [`Db`].
///
/// An instance of this can be placed in every `data` module using the `inventory::submit!` macro.
/// This will register the loader so that the [`Everything`] object can load all defined items.
// Note that this is an enum so that users can more conveniently construct it. It used to be a
// struct with various constructor functions, but that didn't work because the ItemAdder type has a
// &mut in it, and that wasn't allowed in const functions even though the function pointer itself
// is const. See https://github.com/rust-lang/rust/issues/57349 for details.
// TODO: once that issue stabilizes, we can revisit the ItemLoader type.
pub(crate) enum ItemLoader {
    /// A convenience variant for loaders that are the most common type.
    ///
    /// * [`GameFlags`] is which games this item loader is for.
    /// * [`Item`] is the item type being loaded.
    ///
    /// The [`ItemAdder`] function does not have to load exclusively this type of item.
    /// Related items are ok. The main use of the [`Item`] field is to get the path for this item
    /// type, so that files are loaded from that folder.
    ///
    /// `Normal` loaders have extension `.txt`, `LoadAsFile::No`, and `Recursive::Maybe`. They default
    /// to a [`PdxEncoding`] appropriate to the game being validated.
    Normal(GameFlags, Item, ItemAdder),
    /// A variant that allows the full range of item loader behvavior.
    /// * [`PdxEncoding`] indicates whether to expect utf-8 and/or a BOM in the files.
    /// * The `&'static str` is the file extension to look for (including the dot).
    /// * [`LoadAsFile`] is whether to load the whole file as one item, or treat it as normal with a
    ///   series of items in one file.
    /// * [`Recursive`] indicates whether to load subfolders of the item's main folder.
    ///   `Recursive::Maybe` means apply game-dependent logic.
    Full(GameFlags, Item, PdxEncoding, &'static str, LoadAsFile, Recursive, ItemAdder),
}

inventory::collect!(ItemLoader);

impl ItemLoader {
    pub fn for_game(&self, game: Game) -> bool {
        let game_flags = match self {
            ItemLoader::Normal(game_flags, _, _)
            | ItemLoader::Full(game_flags, _, _, _, _, _, _) => game_flags,
        };
        game_flags.contains(GameFlags::from(game))
    }

    pub fn itype(&self) -> Item {
        match self {
            ItemLoader::Normal(_, itype, _) | ItemLoader::Full(_, itype, _, _, _, _, _) => *itype,
        }
    }

    pub fn encoding(&self) -> PdxEncoding {
        match self {
            ItemLoader::Normal(_, _, _) => {
                #[cfg(feature = "hoi4")]
                if Game::is_hoi4() {
                    return PdxEncoding::Utf8NoBom;
                }
                PdxEncoding::Utf8Bom
            }
            ItemLoader::Full(_, _, encoding, _, _, _, _) => *encoding,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ItemLoader::Normal(_, _, _) => ".txt",
            ItemLoader::Full(_, _, _, extension, _, _, _) => extension,
        }
    }

    pub fn whole_file(&self) -> bool {
        match self {
            ItemLoader::Normal(_, _, _) => false,
            ItemLoader::Full(_, _, _, _, load_as_file, _, _) => {
                matches!(load_as_file, LoadAsFile::Yes)
            }
        }
    }

    pub fn recursive(&self) -> bool {
        match self {
            ItemLoader::Normal(_, _, _) => {
                Game::is_ck3() && self.itype().path().starts_with("common/")
            }
            ItemLoader::Full(_, _, _, _, _, recursive, _) => match recursive {
                Recursive::Yes => true,
                Recursive::No => false,
                Recursive::Maybe => Game::is_ck3() && self.itype().path().starts_with("common/"),
            },
        }
    }

    pub fn adder(&self) -> ItemAdder {
        match self {
            ItemLoader::Normal(_, _, adder) | ItemLoader::Full(_, _, _, _, _, _, adder) => *adder,
        }
    }
}

pub enum LoadAsFile {
    Yes,
    No,
}

pub enum Recursive {
    Yes,
    No,
    Maybe,
}
