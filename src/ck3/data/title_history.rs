use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::Block;
use crate::ck3::data::titles::Tier;
use crate::context::ScopeContext;
use crate::date::Date;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{error, error_info, old_warn, warn2, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct TitleHistories {
    histories: FnvHashMap<String, TitleHistory>,
}

impl TitleHistories {
    pub fn load_item(&mut self, key: Token, mut block: Block) {
        if let Some(other) = self.histories.get_mut(key.as_str()) {
            // Multiple entries are valid but could easily be a mistake.
            if other.key.loc.kind >= key.loc.kind {
                warn2(
                    &other.key,
                    ErrorKey::DuplicateItem,
                    "title has two definition blocks, they will be added together",
                    key,
                    "the other one is here",
                );
            }
            other.block.append(&mut block);
        } else {
            self.histories.insert(key.to_string(), TitleHistory::new(key.clone(), block));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.histories.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.histories.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.histories.values() {
            item.validate(data);
        }
    }

    pub fn verify_has_holder(&self, key: &Token, date: Date, data: &Everything) {
        if let Some(item) = self.histories.get(key.as_str()) {
            item.verify_has_holder(key, date, data);
        } else {
            let msg = format!("{key} has no title history");
            error(key, ErrorKey::MissingItem, &msg);
        }
    }
}

impl FileHandler<Block> for TitleHistories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/titles")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read_detect_encoding(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if Tier::try_from(&key).is_ok() {
                self.load_item(key, block);
            } else {
                old_warn(key, ErrorKey::Validation, "expected title");
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TitleHistory {
    key: Token,
    block: Block,
    tier: Tier,
}

impl TitleHistory {
    pub fn new(key: Token, block: Block) -> Self {
        let tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        Self { key, block, tier }
    }

    pub fn verify_has_holder(&self, token: &Token, date: Date, data: &Everything) {
        if let Some(holder) = self.block.get_field_at_date("holder", date) {
            // if holder is not a value then we already warned about that
            if let Some(holder) = holder.get_value() {
                if holder.is("0") {
                    let msg = format!("{token} has no holder at {date}");
                    error_info(
                        token,
                        ErrorKey::History,
                        &msg,
                        "setting the liege will not have effect here",
                    );
                } else if !data.characters.is_alive(holder, date) {
                    let msg = format!("holder of {token} is not alive at {date}");
                    error_info(
                        token,
                        ErrorKey::History,
                        &msg,
                        "setting the liege will not have effect here",
                    );
                }
            }
        } else {
            let msg = format!("{token} has no holder at {date}");
            error_info(
                token,
                ErrorKey::History,
                &msg,
                "setting the liege will not have effect here",
            );
        }
    }

    pub fn validate_history(&self, date: Date, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_numeric("change_development_level");
        if let Some(token) = vd.field_value("holder") {
            if !token.is("0") {
                data.verify_exists(Item::Character, token);
                if data.item_exists(Item::Character, token.as_str()) {
                    data.characters.verify_alive(token, date);
                }
            }
        }
        if let Some(token) = vd.field_value("holder_ignore_head_of_faith_requirement") {
            if !token.is("0") {
                data.verify_exists(Item::Character, token);
                if data.item_exists(Item::Character, token.as_str()) {
                    data.characters.verify_alive(token, date);
                }
            }
        }

        if let Some(token) = vd.field_value("liege") {
            if !token.is("0") {
                data.verify_exists(Item::Title, token);
                if let Some(title) = data.titles.get(token.as_str()) {
                    if title.tier <= self.tier {
                        let msg = format!("liege must be higher tier than {}", self.key);
                        error(token, ErrorKey::TitleTier, &msg);
                    }
                    data.title_history.verify_has_holder(token, date, data);
                }
            }
        }

        if let Some(token) = vd.field_value("de_jure_liege") {
            if !token.is("0") {
                data.verify_exists(Item::Title, token);
                if let Some(title) = data.titles.get(token.as_str()) {
                    if title.tier <= self.tier {
                        let msg = format!("liege must be higher tier than {}", self.key);
                        error(token, ErrorKey::TitleTier, &msg);
                    }
                }
            }
        }

        vd.field_item("government", Item::GovernmentType);

        vd.field_block("succession_laws"); // TODO
        vd.field_bool("remove_succession_laws");

        vd.field_item("name", Item::Localization);
        vd.field_bool("reset_name");

        vd.field_item("insert_title_history", Item::TitleHistory);

        vd.field_validated_key_block("effect", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::Title, &self.key);

        let mut vd = Validator::new(&self.block, data);
        vd.validate_history_blocks(|date, _key, block, data| {
            self.validate_history(date, block, data);
        });
    }
}
