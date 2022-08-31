use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, DefinitionItem};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::pdxfile::PdxFile;
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug, Default)]
pub struct Religions {
    religions: FnvHashMap<String, Religion>,
    faiths: FnvHashMap<String, Faith>,
}

impl Religions {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.religions.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && will_log(key, ErrorKey::Duplicate) {
                error(
                    key,
                    ErrorKey::Duplicate,
                    "religion redefines an existing religion",
                );
                info(
                    &other.key,
                    ErrorKey::Duplicate,
                    "the other religion is here",
                );
            }
        }
        self.religions
            .insert(key.to_string(), Religion::new(key.clone(), block.clone()));

        if let Some(faith_block) = block.get_field_block("faiths") {
            for (faith, b) in faith_block.iter_pure_definitions_warn() {
                if let Some(other) = self.faiths.get(faith.as_str()) {
                    if other.key.loc.kind >= key.loc.kind && will_log(key, ErrorKey::Duplicate) {
                        error(
                            key,
                            ErrorKey::Duplicate,
                            "faith redefines an existing faith",
                        );
                        info(&other.key, ErrorKey::Duplicate, "the other faith is here");
                    }
                }
                let pagan = block.get_field_bool("pagan_roots").unwrap_or(false);
                self.faiths.insert(
                    faith.to_string(),
                    Faith::new(faith.clone(), b.clone(), key.clone(), pagan),
                );
            }
        }
    }

    pub fn validate(&self, data: &Everything) {
        for religion in self.religions.values() {
            religion.validate(data);
        }
        for faith in self.faiths.values() {
            faith.validate(data);

            let religion = &self.religions[faith.religion.as_str()];
            faith.check_have_customs(religion);
        }
    }

    pub fn verify_implied_faith_exists(&self, key: &str, item: &Token) {
        if !self.faiths.contains_key(key) {
            error(
                item,
                ErrorKey::MissingItem,
                "faith not defined in common/religion/religions",
            );
        }
    }

    pub fn verify_faith_exists(&self, item: &Token) {
        if !self.faiths.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "faith not defined in common/religion/religions",
            );
        }
    }

    pub fn verify_faith_exists_opt(&self, item: Option<&Token>) {
        if let Some(item) = item {
            self.verify_faith_exists(item);
        };
    }

    pub fn is_modded_faith(&self, item: &Token) -> bool {
        if let Some(faith) = self.faiths.get(item.as_str()) {
            faith.key.loc.kind != FileKind::VanillaFile
        } else {
            false
        }
    }
}

impl FileHandler for Religions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/religion/religions")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
            Ok(block) => block,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{:#}", e),
                );
                return;
            }
        };

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Assignment(key, _) => {
                    error(key, ErrorKey::Validation, "unknown assignment");
                }
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Definition(key, b) => {
                    self.load_item(key, b);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Religion {
    key: Token,
    block: Block,
}

impl Religion {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        data.localization.verify_exists(&self.key);
        let loca = format!("{}_adj", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_adherent", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_adherent_plural", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_desc", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);

        let mut vd = Validator::new(&self.block, data);

        vd.req_field("family");
        vd.field_value("family");

        vd.req_field("doctrine");
        vd.field_values("doctrine");

        vd.field_blocks("doctrine_selection_pair"); // TODO: validate
        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            let pathname = format!("gfx/interface/icons/faith_doctrines/{}", icon);
            data.fileset.verify_exists_implied(&pathname, icon);
        }
        vd.field_value("piety_icon_group");
        vd.field_value("graphical_faith");
        vd.field_bool("pagan_roots");
        vd.field_validated_block("traits", validate_traits);

        vd.field_list("custom_faith_icons");
        if let Some(icons) = self.block.get_field_list("custom_faith_icons") {
            for icon in &icons {
                let pathname = format!("gfx/interface/icons/faith/{}.dds", icon);
                data.fileset.verify_exists_implied(&pathname, icon);
            }
        }

        vd.field_list("reserved_male_names"); // TODO
        vd.field_list("reserved_female_names"); // TODO

        vd.field_validated_block("holy_order_names", validate_holy_order_names);
        vd.field_list("holy_order_maa");
        vd.field_validated_block("localization", validate_localization);
        vd.field_blocks("faiths");
        vd.warn_remaining();
    }
}

