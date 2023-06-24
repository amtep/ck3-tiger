use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::fileset::FileKind;
use crate::item::Item;
use crate::token::Token;
use crate::validate::{validate_color, validate_traits};

#[derive(Clone, Debug)]
pub struct Religion {}

impl Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("faiths") {
            for (faith, block) in block.iter_pure_definitions() {
                db.add(
                    Item::Faith,
                    faith.clone(),
                    block.clone(),
                    Box::new(Faith {
                        religion: key.clone(),
                    }),
                );
            }
        }
        db.add(Item::Religion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Religion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_adj");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent_plural");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);

        vd.req_field("family");
        vd.field_item("family", Item::ReligionFamily);

        vd.req_field("doctrine");
        vd.field_items("doctrine", Item::Doctrine);

        vd.field_validated_blocks("doctrine_selection_pair", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("requires_dlc_flag", Item::DlcFeature);
            vd.field_item("doctrine", Item::Doctrine);
            vd.field_item("fallback_doctrine", Item::Doctrine);
        });

        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(key, "NGameIcons|FAITH_DOCTRINE_BACKGROUND_PATH")
            {
                let pathname = format!("{icon_path}/{icon}");
                data.fileset.verify_exists_implied(&pathname, icon);
            }
        }
        vd.field_value("piety_icon_group"); // TODO
        vd.field_value("graphical_faith");
        vd.field_bool("pagan_roots");
        vd.field_validated_block("traits", validate_traits);

        vd.field_list("custom_faith_icons");
        if let Some(icons) = block.get_field_list("custom_faith_icons") {
            if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|FAITH_ICON_PATH")
            {
                for icon in &icons {
                    let pathname = format!("{icon_path}/{icon}.dds");
                    data.fileset.verify_exists_implied(&pathname, icon);
                }
            }
        }

        vd.field_list("reserved_male_names"); // TODO
        vd.field_list("reserved_female_names"); // TODO

        vd.field_validated_block("holy_order_names", validate_holy_order_names);
        vd.field_list_items("holy_order_maa", Item::MenAtArms);
        vd.field_validated_block("localization", validate_localization);
        vd.field_validated_block("faiths", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(); // validated by Faith class
        });
    }

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if let Some(block) = block.get_field_block("localization") {
            block.has_key(property)
        } else {
            false
        }
    }
}

fn validate_localization(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for field in CUSTOM_RELIGION_LOCAS {
        vd.field_validated(field, |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::Localization, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                for token in vd.values() {
                    data.verify_exists(Item::Localization, token);
                }
            }
        });
    }
}

fn validate_holy_order_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.field_item("coat_of_arms", Item::Coa);
    }
}

#[derive(Clone, Debug)]
pub struct Faith {
    religion: Token,
}

impl Faith {
    fn check_have_customs(&self, key: &Token, block: &Block, data: &Everything) {
        let locas = block.get_field_block("localization");
        for loca in CUSTOM_RELIGION_LOCAS {
            if let Some(block) = locas {
                if block.has_key(loca) {
                    continue;
                }
            }
            if data.item_has_property(Item::Religion, self.religion.as_str(), loca) {
                continue;
            }
            let msg = format!("faith or religion missing localization for {loca}");
            warn(key, ErrorKey::MissingLocalization, &msg);
        }
    }
}

impl DbKind for Faith {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_adj");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent_plural");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let pagan = block.get_field_bool("pagan_roots").unwrap_or(false);
        if pagan {
            let loca = format!("{key}_old");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_old_adj");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_old_adherent");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_old_adherent_plural");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);

        vd.req_field("color");
        vd.field_validated("color", |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::NamedColor, token),
            BV::Block(block) => validate_color(block, data),
        });
        let icon = vd.field_value("icon").unwrap_or(key);
        if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|FAITH_ICON_PATH") {
            let pathname = format!("{icon_path}/{icon}.dds");
            data.fileset.verify_exists_implied(&pathname, icon);
        }
        if let Some(icon) = vd.field_value("reformed_icon") {
            if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|FAITH_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}.dds");
                data.fileset.verify_exists_implied(&pathname, icon);
            }
        }
        vd.field_value("graphical_faith");
        vd.field_value("piety_icon_group"); // TODO

        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(key, "NGameIcons|FAITH_DOCTRINE_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}");
                data.fileset.verify_exists_implied(&pathname, icon);
            }
        }

        vd.field_item("religious_head", Item::Title);
        vd.req_field("holy_site");
        vd.field_items("holy_site", Item::HolySite);
        vd.req_field("doctrine");
        vd.field_items("doctrine", Item::Doctrine);
        vd.field_validated_blocks("doctrine_selection_pair", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("requires_dlc_flag", Item::DlcFeature);
            vd.field_item("doctrine", Item::Doctrine);
            vd.field_item("fallback_doctrine", Item::Doctrine);
        });

        vd.field_list("reserved_male_names");
        vd.field_list("reserved_female_names");
        vd.field_validated_block("localization", validate_localization);
        vd.field_validated_block("holy_order_names", validate_holy_order_names);
        vd.field_list_items("holy_order_maa", Item::MenAtArms); // TODO: verify this is allowed

        self.check_have_customs(key, block, data);
    }

    fn has_property(
        &self,
        key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "is_modded" {
            return key.loc.kind == FileKind::Mod;
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct ReligionFamily {}

impl ReligionFamily {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ReligionFamily, key, block, Box::new(Self {}));
    }
}

impl DbKind for ReligionFamily {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        vd.field_item("name", Item::Localization);
        if !block.has_key("name") {
            data.verify_exists(Item::Localization, key);
        }

        vd.field_bool("is_pagan");
        vd.field_item("graphical_faith", Item::GraphicalFaith);
        vd.field_value("piety_icon_group"); // TODO
        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            let pathname = format!("gfx/interface/icons/faith_doctrines/{icon}");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }
        vd.field_item("hostility_doctrine", Item::Doctrine);
    }
}

// LAST UPDATED VERSION 1.9.0.2
// Taken from the Faith.random_ values in `data_types_uncategorized.txt`
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
