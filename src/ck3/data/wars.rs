use std::path::PathBuf;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct Wars {
    wars: Vec<War>,
}

impl Wars {
    fn load_item(&mut self, key: Token, block: Block) {
        if key.is("war") {
            self.wars.push(War::new(block));
        } else {
            let msg = format!("unexpected key {key}, expected only `war`");
            err(ErrorKey::History).msg(msg).loc(key).push();
        }
    }

    pub fn validate(&self, data: &Everything) {
        for item in &self.wars {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Wars {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/wars")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read_optional_bom(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct War {
    block: Block,
}

impl War {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("start_date");
        vd.req_field("end_date");

        vd.field_item("name", Item::Localization);
        vd.field_date("start_date");
        vd.field_date("end_date");

        vd.field_list_items("targeted_titles", Item::Title);
        vd.field_item("casus_belli", Item::CasusBelli);
        vd.field_list_items("attackers", Item::Character);
        vd.field_list_items("defenders", Item::Character);
        vd.field_item("claimant", Item::Character);

        vd.unknown_block_fields(|key, block| {
            if data.item_exists(Item::Character, key.as_str()) {
                let mut vd = Validator::new(block, data);
                vd.validate_history_blocks(|_date, block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("location");
                    vd.field_item("location", Item::Province);
                });
            } else {
                let msg = format!("character id {key} not found in history");
                warn(ErrorKey::MissingItem).msg(msg).loc(key).push();
            }
        });
    }
}