fn validate_traits(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    // TODO: parse these. Can be single tokens ("wrathful") or assignments ("wrathful = 3")
    // or even wrathful = { modifier = modifier_key scale = 2 }
    vd.req_field("virtues");
    vd.req_field("sins");
    vd.warn_remaining();
}

fn validate_localization(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for field in CUSTOM_RELIGION_LOCAS {
        vd.field(field);
        if let Some(token) = block.get_field_value(field) {
            data.localization.verify_exists(token);
        } else if let Some(list) = block.get_field_list(field) {
            for token in list {
                data.localization.verify_exists(&token);
            }
        }
    }
    vd.warn_remaining();
}

fn validate_holy_order_names(block: &Block, data: &Everything) {
    if let Some(holy) = block.get_field_block("holy_order_names") {
        for b in holy.get_sub_blocks() {
            if let Some(name) = b.get_field_value("name") {
                data.localization.verify_exists(name);
            }
        }
    }

    // TODO
    // It's a list of sub-blocks, each one having a name key and optional coat_of_arms key
}

#[derive(Clone, Debug)]
pub struct Faith {
    key: Token,
    block: Block,
    religion: Token,
    pagan: bool,
}

impl Faith {
    pub fn new(key: Token, block: Block, religion: Token, pagan: bool) -> Self {
        // TODO: verify that reform_icon is set if a pagan faith
        Self {
            key,
            block,
            religion,
            pagan,
        }
    }

