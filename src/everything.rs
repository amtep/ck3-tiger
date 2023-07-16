use std::borrow::Cow;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use anyhow::Result;
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
    event_themes::{EventBackground, EventTheme, EventTransition},
    events::Events,
    factions::Faction,
    flavorization::Flavorization,
    focus::Focus,
    gameconcepts::GameConcepts,
    gamerules::GameRule,
    government::Government,
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
    opinions::OpinionModifier,
    perks::Perk,
    points_of_interest::PointOfInterest,
    pool::{CharacterBackground, PoolSelector},
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
    genes::Gene,
    gui::Gui,
    localization::Localization,
    on_actions::OnActions,
    portrait::{PortraitAnimation, PortraitCamera, PortraitModifierGroup, PortraitModifierPack},
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
#[cfg(feature = "vic3")]
use crate::parse::json::parse_json_file;
use crate::pdxfile::PdxFile;
use crate::report::{error, old_warn, set_output_style, ErrorKey, OutputStyle, Severity};
use crate::rivers::Rivers;
use crate::token::{Loc, Token};
#[cfg(feature = "vic3")]
use crate::vic3::data::{
    ai_strategies::AiStrategy,
    battle_conditions::BattleCondition,
    buildings::{BuildingGroup, BuildingType},
    character_interactions::CharacterInteraction,
    countries::Country,
    cultures::Culture,
    events::Events,
    gameconcepts::GameConcept,
    goods::Goods,
    ideologies::Ideology,
    institutions::Institution,
    interest_groups::InterestGroup,
    journalentries::Journalentry,
    laws::{LawGroup, LawType},
    media_aliases::MediaAlias,
    modifiers::Modifier,
    pops::PopType,
    production_methods::{ProductionMethod, ProductionMethodGroup},
    provinces::Provinces,
    religions::Religion,
    scripted_buttons::ScriptedButton,
    state_regions::StateRegion,
    state_traits::StateTrait,
    strategic_regions::StrategicRegion,
    technology::{Technology, TechnologyEra},
    terrain::{Terrain, TerrainLabel, TerrainManipulator, TerrainMask, TerrainMaterial},
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
    pub on_actions: OnActions,

    #[cfg(feature = "ck3")]
    pub interaction_cats: InteractionCategories,

    /// Processed map data
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

    pub gui: Gui,
    #[cfg(feature = "ck3")]
    pub data_bindings: DataBindings,

    pub assets: Assets,
    #[cfg(feature = "ck3")]
    pub sounds: Sounds,
    #[cfg(feature = "ck3")]
    pub music: Musics,

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
            Block::new(Loc::for_file(config_file, FileKind::Mod))
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
            on_actions: OnActions::default(),
            #[cfg(feature = "ck3")]
            interaction_cats: InteractionCategories::default(),
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
            gui: Gui::default(),
            #[cfg(feature = "ck3")]
            data_bindings: DataBindings::default(),
            assets: Assets::default(),
            #[cfg(feature = "ck3")]
            sounds: Sounds::default(),
            #[cfg(feature = "ck3")]
            music: Musics::default(),
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
    pub fn load_pdx_files_optional_bom_ext<F>(&mut self, itype: Item, add: F, ext: &str)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        for (key, block) in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            let filename = entry.filename().to_string_lossy();
            if let Some(key) = filename.strip_suffix(ext) {
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

    pub fn load_pdx_files_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block) + Sync + Send,
    {
        self.load_pdx_files_optional_bom_ext(itype, add, ".txt");
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

    #[cfg(feature = "vic3")]
    pub fn load_json<F>(&mut self, itype: Item, add_json: F)
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
            s.spawn(|_| self.fileset.handle(&mut self.scriptvalues));
            s.spawn(|_| self.fileset.handle(&mut self.triggers));
            s.spawn(|_| self.fileset.handle(&mut self.effects));
            s.spawn(|_| self.fileset.handle(&mut self.assets));
            s.spawn(|_| self.fileset.handle(&mut self.gui));
            s.spawn(|_| self.fileset.handle(&mut self.on_actions));
            s.spawn(|_| self.fileset.handle(&mut self.coas));

            // These are items that are different between vic3 and ck3 but share the same name
            s.spawn(|_| self.fileset.handle(&mut self.events));
            s.spawn(|_| self.fileset.handle(&mut self.provinces));
        });

        self.load_pdx_items(Item::Accessory, Accessory::add);
        self.load_pdx_items(Item::AccessoryVariation, AccessoryVariation::add);
        self.load_pdx_items(Item::CoaDesignerColoredEmblem, CoaDesignerColoredEmblem::add);
        self.load_pdx_items(Item::CoaDesignerPattern, CoaDesignerPattern::add);
        self.load_pdx_items_optional_bom(Item::CoaTemplateList, CoaTemplateList::add);
        self.load_pdx_items(Item::CustomLocalization, CustomLocalization::add);
        self.load_pdx_items(Item::EffectLocalization, EffectLocalization::add);
        self.load_pdx_items(Item::Ethnicity, Ethnicity::add);
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

        // These are items that are different between vic3 and ck3 but share the same name
        self.load_pdx_items(Item::Culture, Culture::add);
        self.load_pdx_items(Item::Modifier, Modifier::add);
        self.load_pdx_items(Item::Religion, Religion::add);
    }

    #[cfg(feature = "ck3")]
    fn load_all_ck3(&mut self) {
        scope(|s| {
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
        self.load_pdx_items(Item::GameRule, GameRule::add);
        self.load_pdx_items(Item::TravelOption, TravelOption::add);
        self.load_pdx_items(Item::Story, Story::add);
        self.load_pdx_items(Item::LawGroup, LawGroup::add);
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
        self.load_pdx_items(Item::AiStrategy, AiStrategy::add);
        self.load_pdx_items(Item::BattleCondition, BattleCondition::add);
        self.load_pdx_items(Item::BuildingGroup, BuildingGroup::add);
        self.load_pdx_items(Item::BuildingType, BuildingType::add);
        self.load_pdx_items(Item::CharacterInteraction, CharacterInteraction::add);
        self.load_pdx_items(Item::Country, Country::add);
        self.load_pdx_items(Item::GameConcept, GameConcept::add);
        self.load_pdx_items(Item::Goods, Goods::add);
        self.load_pdx_items(Item::Ideology, Ideology::add);
        self.load_pdx_items(Item::Institution, Institution::add);
        self.load_pdx_items(Item::InterestGroup, InterestGroup::add);
        self.load_pdx_items(Item::Journalentry, Journalentry::add);
        self.load_pdx_items(Item::LawGroup, LawGroup::add);
        self.load_pdx_items(Item::LawType, LawType::add);
        self.load_pdx_items(Item::MediaAlias, MediaAlias::add);
        self.load_pdx_items(Item::PopType, PopType::add);
        self.load_pdx_items(Item::ProductionMethod, ProductionMethod::add);
        self.load_pdx_items(Item::ProductionMethodGroup, ProductionMethodGroup::add);
        self.load_pdx_items(Item::ScriptedButton, ScriptedButton::add);
        self.load_pdx_items(Item::StateRegion, StateRegion::add);
        self.load_pdx_items(Item::StateTrait, StateTrait::add);
        self.load_pdx_items(Item::StrategicRegion, StrategicRegion::add);
        self.load_pdx_items(Item::Technology, Technology::add);
        self.load_pdx_items(Item::TechnologyEra, TechnologyEra::add);
        self.load_pdx_items(Item::Terrain, Terrain::add);
        self.load_pdx_items(Item::TerrainLabel, TerrainLabel::add);
        self.load_pdx_items(Item::TerrainManipulator, TerrainManipulator::add);
        self.load_pdx_files_optional_bom_ext(
            Item::TerrainMaterial,
            TerrainMaterial::add,
            ".settings",
        );
        self.load_json(Item::TerrainMask, TerrainMask::add_json);
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
        s.spawn(|_| self.assets.validate(self));
        s.spawn(|_| self.gui.validate(self));
        s.spawn(|_| self.on_actions.validate(self));

        s.spawn(|_| self.events.validate(self));
        s.spawn(|_| self.provinces.validate(self));
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
        s.spawn(|_| self.coas.validate(self));
    }

    #[cfg(feature = "vic3")]
    fn validate_all_vic3(&self, _s: &Scope) {
        StrategicRegion::crosscheck(self);
    }

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
            Item::Asset => self.assets.asset_exists(key),
            Item::Attitude => ATTITUDES.contains(&key),
            Item::BlendShape => self.assets.blend_shape_exists(key),
            Item::CharacterRole => CHARACTER_ROLES.contains(&key),
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::CountryTier => COUNTRY_TIERS.contains(&key),
            Item::Define => self.defines.exists(key),
            Item::Dlc => DLC.contains(&key),
            Item::DlcFeature => DLC_FEATURES.contains(&key),
            Item::Entity => self.assets.entity_exists(key),
            Item::Event => self.events.exists(key),
            Item::EventNamespace => self.events.namespace_exists(key),
            Item::File => self.fileset.exists(key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::Level => LEVELS.contains(&key),
            Item::Localization => self.localization.exists(key),
            Item::OnAction => self.on_actions.exists(key),
            Item::Pdxmesh => self.assets.mesh_exists(key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::ScriptValue => self.scriptvalues.exists(key),
            Item::SecretGoal => SECRET_GOALS.contains(&key),
            Item::Sound => {
                if let Some(filename) = key.strip_prefix("file://") {
                    self.fileset.exists(filename)
                } else {
                    SOUNDS.contains(&key)
                }
            }
            Item::TextureFile => self.assets.texture_exists(key),
            Item::Wargoal => WARGOALS.contains(&key),
            Item::TutorialLesson => true, // TODO
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
            Item::Province => self.provinces.verify_exists_implied(key, token),
            #[cfg(feature = "ck3")]
            Item::Sound => self.sounds.verify_exists_implied(key, token, self),
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
                    let path = itype.path();
                    let msg = if path.is_empty() {
                        format!("unknown {itype} {key}")
                    } else {
                        format!("{itype} {key} not defined in {path}")
                    };
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
#[cfg(feature = "ck3")]
const ACTIVITY_STATES: &[&str] = &["passive", "travel", "active"];

/// LAST UPDATED VERSION 1.9.2
#[cfg(feature = "ck3")]
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
#[cfg(feature = "ck3")]
const REWARD_ITEMS: &[&str] = &["newsletter_crown"];

/// LAST UPDATED VERSION 1.9.2
#[cfg(feature = "ck3")]
const PRISON_TYPES: &[&str] = &["dungeon", "house_arrest"];

/// LAST UPDATED VERSION 1.9.2
#[cfg(feature = "ck3")]
const SKILLS: &[&str] = &["diplomacy", "intrigue", "learning", "martial", "prowess", "stewardship"];

/// LAST UPDATED VERSION 1.9.2
#[cfg(feature = "ck3")]
const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual", "none"];

/// LAST UPDATED VERSION 1.9.2
#[cfg(feature = "ck3")]
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
#[cfg(feature = "ck3")]
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
#[cfg(feature = "ck3")]
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
#[cfg(feature = "ck3")]
const ARTIFACT_RARITY: &[&str] = &["common", "masterwork", "famed", "illustrious"];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
const ATTITUDES: &[&str] = &[
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
/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
pub const COUNTRY_TIERS: &[&str] =
    &["city_state", "principality", "grand_principality", "kingdom", "empire", "hegemony"];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
pub const LEVELS: &[&str] = &["very_low", "low", "medium", "high", "very_high"];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
pub const SECRET_GOALS: &[&str] =
    &["none", "befriend", "reconcile", "protect", "antagonize", "conquer", "dominate"];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
pub const WARGOALS: &[&str] = &[
    "annex_country",
    "ban_slavery",
    "colonization_rights",
    "conquer_state",
    "contain_threat",
    "force_recognition",
    "humiliation",
    "independence",
    "liberate_country",
    "liberate_subject",
    "make_dominion",
    "make_puppet",
    "make_vassal",
    "open_market",
    "regime_change",
    "return_state",
    "revoke_claim",
    "secession",
    "take_treaty_port",
    "transfer_subject",
    "unification",
    "unification_leadership",
    "war_reparations",
];

/// LAST UPDATED VIC3 VERSION 1.3.6
#[cfg(feature = "vic3")]
// TODO: maybe ruler and heir too?
pub const CHARACTER_ROLES: &[&str] = &["admiral", "agitator", "general", "politician"];

/// LAST UPDATED VIC3 VERSION 1.3.6
/// Taken from the object browser
#[cfg(feature = "vic3")]
const SOUNDS: &[&str] = &[
    "event:/MUSIC/Main/theme_01",
    "event:/MUSIC/Mood/V3/Base/01_A_Prospering_Country",
    "event:/MUSIC/Mood/V3/Base/02_Rule_The_World",
    "event:/MUSIC/Mood/V3/Base/03_Adagio_For_Four_Strings",
    "event:/MUSIC/Mood/V3/Base/04_At_The_Country_Manor",
    "event:/MUSIC/Mood/V3/Base/05_Benedicte",
    "event:/MUSIC/Mood/V3/Base/06_England_1851",
    "event:/MUSIC/Mood/V3/Base/07_Moonlight_Waltz",
    "event:/MUSIC/Mood/V3/Base/08_Our_New_Residence",
    "event:/MUSIC/Mood/V3/Base/09_Over_The_Calm_Ocean",
    "event:/MUSIC/Mood/V3/Base/10_Quite_Noble_Festivities",
    "event:/MUSIC/Mood/V3/Base/11_Remembering_Prince_Albert",
    "event:/MUSIC/Mood/V3/Base/12_Sunrise_Over_London",
    "event:/MUSIC/Mood/V3/Base/13_Sunset_Over_Windsor_Castle",
    "event:/MUSIC/Mood/V3/Base/14_Tea_Time",
    "event:/MUSIC/Mood/V3/Base/15_The_Queen_Is_Actually_Amused",
    "event:/MUSIC/Mood/V3/Base/16_To_Build_A_Factory",
    "event:/MUSIC/Mood/V3/Base/17_Asset_Gathering",
    "event:/MUSIC/Mood/V3/Base/18_British_Soil",
    "event:/MUSIC/Mood/V3/Base/19_Death_March",
    "event:/MUSIC/Mood/V3/Base/20_Glory_To_The_Queen",
    "event:/MUSIC/Stingers/diplomatic_play/begun",
    "event:/MUSIC/Stingers/events/civil",
    "event:/MUSIC/Stingers/events/dramatic",
    "event:/MUSIC/Stingers/events/enthusiastic",
    "event:/MUSIC/Stingers/events/political",
    "event:/MUSIC/Stingers/events/sadness",
    "event:/MUSIC/Stingers/events/spiritual",
    "event:/MUSIC/Stingers/events/tranquil",
    "event:/MUSIC/Stingers/game_over/negative",
    "event:/MUSIC/Stingers/game_over/positive",
    "event:/MUSIC/Stingers/toasts/acquired_technology",
    "event:/MUSIC/Stingers/toasts/country_revolution",
    "event:/MUSIC/Stingers/toasts/election_results_negative",
    "event:/MUSIC/Stingers/toasts/election_results_neutral",
    "event:/MUSIC/Stingers/toasts/election_results_positive",
    "event:/MUSIC/Stingers/toasts/heir_born",
    "event:/MUSIC/Stingers/toasts/journal_entry_completed",
    "event:/MUSIC/Stingers/toasts/law_changed",
    "event:/MUSIC/Stingers/toasts/migration_target_created_other",
    "event:/MUSIC/Stingers/toasts/native_uprising",
    "event:/MUSIC/Stingers/toasts/new_parties",
    "event:/MUSIC/Stingers/toasts/rank_changed",
    "event:/MUSIC/Stingers/toasts/used_favor",
    "event:/MUSIC/Stingers/unique_buildings/angkorwat",
    "event:/MUSIC/Stingers/unique_buildings/bigben",
    "event:/MUSIC/Stingers/unique_buildings/eiffeltower",
    "event:/MUSIC/Stingers/unique_buildings/forbiddencity",
    "event:/MUSIC/Stingers/unique_buildings/hagiasophia",
    "event:/MUSIC/Stingers/unique_buildings/mosqueofdjenna",
    "event:/MUSIC/Stingers/unique_buildings/saintbasilscathedral",
    "event:/MUSIC/Stingers/unique_buildings/statueofliberty",
    "event:/MUSIC/Stingers/unique_buildings/tajmahal",
    "event:/MUSIC/Stingers/unique_buildings/thevatican",
    "event:/MUSIC/Stingers/unique_buildings/thewhitehouse",
    "event:/MUSIC/Stingers/war/outcome_neutral",
    "event:/MUSIC/Stingers/war/start",
    "event:/SFX/Ambience/2D/master",
    "event:/SFX/Ambience/3D/Hub/city_african",
    "event:/SFX/Ambience/3D/Hub/city_arabic",
    "event:/SFX/Ambience/3D/Hub/city_asian",
    "event:/SFX/Ambience/3D/Hub/city_south_american",
    "event:/SFX/Ambience/3D/Hub/city_western",
    "event:/SFX/Ambience/3D/Hub/farm",
    "event:/SFX/Ambience/3D/Hub/forestry",
    "event:/SFX/Ambience/3D/Hub/industry",
    "event:/SFX/Ambience/3D/Hub/mining",
    "event:/SFX/Ambience/3D/Hub/oil_rig",
    "event:/SFX/Ambience/3D/Hub/plantation",
    "event:/SFX/Ambience/3D/Hub/port",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/agitator_speaking",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/barricade",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/conspiring",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/cops_march",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/french_algeria",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/garibaldi",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/gunboat_diplomacy",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/hostile_court",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/monarch_holding_court",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/people_sneaking",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/prison",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/realist_household",
    "event:/SFX/DLC/1.3_ip1/UI/agitator_promote",
    "event:/SFX/DLC/1.3_ip1/UI/character_interaction",
    "event:/SFX/DLC/1.3_ip1/UI/character_invite",
    "event:/SFX/DLC/1.3_ip1/UI/exile_character",
    "event:/SFX/DLC/1.3_ip1/UI/exile_pool_open",
    "event:/SFX/DLC/1.3_ip1/UI/generic_agitator_stinger",
    "event:/SFX/DLC/1.3_ip1/UI/historical_agitator_stinger",
    "event:/SFX/DLC/1.3_ip1/UI/item_revolutionary_movt",
    "event:/SFX/DLC/1.3_ip1/UI/main_menu_illustration",
    "event:/SFX/DLC/1.3_ip1/UI/new_country_start",
    "event:/SFX/DLC/1.3_ip1/UI/open_character_panel",
    "event:/SFX/DLC/1.3_ip1/UI/revolution_progress_tick",
    "event:/SFX/Events/africa/animism",
    "event:/SFX/Events/africa/city_center",
    "event:/SFX/Events/africa/construction_colony",
    "event:/SFX/Events/africa/desert_expedition",
    "event:/SFX/Events/africa/diplomats_negotiating",
    "event:/SFX/Events/africa/leader_arguing",
    "event:/SFX/Events/africa/prosperous_farm",
    "event:/SFX/Events/africa/public_protest",
    "event:/SFX/Events/africa/soldiers_breaking_protest",
    "event:/SFX/Events/asia/buddhism",
    "event:/SFX/Events/asia/confucianism_shinto",
    "event:/SFX/Events/asia/dead_cattle_poor_harvest",
    "event:/SFX/Events/asia/factory_accident",
    "event:/SFX/Events/asia/farmers_market",
    "event:/SFX/Events/asia/hinduism_sikhism",
    "event:/SFX/Events/asia/politician_parliament_motion",
    "event:/SFX/Events/asia/poor_people_moving",
    "event:/SFX/Events/asia/sepoy_mutiny",
    "event:/SFX/Events/asia/union_leader",
    "event:/SFX/Events/asia/westeners_arriving_in_east_asia",
    "event:/SFX/Events/europenorthamerica/american_civil_war",
    "event:/SFX/Events/europenorthamerica/before_the_battle",
    "event:/SFX/Events/europenorthamerica/capitalists_meeting",
    "event:/SFX/Events/europenorthamerica/gold_prospectors",
    "event:/SFX/Events/europenorthamerica/judaism",
    "event:/SFX/Events/europenorthamerica/london_center",
    "event:/SFX/Events/europenorthamerica/native_american",
    "event:/SFX/Events/europenorthamerica/opium_smoker",
    "event:/SFX/Events/europenorthamerica/political_extremism",
    "event:/SFX/Events/europenorthamerica/rich_and_poor",
    "event:/SFX/Events/europenorthamerica/russian_serfs",
    "event:/SFX/Events/europenorthamerica/slaves_breaking_their_chains",
    "event:/SFX/Events/europenorthamerica/springtime_of_nation",
    "event:/SFX/Events/europenorthamerica/sufferage",
    "event:/SFX/Events/generic/civil",
    "event:/SFX/Events/generic/clandestine",
    "event:/SFX/Events/generic/dramatic",
    "event:/SFX/Events/generic/enthusiastic",
    "event:/SFX/Events/generic/political",
    "event:/SFX/Events/generic/sadness",
    "event:/SFX/Events/generic/spiritual",
    "event:/SFX/Events/generic/tranquil",
    "event:/SFX/Events/middleeast/battlefield_trenches",
    "event:/SFX/Events/middleeast/courtroom_upheaval",
    "event:/SFX/Events/middleeast/engineer_blueprint",
    "event:/SFX/Events/middleeast/islam",
    "event:/SFX/Events/middleeast/jungle_expedition",
    "event:/SFX/Events/middleeast/middleclass_cafe",
    "event:/SFX/Events/middleeast/oil_derricks",
    "event:/SFX/Events/middleeast/police_breaking_door",
    "event:/SFX/Events/middleeast/upperclass_party",
    "event:/SFX/Events/misc/1Character_2Flags",
    "event:/SFX/Events/misc/1Character_4Flags",
    "event:/SFX/Events/misc/1Character_Banner",
    "event:/SFX/Events/misc/2Characters",
    "event:/SFX/Events/misc/Icons_Various",
    "event:/SFX/Events/southamerica/aristocrats",
    "event:/SFX/Events/southamerica/child_labor",
    "event:/SFX/Events/southamerica/christianity",
    "event:/SFX/Events/southamerica/election",
    "event:/SFX/Events/southamerica/factory_opening",
    "event:/SFX/Events/southamerica/public_figure_assassination",
    "event:/SFX/Events/southamerica/slaves_night",
    "event:/SFX/Events/southamerica/war_civilians",
    "event:/SFX/Events/unspecific/airplane",
    "event:/SFX/Events/unspecific/airship",
    "event:/SFX/Events/unspecific/arctic",
    "event:/SFX/Events/unspecific/armored_train",
    "event:/SFX/Events/unspecific/art_gallery",
    "event:/SFX/Events/unspecific/automobile",
    "event:/SFX/Events/unspecific/destruction",
    "event:/SFX/Events/unspecific/devastation",
    "event:/SFX/Events/unspecific/factory_closed",
    "event:/SFX/Events/unspecific/gears_pistons",
    "event:/SFX/Events/unspecific/iceberg_in_the_antartica",
    "event:/SFX/Events/unspecific/leader_speaking_to_a_group_of_people",
    "event:/SFX/Events/unspecific/military_parade",
    "event:/SFX/Events/unspecific/naval_battle",
    "event:/SFX/Events/unspecific/sick_people_in_a_field_hospital",
    "event:/SFX/Events/unspecific/signed_contract",
    "event:/SFX/Events/unspecific/steam_ship",
    "event:/SFX/Events/unspecific/temperance_movement",
    "event:/SFX/Events/unspecific/trains",
    "event:/SFX/Events/unspecific/vandalized_storefront",
    "event:/SFX/Events/unspecific/whaling",
    "event:/SFX/Events/unspecific/world_fair",
    "event:/SFX/UI/Alerts/current_situation",
    "event:/SFX/UI/Alerts/event_appear",
    "event:/SFX/UI/Alerts/high_attrition",
    "event:/SFX/UI/Alerts/letter_appear",
    "event:/SFX/UI/Alerts/notification",
    "event:/SFX/UI/Alerts/notification_collapse",
    "event:/SFX/UI/Alerts/notification_dismiss",
    "event:/SFX/UI/Alerts/notification_expand",
    "event:/SFX/UI/Alerts/Toasts/acquired_technology",
    "event:/SFX/UI/Alerts/Toasts/capitulated",
    "event:/SFX/UI/Alerts/Toasts/country_mobilization",
    "event:/SFX/UI/Alerts/Toasts/country_revolution",
    "event:/SFX/UI/Alerts/Toasts/election_results",
    "event:/SFX/UI/Alerts/Toasts/heir_born",
    "event:/SFX/UI/Alerts/Toasts/journal_entry_completed",
    "event:/SFX/UI/Alerts/Toasts/law_changed",
    "event:/SFX/UI/Alerts/Toasts/migration_target",
    "event:/SFX/UI/Alerts/Toasts/native_uprising",
    "event:/SFX/UI/Alerts/Toasts/new_parties",
    "event:/SFX/UI/Alerts/Toasts/peace_agreement",
    "event:/SFX/UI/Alerts/Toasts/rank_changed",
    "event:/SFX/UI/Alerts/Toasts/ranking_to_great_power",
    "event:/SFX/UI/Alerts/Toasts/_transient",
    "event:/SFX/UI/Alerts/Toasts/used_favor",
    "event:/SFX/UI/Alerts/warning_fist_appear",
    "event:/SFX/UI/Budget/coins_lvl_1",
    "event:/SFX/UI/Budget/coins_lvl_2",
    "event:/SFX/UI/Budget/coins_lvl_3",
    "event:/SFX/UI/Budget/coins_lvl_4",
    "event:/SFX/UI/Budget/coins_lvl_5",
    "event:/SFX/UI/Budget/pause_all",
    "event:/SFX/UI/Budget/resume_all",
    "event:/SFX/UI/Frontend/bookmark_bottom_show",
    "event:/SFX/UI/Frontend/start_game",
    "event:/SFX/UI/Frontend/start_panel_show",
    "event:/SFX/UI/Global/alert_remove",
    "event:/SFX/UI/Global/back",
    "event:/SFX/UI/Global/checkbox",
    "event:/SFX/UI/Global/close",
    "event:/SFX/UI/Global/confirm",
    "event:/SFX/UI/Global/decrement",
    "event:/SFX/UI/Global/exit_game",
    "event:/SFX/UI/Global/flag",
    "event:/SFX/UI/Global/game_pause",
    "event:/SFX/UI/Global/game_speed",
    "event:/SFX/UI/Global/game_unpause",
    "event:/SFX/UI/Global/increment",
    "event:/SFX/UI/Global/map_click",
    "event:/SFX/UI/Global/map_hover",
    "event:/SFX/UI/Global/map_hover_interact",
    "event:/SFX/UI/Global/panel_hide",
    "event:/SFX/UI/Global/panel_show",
    "event:/SFX/UI/Global/pause_logo",
    "event:/SFX/UI/Global/play_continue",
    "event:/SFX/UI/Global/play_pause",
    "event:/SFX/UI/Global/pointer_over",
    "event:/SFX/UI/Global/popup_hide",
    "event:/SFX/UI/Global/popup_show",
    "event:/SFX/UI/Global/promote",
    "event:/SFX/UI/Global/select",
    "event:/SFX/UI/Global/shimmer",
    "event:/SFX/UI/Global/situation",
    "event:/SFX/UI/Global/suppress",
    "event:/SFX/UI/Global/tab",
    "event:/SFX/UI/Global/tooltip_lock",
    "event:/SFX/UI/Global/victoria_logo",
    "event:/SFX/UI/Global/zoom",
    "event:/SFX/UI/MapInteraction/build_building",
    "event:/SFX/UI/MapInteraction/build_building_epic",
    "event:/SFX/UI/MapInteraction/civil",
    "event:/SFX/UI/MapInteraction/diplomatic_action_benign",
    "event:/SFX/UI/MapInteraction/diplomatic_action_hostile",
    "event:/SFX/UI/MapInteraction/diplomatic_action_interest",
    "event:/SFX/UI/MapInteraction/diplomatic_action_request",
    "event:/SFX/UI/MapInteraction/diplomatic_play",
    "event:/SFX/UI/MapInteraction/diplomatic_play_epic",
    "event:/SFX/UI/MapInteraction/establish_colony",
    "event:/SFX/UI/MapInteraction/map_interact_transient",
    "event:/SFX/UI/MapInteraction/trade_route",
    "event:/SFX/UI/MapLenses/diplomatic",
    "event:/SFX/UI/MapLenses/diplomatic_stop",
    "event:/SFX/UI/MapLenses/generic",
    "event:/SFX/UI/MapLenses/generic_open",
    "event:/SFX/UI/MapLenses/location_finder",
    "event:/SFX/UI/MapLenses/military",
    "event:/SFX/UI/MapLenses/military_stop",
    "event:/SFX/UI/MapLenses/mobilize_general",
    "event:/SFX/UI/MapLenses/political",
    "event:/SFX/UI/MapLenses/political_stop",
    "event:/SFX/UI/MapLenses/production",
    "event:/SFX/UI/MapLenses/production_stop",
    "event:/SFX/UI/MapLenses/trade",
    "event:/SFX/UI/MapLenses/trade_stop",
    "event:/SFX/UI/Market/filter/industrial",
    "event:/SFX/UI/Market/filter/luxury",
    "event:/SFX/UI/Market/filter/military",
    "event:/SFX/UI/Market/filter/staple",
    "event:/SFX/UI/MaxiMap/activate",
    "event:/SFX/UI/MaxiMap/deactivate",
    "event:/SFX/UI/Military/add_war_goal",
    "event:/SFX/UI/Military/commander_mobilize",
    "event:/SFX/UI/Military/commander_promote",
    "event:/SFX/UI/Military/commander_recruit",
    "event:/SFX/UI/Military/commander_retire",
    "event:/SFX/UI/Military/command_grant",
    "event:/SFX/UI/Military/command_remove",
    "event:/SFX/UI/Military/conscription_center_activate",
    "event:/SFX/UI/Military/order_admiral_convoy_raiding",
    "event:/SFX/UI/Military/order_admiral_intercept",
    "event:/SFX/UI/Military/order_admiral_naval_invasion",
    "event:/SFX/UI/Military/order_admiral_patrol",
    "event:/SFX/UI/Military/order_general_activate",
    "event:/SFX/UI/Military/order_general_front_advance",
    "event:/SFX/UI/Military/order_general_front_defend",
    "event:/SFX/UI/Military/order_general_standby",
    "event:/SFX/UI/Military/strategic_objective_confirm",
    "event:/SFX/UI/MusicPlayer/music_density_slider",
    "event:/SFX/UI/Popups/diplomatic_play_demobilize",
    "event:/SFX/UI/Popups/diplomatic_play_mobilize",
    "event:/SFX/UI/Popups/diplomatic_play_oppose",
    "event:/SFX/UI/Popups/diplomatic_play_support",
    "event:/SFX/UI/Popups/war_breaking_out",
    "event:/SFX/UI/Popups/war_to_arms",
    "event:/SFX/UI/SideBar/budget",
    "event:/SFX/UI/SideBar/budget_stop",
    "event:/SFX/UI/SideBar/buildings",
    "event:/SFX/UI/SideBar/buildings_stop",
    "event:/SFX/UI/SideBar/country",
    "event:/SFX/UI/SideBar/country_stop",
    "event:/SFX/UI/SideBar/culture",
    "event:/SFX/UI/SideBar/culture_stop",
    "event:/SFX/UI/SideBar/diplomacy",
    "event:/SFX/UI/SideBar/diplomacy_stop",
    "event:/SFX/UI/SideBar/journal",
    "event:/SFX/UI/SideBar/journal_stop",
    "event:/SFX/UI/SideBar/list_hide",
    "event:/SFX/UI/SideBar/list_show",
    "event:/SFX/UI/SideBar/markets",
    "event:/SFX/UI/SideBar/markets_stop",
    "event:/SFX/UI/SideBar/military",
    "event:/SFX/UI/SideBar/military_stop",
    "event:/SFX/UI/SideBar/outliner",
    "event:/SFX/UI/SideBar/outliner_stop",
    "event:/SFX/UI/SideBar/politics",
    "event:/SFX/UI/SideBar/politics_stop",
    "event:/SFX/UI/SideBar/population",
    "event:/SFX/UI/SideBar/population_stop",
    "event:/SFX/UI/SideBar/technology",
    "event:/SFX/UI/SideBar/technology_stop",
    "event:/SFX/UI/SideBar/vickypedia",
    "event:/SFX/UI/SideBar/vickypedia_stop",
    "event:/SFX/UI/Technology/confirm",
    "event:/SFX/Vehicles/bleriotxi",
    "event:/SFX/Vehicles/car",
    "event:/SFX/Vehicles/flatbed_truck",
    "event:/SFX/Vehicles/horse_cart",
    "event:/SFX/Vehicles/ships/ship_cargo",
    "event:/SFX/Vehicles/ships/ship_transport",
    "event:/SFX/Vehicles/ships/steamboat",
    "event:/SFX/Vehicles/tractor",
    "event:/SFX/Vehicles/train/cargo/logs",
    "event:/SFX/Vehicles/train/cargo/ore",
    "event:/SFX/Vehicles/train/diesel",
    "event:/SFX/Vehicles/train/electric",
    "event:/SFX/Vehicles/train/european_locomotive",
    "event:/SFX/Vehicles/zeppelin",
    "event:/SFX/Vehicles/zeppelin_2",
    "event:/SFX/VFX/building_demolish",
    "event:/SFX/VFX/building_demote",
    "event:/SFX/VFX/building_promote",
    "event:/SFX/VFX/conscription_center",
    "event:/SFX/VFX/devastation_stage_1",
    "event:/SFX/VFX/devastation_stage_2",
    "event:/SFX/VFX/devastation_stage_3",
    "event:/SFX/VFX/fireworks",
    "event:/SFX/VFX/geyser",
    "event:/SFX/VFX/pollution",
    "event:/SFX/VFX/rain",
    "event:/SFX/VFX/revolution_ongoing",
    "event:/SFX/VFX/sandstorm",
    "event:/SFX/VFX/scaffolding/big_start",
    "event:/SFX/VFX/scaffolding/big_stop",
    "event:/SFX/VFX/scaffolding/sml_start",
    "event:/SFX/VFX/scaffolding/sml_stop",
    "event:/SFX/VFX/scaffolding/special_start",
    "event:/SFX/VFX/scaffolding/special_stop",
    "event:/SFX/VFX/snow",
    "event:/SFX/VFX/unrest_2",
    "event:/SFX/VFX/unrest_3",
    "event:/SFX/VFX/unrest_4",
    "event:/SFX/VFX/volcano",
    "event:/SFX/VFX/war/armored_division/aerial_recon",
    "event:/SFX/VFX/war/armored_division/mechanized_infantry",
    "event:/SFX/VFX/war/artillery_breech",
    "event:/SFX/VFX/war/artillery_chemical",
    "event:/SFX/VFX/war/artillery_mobile",
    "event:/SFX/VFX/war/artillery/mobile/generic",
    "event:/SFX/VFX/war/artillery/siege/aerial_recon",
    "event:/SFX/VFX/war/artillery/siege/chemical",
    "event:/SFX/VFX/war/artillery/siege/generic",
    "event:/SFX/VFX/war/artillery/siege/machine_gunners",
    "event:/SFX/VFX/war/artillery/siege/shrapnel",
    "event:/SFX/VFX/war/artillery/siege/siege_artillery",
    "event:/SFX/VFX/war/campfire",
    "event:/SFX/VFX/war/infantry/irregular/generic",
    "event:/SFX/VFX/war/infantry/irregular/machine_gunners",
    "event:/SFX/VFX/war/infantry/line/cannon_artillery",
    "event:/SFX/VFX/war/infantry/line/flamethrower_company",
    "event:/SFX/VFX/war/infantry/line/generic",
    "event:/SFX/VFX/war/infantry/line/machine_gunners",
    "event:/SFX/VFX/war/infantry/line/skirmish",
    "event:/SFX/VFX/war/infantry/trench/aerial_recon",
    "event:/SFX/VFX/war/infantry/trench/generic",
    "event:/SFX/VFX/war/infantry/trench/machine_gunners",
    "event:/SFX/VFX/war/infantry/trench/squad",
    "event:/SFX/VFX/war/musket",
    "event:/SFX/VFX/war/rifle",
    "event:/SFX/VFX/war/rifle_bolt",
    "event:/SFX/VFX/war/rifle_repeating",
    "event:/SFX/VFX/war/ships/battleship",
    "event:/SFX/VFX/war/ships/ironclad",
    "event:/SFX/VFX/war/ships/ship_of_the_line",
    "event:/SFX/VFX/war/zone_center",
    "event:/SFX/VFX/war/zone_side",
    "event:/SFX/VFX/war/zone_snapshot_mute_2Damb",
    "event:/SFX/VFX/waterfall",
    "event:/SFX/VFX/whale_exhale",
];
