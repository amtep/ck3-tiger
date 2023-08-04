//! Stores everything known about the game and mod being validated.
//!
//! References to [`Everything`] are passed down through nearly all of the validation logic, so
//! that individual functions can access all the defined game items.

use std::borrow::Cow;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
#[cfg(feature = "ck3")]
use std::sync::RwLock;

use anyhow::Result;
#[cfg(feature = "ck3")]
use fnv::FnvHashSet;
use rayon::{scope, Scope};
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::block::Block;
#[cfg(feature = "ck3")]
use crate::ck3::data::{
    accolades::{AccoladeIcon, AccoladeName, AccoladeType},
    activities::{ActivityIntent, ActivityLocale, ActivityType, GuestInviteRule, PulseAction},
    amenities::Amenity,
    artifacts::{
        ArtifactFeature, ArtifactFeatureGroup, ArtifactSlot, ArtifactTemplate, ArtifactType,
        ArtifactVisual,
    },
    bookmarks::{Bookmark, BookmarkGroup, BookmarkPortrait},
    buildings::Building,
    casusbelli::{CasusBelli, CasusBelliGroup},
    character_templates::CharacterTemplate,
    characters::Characters,
    combat::CombatPhaseEvent,
    combat_effects::CombatEffect,
    council::{CouncilPosition, CouncilTask},
    court_scene::{CourtSceneCulture, CourtSceneGroup, CourtSceneRole, CourtSceneSetting},
    court_type::CourtType,
    courtpos::{CourtPosition, CourtPositionCategory},
    culture_history::CultureHistory,
    cultures::{
        Ck3Culture, CultureAesthetic, CultureCreationName, CultureEra, CulturePillar,
        CultureTradition,
    },
    data_binding::DataBindings,
    deathreasons::DeathReason,
    decisions::Ck3Decision,
    diarchies::{DiarchyMandate, DiarchyType},
    difficulty::PlayableDifficultyInfo,
    dna::Dna,
    doctrines::Doctrines,
    dynasties::Dynasty,
    dynasty_legacies::{DynastyLegacy, DynastyPerk},
    election::Election,
    environment::PortraitEnvironment,
    event_themes::{EventBackground, EventTheme, EventTransition},
    events::Ck3Events,
    factions::Faction,
    flavorization::Flavorization,
    focus::Focus,
    gameconcepts::GameConcepts,
    gamerules::Ck3GameRule,
    government::Government,
    holdings::Holding,
    holysites::HolySite,
    hooks::Hook,
    houses::House,
    important_actions::ImportantAction,
    innovations::Innovation,
    inspirations::Inspiration,
    interaction_cats::CharacterInteractionCategories,
    interactions::Ck3CharacterInteraction,
    laws::Ck3LawGroup,
    lifestyles::Lifestyle,
    maa::MenAtArmsTypes,
    map_environment::MapEnvironment,
    mapmodes::MapMode,
    memories::MemoryType,
    messages::Message,
    modif::ModifierFormat,
    modifiers::Ck3Modifier,
    mottos::{Motto, MottoInsert},
    music::Musics,
    namelists::NameList,
    nickname::Nickname,
    opinions::OpinionModifier,
    perks::Perk,
    points_of_interest::PointOfInterest,
    pool::{CharacterBackground, PoolSelector},
    prov_history::ProvinceHistories,
    provinces::Ck3Provinces,
    regions::Region,
    relations::Relation,
    religions::{Ck3Religion, ReligionFamily},
    schemes::Scheme,
    scripted_animations::ScriptedAnimation,
    scripted_costs::ScriptedCost,
    scripted_guis::ScriptedGui,
    scripted_illustrations::ScriptedIllustration,
    scripted_rules::ScriptedRule,
    secrets::Secret,
    sound::Sounds,
    stories::Story,
    struggle::{Catalyst, Struggle},
    suggestions::Suggestion,
    terrain::Ck3Terrain,
    title_history::TitleHistories,
    titles::Titles,
    traits::Traits,
    travel::TravelOption,
    vassalcontract::VassalContract,
    vassalstance::VassalStance,
};
#[cfg(feature = "ck3")]
use crate::ck3::tables::misc::*;
use crate::config_load::{check_for_legacy_ignore, load_filter};
use crate::context::ScopeContext;
#[cfg(feature = "ck3")]
use crate::data::coa::CoaDynamicDefinition;
#[cfg(feature = "ck3")]
use crate::data::coadesigner::{CoaDesignerColorPalette, CoaDesignerEmblemLayout};
use crate::data::{
    accessory::{Accessory, AccessoryVariation},
    assets::Assets,
    coa::{CoaTemplateList, Coas},
    coadesigner::{CoaDesignerColoredEmblem, CoaDesignerPattern},
    colors::NamedColor,
    customloca::CustomLocalization,
    defines::Defines,
    effect_localization::EffectLocalization,
    ethnicity::Ethnicity,
    fonts::Font,
    genes::Gene,
    gui::Gui,
    localization::Localization,
    on_actions::OnActions,
    portrait::{PortraitAnimation, PortraitCamera, PortraitModifierGroup, PortraitModifierPack},
    script_values::ScriptValues,
    scripted_effects::{Effect, Effects},
    scripted_lists::ScriptedLists,
    scripted_modifiers::ScriptedModifiers,
    scripted_triggers::{Trigger, Triggers},
    trigger_localization::TriggerLocalization,
};
use crate::db::{Db, DbKind};
use crate::dds::DdsFiles;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::game::Game;
#[cfg(feature = "imperator")]
use crate::imperator::data::goods::TradeGood;
#[cfg(feature = "imperator")]
use crate::imperator::tables::misc::*;
use crate::item::Item;
use crate::lowercase::Lowercase;
#[cfg(feature = "vic3")]
use crate::parse::json::parse_json_file;
use crate::pdxfile::PdxFile;
#[cfg(feature = "ck3")]
use crate::report::err;
use crate::report::{report, set_output_style, ErrorKey, OutputStyle, Severity};
use crate::rivers::Rivers;
use crate::token::{Loc, Token};
#[cfg(feature = "vic3")]
use crate::vic3::data::{
    ai_strategies::AiStrategy,
    battle_conditions::BattleCondition,
    buildings::{BuildingGroup, BuildingType},
    canals::CanalType,
    character_interactions::Vic3CharacterInteraction,
    combat_units::CombatUnit,
    countries::{Country, CountryFormation, CountryRank, CountryType},
    cultures::Vic3Culture,
    decisions::Vic3Decision,
    diplomatic_actions::DiplomaticAction,
    diplomatic_plays::DiplomaticPlay,
    events::Vic3Events,
    gameconcepts::GameConcept,
    gamerules::Vic3GameRule,
    goods::Goods,
    governments::GovernmentType,
    history::History,
    ideologies::Ideology,
    institutions::Institution,
    interest_groups::{InterestGroup, InterestGroupTrait},
    journalentries::Journalentry,
    laws::{LawType, Vic3LawGroup},
    map::MapLayer,
    media_aliases::MediaAlias,
    modifier_types::ModifierType,
    modifiers::Vic3Modifier,
    objectives::{Objective, ObjectiveSubgoal, ObjectiveSubgoalCategory},
    pops::PopType,
    production_methods::{ProductionMethod, ProductionMethodGroup},
    provinces::Vic3Provinces,
    religions::Vic3Religion,
    scripted_buttons::ScriptedButton,
    state_regions::StateRegion,
    state_traits::StateTrait,
    strategic_regions::StrategicRegion,
    subject_types::SubjectType,
    technology::{Technology, TechnologyEra},
    terrain::{TerrainLabel, TerrainManipulator, TerrainMask, TerrainMaterial, Vic3Terrain},
};
#[cfg(feature = "vic3")]
use crate::vic3::tables::misc::*;

