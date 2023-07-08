use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::Result;
use fnv::FnvHashSet;
use rayon::{scope, Scope};
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::block::Block;
#[cfg(feature = "ck3")]
use crate::ck3::data::{
    accessory::{Accessory, AccessoryVariation},
    accolades::{AccoladeIcon, AccoladeName, AccoladeType},
    activities::{ActivityIntent, ActivityLocale, ActivityType, GuestInviteRule, PulseAction},
    amenities::Amenity,
    artifacts::{
        ArtifactFeature, ArtifactFeatureGroup, ArtifactSlot, ArtifactTemplate, ArtifactType,
        ArtifactVisual,
    },
    assets::Assets,
    bookmarks::{Bookmark, BookmarkGroup, BookmarkPortrait},
    buildings::Building,
    casusbelli::{CasusBelli, CasusBelliGroup},
    character_templates::CharacterTemplate,
    characters::Characters,
    coa::{CoaDynamicDefinition, CoaTemplateList, Coas},
    coadesigner::{
        CoaDesignerColorPalette, CoaDesignerColoredEmblem, CoaDesignerEmblemLayout,
        CoaDesignerPattern,
    },
    combat::CombatPhaseEvent,
    combat_effects::CombatEffect,
    council::{CouncilPosition, CouncilTask},
    court_scene::{CourtSceneCulture, CourtSceneGroup, CourtSceneRole, CourtSceneSetting},
    court_type::CourtType,
    courtpos::{CourtPosition, CourtPositionCategory},
    culture_history::CultureHistory,
    cultures::{
        Culture, CultureAesthetic, CultureCreationName, CultureEra, CulturePillar, CultureTradition,
    },
    data_binding::DataBindings,
    deathreasons::DeathReason,
    decisions::Decision,
    diarchies::{DiarchyMandate, DiarchyType},
    difficulty::PlayableDifficultyInfo,
    dna::Dna,
    doctrines::Doctrines,
    dynasties::Dynasty,
    dynasty_legacies::{DynastyLegacy, DynastyPerk},
    election::Election,
    environment::Environment,
    ethnicity::Ethnicity,
    event_themes::{EventBackground, EventTheme, EventTransition},
    events::Events,
    factions::Faction,
    flavorization::Flavorization,
    focus::Focus,
    gameconcepts::GameConcepts,
    gamerules::GameRule,
    genes::Gene,
    government::Government,
    gui::Gui,
    holdings::Holding,
    holysites::HolySite,
    hooks::Hook,
    houses::House,
    important_actions::ImportantAction,
    innovations::Innovation,
    inspirations::Inspiration,
    interaction_cats::InteractionCategories,
    interactions::Interaction,
    laws::LawGroup,
    lifestyles::Lifestyle,
    maa::MenAtArmsTypes,
    map_environment::MapEnvironment,
    mapmodes::MapMode,
    memories::MemoryType,
    messages::Message,
    modif::ModifierFormat,
    modifiers::Modifier,
    mottos::{Motto, MottoInsert},
    music::Musics,
    namelists::NameList,
    nickname::Nickname,
    on_actions::OnActions,
    opinions::OpinionModifier,
    perks::Perk,
    points_of_interest::PointOfInterest,
    pool::{CharacterBackground, PoolSelector},
    portrait::{PortraitAnimation, PortraitCamera, PortraitModifierGroup, PortraitModifierPack},
    prov_history::ProvinceHistories,
    provinces::Provinces,
    regions::Region,
    relations::Relation,
    religions::{Religion, ReligionFamily},
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
    terrain::Terrain,
    title_history::TitleHistories,
    titles::Titles,
    traits::Traits,
    travel::TravelOption,
    vassalcontract::VassalContract,
    vassalstance::VassalStance,
};
use crate::config_load::{check_for_legacy_ignore, load_filter};
use crate::context::ScopeContext;
use crate::data::{
    colors::NamedColor,
    customloca::CustomLocalization,
    defines::Defines,
    effect_localization::EffectLocalization,
    localization::Localization,
    scripted_effects::{Effect, Effects},
    scripted_lists::ScriptedLists,
    scripted_modifiers::ScriptedModifiers,
    scripted_triggers::{Trigger, Triggers},
    scriptvalues::ScriptValues,
    trigger_localization::TriggerLocalization,
};
use crate::db::{Db, DbKind};
use crate::dds::DdsFiles;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{error, old_warn, set_output_style, ErrorKey, OutputStyle, Severity};
use crate::rivers::Rivers;
use crate::token::{Loc, Token};
#[cfg(feature = "vic3")]
use crate::vic3::data::{
    battle_conditions::BattleCondition,
    buildings::{BuildingGroup, BuildingType},
    countries::Country,
    cultures::Culture,
    events::Events,
    modifiers::Modifier,
    production_methods::ProductionMethod,
    religions::Religion,
    state_regions::StateRegion,
    technology::{Technology, TechnologyEra},
    terrain_manipulator::TerrainManipulator,
};

