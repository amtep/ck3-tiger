//! Stores everything known about the game and mod being validated.
//!
//! References to [`Everything`] are passed down through nearly all of the validation logic, so
//! that individual functions can access all the defined game items.

use std::borrow::Cow;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
#[cfg(any(feature = "ck3", feature = "vic3"))]
use std::sync::RwLock;

use anyhow::Result;
use rayon::{scope, Scope};
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::block::Block;
#[cfg(any(feature = "ck3", feature = "vic3"))]
use crate::block::BV;
#[cfg(feature = "ck3")]
use crate::ck3::data::{
    characters::Characters,
    climate::Climate,
    doctrines::Doctrines,
    gameconcepts::GameConcepts,
    interaction_cats::CharacterInteractionCategories,
    maa::MenAtArmsTypes,
    prov_history::ProvinceHistories,
    prov_terrain::{ProvinceProperties, ProvinceTerrains},
    provinces::Ck3Provinces,
    title_history::TitleHistories,
    titles::Titles,
    traits::Traits,
    wars::Wars,
};
#[cfg(feature = "ck3")]
use crate::ck3::tables::misc::*;
use crate::config_load::{check_for_legacy_ignore, load_filter};
use crate::context::ScopeContext;
#[cfg(any(feature = "ck3", feature = "vic3"))]
use crate::data::data_binding::DataBindings;
use crate::data::{
    assets::Assets,
    defines::Defines,
    events::Events,
    gui::Gui,
    localization::Localization,
    music::Musics,
    on_actions::OnActions,
    scripted_effects::{Effect, Effects},
    scripted_triggers::{Trigger, Triggers},
};
#[cfg(feature = "jomini")]
use crate::data::{
    coa::Coas, script_values::ScriptValues, scripted_lists::ScriptedLists,
    scripted_modifiers::ScriptedModifiers,
};
use crate::db::{Db, DbKind};
use crate::dds::DdsFiles;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::game::Game;
#[cfg(any(feature = "ck3", feature = "vic3"))]
use crate::helpers::TigerHashSet;
#[cfg(feature = "hoi4")]
use crate::hoi4::data::provinces::Hoi4Provinces;
#[cfg(feature = "imperator")]
use crate::imperator::data::{decisions::Decisions, provinces::ImperatorProvinces};
#[cfg(feature = "imperator")]
use crate::imperator::tables::misc::*;
use crate::item::{Item, ItemLoader};
use crate::lowercase::Lowercase;
use crate::macros::MACRO_MAP;
#[cfg(feature = "vic3")]
use crate::parse::json::parse_json_file;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
#[cfg(any(feature = "ck3", feature = "vic3"))]
use crate::report::err;
use crate::report::{report, set_output_style, ErrorKey, OutputStyle, Severity};
use crate::rivers::Rivers;
use crate::token::{Loc, Token};
#[cfg(feature = "vic3")]
use crate::vic3::data::{
    buy_packages::BuyPackage, history::History, provinces::Vic3Provinces,
    strategic_regions::StrategicRegion, terrain::TerrainMask,
};
#[cfg(feature = "vic3")]
use crate::vic3::tables::misc::*;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
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

    /// The global parser state, carrying information between files.
    /// Currently only used by the pdxfile parser, to handle the `reader_export` directory,
    /// which is specially processed before all other files.
    pub parser: ParserMemory,

    /// A cache of define values (from common/defines) that are missing and that have already been
    /// warned about as missing. This is to avoid duplicate warnings.
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    warned_defines: RwLock<TigerHashSet<String>>,

    /// Tracks all the files (vanilla and mods) that are relevant to the current validation.
    pub(crate) fileset: Fileset,

    /// Tracks specifically the .dds files, and their formats and sizes.
    pub(crate) dds: DdsFiles,

    /// A general database of item types. Most items go here. The ones that need special handling
    /// go in the separate databases listed below.
    pub(crate) database: Db,

    pub(crate) localization: Localization,

    #[cfg(feature = "jomini")]
    pub(crate) scripted_lists: ScriptedLists,

    pub(crate) defines: Defines,

    pub(crate) events: Events,
    #[cfg(feature = "imperator")]
    pub(crate) decisions_imperator: Decisions,

    #[cfg(feature = "jomini")]
    pub(crate) scripted_modifiers: ScriptedModifiers,
    pub(crate) on_actions: OnActions,

    #[cfg(feature = "ck3")]
    pub(crate) interaction_cats: CharacterInteractionCategories,

    #[cfg(feature = "ck3")]
    pub(crate) provinces_ck3: Ck3Provinces,
    #[cfg(feature = "vic3")]
    pub(crate) provinces_vic3: Vic3Provinces,
    #[cfg(feature = "imperator")]
    pub(crate) provinces_imperator: ImperatorProvinces,
    #[cfg(feature = "hoi4")]
    pub(crate) provinces_hoi4: Hoi4Provinces,

    #[cfg(feature = "ck3")]
    pub(crate) province_histories: ProvinceHistories,
    #[cfg(feature = "ck3")]
    pub(crate) province_properties: ProvinceProperties,
    #[cfg(feature = "ck3")]
    pub(crate) province_terrains: ProvinceTerrains,

    #[cfg(feature = "ck3")]
    pub(crate) gameconcepts: GameConcepts,

    #[cfg(feature = "ck3")]
    pub(crate) titles: Titles,

    #[cfg(feature = "ck3")]
    pub(crate) characters: Characters,

    #[cfg(feature = "jomini")]
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
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    pub(crate) data_bindings: DataBindings,

    pub(crate) assets: Assets,
    pub(crate) music: Musics,

    #[cfg(feature = "jomini")]
    pub(crate) coas: Coas,

    #[cfg(feature = "vic3")]
    pub(crate) history: History,

    #[cfg(feature = "ck3")]
    pub(crate) wars: Wars,
}