#[derive(Debug, Error)]
pub enum FilesError {
    #[error("Could not read game files at {path}")]
    VanillaUnreadable { path: PathBuf, source: walkdir::Error },
    #[error("Could not read mod files at {path}")]
    ModUnreadable { path: PathBuf, source: walkdir::Error },
    #[error("Could not read config file at {path}")]
    ConfigUnreadable { path: PathBuf },
}

/// A record of everything known about the game and mod being validated.
///
/// References to [`Everything`] are passed down through nearly all of the validation logic, so
/// that individual functions can access all the defined game items.
///
/// The validator has two main phases: parsing and validation.
/// * During parsing, the script files are read, parsed, and loaded into the various databases.
///   `Everything` is mutable during this period.
/// * During validation, `Everything` is immutable and cross-checking between item types can be done safely.
#[derive(Debug)]
pub struct Everything {
    /// Config from file
    config: Block,

    /// A cache of define values (from common/defines) that are missing and that have already been
    /// warned about as missing. This is to avoid duplicate warnings.
    #[cfg(feature = "ck3")] // happens not to be used by vic3
    warned_defines: RwLock<FnvHashSet<String>>,

    /// Tracks all the files (vanilla and mods) that are relevant to the current validation.
    pub(crate) fileset: Fileset,

    /// Tracks specifically the .dds files, and their formats and sizes.
    pub(crate) dds: DdsFiles,

    /// A general database of item types. Most items go here. The ones that need special handling
    /// go in the separate databases listed below.
    pub(crate) database: Db,

    pub(crate) localization: Localization,

    pub(crate) scripted_lists: ScriptedLists,

    pub(crate) defines: Defines,

    #[cfg(feature = "ck3")]
    pub(crate) events_ck3: Ck3Events,
    #[cfg(feature = "vic3")]
    pub(crate) events_vic3: Vic3Events,

    pub(crate) scripted_modifiers: ScriptedModifiers,
    pub(crate) on_actions: OnActions,

    #[cfg(feature = "ck3")]
    pub(crate) interaction_cats: CharacterInteractionCategories,

    #[cfg(feature = "ck3")]
    pub(crate) provinces_ck3: Ck3Provinces,
    #[cfg(feature = "vic3")]
    pub(crate) provinces_vic3: Vic3Provinces,

    #[cfg(feature = "ck3")]
    pub(crate) province_histories: ProvinceHistories,

    #[cfg(feature = "ck3")]
    pub(crate) gameconcepts: GameConcepts,

    #[cfg(feature = "ck3")]
    pub(crate) titles: Titles,

    #[cfg(feature = "ck3")]
    pub(crate) characters: Characters,

    pub(crate) script_values: ScriptValues,

    pub(crate) triggers: Triggers,
    pub(crate) effects: Effects,

    #[cfg(feature = "ck3")]
    pub(crate) traits: Traits,

    #[cfg(feature = "ck3")]
    pub(crate) title_history: TitleHistories,

    #[cfg(feature = "ck3")]
    pub(crate) doctrines: Doctrines,

    #[cfg(feature = "ck3")]
    pub(crate) menatarmstypes: MenAtArmsTypes,

    pub(crate) gui: Gui,
    #[cfg(feature = "ck3")]
    pub(crate) data_bindings: DataBindings,

    pub(crate) assets: Assets,
    #[cfg(feature = "ck3")]
    pub(crate) sounds: Sounds,
    #[cfg(feature = "ck3")]
    pub(crate) music: Musics,

    pub(crate) coas: Coas,

    #[cfg(feature = "vic3")]
    pub(crate) history: History,
}

impl Everything {
    pub fn new(
        vanilla_dir: &Path,
        mod_root: &Path,
        replace_paths: Vec<PathBuf>,
    ) -> Result<Self, FilesError> {
        let mut fileset =
            Fileset::new(vanilla_dir.to_path_buf(), mod_root.to_path_buf(), replace_paths);

        let config_file_name = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => "ck3-tiger.conf",
            #[cfg(feature = "vic3")]
            Game::Vic3 => "vic3-tiger.conf",
            #[cfg(feature = "imperator")]
            Game::Imperator => "imperator-tiger.conf",
        };

        let config_file = mod_root.join(config_file_name);
        let config = if config_file.is_file() {
            Self::_read_config(config_file_name, &config_file)
                .ok_or(FilesError::ConfigUnreadable { path: config_file })?
        } else {
            Block::new(Loc::for_file(config_file, FileKind::Mod))
        };

        fileset.config(config.clone());

        fileset.scan_all()?;
        fileset.finalize();

