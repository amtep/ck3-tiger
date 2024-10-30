use std::path::PathBuf;

use crate::block::{Block, BV};
use crate::ck3::data::provinces::ProvId;
use crate::ck3::data::titles::Titles;
use crate::date::Date;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::Severity;

#[derive(Clone, Debug, Default)]
pub struct ProvinceHistories {
    provinces: TigerHashMap<ProvId, ProvinceHistory>,
}

impl ProvinceHistories {
    fn load_item(&mut self, id: ProvId, key: Token, mut block: Block) {
        if let Some(province) = self.provinces.get_mut(&id) {
            // Multiple entries are valid but could easily be a mistake.
            if province.key.loc.kind >= key.loc.kind {
                dup_error(&key, &province.key, "province");
            }
            province.block.append(&mut block);
        } else {
            self.provinces.insert(id, ProvinceHistory::new(key, block));
        }
    }

    pub fn validate(&self, data: &Everything) {
        for (provid, item) in &self.provinces {
            item.validate(*provid, data);
        }
    }

    pub fn check_pod_faiths(&self, data: &Everything, titles: &Titles) {
        for bookmark in [
            Date::new(1230, 1, 4),
            Date::new(1230, 1, 5),
            Date::new(1230, 1, 6),
            Date::new(1375, 7, 5),
            Date::new(1510, 1, 3),
        ] {
            for (provid, provhist) in &self.provinces {
                if let Some(capital) = titles.capital_of(*provid) {
                    let religion = provhist.block.get_field_at_date("religion", bookmark);
                    if let Some(religion) = religion.and_then(BV::get_value) {
                        if !data.item_has_property(Item::Faith, religion.as_str(), "is_modded") {
                            let msg = format!(
                                "Vanilla or unknown religion in prov {} (county {}) at {}",
                                provhist.key, capital, bookmark
                            );
                            warn(ErrorKey::PrincesOfDarkness).msg(msg).loc(religion).push();
                        }
                    } else {
                        warn(ErrorKey::PrincesOfDarkness)
                            .msg("no religion")
                            .loc(&provhist.key)
                            .push();
                    }
                }
            }
        }
    }
}

impl FileHandler<Block> for ProvinceHistories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/provinces")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read_detect_encoding(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if let Ok(id) = key.as_str().parse() {
                self.load_item(id, key, block);
            } else {
                let msg = "unexpected key, expected only province ids";
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProvinceHistory {
    key: Token,
    block: Block,
}

impl ProvinceHistory {
    fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    fn validate_common(vd: &mut Validator, data: &Everything) {
        vd.field_item("culture", Item::Culture);
        vd.field_item("religion", Item::Faith);
        vd.field_item("faith", Item::Faith);
        if let Some(token) = vd.field_value("holding") {
            if !token.is("auto") && !token.is("none") {
                data.verify_exists(Item::HoldingType, token);
            }
        }
        vd.field_list_items("buildings", Item::Building);
        vd.multi_field_item("special_building_slot", Item::SpecialBuilding);
        vd.multi_field_item("special_building", Item::SpecialBuilding);
        // TODO: check if province is duchy capital
        // TODO: check if building is duchy capital building
        vd.field_item("duchy_capital_building", Item::Building);

        vd.field_validated_block_rooted("effect", Scopes::Province, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No);
        });
    }

    fn validate_history(_date: Date, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        Self::validate_common(&mut vd, data);
    }

    fn validate(&self, provid: ProvId, data: &Everything) {
        data.provinces_ck3.verify_exists_provid(provid, &self.key, Severity::Error);
        // TODO: verify that all county-capital provinces have a culture and religion
        // This needs province mappings to be loaded too
        let mut vd = Validator::new(&self.block, data);
        Self::validate_common(&mut vd, data);
        vd.field_value("terrain"); // TODO: this does not seem to be an Item::Terrain
        vd.validate_history_blocks(Self::validate_history);
    }
}