    pub fn validate(&self, data: &Everything) {
        data.localization.verify_exists(&self.key);
        let loca = format!("{}_adj", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_adherent", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_adherent_plural", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);

        if self.pagan {
            let loca = format!("{}_old", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
            let loca = format!("{}_old_adj", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
            let loca = format!("{}_old_adherent", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
            let loca = format!("{}_old_adherent_plural", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        let mut vd = Validator::new(&self.block, data);

        vd.req_field("color");
        vd.field_validated_block("color", validate_color);
        if let Some(icon) = vd.field_value("icon") {
            let pathname = format!("gfx/interface/icons/faith/{}.dds", icon);
            data.fileset.verify_exists_implied(&pathname, icon);
        } else {
            let pathname = format!("gfx/interface/icons/faith/{}.dds", self.key);
            data.fileset.verify_exists_implied(&pathname, &self.key);
        }
        if let Some(icon) = vd.field_value("reformed_icon") {
            let pathname = format!("gfx/interface/icons/faith/{}.dds", icon);
            data.fileset.verify_exists_implied(&pathname, icon);
        }
        vd.field_value("graphical_faith");
        vd.field_value("piety_icon_group");

        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            let pathname = format!("gfx/interface/icons/faith_doctrines/{}", icon);
            data.fileset.verify_exists_implied(&pathname, icon);
        }

        vd.field_value("religious_head");
        vd.req_field("holy_site");
        vd.field_values("holy_site");
        vd.req_field("doctrine");
        vd.field_values("doctrine");
        vd.field_blocks("doctrine_selection_pair"); // TODO: validate
        vd.field_list("reserved_male_names");
        vd.field_list("reserved_female_names");
        vd.field_validated_block("localization", validate_localization);
        vd.field_validated_block("holy_order_names", validate_holy_order_names);
        vd.field_list("holy_order_maa"); // TODO: verify this is allowed
        vd.warn_remaining();
    }

    pub fn check_have_customs(&self, religion: &Religion) {
        let self_block = self.block.get_field_block("localization");
        let religion_block = religion.block.get_field_block("localization");
        for s in CUSTOM_RELIGION_LOCAS {
            if let Some(block) = self_block {
                if block.get_key(s).is_some() {
                    continue;
                }
            }
            if let Some(block) = religion_block {
                if block.get_key(s).is_some() {
                    continue;
                }
            }
            let msg = format!("faith or religion missing localization for `{}`", s);
            warn(&self.key, ErrorKey::MissingLocalization, &msg);
        }
    }
}

// LAST UPDATED VERSION 1.6.2.2
const CUSTOM_RELIGION_LOCAS: &[&str] = &[
    "AltPriestTermPlural",
    "BishopFemale",
    "BishopFemalePlural",
    "BishopMale",
    "BishopMalePlural",
    "BishopNeuter",
    "BishopNeuterPlural",
    "CreatorHerHim",
    "CreatorHerHis",
    "CreatorName",
    "CreatorNamePossessive",
    "CreatorSheHe",
    "DeathDeityHerHis",
    "DeathDeityName",
    "DeathDeityNamePossessive",
    "DeathDeitySheHe",
    "DevilHerHis",
    "DevilHerselfHimself",
    "DevilName",
    "DevilNamePossessive",
    "DevilSheHe",
    "DevoteeFemale",
    "DevoteeFemalePlural",
    "DevoteeMale",
    "DevoteeMalePlural",
    "DevoteeNeuter",
    "DevoteeNeuterPlural",
    "DivineRealm",
    "DivineRealm2",
    "DivineRealm3",
    "EvilGodNames",
    "FateGodHerHim",
    "FateGodHerHis",
    "FateGodName",
    "FateGodNamePossessive",
    "FateGodSheHe",
    "FertilityGodHerHim",
    "FertilityGodHerHis",
    "FertilityGodName",
    "FertilityGodNamePossessive",
    "FertilityGodSheHe",
    "GHWName",
    "GHWNamePlural",
    "GoodGodNames",
    "HealthGodHerHim",
    "HealthGodHerHis",
    "HealthGodName",
    "HealthGodNamePossessive",
    "HealthGodSheHe",
    "HighGodHerHis",
    "HighGodHerselfHimself",
    "HighGodName",
    "HighGodName2",
    "HighGodNameAlternate",
    "HighGodNameAlternatePossessive",
    "HighGodNamePossessive",
    "HighGodNameSheHe",
    "HouseOfWorship",
    "HouseOfWorship2",
    "HouseOfWorship3",
    "HouseOfWorshipPlural",
    "HouseholdGodHerHim",
    "HouseholdGodHerHis",
    "HouseholdGodName",
    "HouseholdGodNamePossessive",
    "HouseholdGodSheHe",
    "KnowledgeGodHerHim",
    "KnowledgeGodHerHis",
    "KnowledgeGodName",
    "KnowledgeGodNamePossessive",
    "KnowledgeGodSheHe",
    "NegativeAfterLife",
    "NegativeAfterLife2",
    "NegativeAfterLife3",
    "NightGodHerHim",
    "NightGodHerHis",
    "NightGodName",
    "NightGodNamePossessive",
    "NightGodSheHe",
    "PantheonTerm",
    "PantheonTerm2",
    "PantheonTerm3",
    "PantheonTermHasHave",
    "PositiveAfterLife",
    "PositiveAfterLife2",
    "PositiveAfterLife3",
    "PriestFemale",
    "PriestFemalePlural",
    "PriestMale",
    "PriestMalePlural",
    "PriestNeuter",
    "PriestNeuterPlural",
    "ReligiousHeadName",
    "ReligiousHeadTitleName",
    "ReligiousSymbol",
    "ReligiousSymbol2",
    "ReligiousSymbol3",
    "ReligiousText",
    "ReligiousText2",
    "ReligiousText3",
    "TricksterGodHerHim",
    "TricksterGodHerHis",
    "TricksterGodName",
    "TricksterGodNamePossessive",
    "TricksterGodSheHe",
    "WarGodHerHim",
    "WarGodHerHis",
    "WarGodName",
    "WarGodNamePossessive",
    "WarGodSheHe",
    "WaterGodHerHim",
    "WaterGodHerHis",
    "WaterGodName",
    "WaterGodNamePossessive",
    "WaterGodSheHe",
    "WealthGodHerHim",
    "WealthGodHerHis",
    "WealthGodName",
    "WealthGodNamePossessive",
    "WealthGodSheHe",
    "WitchGodHerHim",
    "WitchGodHerHis",
    "WitchGodMistressMaster",
    "WitchGodMotherFather",
    "WitchGodName",
    "WitchGodNamePossessive",
    "WitchGodSheHe",
];
