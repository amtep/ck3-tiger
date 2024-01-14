use crate::block::{Block, BV};
use crate::ck3::validate::validate_traits;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::fileset::FileKind;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{old_warn, ErrorKey};
use crate::token::Token;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Religion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Religion, Religion::add)
}

impl Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("faiths") {
            for (faith, block) in block.iter_definitions() {
                if let Some(token) = block.get_field_value("graphical_faith") {
                    db.add_flag(Item::GraphicalFaith, token.clone());
                }
                if let Some(token) = block.get_field_value("icon") {
                    db.add_flag(Item::FaithIcon, token.clone());
                } else {
                    db.add_flag(Item::FaithIcon, faith.clone());
                }
                if let Some(token) = block.get_field_value("reformed_icon") {
                    db.add_flag(Item::FaithIcon, token.clone());
                }
                let kind = Box::new(Faith { religion: key.clone() });
                db.add(Item::Faith, faith.clone(), block.clone(), kind);
            }
        }
        if let Some(block) = block.get_field_block("custom_faith_icons") {
            for token in block.iter_values() {
                db.add_flag(Item::FaithIcon, token.clone());
            }
        }
        if let Some(token) = block.get_field_value("graphical_faith") {
            db.add_flag(Item::GraphicalFaith, token.clone());
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
        vd.multi_field_item("doctrine", Item::Doctrine);

        vd.multi_field_validated_block("doctrine_selection_pair", |block, data| {
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
                data.verify_exists_implied(Item::File, &pathname, icon);
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
                    data.verify_exists_implied(Item::File, &pathname, icon);
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
            vd.unknown_block_fields(|_, _| ()); // validated by Faith class
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

/// Loaded via [`Religion`]
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
            old_warn(key, ErrorKey::MissingLocalization, &msg);
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

        let pagan = block
            .get_field_values("doctrine")
            .iter()
            .any(|value| data.doctrines.unreformed(value.as_str()));
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
        vd.field_validated("color", validate_possibly_named_color);

        let icon = vd.field_value("icon").unwrap_or(key);
        if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|FAITH_ICON_PATH") {
            let pathname = format!("{icon_path}/{icon}.dds");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }
        if pagan {
            vd.req_field_fatal("reformed_icon");
        } else {
            vd.ban_field("reformed_icon", || "unreformed faiths");
        }
        if let Some(icon) = vd.field_value("reformed_icon") {
            if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|FAITH_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}.dds");
                data.fileset.verify_exists_implied_crashes(&pathname, icon);
            }
        }
        vd.field_value("graphical_faith");
        vd.field_value("piety_icon_group"); // TODO

        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(key, "NGameIcons|FAITH_DOCTRINE_BACKGROUND_PATH")
            {
                let pathname = format!("{icon_path}/{icon}");
                data.verify_exists_implied(Item::File, &pathname, icon);
            }
        }

        vd.field_item("religious_head", Item::Title);
        vd.req_field("holy_site");
        vd.multi_field_item("holy_site", Item::HolySite);
        vd.req_field("doctrine");
        vd.multi_field_item("doctrine", Item::Doctrine);
        vd.multi_field_validated_block("doctrine_selection_pair", |block, data| {
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

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ReligionFamily, ReligionFamily::add)
}

impl ReligionFamily {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(token) = block.get_field_value("graphical_faith") {
            db.add_flag(Item::GraphicalFaith, token.clone());
        }
        db.add(Item::ReligionFamily, key, block, Box::new(Self {}));
    }
}

impl DbKind for ReligionFamily {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let name = vd.field_value("name").unwrap_or(key);
        data.verify_exists(Item::Localization, name);
        let loca = format!("{name}_desc");
        data.verify_exists_implied(Item::Localization, &loca, name);

        vd.field_bool("is_pagan");
        vd.field_value("graphical_faith");
        vd.field_value("piety_icon_group"); // TODO
        if let Some(icon) = vd.field_value("doctrine_background_icon") {
            let pathname = format!("gfx/interface/icons/faith_doctrines/{icon}");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }
        vd.field_item("hostility_doctrine", Item::Doctrine);
    }
}

// LAST UPDATED VERSION 1.11.3
// Taken from the Faith.random_ values in `data_types_uncategorized.txt`
pub const CUSTOM_RELIGION_LOCAS: &[&str] = &[
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