#[derive(Debug, Error)]
pub enum FilesError {
    #[error("Could not read game files at {path}")]
    VanillaUnreadable { path: PathBuf, source: walkdir::Error },
    #[error("Could not read mod files at {path}")]
    ModUnreadable { path: PathBuf, source: walkdir::Error },
    #[error("Could not read config file at {path}")]
    ConfigUnreadable { path: PathBuf },
}

#[derive(Debug)]
pub struct Everything {
    /// Config from file
    config: Block,

    warned_defines: RwLock<FnvHashSet<String>>,

    /// The vanilla and mod files
    pub fileset: Fileset,

    pub dds: DdsFiles,

    pub database: Db,

    /// Processed localization files
    pub localization: Localization,

    pub scripted_lists: ScriptedLists,

    pub defines: Defines,

    /// Processed event files
    pub events: Events,

    pub scripted_modifiers: ScriptedModifiers,
    #[cfg(feature = "ck3")]
    pub on_actions: OnActions,

    #[cfg(feature = "ck3")]
    pub interaction_cats: InteractionCategories,

    /// Processed map data
    #[cfg(feature = "ck3")]
    pub provinces: Provinces,

    /// Processed history/provinces data
    #[cfg(feature = "ck3")]
    pub province_histories: ProvinceHistories,

    /// Processed game concepts
    #[cfg(feature = "ck3")]
    pub gameconcepts: GameConcepts,

    /// Landed titles
    #[cfg(feature = "ck3")]
    pub titles: Titles,

    #[cfg(feature = "ck3")]
    pub characters: Characters,

    pub scriptvalues: ScriptValues,

    pub triggers: Triggers,
    pub effects: Effects,

    #[cfg(feature = "ck3")]
    pub traits: Traits,

    #[cfg(feature = "ck3")]
    pub title_history: TitleHistories,

    #[cfg(feature = "ck3")]
    pub doctrines: Doctrines,

    #[cfg(feature = "ck3")]
    pub menatarmstypes: MenAtArmsTypes,

    #[cfg(feature = "ck3")]
    pub gui: Gui,
    #[cfg(feature = "ck3")]
    pub data_bindings: DataBindings,

    #[cfg(feature = "ck3")]
    pub assets: Assets,
    #[cfg(feature = "ck3")]
    pub sounds: Sounds,
    #[cfg(feature = "ck3")]
    pub music: Musics,

    #[cfg(feature = "ck3")]
    pub coas: Coas,
}