        Ok(Everything {
            fileset,
            dds: DdsFiles::default(),
            config,
            #[cfg(feature = "ck3")]
            warned_defines: RwLock::new(FnvHashSet::default()),
            database: Db::default(),
            localization: Localization::default(),
            scripted_lists: ScriptedLists::default(),
            defines: Defines::default(),
            #[cfg(feature = "ck3")]
            events_ck3: Ck3Events::default(),
            #[cfg(feature = "vic3")]
            events_vic3: Vic3Events::default(),
            scripted_modifiers: ScriptedModifiers::default(),
            on_actions: OnActions::default(),
            #[cfg(feature = "ck3")]
            interaction_cats: CharacterInteractionCategories::default(),
            #[cfg(feature = "ck3")]
            provinces_ck3: Ck3Provinces::default(),
            #[cfg(feature = "vic3")]
            provinces_vic3: Vic3Provinces::default(),
            #[cfg(feature = "ck3")]
            province_histories: ProvinceHistories::default(),
            #[cfg(feature = "ck3")]
            gameconcepts: GameConcepts::default(),
            #[cfg(feature = "ck3")]
            titles: Titles::default(),
            #[cfg(feature = "ck3")]
            characters: Characters::default(),
            script_values: ScriptValues::default(),
            triggers: Triggers::default(),
            effects: Effects::default(),
            #[cfg(feature = "ck3")]
            traits: Traits::default(),
            #[cfg(feature = "ck3")]
            title_history: TitleHistories::default(),
            #[cfg(feature = "ck3")]
            doctrines: Doctrines::default(),
            #[cfg(feature = "ck3")]
            menatarmstypes: MenAtArmsTypes::default(),
            gui: Gui::default(),
            #[cfg(feature = "ck3")]
            data_bindings: DataBindings::default(),
            assets: Assets::default(),
            #[cfg(feature = "ck3")]
            sounds: Sounds::default(),
            #[cfg(feature = "ck3")]
            music: Musics::default(),
            coas: Coas::default(),
            #[cfg(feature = "vic3")]
            history: History::default(),
        })
    }

    fn _read_config(name: &str, path: &Path) -> Option<Block> {
        let entry = FileEntry::new(PathBuf::from(name), FileKind::Mod);
        PdxFile::read_no_bom(&entry, path)
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        self.fileset.fullpath(entry)
    }

    pub fn load_config_filtering_rules(&self) {
        check_for_legacy_ignore(&self.config);
        load_filter(&self.config);
    }

    /// Load the `OutputStyle` settings from the config.
    /// Note that the settings from the config can still be overridden
    /// by supplying the --no-color flag.
    fn load_output_styles(&self, default_color: bool) -> OutputStyle {
        // Treat a missing output_style block and an empty output_style block exactly the same.
        let block = match self.config.get_field_block("output_style") {
            Some(block) => Cow::Borrowed(block),
            None => Cow::Owned(Block::new(self.config.loc.clone())),
        };
        if !block.get_field_bool("enable").unwrap_or(default_color) {
            return OutputStyle::no_color();
        }
        let mut style = OutputStyle::default();
        for severity in Severity::iter() {
            if let Some(error_block) =
                block.get_field_block(format!("{severity}").to_ascii_lowercase().as_str())
            {
                if let Some(color) = error_block.get_field_value("color") {
                    style.set(severity, color.as_str());
                }
            }
        }
        style
    }

    /// A helper function for categories of items that follow the usual pattern of
    /// `.txt` files containing a block with definitions
    fn load_pdx_items<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        self.load_pdx_items_ext(itype, add, ".txt");
    }

    /// Like `load_pdx_items` but does not complain about a missing BOM
    fn load_pdx_items_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        self.load_pdx_items_optional_bom_ext(itype, add, ".txt");
    }

    /// Like `load_pdx_items_ext` but does not complain about a missing BOM
    fn load_pdx_items_optional_bom_ext<F>(&mut self, itype: Item, add: F, ext: &str)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for mut block in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            if entry.filename().to_string_lossy().ends_with(ext) {
                PdxFile::read_optional_bom(entry, &self.fileset.fullpath(entry))
            } else {
                None
            }
        }) {
            for (key, block) in block.drain_definitions_warn() {
                add(&mut self.database, key, block);
            }
        }
    }

    /// A helper function for categories of items that follow the usual pattern of
    /// files containing a block with definitions
    fn load_pdx_items_ext<F>(&mut self, itype: Item, add: F, ext: &str)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for mut block in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            if entry.filename().to_string_lossy().ends_with(ext) {
                PdxFile::read(entry, &self.fileset.fullpath(entry))
            } else {
                None
            }
        }) {
            for (key, block) in block.drain_definitions_warn() {
                add(&mut self.database, key, block);
            }
        }
    }

    /// A helper function for categories of items that are unusual in having each item in one file.
    fn load_pdx_files_optional_bom_ext<F>(&mut self, itype: Item, add: F, ext: &str)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for (key, block) in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            let filename = entry.filename().to_string_lossy();
            if let Some(key) = filename.strip_suffix(ext) {
                if let Some(block) =
                    PdxFile::read_optional_bom(entry, &self.fileset.fullpath(entry))
                {
                    let key = Token::new(key, Loc::for_entry(entry));
                    Some((key, block))
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            add(&mut self.database, key, block);
        }
    }

    #[cfg(feature = "ck3")] // happens not to be used by vic3
    fn load_pdx_files_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        self.load_pdx_files_optional_bom_ext(itype, add, ".txt");
    }

    /// A helper function for categories of items that are unusual in having each item in one file.
    #[cfg(feature = "ck3")] // happens not to be used by vic3
    fn load_pdx_files_cp1252<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for (key, block) in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            let filename = entry.filename().to_string_lossy();
            if let Some(key) = filename.strip_suffix(".txt") {
                if let Some(block) = PdxFile::read_cp1252(entry, &self.fileset.fullpath(entry)) {
                    let key = Token::new(key, Loc::for_entry(entry));
                    Some((key, block))
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            add(&mut self.database, key, block);
        }
    }

    #[cfg(feature = "vic3")]
    fn load_json<F>(&mut self, itype: Item, add_json: F)
    where
        F: Fn(&mut Db, Block) + Sync + Send,
    {
        for block in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            if entry.filename().to_string_lossy().ends_with(".json") {
                parse_json_file(entry, &self.fileset.fullpath(entry))
            } else {
                None
            }
        }) {
            add_json(&mut self.database, block);
        }
    }

    pub fn load_output_settings(&self, default_colors: bool) {
        set_output_style(self.load_output_styles(default_colors));
    }

    fn load_all_generic(&mut self) {
        scope(|s| {
            s.spawn(|_| self.fileset.handle(&mut self.dds));
            s.spawn(|_| self.fileset.handle(&mut self.localization));
            s.spawn(|_| self.fileset.handle(&mut self.scripted_lists));
            s.spawn(|_| self.fileset.handle(&mut self.defines));
            s.spawn(|_| self.fileset.handle(&mut self.scripted_modifiers));
            s.spawn(|_| self.fileset.handle(&mut self.script_values));
            s.spawn(|_| self.fileset.handle(&mut self.triggers));
            s.spawn(|_| self.fileset.handle(&mut self.effects));
            s.spawn(|_| self.fileset.handle(&mut self.assets));
            s.spawn(|_| self.fileset.handle(&mut self.gui));
            s.spawn(|_| self.fileset.handle(&mut self.on_actions));
            s.spawn(|_| self.fileset.handle(&mut self.coas));
        });

        self.load_pdx_items(Item::Accessory, Accessory::add);
        self.load_pdx_items(Item::AccessoryVariation, AccessoryVariation::add);
        self.load_pdx_items(Item::CoaDesignerColoredEmblem, CoaDesignerColoredEmblem::add);
        self.load_pdx_items(Item::CoaDesignerPattern, CoaDesignerPattern::add);
        self.load_pdx_items_optional_bom(Item::CoaTemplateList, CoaTemplateList::add);
        self.load_pdx_items(Item::CustomLocalization, CustomLocalization::add);
        self.load_pdx_items(Item::EffectLocalization, EffectLocalization::add);
        self.load_pdx_items(Item::Ethnicity, Ethnicity::add);
        self.load_pdx_items_optional_bom_ext(Item::Font, Font::add, ".font");
        self.load_pdx_items(Item::GeneCategory, Gene::add);
        self.load_pdx_items_optional_bom(Item::NamedColor, NamedColor::add);
        self.load_pdx_items(Item::PortraitAnimation, PortraitAnimation::add);
        self.load_pdx_items(Item::PortraitCamera, PortraitCamera::add);
        self.load_pdx_items(Item::PortraitModifierGroup, PortraitModifierGroup::add);
        self.load_pdx_items_ext(
            Item::PortraitModifierPack,
            PortraitModifierPack::add,
            ".modifierpack",
        );
        self.load_pdx_items(Item::TriggerLocalization, TriggerLocalization::add);
    }

    #[cfg(feature = "ck3")]
    fn load_all_ck3(&mut self) {
        scope(|s| {
            s.spawn(|_| self.fileset.handle(&mut self.events_ck3));
            s.spawn(|_| self.fileset.handle(&mut self.interaction_cats));
            s.spawn(|_| self.fileset.handle(&mut self.province_histories));
            s.spawn(|_| self.fileset.handle(&mut self.gameconcepts));
            s.spawn(|_| self.fileset.handle(&mut self.titles));
            s.spawn(|_| self.fileset.handle(&mut self.characters));
            s.spawn(|_| self.fileset.handle(&mut self.traits));
            s.spawn(|_| self.fileset.handle(&mut self.title_history));
            s.spawn(|_| self.fileset.handle(&mut self.doctrines));
            s.spawn(|_| self.fileset.handle(&mut self.menatarmstypes));
            s.spawn(|_| self.fileset.handle(&mut self.data_bindings));
            s.spawn(|_| self.fileset.handle(&mut self.sounds));
            s.spawn(|_| self.fileset.handle(&mut self.music));
            s.spawn(|_| self.fileset.handle(&mut self.provinces_ck3));
        });
        self.load_pdx_items(Item::CharacterInteraction, Ck3CharacterInteraction::add);
        self.load_pdx_items(Item::Culture, Ck3Culture::add);
        self.load_pdx_items(Item::Decision, Ck3Decision::add);
        self.load_pdx_items(Item::Modifier, Ck3Modifier::add);
        self.load_pdx_items(Item::Religion, Ck3Religion::add);
        self.load_pdx_items(Item::ReligionFamily, ReligionFamily::add);
        self.load_pdx_items(Item::Dynasty, Dynasty::add);
        self.load_pdx_items(Item::House, House::add);
        self.load_pdx_items(Item::NameList, NameList::add);
        self.load_pdx_items(Item::Lifestyle, Lifestyle::add);
        self.load_pdx_items(Item::CourtPositionCategory, CourtPositionCategory::add);
        self.load_pdx_items(Item::CourtPosition, CourtPosition::add);
        self.load_pdx_items(Item::EventTheme, EventTheme::add);
        self.load_pdx_items(Item::EventBackground, EventBackground::add);
        self.load_pdx_items(Item::EventTransition, EventTransition::add);
        self.load_pdx_items(Item::ScriptedRule, ScriptedRule::add);
        self.load_pdx_items(Item::Faction, Faction::add);
        self.load_pdx_items(Item::Relation, Relation::add);
        self.load_pdx_items(Item::Terrain, Ck3Terrain::add);
        self.load_pdx_items(Item::Region, Region::add);
        self.load_pdx_items(Item::ScriptedGui, ScriptedGui::add);
        self.load_pdx_items(Item::Amenity, Amenity::add);
        self.load_pdx_items(Item::CasusBelliGroup, CasusBelliGroup::add);
        self.load_pdx_items(Item::CasusBelli, CasusBelli::add);
        self.load_pdx_items(Item::Holding, Holding::add);
        self.load_pdx_items(Item::Focus, Focus::add);
        self.load_pdx_items(Item::Perk, Perk::add);
        self.load_pdx_items(Item::OpinionModifier, OpinionModifier::add);
        self.load_pdx_items(Item::CharacterTemplate, CharacterTemplate::add);
        self.load_pdx_items(Item::DeathReason, DeathReason::add);
        self.load_pdx_items(Item::ArtifactSlot, ArtifactSlot::add);
        self.load_pdx_items(Item::ArtifactType, ArtifactType::add);
        self.load_pdx_items(Item::ArtifactTemplate, ArtifactTemplate::add);
        self.load_pdx_items(Item::ArtifactVisual, ArtifactVisual::add);
        self.load_pdx_items(Item::ArtifactFeature, ArtifactFeature::add);
        self.load_pdx_items(Item::ArtifactFeatureGroup, ArtifactFeatureGroup::add);
        self.load_pdx_items(Item::Nickname, Nickname::add);
        self.load_pdx_items(Item::Building, Building::add);
        self.load_pdx_items(Item::CultureEra, CultureEra::add);
        self.load_pdx_items(Item::CulturePillar, CulturePillar::add);
        self.load_pdx_items(Item::CultureTradition, CultureTradition::add);
        self.load_pdx_items(Item::CultureAesthetic, CultureAesthetic::add);
        self.load_pdx_items(Item::CultureCreationName, CultureCreationName::add);
        self.load_pdx_items(Item::Innovation, Innovation::add);
        self.load_pdx_items(Item::AccoladeIcon, AccoladeIcon::add);
        self.load_pdx_items(Item::AccoladeName, AccoladeName::add);
        self.load_pdx_items(Item::AccoladeType, AccoladeType::add);
        self.load_pdx_items(Item::VassalStance, VassalStance::add);
        self.load_pdx_items(Item::Dna, Dna::add);
        self.load_pdx_items(Item::Bookmark, Bookmark::add);
        self.load_pdx_items(Item::BookmarkGroup, BookmarkGroup::add);
        self.load_pdx_items_optional_bom(Item::BookmarkPortrait, BookmarkPortrait::add);
        self.load_pdx_items(Item::GovernmentType, Government::add);
        self.load_pdx_items(Item::Hook, Hook::add);
        self.load_pdx_items(Item::CouncilPosition, CouncilPosition::add);
        self.load_pdx_items(Item::CouncilTask, CouncilTask::add);
        self.load_pdx_items(Item::PoolSelector, PoolSelector::add);
        self.load_pdx_items(Item::CharacterBackground, CharacterBackground::add);
        self.load_pdx_items(Item::HolySite, HolySite::add);
        self.load_pdx_items(Item::PortraitEnvironment, PortraitEnvironment::add);
        self.load_pdx_items(Item::Struggle, Struggle::add);
        self.load_pdx_items(Item::Catalyst, Catalyst::add);
        self.load_pdx_items(Item::ImportantAction, ImportantAction::add);
        self.load_pdx_items(Item::Suggestion, Suggestion::add);
        self.load_pdx_items(Item::Scheme, Scheme::add);
        self.load_pdx_items(Item::ModifierFormat, ModifierFormat::add);
        self.load_pdx_items(Item::MemoryType, MemoryType::add);
        self.load_pdx_items(Item::MapMode, MapMode::add);
        self.load_pdx_items(Item::VassalContract, VassalContract::add);
        self.load_pdx_items(Item::CourtType, CourtType::add);
        self.load_pdx_items(Item::Secret, Secret::add);
        self.load_pdx_items(Item::ActivityType, ActivityType::add);
        self.load_pdx_items(Item::ActivityLocale, ActivityLocale::add);
        self.load_pdx_items(Item::ActivityIntent, ActivityIntent::add);
        self.load_pdx_items(Item::GuestInviteRule, GuestInviteRule::add);
        self.load_pdx_items(Item::PulseAction, PulseAction::add);
        self.load_pdx_items(Item::ScriptedAnimation, ScriptedAnimation::add);
        self.load_pdx_items(Item::CourtSceneCulture, CourtSceneCulture::add);
        self.load_pdx_items(Item::CourtSceneGroup, CourtSceneGroup::add);
        self.load_pdx_items(Item::CourtSceneRole, CourtSceneRole::add);
        self.load_pdx_files_optional_bom(Item::CourtSceneSetting, CourtSceneSetting::add);
        self.load_pdx_files_optional_bom(Item::MapEnvironment, MapEnvironment::add);
        self.load_pdx_items(Item::GameRule, Ck3GameRule::add);
        self.load_pdx_items(Item::TravelOption, TravelOption::add);
        self.load_pdx_items(Item::Story, Story::add);
        self.load_pdx_items(Item::LawGroup, Ck3LawGroup::add);
        self.load_pdx_items(Item::SuccessionElection, Election::add);
        self.load_pdx_items(Item::DiarchyType, DiarchyType::add);
        self.load_pdx_items(Item::DiarchyMandate, DiarchyMandate::add);
        self.load_pdx_items(Item::Inspiration, Inspiration::add);
        self.load_pdx_items(Item::CoaDesignerColorPalette, CoaDesignerColorPalette::add);
        self.load_pdx_items(Item::CoaDesignerEmblemLayout, CoaDesignerEmblemLayout::add);
        self.load_pdx_items(Item::PointOfInterest, PointOfInterest::add);
        self.load_pdx_items(Item::DynastyLegacy, DynastyLegacy::add);
        self.load_pdx_items(Item::DynastyPerk, DynastyPerk::add);
        self.load_pdx_items(Item::CombatEffect, CombatEffect::add);
        self.load_pdx_items(Item::ScriptedIllustration, ScriptedIllustration::add);
        self.load_pdx_items(Item::Flavorization, Flavorization::add);
        self.load_pdx_files_cp1252(Item::CultureHistory, CultureHistory::add);
        self.load_pdx_items(Item::Motto, Motto::add);
        self.load_pdx_items(Item::MottoInsert, MottoInsert::add);
        self.load_pdx_items(Item::CombatPhaseEvent, CombatPhaseEvent::add);
        self.load_pdx_items(Item::ScriptedCost, ScriptedCost::add);
        self.load_pdx_items(Item::PlayableDifficultyInfo, PlayableDifficultyInfo::add);
        self.load_pdx_items(Item::Message, Message::add);
        self.load_pdx_items(Item::CoaDynamicDefinition, CoaDynamicDefinition::add);
        Building::finalize(&mut self.database);
    }

    #[cfg(feature = "vic3")]
    fn load_all_vic3(&mut self) {
        self.fileset.handle(&mut self.history);
        self.fileset.handle(&mut self.events_vic3);
        self.fileset.handle(&mut self.provinces_vic3);
        self.load_pdx_items(Item::AiStrategy, AiStrategy::add);
        self.load_pdx_items(Item::BattleCondition, BattleCondition::add);
        self.load_pdx_items(Item::BuildingGroup, BuildingGroup::add);
        self.load_pdx_items(Item::BuildingType, BuildingType::add);
        self.load_pdx_items(Item::CanalType, CanalType::add);
        self.load_pdx_items(Item::CharacterInteraction, Vic3CharacterInteraction::add);
        self.load_pdx_items(Item::CombatUnit, CombatUnit::add);
        self.load_pdx_items(Item::Country, Country::add);
        self.load_pdx_items(Item::CountryFormation, CountryFormation::add);
        self.load_pdx_items(Item::CountryType, CountryType::add);
        self.load_pdx_items(Item::CountryRank, CountryRank::add);
        self.load_pdx_items(Item::Culture, Vic3Culture::add);
        self.load_pdx_items(Item::Decision, Vic3Decision::add);
        self.load_pdx_items(Item::DiplomaticAction, DiplomaticAction::add);
        self.load_pdx_items(Item::DiplomaticPlay, DiplomaticPlay::add);
        self.load_pdx_items(Item::GameConcept, GameConcept::add);
        self.load_pdx_items(Item::GameRule, Vic3GameRule::add);
        self.load_pdx_items(Item::Goods, Goods::add);
        self.load_pdx_items(Item::GovernmentType, GovernmentType::add);
        self.load_pdx_items(Item::Ideology, Ideology::add);
        self.load_pdx_items(Item::Institution, Institution::add);
        self.load_pdx_items(Item::InterestGroup, InterestGroup::add);
        self.load_pdx_items(Item::InterestGroupTrait, InterestGroupTrait::add);
        self.load_pdx_items(Item::Journalentry, Journalentry::add);
        self.load_pdx_items(Item::LawGroup, Vic3LawGroup::add);
        self.load_pdx_items(Item::LawType, LawType::add);
        self.load_pdx_items(Item::MapLayer, MapLayer::add);
        self.load_pdx_items(Item::MediaAlias, MediaAlias::add);
        self.load_pdx_items(Item::Modifier, Vic3Modifier::add);
        self.load_pdx_items(Item::ModifierType, ModifierType::add);
        self.load_pdx_items(Item::Objective, Objective::add);
        self.load_pdx_items(Item::ObjectiveSubgoal, ObjectiveSubgoal::add);
        self.load_pdx_items(Item::ObjectiveSubgoalCategory, ObjectiveSubgoalCategory::add);
        self.load_pdx_items(Item::PopType, PopType::add);
        self.load_pdx_items(Item::ProductionMethod, ProductionMethod::add);
        self.load_pdx_items(Item::ProductionMethodGroup, ProductionMethodGroup::add);
        self.load_pdx_items(Item::Religion, Vic3Religion::add);
        self.load_pdx_items(Item::ScriptedButton, ScriptedButton::add);
        self.load_pdx_items(Item::StateRegion, StateRegion::add);
        self.load_pdx_items(Item::StateTrait, StateTrait::add);
        self.load_pdx_items(Item::StrategicRegion, StrategicRegion::add);
        self.load_pdx_items(Item::SubjectType, SubjectType::add);
        self.load_pdx_items(Item::Technology, Technology::add);
        self.load_pdx_items(Item::TechnologyEra, TechnologyEra::add);
        self.load_pdx_items(Item::Terrain, Vic3Terrain::add);
        self.load_pdx_items(Item::TerrainLabel, TerrainLabel::add);
        self.load_pdx_items(Item::TerrainManipulator, TerrainManipulator::add);
        self.load_pdx_files_optional_bom_ext(
            Item::TerrainMaterial,
            TerrainMaterial::add,
            ".settings",
        );
        self.load_json(Item::TerrainMask, TerrainMask::add_json);
    }

    #[cfg(feature = "imperator")]
    fn load_all_imperator(&mut self) {
        self.load_pdx_items(Item::TradeGood, TradeGood::add);
    }

    pub fn load_all(&mut self) {
        self.load_all_generic();
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => self.load_all_ck3(),
            #[cfg(feature = "vic3")]
            Game::Vic3 => self.load_all_vic3(),
            #[cfg(feature = "imperator")]
            Game::Imperator => self.load_all_imperator(),
        }
    }

    fn validate_all_generic<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.fileset.validate(self));
        s.spawn(|_| self.scripted_lists.validate(self));
        s.spawn(|_| self.defines.validate(self));
        s.spawn(|_| self.scripted_modifiers.validate(self));
        s.spawn(|_| self.script_values.validate(self));
        s.spawn(|_| self.triggers.validate(self));
        s.spawn(|_| self.effects.validate(self));
        s.spawn(|_| self.assets.validate(self));
        s.spawn(|_| self.gui.validate(self));
        s.spawn(|_| self.on_actions.validate(self));
        s.spawn(|_| self.coas.validate(self));
    }

    #[cfg(feature = "ck3")]
    fn validate_all_ck3<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.interaction_cats.validate(self));
        s.spawn(|_| self.province_histories.validate(self));
        s.spawn(|_| self.gameconcepts.validate(self));
        s.spawn(|_| self.titles.validate(self));
        s.spawn(|_| self.characters.validate(self));
        s.spawn(|_| self.traits.validate(self));
        s.spawn(|_| self.title_history.validate(self));
        s.spawn(|_| self.doctrines.validate(self));
        s.spawn(|_| self.menatarmstypes.validate(self));
        s.spawn(|_| self.data_bindings.validate(self));
        s.spawn(|_| self.sounds.validate(self));
        s.spawn(|_| self.music.validate(self));
        s.spawn(|_| self.events_ck3.validate(self));
        s.spawn(|_| self.provinces_ck3.validate(self));
    }

    #[cfg(feature = "vic3")]
    fn validate_all_vic3<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.events_vic3.validate(self));
        s.spawn(|_| self.history.validate(self));
        s.spawn(|_| self.provinces_vic3.validate(self));
        s.spawn(|_| StrategicRegion::crosscheck(self));
    }

    // Imperator one goes here when needed

    pub fn validate_all(&self) {
        scope(|s| {
            self.validate_all_generic(s);
            match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.validate_all_ck3(s),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.validate_all_vic3(s),
                #[cfg(feature = "imperator")]
                Game::Imperator => (), // TODO - imperator -
            }
        });
        self.database.validate(self);

        self.localization.validate_pass2(self);
    }

    pub fn check_rivers(&mut self) {
        let mut rivers = Rivers::default();
        self.fileset.handle(&mut rivers);
        rivers.validate(self);
    }

    #[cfg(feature = "ck3")]
    pub fn check_pod(&mut self) {
        self.province_histories.check_pod_faiths(self, &self.titles);
        self.characters.check_pod_flags(self);
        self.localization.check_pod_loca(self);
    }

    pub fn check_unused(&mut self) {
        self.localization.check_unused(self);
        self.fileset.check_unused_dds(self);
    }

    pub(crate) fn item_has_property(&self, itype: Item, key: &str, property: &str) -> bool {
        self.database.has_property(itype, key, property, self)
    }

    #[cfg(feature = "ck3")]
    pub(crate) fn item_exists_ck3(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::ActivityState => ACTIVITY_STATES.contains(&key),
            Item::ArtifactHistory => ARTIFACT_HISTORY.contains(&key),
            Item::ArtifactRarity => ARTIFACT_RARITY.contains(&key),
            Item::Character => self.characters.exists(key),
            Item::CharacterInteractionCategory => self.interaction_cats.exists(key),
            Item::DangerType => DANGER_TYPES.contains(&key),
            Item::Dlc => DLC_CK3.contains(&key),
            Item::DlcFeature => DLC_FEATURES_CK3.contains(&key),
            Item::Doctrine => self.doctrines.exists(key),
            Item::DoctrineParameter => self.doctrines.parameter_exists(key),
            Item::Event => self.events_ck3.exists(key),
            Item::EventNamespace => self.events_ck3.namespace_exists(key),
            Item::GameConcept => self.gameconcepts.exists(key),
            Item::GeneticConstraint => self.traits.constraint_exists(key),
            Item::MenAtArms => self.menatarmstypes.exists(key),
            Item::MenAtArmsBase => self.menatarmstypes.base_exists(key),
            Item::Music => self.music.exists(key),
            Item::PrisonType => PRISON_TYPES.contains(&key),
            Item::Province => self.provinces_ck3.exists(key),
            Item::RewardItem => REWARD_ITEMS.contains(&key),
            Item::Sexuality => SEXUALITIES.contains(&key),
            Item::Skill => SKILLS.contains(&key),
            Item::Sound => self.sounds.exists(key),
            Item::Title => self.titles.exists(key),
            Item::TitleHistory => self.title_history.exists(key),
            Item::TitleHistoryType => TITLE_HISTORY_TYPES.contains(&key),
            Item::Trait => self.traits.exists(key),
            Item::TraitFlag => self.traits.flag_exists(key),
            Item::TraitTrack => self.traits.track_exists(key),
            Item::TraitCategory => TRAIT_CATEGORIES.contains(&key),
            _ => self.database.exists(itype, key),
        }
    }

    #[cfg(feature = "vic3")]
    pub(crate) fn item_exists_vic3(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Approval => APPROVALS.contains(&key),
            Item::Attitude => ATTITUDES.contains(&key),
            Item::CharacterRole => CHARACTER_ROLES.contains(&key),
            Item::CountryTier => COUNTRY_TIERS.contains(&key),
            Item::Dlc => DLC_VIC3.contains(&key),
            Item::DlcFeature => DLC_FEATURES_VIC3.contains(&key),
            Item::Event => self.events_vic3.exists(key),
            Item::EventNamespace => self.events_vic3.namespace_exists(key),
            Item::InfamyThreshold => INFAMY_THRESHOLDS.contains(&key),
            Item::Level => LEVELS.contains(&key),
            Item::PoliticalMovement => POLITICAL_MOVEMENTS.contains(&key),
            Item::SecretGoal => SECRET_GOALS.contains(&key),
            Item::Sound => {
                if let Some(filename) = key.strip_prefix("file://") {
                    self.fileset.exists(filename)
                } else {
                    SOUNDS_VIC3.contains(&key)
                }
            }
            Item::Strata => STRATA.contains(&key),
            Item::TransferOfPower => TRANSFER_OF_POWER.contains(&key),
            Item::Wargoal => WARGOALS.contains(&key),
            Item::CharacterTemplate
            | Item::CharacterTrait
            | Item::CommanderOrder
            | Item::Party
            | Item::CultureGraphics
            | Item::Decree
            | Item::TutorialLesson => true, // TODO
            _ => self.database.exists(itype, key),
        }
    }

    #[cfg(feature = "imperator")]
    pub(crate) fn item_exists_imperator(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Dlc => DLC_IMPERATOR.contains(&key),
            Item::DlcFeature => DLC_FEATURES_IMPERATOR.contains(&key),
            Item::Sound => {
                if let Some(filename) = key.strip_prefix("file://") {
                    self.fileset.exists(filename)
                } else {
                    SOUNDS_IMPERATOR.contains(&key)
                }
            }
            _ => self.database.exists(itype, key),
        }
    }

    pub(crate) fn item_exists(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Asset => self.assets.asset_exists(key),
            Item::BlendShape => self.assets.blend_shape_exists(key),
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::Define => self.defines.exists(key),
            Item::Entity => self.assets.entity_exists(key),
            Item::File => self.fileset.exists(key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::GuiLayer => self.gui.layer_exists(key),
            Item::GuiTemplate => self.gui.template_exists(key),
            Item::GuiType => self.gui.type_exists(&Lowercase::new(key)),
            Item::Localization => self.localization.exists(key),
            Item::OnAction => self.on_actions.exists(key),
            Item::Pdxmesh => self.assets.mesh_exists(key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::ScriptValue => self.script_values.exists(key),
            Item::TextFormat => self.gui.textformat_exists(key),
            Item::TextIcon => self.gui.texticon_exists(key),
            Item::TextureFile => self.assets.texture_exists(key),
            Item::Shortcut => true, // TODO
            _ => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.item_exists_ck3(itype, key),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.item_exists_vic3(itype, key),
                #[cfg(feature = "imperator")]
                Game::Imperator => self.item_exists_imperator(itype, key),
            },
        }
    }

    pub(crate) fn mark_used(&self, itype: Item, key: &str) {
        match itype {
            Item::File => self.fileset.mark_used(key),
            Item::Localization => self.localization.mark_used(key),
            _ => (),
        }
    }

    pub(crate) fn verify_exists(&self, itype: Item, token: &Token) {
        self.verify_exists_implied(itype, token.as_str(), token);
    }

    pub(crate) fn verify_exists_max_sev(&self, itype: Item, token: &Token, max_sev: Severity) {
        self.verify_exists_implied_max_sev(itype, token.as_str(), token, max_sev);
    }

    pub(crate) fn verify_exists_implied_max_sev(
        &self,
        itype: Item,
        key: &str,
        token: &Token,
        max_sev: Severity,
    ) {
        match itype {
            Item::File => self.fileset.verify_exists_implied(key, token, max_sev),
            Item::Localization => self.localization.verify_exists_implied(key, token, max_sev),
            #[cfg(feature = "ck3")]
            Item::Music => self.music.verify_exists_implied(key, token, max_sev),
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            Item::Province => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.provinces_ck3.verify_exists_implied(key, token, max_sev),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.provinces_vic3.verify_exists_implied(key, token, max_sev),
            },
            #[cfg(feature = "ck3")]
            Item::Sound => self.sounds.verify_exists_implied(key, token, self, max_sev),
            Item::TextureFile => {
                if let Some(entry) = self.assets.get_texture(key) {
                    // TODO: avoid allocating a string here
                    self.fileset.mark_used(&entry.path().to_string_lossy());
                } else {
                    let msg = format!("no texture file {key} anywhere under {}", itype.path());
                    report(ErrorKey::MissingFile, itype.severity().at_most(max_sev))
                        .conf(itype.confidence())
                        .msg(msg)
                        .loc(token)
                        .push();
                }
            }
            _ => {
                if !self.item_exists(itype, key) {
                    let path = itype.path();
                    let msg = if path.is_empty() {
                        format!("unknown {itype} {key}")
                    } else {
                        format!("{itype} {key} not defined in {path}")
                    };
                    report(ErrorKey::MissingItem, itype.severity().at_most(max_sev))
                        .conf(itype.confidence())
                        .msg(msg)
                        .loc(token)
                        .push();
                }
            }
        }
    }

    pub(crate) fn verify_exists_implied(&self, itype: Item, key: &str, token: &Token) {
        self.verify_exists_implied_max_sev(itype, key, token, Severity::Error);
    }

    pub(crate) fn validate_use(&self, itype: Item, key: &Token, block: &Block) {
        self.database.validate_use(itype, key, block, self);
    }

    #[cfg(feature = "ck3")] // happens not to be used by vic3
    pub(crate) fn validate_call(
        &self,
        itype: Item,
        key: &Token,
        block: &Block,
        sc: &mut ScopeContext,
    ) {
        self.database.validate_call(itype, key, block, self, sc);
    }

    /// Validate the use of a localization within a specific `ScopeContext`.
    /// This allows validation of the named scopes used within the localization's datafunctions.
    pub(crate) fn validate_localization_sc(&self, key: &str, sc: &mut ScopeContext) {
        self.localization.validate_use(key, self, sc);
    }

    #[allow(dead_code)] // not currently used, but was hard to write...
    pub(crate) fn get_item<T: DbKind>(
        &self,
        itype: Item,
        key: &str,
    ) -> Option<(&Token, &Block, &T)> {
        self.database.get_item(itype, key)
    }

    pub(crate) fn get_key_block(&self, itype: Item, key: &str) -> Option<(&Token, &Block)> {
        self.database.get_key_block(itype, key)
    }

    pub(crate) fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        #[cfg(feature = "ck3")]
        if Game::is_ck3() {
            if let Some(trigger) = self.triggers.get(key.as_str()) {
                return Some(trigger);
            }
            if let Some(trigger) = self.events_ck3.get_trigger(key) {
                return Some(trigger);
            }
            return None;
        }
        self.triggers.get(key.as_str())
    }

    pub(crate) fn get_effect(&self, key: &Token) -> Option<&Effect> {
        #[cfg(feature = "ck3")]
        if Game::is_ck3() {
            if let Some(effect) = self.effects.get(key.as_str()) {
                return Some(effect);
            }
            if let Some(effect) = self.events_ck3.get_effect(key) {
                return Some(effect);
            }
            return None;
        }
        self.effects.get(key.as_str())
    }

    #[allow(unused_variables)] // TODO - imperator - does not use
    pub(crate) fn check_event_scope(&self, token: &Token, sc: &mut ScopeContext) {
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => self.events_ck3.check_scope(token, sc),
            #[cfg(feature = "vic3")]
            Game::Vic3 => self.events_vic3.check_scope(token, sc),
            #[cfg(feature = "imperator")]
            Game::Imperator => (), // TODO - imperator -
        };
    }

    #[cfg(feature = "ck3")] // happens not to be used by vic3
    pub(crate) fn get_defined_string(&self, key: &str) -> Option<&Token> {
        self.defines.get_string(key)
    }

    #[allow(clippy::missing_panics_doc)] // only panics on poisoned mutex
    #[cfg(feature = "ck3")] // happens not to be used by vic3
    pub(crate) fn get_defined_string_warn(&self, token: &Token, key: &str) -> Option<&Token> {
        let result = self.get_defined_string(key);
        let mut cache = self.warned_defines.write().unwrap();
        if result.is_none() && !cache.contains(key) {
            let msg = format!("{key} not defined in common/defines/");
            err(ErrorKey::MissingItem).msg(msg).loc(token).push();
            cache.insert(key.to_string());
        }
        result
    }
}
