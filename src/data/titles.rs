use fnv::FnvHashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::block::Block;
use crate::data::provinces::ProvId;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Titles {
    titles: FnvHashMap<String, Rc<Title>>,
    baronies: FnvHashMap<ProvId, Rc<Title>>,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Tier {
    Barony,
    County,
    Duchy,
    Kingdom,
    Empire,
}

impl TryFrom<&Token> for Tier {
    type Error = std::fmt::Error;
    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        let s = value.as_str();
        if s.starts_with("b_") {
            Ok(Tier::Barony)
        } else if s.starts_with("c_") {
            Ok(Tier::County)
        } else if s.starts_with("d_") {
            Ok(Tier::Duchy)
        } else if s.starts_with("k_") {
            Ok(Tier::Kingdom)
        } else if s.starts_with("e_") {
            Ok(Tier::Empire)
        } else {
            Err(std::fmt::Error)
        }
    }
}

impl TryFrom<Token> for Tier {
    type Error = std::fmt::Error;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Tier::try_from(&value)
    }
}

impl Display for Tier {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Tier::Barony => write!(f, "barony"),
            Tier::County => write!(f, "county"),
            Tier::Duchy => write!(f, "duchy"),
            Tier::Kingdom => write!(f, "kingdom"),
            Tier::Empire => write!(f, "empire"),
        }
    }
}

impl Titles {
    pub fn load_item(&mut self, key: Token, block: &Block, capital_of: Option<Token>) {
        if let Some(other) = self.titles.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "title");
            }
        }
        let title = Rc::new(Title::new(key.clone(), block.clone(), capital_of));
        self.titles.insert(key.to_string(), title.clone());

        let parent_tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        if parent_tier == Tier::Barony {
            if let Some(provid) = block.get_field_integer("province") {
                if let Ok(provid) = ProvId::try_from(provid) {
                    self.baronies.insert(provid, title);
                } else {
                    error(
                        block.get_field_value("province").unwrap(),
                        ErrorKey::Validation,
                        "province id out of range",
                    );
                }
            } else {
                error(&key, ErrorKey::Validation, "barony without province id");
            }
        }

        let mut capital = parent_tier == Tier::County;
        for (k, v) in block.iter_pure_definitions() {
            if let Ok(tier) = Tier::try_from(k) {
                if tier >= parent_tier {
                    let msg = format!("can't put a {} inside a {}", tier, parent_tier);
                    error(k, ErrorKey::Validation, &msg);
                }
                let capital_of = if capital { Some(key.clone()) } else { None };
                self.load_item(k.clone(), v, capital_of);
                capital = false;
            }
        }
        if capital {
            error(key, ErrorKey::Validation, "county with no baronies!");
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.titles.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.titles.values() {
            item.validate(data);
        }
    }

    pub fn capital_of(&self, prov: ProvId) -> Option<&Token> {
        self.baronies.get(&prov).and_then(|b| b.capital_of.as_ref())
    }
}

impl FileHandler for Titles {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/landed_titles")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for (key, block) in block.iter_pure_definitions_warn() {
            if Tier::try_from(key).is_ok() {
                self.load_item(key.clone(), block, None);
            } else {
                warn(key, ErrorKey::Validation, "expected title");
            }
        }
    }

    fn finalize(&mut self) {
        for title in self.titles.values() {
            if let Some(capital) = title.block.get_field_value("capital") {
                if self.titles.get(capital.as_str()).is_none() {
                    error(
                        capital,
                        ErrorKey::Validation,
                        "capital is not defined as a title",
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Title {
    key: Token,
    block: Block,
    tier: Tier,
    capital_of: Option<Token>, // for baronies
}

impl Title {
    pub fn new(key: Token, block: Block, capital_of: Option<Token>) -> Self {
        let tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        Self {
            key,
            block,
            tier,
            capital_of,
        }
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::TitleHistory, &self.key);
        data.localization.verify_exists(&self.key);
        // TODO: figure out when to recommend adding _adj or _pre titles
        // The _adj key is optional
        // The _pre key is optional

        if let Some(names) = self.block.get_field_block("cultural_names") {
            for (_, t) in names.get_assignments() {
                data.localization.verify_exists(t);
                // The _adj key is optional
            }
        }
    }
}
