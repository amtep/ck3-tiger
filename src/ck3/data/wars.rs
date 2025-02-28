use std::path::PathBuf;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{ErrorKey, err, warn};
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

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read_optional_bom(entry, parser)
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

        // These are not actually required by the game engine,
        // but they are logically needed to define a war.
        vd.req_field("start_date");
        vd.req_field("end_date");
        vd.req_field("casus_belli");
        vd.req_field("attackers");
        vd.req_field("defenders");

        vd.field_item("name", Item::Localization);

        let mut start_date_token = None;
        let mut end_date_token = None;
        vd.field_validated_value("start_date", |_, mut vd| {
            vd.date();
            start_date_token = Some(vd.value().clone());
        });
        vd.field_validated_value("end_date", |_, mut vd| {
            vd.date();
            end_date_token = Some(vd.value().clone());
        });

        if let Some(start_date) = start_date_token.as_ref().and_then(Token::get_date) {
            if let Some(end_date) = end_date_token.as_ref().and_then(Token::get_date) {
                if start_date > end_date {
                    err(ErrorKey::Range)
                        .msg("start date is after end date")
                        .loc_msg(start_date_token.as_ref().unwrap(), "start date")
                        .loc_msg(end_date_token.as_ref().unwrap(), "end date")
                        .push();
                }
            }
        }

        vd.field_list_items("targeted_titles", Item::Title);
        vd.field_item("casus_belli", Item::CasusBelli);
        vd.field_list_items("attackers", Item::Character);
        vd.field_list_items("defenders", Item::Character);
        vd.field_item("claimant", Item::Character);

        vd.unknown_block_fields(|key, block| {
            if data.item_exists(Item::Character, key.as_str()) {
                let mut vd = Validator::new(block, data);
                vd.validate_history_blocks(|date, key, block, data| {
                    if let Some(start_date) = start_date_token.as_ref().and_then(Token::get_date) {
                        if date < start_date {
                            err(ErrorKey::Range)
                                .msg("date is before start date")
                                .loc(key)
                                .loc_msg(start_date_token.as_ref().unwrap(), "start date")
                                .push();
                        }
                    }
                    if let Some(end_date) = end_date_token.as_ref().and_then(Token::get_date) {
                        if date > end_date {
                            err(ErrorKey::Range)
                                .msg("date is after end date")
                                .loc(key)
                                .loc_msg(end_date_token.as_ref().unwrap(), "end date")
                                .push();
                        }
                    }
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