impl Everything {
    /// Create a new `Everything` instance, ready for validating a mod.
    ///
    /// `vanilla_dir` is the path to the base game files. If it's `None`, then no vanilla files
    /// will be loaded. This will seriously affect validation, but it's ok if you just want to load
    /// and examine the mod files.
    ///
    /// `mod_root` is the path to the mod files. The config file will also be looked for there.
    ///
    /// `replace_paths` is from the similarly named field in the `.mod` file.
    pub fn new(
        config_filepath: Option<&Path>,
        vanilla_dir: Option<&Path>,
        mod_root: &Path,
        replace_paths: Vec<PathBuf>,
    ) -> Result<Self> {
        let mut fileset = Fileset::new(vanilla_dir, mod_root.to_path_buf(), replace_paths);

        let config_file_name = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => "ck3-tiger.conf",
            #[cfg(feature = "vic3")]
            Game::Vic3 => "vic3-tiger.conf",
            #[cfg(feature = "imperator")]
            Game::Imperator => "imperator-tiger.conf",
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => "hoi4-tiger.conf",
        };

        let config_file = match config_filepath {
            Some(path) => path.to_path_buf(),
            None => mod_root.join(config_file_name),
        };

        let config = if config_file.is_file() {
            Self::read_config(config_file_name, &config_file)
                .ok_or(FilesError::ConfigUnreadable { path: config_file })?
        } else {
            Block::new(Loc::for_file(config_file.clone(), FileKind::Mod, config_file.clone()))
        };

        fileset.config(config.clone())?;

        fileset.scan_all()?;
        fileset.finalize();

