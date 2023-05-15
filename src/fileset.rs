use anyhow::Result;
use fnv::FnvHashSet;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use walkdir::WalkDir;

use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::Everything;
use crate::token::{Loc, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileKind {
    Vanilla,
    Mod,
}

impl Display for FileKind {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            FileKind::Vanilla => write!(fmt, "CK3"),
            FileKind::Mod => write!(fmt, "MOD"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileEntry {
    /// Pathname components below the mod directory or the vanilla game dir
    /// Must not be empty.
    path: PathBuf,
    /// Whether it's a vanilla or mod file
    kind: FileKind,
}

impl FileEntry {
    pub fn new(path: PathBuf, kind: FileKind) -> Self {
        assert!(path.file_name().is_some());
        Self { path, kind }
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Convenience function
    /// Won't panic because `FileEntry` with empty filename is not allowed.
    #[allow(clippy::missing_panics_doc)]
    pub fn filename(&self) -> &OsStr {
        self.path.file_name().unwrap()
    }
}

impl Display for FileEntry {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.path.display())
    }
}

impl From<&FileEntry> for Loc {
    fn from(entry: &FileEntry) -> Self {
        Loc::for_file(Rc::new(entry.path().to_path_buf()), entry.kind)
    }
}

impl From<&FileEntry> for Token {
    fn from(entry: &FileEntry) -> Self {
        Token::from(Loc::from(entry))
    }
}

/// A trait for a submodule that can process files.
pub trait FileHandler {
    /// The `FileHandler` can read settings it needs from the ck3-tiger config.
    fn config(&mut self, _config: &Block) {}

    /// Which files this handler is interested in.
    /// This is a directory prefix of files it wants to handle,
    /// relative to the mod or vanilla root.
    fn subpath(&self) -> PathBuf;

    /// This is called for each matching file in turn, in lexical order.
    /// That's the order in which the CK3 game engine loads them too.
    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path);

    /// This is called after all files have been handled.
    /// The `FileHandler` can generate indexes, perform full-data checks, etc.
    fn finalize(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct Fileset {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,

    /// A list of directories that should not be read from vanilla.
    replace_paths: Vec<PathBuf>,

    /// The ck3-tiger config
    config: Option<Block>,

    /// The CK3 and mod files in arbitrary order (will be empty after `finalize`)
    files: Vec<FileEntry>,

    /// The CK3 and mod files in the order the game would load them
    ordered_files: Vec<FileEntry>,

    /// All filenames from ordered_files, for quick lookup
    filenames: FnvHashSet<PathBuf>,
}

impl Fileset {
    pub fn new(vanilla_root: PathBuf, mod_root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Fileset {
            vanilla_root,
            mod_root,
            replace_paths,
            config: None,
            files: Vec::new(),
            ordered_files: Vec::new(),
            filenames: FnvHashSet::default(),
        }
    }

    pub fn config(&mut self, config: Block) {
        self.config = Some(config);
    }

    pub fn scan(&mut self, path: &Path, kind: FileKind) -> Result<(), walkdir::Error> {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.depth() == 0 || !entry.file_type().is_file() {
                continue;
            }
            // unwrap is safe here because WalkDir gives us paths with this prefix.
            let inner_path = entry.path().strip_prefix(path).unwrap();
            if kind == FileKind::Vanilla && self.replace_paths.iter().any(|p| p == inner_path) {
                continue;
            }
            self.files
                .push(FileEntry::new(inner_path.to_path_buf(), kind));
        }
        Ok(())
    }

    pub fn finalize(&mut self) {
        // This places `Mod` entries after `Vanilla` entries
        self.files.sort();

        // When there are identical paths, only keep the last entry of them.
        for entry in self.files.drain(..) {
            if let Some(prev) = self.ordered_files.last_mut() {
                if entry.path == prev.path {
                    *prev = entry;
                } else {
                    self.ordered_files.push(entry);
                }
            } else {
                self.ordered_files.push(entry);
            }
        }

        for entry in &self.ordered_files {
            self.filenames.insert(entry.path.clone());
        }
    }

    pub fn get_files_under<'a>(&'a self, subpath: &'a Path) -> Files<'a> {
        Files {
            iter: self.ordered_files.iter(),
            subpath,
        }
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        match entry.kind {
            FileKind::Vanilla => self.vanilla_root.join(entry.path()),
            FileKind::Mod => self.mod_root.join(entry.path()),
        }
    }

    pub fn handle<H: FileHandler>(&self, handler: &mut H) {
        if let Some(config) = &self.config {
            handler.config(config);
        }
        let subpath = handler.subpath();
        for entry in self.get_files_under(&subpath) {
            handler.handle_file(entry, &self.fullpath(entry));
        }
        handler.finalize();
    }

    pub fn exists(&self, key: &str) -> bool {
        let filepath = PathBuf::from(key);
        self.filenames.contains(&filepath)
    }

    pub fn verify_exists(&self, file: &Token) {
        let filepath = PathBuf::from(file.as_str());
        if !self.filenames.contains(&filepath) {
            error(
                file,
                ErrorKey::MissingFile,
                "referenced file does not exist",
            );
        }
    }

    pub fn verify_exists_implied(&self, file: &str, t: &Token) {
        let filepath = PathBuf::from(file);
        if !self.filenames.contains(&filepath) {
            error(
                t,
                ErrorKey::MissingFile,
                &format!("file {file} does not exist"),
            );
        }
    }

    pub fn validate(&self, _data: &Everything) {
        // Check the files in directories in common/ to make sure they are in known directories
        let mut warned: Vec<&Path> = Vec::new();
        'outer: for entry in &self.ordered_files {
            if !entry.path.starts_with("common") || !entry.path.to_string_lossy().ends_with(".txt")
            {
                continue;
            }
            let dirname = entry.path.parent().unwrap();
            if warned.contains(&dirname) {
                continue;
            }
            // TODO: check if subdirectories are ok in the different common/ directories
            for valid in COMMON_DIRS {
                if entry.path.starts_with(valid) {
                    continue 'outer;
                }
            }
            if entry.path.starts_with("common/scripted_values") {
                let msg = "file should be in common/script_values/";
                error(entry, ErrorKey::Filename, msg);
            } else if entry.path.starts_with("common/on_actions") {
                let msg = "file should be in common/on_action/";
                error(entry, ErrorKey::Filename, msg);
            } else {
                let msg = "file in unexpected directory";
                error(entry, ErrorKey::Filename, msg);
            }
            warned.push(dirname);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Files<'a> {
    iter: std::slice::Iter<'a, FileEntry>,
    subpath: &'a Path,
}

impl<'a> Iterator for Files<'a> {
    type Item = &'a FileEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .by_ref()
            .find(|&entry| entry.path.starts_with(self.subpath))
    }
}

/// LAST UPDATED VERSION 1.9.0.2
const COMMON_DIRS: &[&str] = &[
    "common/accolade_icons",
    "common/accolade_names",
    "common/accolade_types",
    "common/achievement_groups.txt", // exception for this file
    "common/achievements",
    "common/activities/activity_locales",
    "common/activities/activity_types",
    "common/activities/guest_invite_rules",
    "common/activities/intents",
    "common/activities/pulse_actions",
    "common/ai_goaltypes",
    "common/ai_war_stances",
    "common/artifacts/blueprints",
    "common/artifacts/feature_groups",
    "common/artifacts/features",
    "common/artifacts/slots",
    "common/artifacts/templates",
    "common/artifacts/types",
    "common/artifacts/visuals",
    "common/bookmark_portraits",
    "common/bookmarks/bookmarks",
    "common/bookmarks/groups",
    "common/buildings",
    "common/casus_belli_groups",
    "common/casus_belli_types",
    "common/character_backgrounds",
    "common/character_interaction_categories",
    "common/character_interactions",
    "common/character_memory_types",
    "common/coat_of_arms/coat_of_arms",
    "common/coat_of_arms/dynamic_definitions",
    "common/coat_of_arms/options",
    "common/coat_of_arms/template_lists",
    "common/combat_effects",
    "common/combat_phase_events",
    "common/console_groups",
    "common/council_positions",
    "common/council_tasks",
    "common/court_amenities",
    "common/courtier_guest_management",
    "common/court_positions/categories",
    "common/court_positions/types",
    "common/court_types",
    "common/culture/aesthetics_bundles",
    "common/culture/creation_names",
    "common/culture/cultures",
    "common/culture/eras",
    "common/culture/innovations",
    "common/culture/name_equivalency",
    "common/culture/name_lists",
    "common/culture/pillars",
    "common/culture/traditions",
    "common/customizable_localization",
    "common/deathreasons",
    "common/decisions",
    "common/defines",
    "common/diarchies/diarchy_mandates",
    "common/diarchies/diarchy_types",
    "common/dna_data",
    "common/dynasties",
    "common/dynasty_house_motto_inserts",
    "common/dynasty_house_mottos",
    "common/dynasty_houses",
    "common/dynasty_legacies",
    "common/dynasty_perks",
    "common/effect_localization",
    "common/ethnicities",
    "common/event_backgrounds",
    "common/event_themes",
    "common/event_transitions",
    "common/factions",
    "common/flavorization",
    "common/focuses",
    "common/game_concepts",
    "common/game_rules",
    "common/genes",
    "common/governments",
    "common/guest_system",
    "common/holdings",
    "common/hook_types",
    "common/important_actions",
    "common/inspirations",
    "common/landed_titles",
    "common/laws",
    "common/lease_contracts",
    "common/lifestyle_perks",
    "common/lifestyles",
    "common/men_at_arms_types",
    "common/messages",
    "common/modifier_definition_formats",
    "common/modifier_icons",
    "common/modifiers",
    "common/named_colors",
    "common/nicknames",
    "common/on_action",
    "common/opinion_modifiers",
    "common/playable_difficulty_infos",
    "common/pool_character_selectors",
    "common/province_terrain",
    "common/religion/doctrines",
    "common/religion/fervor_modifiers",
    "common/religion/holy_sites",
    "common/religion/religion_families",
    "common/religion/religions",
    "common/schemes",
    "common/scripted_animations",
    "common/scripted_character_templates",
    "common/scripted_costs",
    "common/scripted_effects",
    "common/scripted_guis",
    "common/scripted_lists",
    "common/scripted_modifiers",
    "common/scripted_relations",
    "common/scripted_rules",
    "common/scripted_triggers",
    "common/script_values",
    "common/secret_types",
    "common/story_cycles",
    "common/struggle/catalysts",
    "common/struggle/struggles",
    "common/succession_election",
    "common/suggestions",
    "common/terrain_types",
    "common/traits",
    "common/travel/travel_options",
    "common/trigger_localization",
    "common/tutorial_lesson_chains",
    "common/tutorial_lessons",
    "common/vassal_contracts",
    "common/vassal_stances",
];