impl Everything {
    pub fn new(
        vanilla_dir: &Path,
        mod_root: &Path,
        replace_paths: Vec<PathBuf>,
    ) -> Result<Self, FilesError> {
        let mut fileset =
            Fileset::new(vanilla_dir.to_path_buf(), mod_root.to_path_buf(), replace_paths);

        let config_file = mod_root.join("ck3-tiger.conf");
        let config = if config_file.is_file() {
            Self::_read_config("ck3-tiger.conf", &config_file)
                .ok_or(FilesError::ConfigUnreadable { path: config_file })?
        } else {
            Block::new(Loc::for_file(Arc::new(config_file), FileKind::Mod))
        };

        fileset.config(config.clone());

        fileset.scan_all()?;
        fileset.finalize();

        Ok(Everything {
            fileset,
            dds: DdsFiles::default(),
            config,
            warned_defines: RwLock::new(FnvHashSet::default()),
            database: Db::default(),
            localization: Localization::default(),
            scripted_lists: ScriptedLists::default(),
            defines: Defines::default(),
            events: Events::default(),
            scripted_modifiers: ScriptedModifiers::default(),
            #[cfg(feature = "ck3")]
            on_actions: OnActions::default(),
            #[cfg(feature = "ck3")]
            interaction_cats: InteractionCategories::default(),
            #[cfg(feature = "ck3")]
            provinces: Provinces::default(),
            #[cfg(feature = "ck3")]
            province_histories: ProvinceHistories::default(),
            #[cfg(feature = "ck3")]
            gameconcepts: GameConcepts::default(),
            #[cfg(feature = "ck3")]
            titles: Titles::default(),
            #[cfg(feature = "ck3")]
            characters: Characters::default(),
            scriptvalues: ScriptValues::default(),
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
            #[cfg(feature = "ck3")]
            gui: Gui::default(),
            #[cfg(feature = "ck3")]
            data_bindings: DataBindings::default(),
            #[cfg(feature = "ck3")]
            assets: Assets::default(),
            #[cfg(feature = "ck3")]
            sounds: Sounds::default(),
            #[cfg(feature = "ck3")]
            music: Musics::default(),
            #[cfg(feature = "ck3")]
            coas: Coas::default(),
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
    /// Returns None if no settings are defined.
    /// Otherwise, returns the overwritten `OutputStyles`.
    ///
    /// Note that the settings from the config can still be overridden
    /// by supplying the --no-color flag.
    fn load_output_styles(&self) -> Option<OutputStyle> {
        let block = self.config.get_field_block("output_style")?;
        if !block.get_field_bool("enable").unwrap_or(true) {
            return Some(OutputStyle::no_color());
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
        Some(style)
    }

    /// A helper function for categories of items that follow the usual pattern of
    /// `.txt` files containing a block with definitions
    pub fn load_pdx_items<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        self.load_pdx_items_ext(itype, add, ".txt");
    }

    /// Like `load_pdx_items` but does not complain about a missing BOM
    pub fn load_pdx_items_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for mut block in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            if entry.filename().to_string_lossy().ends_with(".txt") {
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

    /// Like `load_pdx_items` but does not expect a Unicode file.
    /// Non-Unicode files are mostly used in the history/ section.
    pub fn load_pdx_items_cp1252<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for mut block in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            if entry.filename().to_string_lossy().ends_with(".txt") {
                PdxFile::read_cp1252(entry, &self.fileset.fullpath(entry))
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
    pub fn load_pdx_items_ext<F>(&mut self, itype: Item, add: F, ext: &str)
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
    pub fn load_pdx_files_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for (key, block) in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            let filename = entry.filename().to_string_lossy();
            if let Some(key) = filename.strip_suffix(".txt") {
                if let Some(block) =
                    PdxFile::read_optional_bom(entry, &self.fileset.fullpath(entry))
                {
                    let key = Token::new(key.to_string(), Loc::for_entry(entry));
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

    /// A helper function for categories of items that are unusual in having each item in one file.
    pub fn load_pdx_files_cp1252<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for (key, block) in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            let filename = entry.filename().to_string_lossy();
            if let Some(key) = filename.strip_suffix(".txt") {
                if let Some(block) = PdxFile::read_cp1252(entry, &self.fileset.fullpath(entry)) {
                    let key = Token::new(key.to_string(), Loc::for_entry(entry));
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

    pub fn load_output_settings(&self) {
        if let Some(style) = self.load_output_styles() {
            set_output_style(style);
        }
    }

    fn load_all_generic(&mut self) {
        scope(|s| {
            s.spawn(|_| self.fileset.handle(&mut self.dds));
            s.spawn(|_| self.fileset.handle(&mut self.localization));
            s.spawn(|_| self.fileset.handle(&mut self.scripted_lists));
            s.spawn(|_| self.fileset.handle(&mut self.defines));
            s.spawn(|_| self.fileset.handle(&mut self.scripted_modifiers));
            s.spawn(|_| self.fileset.handle(&mut self.scriptvalues));
            s.spawn(|_| self.fileset.handle(&mut self.triggers));
            s.spawn(|_| self.fileset.handle(&mut self.effects));

            s.spawn(|_| self.fileset.handle(&mut self.events));
        });

        self.load_pdx_items(Item::TriggerLocalization, TriggerLocalization::add);
        self.load_pdx_items(Item::EffectLocalization, EffectLocalization::add);
        self.load_pdx_items(Item::CustomLocalization, CustomLocalization::add);
        self.load_pdx_items_optional_bom(Item::NamedColor, NamedColor::add);

        self.load_pdx_items(Item::Culture, Culture::add);
        self.load_pdx_items(Item::Modifier, Modifier::add);
        self.load_pdx_items(Item::Religion, Religion::add);
    }

    #[cfg(feature = "ck3")]
    fn load_all_ck3(&mut self) {
        scope(|s| {
            s.spawn(|_| self.fileset.handle(&mut self.on_actions));
            s.spawn(|_| self.fileset.handle(&mut self.interaction_cats));
            s.spawn(|_| self.fileset.handle(&mut self.provinces));
            s.spawn(|_| self.fileset.handle(&mut self.province_histories));
            s.spawn(|_| self.fileset.handle(&mut self.gameconcepts));
            s.spawn(|_| self.fileset.handle(&mut self.titles));
            s.spawn(|_| self.fileset.handle(&mut self.characters));
            s.spawn(|_| self.fileset.handle(&mut self.traits));
            s.spawn(|_| self.fileset.handle(&mut self.title_history));
            s.spawn(|_| self.fileset.handle(&mut self.doctrines));
            s.spawn(|_| self.fileset.handle(&mut self.menatarmstypes));
            s.spawn(|_| self.fileset.handle(&mut self.gui));
            s.spawn(|_| self.fileset.handle(&mut self.data_bindings));
            s.spawn(|_| self.fileset.handle(&mut self.assets));
            s.spawn(|_| self.fileset.handle(&mut self.sounds));
            s.spawn(|_| self.fileset.handle(&mut self.music));
        });
        self.load_pdx_items(Item::Decision, Decision::add);
        self.load_pdx_items(Item::Interaction, Interaction::add);
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
        self.load_pdx_items(Item::Terrain, Terrain::add);
        self.load_pdx_items(Item::Region, Region::add);
        self.load_pdx_items(Item::ScriptedGui, ScriptedGui::add);
        self.load_pdx_items(Item::GeneCategory, Gene::add);
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
        self.load_pdx_items(Item::Accessory, Accessory::add);
        self.load_pdx_items(Item::AccessoryVariation, AccessoryVariation::add);
        self.load_pdx_items(Item::PortraitModifierGroup, PortraitModifierGroup::add);
        self.load_pdx_items_ext(
            Item::PortraitModifierPack,
            PortraitModifierPack::add,
            ".modifierpack",
        );
        self.load_pdx_items(Item::PortraitCamera, PortraitCamera::add);
        self.load_pdx_items(Item::AccoladeIcon, AccoladeIcon::add);
        self.load_pdx_items(Item::AccoladeName, AccoladeName::add);
        self.load_pdx_items(Item::AccoladeType, AccoladeType::add);
        self.load_pdx_items(Item::VassalStance, VassalStance::add);
        self.load_pdx_items(Item::Dna, Dna::add);
        self.load_pdx_items(Item::Bookmark, Bookmark::add);
        self.load_pdx_items(Item::BookmarkGroup, BookmarkGroup::add);
        self.load_pdx_items_optional_bom(Item::BookmarkPortrait, BookmarkPortrait::add);
        self.load_pdx_items(Item::Ethnicity, Ethnicity::add);
        self.load_pdx_items(Item::GovernmentType, Government::add);
        self.load_pdx_items(Item::Hook, Hook::add);
        self.load_pdx_items(Item::CouncilPosition, CouncilPosition::add);
        self.load_pdx_items(Item::CouncilTask, CouncilTask::add);
        self.load_pdx_items(Item::PoolSelector, PoolSelector::add);
        self.load_pdx_items(Item::CharacterBackground, CharacterBackground::add);
        self.load_pdx_items(Item::HolySite, HolySite::add);
        self.fileset.handle(&mut self.coas);
        self.load_pdx_items_optional_bom(Item::CoaTemplateList, CoaTemplateList::add);
        self.load_pdx_items(Item::CoaDynamicDefinition, CoaDynamicDefinition::add);
        self.load_pdx_items(Item::Environment, Environment::add);
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
        self.load_pdx_items(Item::PortraitAnimation, PortraitAnimation::add);
        self.load_pdx_items(Item::GameRule, GameRule::add);
        self.load_pdx_items(Item::TravelOption, TravelOption::add);
        self.load_pdx_items(Item::Story, Story::add);
        self.load_pdx_items(Item::LawGroup, LawGroup::add);
        self.load_pdx_items(Item::SuccessionElection, Election::add);
        self.load_pdx_items(Item::DiarchyType, DiarchyType::add);
        self.load_pdx_items(Item::DiarchyMandate, DiarchyMandate::add);
        self.load_pdx_items(Item::Inspiration, Inspiration::add);
        self.load_pdx_items(Item::CoaDesignerColoredEmblem, CoaDesignerColoredEmblem::add);
        self.load_pdx_items(Item::CoaDesignerColorPalette, CoaDesignerColorPalette::add);
        self.load_pdx_items(Item::CoaDesignerEmblemLayout, CoaDesignerEmblemLayout::add);
        self.load_pdx_items(Item::CoaDesignerPattern, CoaDesignerPattern::add);
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
        Building::finalize(&mut self.database);
    }

    #[cfg(feature = "vic3")]
    fn load_all_vic3(&mut self) {
        self.load_pdx_items(Item::BattleCondition, BattleCondition::add);
        self.load_pdx_items(Item::BuildingGroup, BuildingGroup::add);
        self.load_pdx_items(Item::BuildingType, BuildingType::add);
        self.load_pdx_items(Item::Country, Country::add);
        self.load_pdx_items(Item::ProductionMethod, ProductionMethod::add);
        self.load_pdx_items(Item::StateRegion, StateRegion::add);
        self.load_pdx_items(Item::Technology, Technology::add);
        self.load_pdx_items(Item::TechnologyEra, TechnologyEra::add);
        self.load_pdx_items(Item::TerrainManipulator, TerrainManipulator::add);
    }

    pub fn load_all(&mut self) {
        self.load_all_generic();
        #[cfg(feature = "ck3")]
        self.load_all_ck3();
        #[cfg(feature = "vic3")]
        self.load_all_vic3();
    }

    fn validate_all_generic<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.fileset.validate(self));
        s.spawn(|_| self.localization.validate(self));
        s.spawn(|_| self.scripted_lists.validate(self));
        s.spawn(|_| self.defines.validate(self));
        s.spawn(|_| self.scripted_modifiers.validate(self));
        s.spawn(|_| self.scriptvalues.validate(self));
        s.spawn(|_| self.triggers.validate(self));
        s.spawn(|_| self.effects.validate(self));

        s.spawn(|_| self.events.validate(self));
    }

    #[cfg(feature = "ck3")]
    fn validate_all_ck3<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.on_actions.validate(self));
        s.spawn(|_| self.interaction_cats.validate(self));
        s.spawn(|_| self.provinces.validate(self));
        s.spawn(|_| self.province_histories.validate(self));
        s.spawn(|_| self.gameconcepts.validate(self));
        s.spawn(|_| self.titles.validate(self));
        s.spawn(|_| self.characters.validate(self));
        s.spawn(|_| self.traits.validate(self));
        s.spawn(|_| self.title_history.validate(self));
        s.spawn(|_| self.doctrines.validate(self));
        s.spawn(|_| self.menatarmstypes.validate(self));
        s.spawn(|_| self.gui.validate(self));
        s.spawn(|_| self.data_bindings.validate(self));
        s.spawn(|_| self.assets.validate(self));
        s.spawn(|_| self.sounds.validate(self));
        s.spawn(|_| self.music.validate(self));
        s.spawn(|_| self.coas.validate(self));
    }

    #[cfg(feature = "vic3")]
    fn validate_all_vic3(&self, _s: &Scope) {}

    pub fn validate_all(&self) {
        scope(|s| {
            self.validate_all_generic(s);
            #[cfg(feature = "ck3")]
            self.validate_all_ck3(s);
            #[cfg(feature = "vic3")]
            self.validate_all_vic3(s);
        });
        self.database.validate(self);
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

    pub fn item_has_property(&self, itype: Item, key: &str, property: &str) -> bool {
        self.database.has_property(itype, key, property, self)
    }

    #[cfg(feature = "ck3")]
    pub fn item_exists(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::ActivityState => ACTIVITY_STATES.contains(&key),
            Item::ArtifactHistory => ARTIFACT_HISTORY.contains(&key),
            Item::ArtifactRarity => ARTIFACT_RARITY.contains(&key),
            Item::Asset => self.assets.asset_exists(key),
            Item::BlendShape => self.assets.blend_shape_exists(key),
            Item::Character => self.characters.exists(key),
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::DangerType => DANGER_TYPES.contains(&key),
            Item::Define => self.defines.exists(key),
            Item::Dlc => DLC.contains(&key),
            Item::DlcFeature => DLC_FEATURES.contains(&key),
            Item::Doctrine => self.doctrines.exists(key),
            Item::DoctrineParameter => self.doctrines.parameter_exists(key),
            Item::Entity => self.assets.entity_exists(key),
            Item::Event => self.events.exists(key),
            Item::EventNamespace => self.events.namespace_exists(key),
            Item::File => self.fileset.exists(key),
            Item::GameConcept => self.gameconcepts.exists(key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::GeneticConstraint => self.traits.constraint_exists(key),
            Item::InteractionCategory => self.interaction_cats.exists(key),
            Item::Localization => self.localization.exists(key),
            Item::MenAtArms => self.menatarmstypes.exists(key),
            Item::MenAtArmsBase => self.menatarmstypes.base_exists(key),
            Item::Music => self.music.exists(key),
            Item::OnAction => self.on_actions.exists(key),
            Item::Pdxmesh => self.assets.mesh_exists(key),
            Item::PrisonType => PRISON_TYPES.contains(&key),
            Item::Province => self.provinces.exists(key),
            Item::RewardItem => REWARD_ITEMS.contains(&key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::ScriptValue => self.scriptvalues.exists(key),
            Item::Sexuality => SEXUALITIES.contains(&key),
            Item::Skill => SKILLS.contains(&key),
            Item::Sound => self.sounds.exists(key),
            Item::TextureFile => self.assets.texture_exists(key),
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
    pub fn item_exists(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Attitude => ATTITUDE.contains(&key),
            Item::Define => self.defines.exists(key),
            Item::Dlc => DLC.contains(&key),
            Item::DlcFeature => DLC_FEATURES.contains(&key),
            Item::Event => self.events.exists(key),
            Item::EventNamespace => self.events.namespace_exists(key),
            Item::File => self.fileset.exists(key),
            Item::Localization => self.localization.exists(key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::ScriptValue => self.scriptvalues.exists(key),
            Item::Sound => true, // TODO
            _ => self.database.exists(itype, key),
        }
    }

    pub fn item_used(&self, itype: Item, key: &str) {
        match itype {
            Item::File => self.fileset.mark_used(key),
            Item::Localization => self.localization.mark_used(key),
            _ => (),
        }
    }

    pub fn verify_exists(&self, itype: Item, token: &Token) {
        self.verify_exists_implied(itype, token.as_str(), token);
    }

    pub fn verify_exists_implied(&self, itype: Item, key: &str, token: &Token) {
        match itype {
            Item::File => self.fileset.verify_exists_implied(key, token),
            Item::Localization => self.localization.verify_exists_implied(key, token),
            #[cfg(feature = "ck3")]
            Item::Music => self.music.verify_exists_implied(key, token),
            #[cfg(feature = "ck3")]
            Item::Province => self.provinces.verify_exists_implied(key, token),
            #[cfg(feature = "ck3")]
            Item::Sound => self.sounds.verify_exists_implied(key, token, self),
            #[cfg(feature = "ck3")]
            Item::TextureFile => {
                if let Some(entry) = self.assets.get_texture(key) {
                    // TODO: avoid allocating a string here
                    self.fileset.mark_used(&entry.path().to_string_lossy());
                } else {
                    let msg = format!("no texture file {key} anywhere under {}", itype.path());
                    error(token, ErrorKey::MissingFile, &msg);
                }
            }
            _ => {
                if !self.item_exists(itype, key) {
                    let msg = format!("{} {} not defined in {}", itype, key, itype.path());
                    error(token, ErrorKey::MissingItem, &msg);
                }
            }
        }
    }

    pub fn validate_use(&self, itype: Item, key: &Token, block: &Block) {
        self.database.validate_use(itype, key, block, self);
    }

    pub fn validate_call(&self, itype: Item, key: &Token, block: &Block, sc: &mut ScopeContext) {
        self.database.validate_call(itype, key, block, self, sc);
    }

    pub fn get_item<T: DbKind>(&self, itype: Item, key: &str) -> Option<(&Token, &Block, &T)> {
        self.database.get_item(itype, key)
    }

    pub fn get_key_block(&self, itype: Item, key: &str) -> Option<(&Token, &Block)> {
        self.database.get_key_block(itype, key)
    }

    #[cfg(feature = "ck3")]
    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        if let Some(trigger) = self.triggers.get(key.as_str()) {
            Some(trigger)
        } else if let Some(trigger) = self.events.get_trigger(key) {
            Some(trigger)
        } else {
            None
        }
    }
    #[cfg(feature = "vic3")]
    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        self.triggers.get(key.as_str())
    }

    #[cfg(feature = "ck3")]
    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        if let Some(effect) = self.effects.get(key.as_str()) {
            Some(effect)
        } else if let Some(effect) = self.events.get_effect(key) {
            Some(effect)
        } else {
            None
        }
    }
    #[cfg(feature = "vic3")]
    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        self.effects.get(key.as_str())
    }

    pub fn get_defined_string(&self, key: &str) -> Option<&Token> {
        self.defines.get_string(key)
    }

    pub fn get_defined_string_warn(&self, token: &Token, key: &str) -> Option<&Token> {
        let result = self.get_defined_string(key);
        let mut cache = self.warned_defines.write().unwrap();
        if result.is_none() && !cache.contains(key) {
            old_warn(
                token,
                ErrorKey::MissingItem,
                &format!("{key} not defined in common/defines/"),
            );
            cache.insert(key.to_string());
        }
        result
    }
}

/// LAST UPDATED VERSION 1.9.2
const ACTIVITY_STATES: &[&str] = &["passive", "travel", "active"];

/// LAST UPDATED VERSION 1.9.2
const ARTIFACT_HISTORY: &[&str] = &[
    "created_before_history",
    "created",
    "prize_created",
    "discovered",
    "creator_discovered",
    "claimed_by_house",
    "given",
    "stolen",
    "inherited",
    "conquest",
    "taken_in_siege",
    "taken_in_battle",
    "won_in_duel",
    "purchased",
    "prize_awarded",
    "ransomed",
    "reforged",
];

/// LAST UPDATED VERSION 1.9.2
// TODO: parse it from dlc_metadata/ ? Unfortunately Tours and Tournaments
// is an exception.
#[cfg(feature = "ck3")]
const DLC: &[&str] = &[
    "Fashion of the Abbasid Court",
    "The Northern Lords",
    "Garments of the Holy Roman Empire",
    "The Fate of Iberia",
    "The Royal Court",
    "Friends and Foes",
    "tours_and_tournaments",
    "Elegance of the Empire",
];

/// LAST UPDATED VERSION 1.9.2
#[cfg(feature = "ck3")]
const DLC_FEATURES: &[&str] = &[
    "garments_of_the_hre",
    "fashion_of_the_abbasid_court",
    "the_northern_lords",
    "hybridize_culture",
    "diverge_culture",
    "royal_court",
    "reform_culture",
    "court_artifacts",
    "the_fate_of_iberia",
    "friends_and_foes",
    "tours_and_tournaments",
    "advanced_activities",
    "accolades",
    "elegance_of_the_empire",
];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
const DLC: &[&str] = &["dlc001", "dlc002", "dlc003", "dlc004", "dlc005", "dlc006"];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
const DLC_FEATURES: &[&str] =
    &["voice_of_the_people_content", "voice_of_the_people_preorder", "agitators"];

/// LAST UPDATED VERSION 1.9.2
const REWARD_ITEMS: &[&str] = &["newsletter_crown"];

/// LAST UPDATED VERSION 1.9.2
const PRISON_TYPES: &[&str] = &["dungeon", "house_arrest"];

/// LAST UPDATED VERSION 1.9.2
const SKILLS: &[&str] = &["diplomacy", "intrigue", "learning", "martial", "prowess", "stewardship"];

/// LAST UPDATED VERSION 1.9.2
const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual", "none"];

/// LAST UPDATED VERSION 1.9.2
const TITLE_HISTORY_TYPES: &[&str] = &[
    "conquest",
    "conquest_holy_war",
    "conquest_claim",
    "conquest_populist",
    "election",
    "inheritance",
    "abdication",
    "created",
    "destroyed",
    "usurped",
    "granted",
    "revoked",
    "independency",
    "leased_out",
    "lease_revoked",
    "returned",
    "faction_demand",
    "swear_fealty",
];

/// LAST UPDATED VERSION 1.9.2
const TRAIT_CATEGORIES: &[&str] = &[
    "personality",
    "education",
    "childhood",
    "commander",
    "winter_commander",
    "lifestyle",
    "court_type",
    "fame",
    "health",
];

/// LAST UPDATED VERSION 1.9.2
const DANGER_TYPES: &[&str] = &[
    "default",
    "battle",
    "raid",
    "siege",
    "army",
    "occupation",
    "county_control",
    "county_opinion",
    "owner_opinion",
];

/// LAST UPDATED VERSION 1.9.2
const ARTIFACT_RARITY: &[&str] = &["common", "masterwork", "famed", "illustrious"];

/// LAST UPDATED VIC3 VERSION 1.3.6
const ATTITUDE: &[&str] = &[
    "antagonistic",
    "belligerent",
    "cautious",
    "conciliatory",
    "cooperative",
    "disinterested",
    "domineering",
    "genial",
    "human",
    "loyal",
    "protective",
    "rebellious",
    "wary",
];