        Ok(Everything {
            parser: ParserMemory::default(),
            fileset,
            dds: DdsFiles::default(),
            config,
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            warned_defines: RwLock::new(TigerHashSet::default()),
            database: Db::default(),
            localization: Localization::default(),
            #[cfg(feature = "jomini")]
            scripted_lists: ScriptedLists::default(),
            defines: Defines::default(),
            events: Events::default(),
            #[cfg(feature = "imperator")]
            decisions_imperator: Decisions::default(),
            #[cfg(feature = "jomini")]
            scripted_modifiers: ScriptedModifiers::default(),
            on_actions: OnActions::default(),
            #[cfg(feature = "ck3")]
            interaction_cats: CharacterInteractionCategories::default(),
            #[cfg(feature = "ck3")]
            provinces_ck3: Ck3Provinces::default(),
            #[cfg(feature = "vic3")]
            provinces_vic3: Vic3Provinces::default(),
            #[cfg(feature = "imperator")]
            provinces_imperator: ImperatorProvinces::default(),
            #[cfg(feature = "hoi4")]
            provinces_hoi4: Hoi4Provinces::default(),
            #[cfg(feature = "ck3")]
            province_histories: ProvinceHistories::default(),
            #[cfg(feature = "ck3")]
            province_properties: ProvinceProperties::default(),
            #[cfg(feature = "ck3")]
            province_terrains: ProvinceTerrains::default(),
            #[cfg(feature = "ck3")]
            gameconcepts: GameConcepts::default(),
            #[cfg(feature = "ck3")]
            titles: Titles::default(),
            #[cfg(feature = "ck3")]
            characters: Characters::default(),
            #[cfg(feature = "jomini")]
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
            #[cfg(any(feature = "ck3", feature = "vic3"))]
            data_bindings: DataBindings::default(),
            assets: Assets::default(),
            music: Musics::default(),
            #[cfg(feature = "jomini")]
            coas: Coas::default(),
            #[cfg(feature = "vic3")]
            history: History::default(),
            #[cfg(feature = "ck3")]
            wars: Wars::default(),
        })
    }

    fn read_config(name: &str, path: &Path) -> Option<Block> {
        let entry = FileEntry::new(PathBuf::from(name), FileKind::Mod, path.to_path_buf());
        PdxFile::read_optional_bom(&entry, &ParserMemory::default())
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
            None => Cow::Owned(Block::new(self.config.loc)),
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

    pub fn load_output_settings(&self, default_colors: bool) {
        set_output_style(self.load_output_styles(default_colors));
    }

    #[cfg(feature = "vic3")]
    fn load_json<F>(&mut self, itype: Item, add_json: F)
    where
        F: Fn(&mut Db, Block) + Sync + Send,
    {
        for block in self.fileset.filter_map_under(&PathBuf::from(itype.path()), |entry| {
            if entry.filename().to_string_lossy().ends_with(".json") {
                parse_json_file(entry)
            } else {
                None
            }
        }) {
            add_json(&mut self.database, block);
        }
    }

    #[cfg(feature = "ck3")]
    fn load_reader_export(&mut self) {
        let path = PathBuf::from("reader_export");
        for entry in self.fileset.get_files_under(&path) {
            if entry.filename().to_string_lossy().ends_with(".txt") {
                PdxFile::reader_export(entry, &mut self.parser.pdxfile);
            }
        }
    }

    fn load_pdx_files(&mut self, loader: &ItemLoader) {
        let path = PathBuf::from(loader.itype().path());
        let recursive = loader.recursive();
        let expect_count = path.components().count() + 1;
        for mut block in self.fileset.filter_map_under(&path, |entry| {
            // It's <= expect_count because some loader paths are files not directories
            if (recursive || entry.path().components().count() <= expect_count)
                && entry.filename().to_string_lossy().ends_with(loader.extension())
            {
                PdxFile::read_encoded(entry, loader.encoding(), &self.parser)
            } else {
                None
            }
        }) {
            if loader.whole_file() {
                let fname = block.loc.filename();
                // unwrap is safe here because of the ends_with check above.
                let key = fname.strip_suffix(loader.extension()).unwrap();
                let key = Token::new(key, block.loc);
                (loader.adder())(&mut self.database, key, block);
            } else {
                for (key, block) in block.drain_definitions_warn() {
                    (loader.adder())(&mut self.database, key, block);
                }
            }
        }
    }

    fn load_all_normal_pdx_files(&mut self) {
        for loader in inventory::iter::<ItemLoader> {
            if loader.for_game(Game::game()) {
                self.load_pdx_files(loader);
            }
        }
    }

    fn load_all_generic(&mut self) {
        scope(|s| {
            s.spawn(|_| self.fileset.handle(&mut self.dds, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.events, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.localization, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.defines, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.triggers, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.effects, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.assets, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.gui, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.on_actions, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.music, &self.parser));
        });

        self.load_all_normal_pdx_files();
    }

    #[cfg(feature = "ck3")]
    fn load_all_ck3(&mut self) {
        scope(|s| {
            s.spawn(|_| self.fileset.handle(&mut self.interaction_cats, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.province_histories, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.province_properties, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.province_terrains, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.gameconcepts, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.titles, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.characters, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.traits, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.title_history, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.doctrines, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.menatarmstypes, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.data_bindings, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.provinces_ck3, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.scripted_lists, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.wars, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.coas, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.scripted_modifiers, &self.parser));
            s.spawn(|_| self.fileset.handle(&mut self.script_values, &self.parser));
        });
        crate::ck3::data::buildings::Building::finalize(&mut self.database);
    }

    #[cfg(feature = "vic3")]
    fn load_all_vic3(&mut self) {
        self.fileset.handle(&mut self.history, &self.parser);
        self.fileset.handle(&mut self.provinces_vic3, &self.parser);
        self.fileset.handle(&mut self.data_bindings, &self.parser);
        self.fileset.handle(&mut self.coas, &self.parser);
        self.fileset.handle(&mut self.scripted_lists, &self.parser);
        self.fileset.handle(&mut self.scripted_modifiers, &self.parser);
        self.fileset.handle(&mut self.script_values, &self.parser);
        self.load_json(Item::TerrainMask, TerrainMask::add_json);
    }

    #[cfg(feature = "imperator")]
    fn load_all_imperator(&mut self) {
        self.fileset.handle(&mut self.decisions_imperator, &self.parser);
        self.fileset.handle(&mut self.provinces_imperator, &self.parser);
        self.fileset.handle(&mut self.coas, &self.parser);
        self.fileset.handle(&mut self.scripted_lists, &self.parser);
        self.fileset.handle(&mut self.scripted_modifiers, &self.parser);
        self.fileset.handle(&mut self.script_values, &self.parser);
    }

    #[cfg(feature = "hoi4")]
    fn load_all_hoi4(&mut self) {}

    pub fn load_all(&mut self) {
        #[cfg(feature = "ck3")]
        self.load_reader_export();
        self.load_all_generic();
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => self.load_all_ck3(),
            #[cfg(feature = "vic3")]
            Game::Vic3 => self.load_all_vic3(),
            #[cfg(feature = "imperator")]
            Game::Imperator => self.load_all_imperator(),
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => self.load_all_hoi4(),
        }
        self.database.add_subitems();
    }

    fn validate_all_generic<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.fileset.validate(self));
        s.spawn(|_| self.defines.validate(self));
        s.spawn(|_| self.triggers.validate(self));
        s.spawn(|_| self.effects.validate(self));
        s.spawn(|_| self.events.validate(self));
        s.spawn(|_| self.assets.validate(self));
        s.spawn(|_| self.gui.validate(self));
        s.spawn(|_| self.on_actions.validate(self));
        s.spawn(|_| self.music.validate(self));
    }

    #[cfg(feature = "ck3")]
    fn validate_all_ck3<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.interaction_cats.validate(self));
        s.spawn(|_| self.province_histories.validate(self));
        s.spawn(|_| self.province_properties.validate(self));
        s.spawn(|_| self.province_terrains.validate(self));
        s.spawn(|_| self.gameconcepts.validate(self));
        s.spawn(|_| self.titles.validate(self));
        s.spawn(|_| self.characters.validate(self));
        s.spawn(|_| self.traits.validate(self));
        s.spawn(|_| self.title_history.validate(self));
        s.spawn(|_| self.doctrines.validate(self));
        s.spawn(|_| self.menatarmstypes.validate(self));
        s.spawn(|_| self.data_bindings.validate(self));
        s.spawn(|_| self.provinces_ck3.validate(self));
        s.spawn(|_| self.wars.validate(self));
        s.spawn(|_| self.coas.validate(self));
        s.spawn(|_| self.scripted_lists.validate(self));
        s.spawn(|_| self.scripted_modifiers.validate(self));
        s.spawn(|_| self.script_values.validate(self));
        s.spawn(|_| Climate::validate_all(&self.database, self));
    }

    #[cfg(feature = "vic3")]
    fn validate_all_vic3<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.history.validate(self));
        s.spawn(|_| self.provinces_vic3.validate(self));
        s.spawn(|_| self.data_bindings.validate(self));
        s.spawn(|_| self.coas.validate(self));
        s.spawn(|_| self.scripted_lists.validate(self));
        s.spawn(|_| self.scripted_modifiers.validate(self));
        s.spawn(|_| self.script_values.validate(self));
        s.spawn(|_| StrategicRegion::crosscheck(self));
        s.spawn(|_| BuyPackage::crosscheck(self));
    }

    #[cfg(feature = "imperator")]
    fn validate_all_imperator<'a>(&'a self, s: &Scope<'a>) {
        s.spawn(|_| self.decisions_imperator.validate(self));
        s.spawn(|_| self.provinces_imperator.validate(self));
        s.spawn(|_| self.coas.validate(self));
        s.spawn(|_| self.scripted_lists.validate(self));
        s.spawn(|_| self.scripted_modifiers.validate(self));
        s.spawn(|_| self.script_values.validate(self));
    }

    #[cfg(feature = "hoi4")]
    fn validate_all_hoi4<'a>(&'a self, _s: &Scope<'a>) {}

    pub fn validate_all(&self) {
        scope(|s| {
            self.validate_all_generic(s);
            match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.validate_all_ck3(s),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.validate_all_vic3(s),
                #[cfg(feature = "imperator")]
                Game::Imperator => self.validate_all_imperator(s),
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => self.validate_all_hoi4(s),
            }
        });
        self.database.validate(self);

        self.localization.validate_pass2(self);
    }

    pub fn check_rivers(&mut self) {
        if !Game::is_hoi4() {
            // TODO HOI4
            let mut rivers = Rivers::default();
            self.fileset.handle(&mut rivers, &self.parser);
            rivers.validate(self);
        }
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

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub(crate) fn item_lc_has_property(
        &self,
        itype: Item,
        key: &Lowercase,
        property: &str,
    ) -> bool {
        self.database.lc_has_property(itype, key, property, self)
    }

    #[cfg(feature = "ck3")]
    fn item_exists_ck3(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::ActivityState => ACTIVITY_STATES.contains(&key),
            Item::ArtifactHistory => ARTIFACT_HISTORY.contains(&key),
            Item::ArtifactRarity => ARTIFACT_RARITIES.contains(&&*key.to_ascii_lowercase()),
            Item::Character => self.characters.exists(key),
            Item::CharacterInteractionCategory => self.interaction_cats.exists(key),
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::DangerType => DANGER_TYPES.contains(&key),
            Item::DlcFeature => DLC_FEATURES_CK3.contains(&key),
            Item::Doctrine => self.doctrines.exists(key),
            Item::DoctrineCategory => self.doctrines.category_exists(key),
            Item::DoctrineParameter => self.doctrines.parameter_exists(key),
            Item::GameConcept => self.gameconcepts.exists(key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::GeneticConstraint => self.traits.constraint_exists(key),
            Item::MenAtArms => self.menatarmstypes.exists(key),
            Item::MenAtArmsBase => self.menatarmstypes.base_exists(key),
            Item::PrisonType => PRISON_TYPES.contains(&key),
            Item::Province => self.provinces_ck3.exists(key),
            Item::RewardItem => REWARD_ITEMS.contains(&key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptValue => self.script_values.exists(key),
            Item::Sexuality => SEXUALITIES.contains(&key),
            Item::Skill => SKILLS.contains(&key),
            Item::Sound => self.valid_sound(key),
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
    fn item_exists_vic3(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Approval => APPROVALS.contains(&key),
            Item::Attitude => ATTITUDES.contains(&&*key.to_lowercase()),
            Item::CharacterRole => CHARACTER_ROLES.contains(&key),
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::CountryTier => COUNTRY_TIERS.contains(&key),
            Item::DlcFeature => DLC_FEATURES_VIC3.contains(&key),
            Item::EventCategory => EVENT_CATEGORIES.contains(&key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::InfamyThreshold => INFAMY_THRESHOLDS.contains(&key),
            Item::Level => LEVELS.contains(&key),
            Item::RelationsThreshold => RELATIONS.contains(&key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptValue => self.script_values.exists(key),
            Item::SecretGoal => SECRET_GOALS.contains(&key),
            Item::Sound => self.valid_sound(key),
            Item::Strata => STRATA.contains(&key),
            Item::TerrainKey => TERRAIN_KEYS.contains(&key),
            Item::TransferOfPower => TRANSFER_OF_POWER.contains(&key),
            Item::Wargoal => WARGOALS.contains(&key),
            _ => self.database.exists(itype, key),
        }
    }

    #[cfg(feature = "imperator")]
    fn item_exists_imperator(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::DlcName => DLC_NAME_IMPERATOR.contains(&key),
            Item::Decision => self.decisions_imperator.exists(key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::Province => self.provinces_imperator.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptValue => self.script_values.exists(key),
            Item::Sound => self.valid_sound(key),
            _ => self.database.exists(itype, key),
        }
    }

    #[cfg(feature = "hoi4")]
    fn item_exists_hoi4(&self, itype: Item, key: &str) -> bool {
        self.database.exists(itype, key)
    }

    pub(crate) fn item_exists(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::Asset => self.assets.asset_exists(key),
            Item::BlendShape => self.assets.blend_shape_exists(key),
            Item::Define => self.defines.exists(key),
            Item::Entity => self.assets.entity_exists(key),
            Item::Entry => self.fileset.entry_exists(key),
            Item::Event => self.events.exists(key),
            Item::EventNamespace => self.events.namespace_exists(key),
            Item::File => self.fileset.exists(key),
            Item::GuiLayer => self.gui.layer_exists(key),
            Item::GuiTemplate => self.gui.template_exists(key),
            Item::GuiType => self.gui.type_exists(&Lowercase::new(key)),
            Item::Localization => self.localization.exists(key),
            Item::Music => self.music.exists(key),
            Item::OnAction => self.on_actions.exists(key),
            Item::Pdxmesh => self.assets.mesh_exists(key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::TextFormat => self.gui.textformat_exists(key),
            Item::TextIcon => self.gui.texticon_exists(key),
            Item::TextureFile => self.assets.texture_exists(key),
            Item::WidgetName => self.gui.name_exists(key),
            Item::Directory | Item::Shortcut => true, // TODO
            _ => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.item_exists_ck3(itype, key),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.item_exists_vic3(itype, key),
                #[cfg(feature = "imperator")]
                Game::Imperator => self.item_exists_imperator(itype, key),
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => self.item_exists_hoi4(itype, key),
            },
        }
    }

    /// Return true iff the item `key` is found with a case insensitive match.
    /// This function is **incomplete**. It only contains the item types for which case insensitive
    /// matches are needed; this is currently the ones used in `src/ck3/tables/modif.rs`.
    #[cfg(feature = "ck3")]
    fn item_exists_lc_ck3(&self, itype: Item, key: &Lowercase) -> bool {
        match itype {
            Item::MenAtArmsBase => self.menatarmstypes.base_exists_lc(key),
            Item::Trait => self.traits.exists_lc(key),
            Item::TraitTrack => self.traits.track_exists_lc(key),
            _ => self.database.exists_lc(itype, key),
        }
    }

    /// Return true iff the item `key` is found with a case insensitive match.
    /// This function is **incomplete**. It only contains the item types for which case insensitive
    /// matches are needed; this is currently the ones used in `src/vic3/tables/modif.rs`.
    #[cfg(feature = "vic3")]
    fn item_exists_lc_vic3(&self, itype: Item, key: &Lowercase) -> bool {
        match itype {
            Item::TerrainKey => TERRAIN_KEYS.contains(&key.as_str()),
            _ => self.database.exists_lc(itype, key),
        }
    }

    /// Return true iff the item `key` is found with a case insensitive match.
    /// This function is **incomplete**. It only contains the item types for which case insensitive
    /// matches are needed; this is currently the ones used in `src/imperator/tables/modif.rs`.
    #[cfg(feature = "imperator")]
    fn item_exists_lc_imperator(&self, itype: Item, key: &Lowercase) -> bool {
        #[allow(clippy::match_single_binding)]
        match itype {
            _ => self.database.exists_lc(itype, key),
        }
    }

    /// Return true iff the item `key` is found with a case insensitive match.
    /// This function is **incomplete**. It only contains the item types for which case insensitive
    /// matches are needed; this is currently the ones used in `src/hoi4/tables/modif.rs`.
    #[cfg(feature = "hoi4")]
    fn item_exists_lc_hoi4(&self, itype: Item, key: &Lowercase) -> bool {
        #[allow(clippy::match_single_binding)]
        match itype {
            _ => self.database.exists_lc(itype, key),
        }
    }
    /// Return true iff the item `key` is found with a case insensitive match.
    /// This function is **incomplete**. It only contains the item types for which case insensitive
    /// matches are needed; this is currently the ones used in modif lookups.
    pub(crate) fn item_exists_lc(&self, itype: Item, key: &Lowercase) -> bool {
        #[allow(clippy::match_single_binding)]
        match itype {
            _ => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.item_exists_lc_ck3(itype, key),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.item_exists_lc_vic3(itype, key),
                #[cfg(feature = "imperator")]
                Game::Imperator => self.item_exists_lc_imperator(itype, key),
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => self.item_exists_lc_hoi4(itype, key),
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
            Item::Entry => self.fileset.verify_entry_exists(key, token, max_sev),
            Item::File => self.fileset.verify_exists_implied(key, token, max_sev),
            Item::Localization => self.localization.verify_exists_implied(key, token, max_sev),
            Item::Music => self.music.verify_exists_implied(key, token, max_sev),
            Item::Province => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.provinces_ck3.verify_exists_implied(key, token, max_sev),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.provinces_vic3.verify_exists_implied(key, token, max_sev),
                #[cfg(feature = "imperator")]
                Game::Imperator => {
                    self.provinces_imperator.verify_exists_implied(key, token, max_sev);
                }
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => {
                    self.provinces_hoi4.verify_exists_implied(key, token, max_sev);
                }
            },
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

    #[cfg(feature = "ck3")]
    pub(crate) fn verify_icon(&self, define: &str, token: &Token, suffix: &str) {
        if let Some(icon_path) = self.get_defined_string_warn(token, define) {
            let pathname = format!("{icon_path}/{token}{suffix}");
            // It's `Severity::Warning` because a missing icon is only a UI issue.
            self.verify_exists_implied_max_sev(Item::File, &pathname, token, Severity::Warning);
        }
    }

    #[cfg(feature = "ck3")]
    pub(crate) fn mark_used_icon(&self, define: &str, token: &Token, suffix: &str) {
        if let Some(icon_path) = self.get_defined_string_warn(token, define) {
            let pathname = format!("{icon_path}/{token}{suffix}");
            self.fileset.mark_used(&pathname);
        }
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

    #[allow(dead_code)]
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
            if let Some(trigger) = self.events.get_trigger(key) {
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
            if let Some(effect) = self.events.get_effect(key) {
                return Some(effect);
            }
            return None;
        }
        self.effects.get(key.as_str())
    }

    #[cfg(feature = "ck3")] // happens not to be used by vic3
    pub(crate) fn get_defined_string(&self, key: &str) -> Option<&Token> {
        self.defines.get_bv(key).and_then(BV::get_value)
    }

    #[cfg(any(feature = "ck3", feature = "vic3"))]
    pub(crate) fn get_defined_array(&self, key: &str) -> Option<&Block> {
        self.defines.get_bv(key).and_then(BV::get_block)
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

    #[allow(clippy::missing_panics_doc)] // only panics on poisoned mutex
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    pub(crate) fn get_defined_array_warn(&self, token: &Token, key: &str) -> Option<&Block> {
        let result = self.get_defined_array(key);
        let mut cache = self.warned_defines.write().unwrap();
        if result.is_none() && !cache.contains(key) {
            let msg = format!("{key} not defined in common/defines/");
            err(ErrorKey::MissingItem).msg(msg).loc(token).push();
            cache.insert(key.to_string());
        }
        result
    }

    #[cfg(feature = "ck3")]
    pub fn iter_keys_ck3<'a>(&'a self, itype: Item) -> Box<dyn Iterator<Item = &'a Token> + 'a> {
        match itype {
            Item::Coa => Box::new(self.coas.iter_keys()),
            Item::CoaTemplate => Box::new(self.coas.iter_template_keys()),
            Item::Character => Box::new(self.characters.iter_keys()),
            Item::CharacterInteractionCategory => Box::new(self.interaction_cats.iter_keys()),
            Item::Doctrine => Box::new(self.doctrines.iter_keys()),
            Item::DoctrineCategory => Box::new(self.doctrines.iter_category_keys()),
            Item::DoctrineParameter => Box::new(self.doctrines.iter_parameter_keys()),
            Item::GameConcept => Box::new(self.gameconcepts.iter_keys()),
            Item::GeneAttribute => Box::new(self.assets.iter_attribute_keys()),
            Item::GeneticConstraint => Box::new(self.traits.iter_constraint_keys()),
            Item::MenAtArms => Box::new(self.menatarmstypes.iter_keys()),
            Item::MenAtArmsBase => Box::new(self.menatarmstypes.iter_base_keys()),
            Item::Province => Box::new(self.provinces_ck3.iter_keys()),
            Item::ScriptedList => Box::new(self.scripted_lists.iter_keys()),
            Item::ScriptedModifier => Box::new(self.scripted_modifiers.iter_keys()),
            Item::ScriptValue => Box::new(self.script_values.iter_keys()),
            Item::Title => Box::new(self.titles.iter_keys()),
            Item::TitleHistory => Box::new(self.title_history.iter_keys()),
            Item::Trait => Box::new(self.traits.iter_keys()),
            Item::TraitFlag => Box::new(self.traits.iter_flag_keys()),
            Item::TraitTrack => Box::new(self.traits.iter_track_keys()),
            _ => Box::new(self.database.iter_keys(itype)),
        }
    }

    #[cfg(feature = "vic3")]
    fn iter_keys_vic3<'a>(&'a self, itype: Item) -> Box<dyn Iterator<Item = &'a Token> + 'a> {
        match itype {
            Item::Coa => Box::new(self.coas.iter_keys()),
            Item::CoaTemplate => Box::new(self.coas.iter_template_keys()),
            Item::GeneAttribute => Box::new(self.assets.iter_attribute_keys()),
            Item::ScriptedList => Box::new(self.scripted_lists.iter_keys()),
            Item::ScriptedModifier => Box::new(self.scripted_modifiers.iter_keys()),
            Item::ScriptValue => Box::new(self.script_values.iter_keys()),
            _ => Box::new(self.database.iter_keys(itype)),
        }
    }

    #[cfg(feature = "imperator")]
    fn iter_keys_imperator<'a>(&'a self, itype: Item) -> Box<dyn Iterator<Item = &'a Token> + 'a> {
        match itype {
            Item::Coa => Box::new(self.coas.iter_keys()),
            Item::CoaTemplate => Box::new(self.coas.iter_template_keys()),
            Item::Decision => Box::new(self.decisions_imperator.iter_keys()),
            Item::GeneAttribute => Box::new(self.assets.iter_attribute_keys()),
            Item::Province => Box::new(self.provinces_imperator.iter_keys()),
            Item::ScriptedList => Box::new(self.scripted_lists.iter_keys()),
            Item::ScriptedModifier => Box::new(self.scripted_modifiers.iter_keys()),
            Item::ScriptValue => Box::new(self.script_values.iter_keys()),
            _ => Box::new(self.database.iter_keys(itype)),
        }
    }

    #[cfg(feature = "hoi4")]
    fn iter_keys_hoi4<'a>(&'a self, itype: Item) -> Box<dyn Iterator<Item = &'a Token> + 'a> {
        Box::new(self.database.iter_keys(itype))
    }

    pub fn iter_keys<'a>(&'a self, itype: Item) -> Box<dyn Iterator<Item = &'a Token> + 'a> {
        match itype {
            Item::Asset => Box::new(self.assets.iter_asset_keys()),
            Item::BlendShape => Box::new(self.assets.iter_blend_shape_keys()),
            Item::Define => Box::new(self.defines.iter_keys()),
            Item::Entity => Box::new(self.assets.iter_entity_keys()),
            Item::Event => Box::new(self.events.iter_keys()),
            Item::EventNamespace => Box::new(self.events.iter_namespace_keys()),
            Item::File => Box::new(self.fileset.iter_keys()),
            Item::GuiLayer => Box::new(self.gui.iter_layer_keys()),
            Item::GuiTemplate => Box::new(self.gui.iter_template_keys()),
            Item::GuiType => Box::new(self.gui.iter_type_keys()),
            Item::Localization => Box::new(self.localization.iter_keys()),
            Item::Music => Box::new(self.music.iter_keys()),
            Item::OnAction => Box::new(self.on_actions.iter_keys()),
            Item::Pdxmesh => Box::new(self.assets.iter_mesh_keys()),
            Item::ScriptedEffect => Box::new(self.effects.iter_keys()),
            Item::ScriptedTrigger => Box::new(self.triggers.iter_keys()),
            Item::TextFormat => Box::new(self.gui.iter_textformat_keys()),
            Item::TextIcon => Box::new(self.gui.iter_texticon_keys()),
            Item::TextureFile => Box::new(self.assets.iter_texture_keys()),
            Item::WidgetName => Box::new(self.gui.iter_names()),
            _ => match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => self.iter_keys_ck3(itype),
                #[cfg(feature = "vic3")]
                Game::Vic3 => self.iter_keys_vic3(itype),
                #[cfg(feature = "imperator")]
                Game::Imperator => self.iter_keys_imperator(itype),
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => self.iter_keys_hoi4(itype),
            },
        }
    }

    fn valid_sound(&self, name: &str) -> bool {
        // TODO: verify that file:/ values work
        if let Some(filename) = name.strip_prefix("file:/") {
            self.fileset.exists(filename)
        } else {
            let sounds_set = match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => &crate::ck3::tables::sounds::SOUNDS_SET,
                #[cfg(feature = "vic3")]
                Game::Vic3 => &crate::vic3::tables::sounds::SOUNDS_SET,
                #[cfg(feature = "imperator")]
                Game::Imperator => &crate::imperator::tables::sounds::SOUNDS_SET,
                #[cfg(feature = "hoi4")]
                Game::Hoi4 => &crate::hoi4::tables::sounds::SOUNDS_SET,
            };
            sounds_set.contains(&Lowercase::new(name))
        }
    }

    /// Return true iff a script value of the given name is defined.
    pub(crate) fn script_value_exists(&self, name: &str) -> bool {
        if Game::is_jomini() {
            #[cfg(feature = "jomini")]
            return self.script_values.exists(name);
        }
        false
    }
}

impl Drop for Everything {
    fn drop(&mut self) {
        // For the sake of the benchmark code, restore MACRO_MAP to a clean slate
        MACRO_MAP.clear();
    }
}
