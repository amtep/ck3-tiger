use fnv::FnvHashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::provinces::ProvId;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_possibly_named_color;

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
    pub fn load_item(
        &mut self,
        key: Token,
        block: Block,
        parent: Option<&str>,
        is_county_capital: bool,
    ) {
        if let Some(other) = self.titles.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "title");
            }
        }
        let title = Rc::new(Title::new(
            key.clone(),
            block.clone(),
            parent,
            is_county_capital,
        ));
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

        let mut is_county_capital = parent_tier == Tier::County;
        for (k, v) in block.iter_definitions() {
            if let Ok(tier) = Tier::try_from(k) {
                if tier >= parent_tier {
                    let msg = format!("can't put a {tier} inside a {parent_tier}");
                    error(k, ErrorKey::TitleTier, &msg);
                }
                self.load_item(k.clone(), v.clone(), Some(key.as_str()), is_county_capital);
                is_county_capital = false;
            }
        }
        if is_county_capital {
            error(key, ErrorKey::Validation, "county with no baronies!");
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.titles.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<Rc<Title>> {
        self.titles.get(key).cloned()
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.titles.values() {
            item.validate(data);
        }
    }

    pub fn capital_of(&self, prov: ProvId) -> Option<&str> {
        self.baronies.get(&prov).and_then(|b| b.capital_of())
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

        let Some(mut block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.drain_definitions_warn() {
            if Tier::try_from(&key).is_ok() {
                self.load_item(key, block, None, false);
            } else {
                warn(key, ErrorKey::Validation, "expected title");
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Title {
    key: Token,
    block: Block,
    pub tier: Tier,
    pub parent: Option<String>,
    is_county_capital: bool, // for baronies
}

impl Title {
    pub fn new(key: Token, block: Block, parent: Option<&str>, is_county_capital: bool) -> Self {
        let tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        let parent = parent.map(String::from);
        Self {
            key,
            block,
            tier,
            parent,
            is_county_capital,
        }
    }

    pub fn validate(&self, data: &Everything) {
        // NOTE: There used to be a check that non-barony titles existed in the
        // title history, but that seems to be optional.
        data.verify_exists(Item::Localization, &self.key);
        // TODO: figure out when to recommend adding _adj or _pre or _article loca
        let loca = format!("{}_adj", &self.key);
        data.item_used(Item::Localization, &loca);
        let loca = format!("{}_pre", &self.key);
        data.item_used(Item::Localization, &loca);
        let definite_form = self.block.field_value_is("definite_form", "yes");
        if definite_form {
            let loca = format!("{}_article", &self.key);
            data.item_used(Item::Localization, &loca);
        }

        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());

        vd.field_validated("color", validate_possibly_named_color);
        vd.advice_field("color2", "no longer used");
        if let Some(token) = vd.field_value("capital") {
            data.verify_exists(Item::Title, token);
            if Tier::try_from(token) != Ok(Tier::County) {
                let msg = "capital must be a county";
                error(token, ErrorKey::TitleTier, msg);
            }
        }
        vd.field_bool("definite_form");
        vd.field_bool("ruler_uses_title_name");
        vd.field_bool("can_be_named_after_dynasty");
        vd.field_bool("landless");
        vd.field_bool("no_automatic_claims");
        vd.field_bool("always_follows_primary_heir");
        vd.field_bool("destroy_if_invalid_heir");
        vd.field_bool("destroy_on_succession");
        vd.field_bool("delete_on_destroy");
        vd.field_bool("delete_on_gain_same_tier");
        vd.field_bool("de_jure_drift_disabled");
        vd.field_bool("ignore_titularity_for_title_weighting");

        vd.field_list_items("male_names", Item::Localization);
        vd.field_list_items("female_names", Item::Localization);

        if Tier::try_from(&self.key) == Ok(Tier::Barony) {
            // TODO: check that no two baronies have the same province
            vd.field_item("province", Item::Province);
        }

        vd.field_script_value_no_breakdown("ai_primary_priority", &mut sc);

        vd.field_validated_block("can_create", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_create_on_partition", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("cultural_names", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, token) in vd.unknown_value_fields() {
                data.verify_exists(Item::NameList, key);
                data.verify_exists(Item::Localization, token);
                let loca = format!("{token}_adj");
                data.item_used(Item::Localization, &loca);
                if definite_form {
                    let loca = format!("{token}_article");
                    data.item_used(Item::Localization, &loca);
                }
            }
        });

        for (key, _) in vd.unknown_block_fields() {
            if Tier::try_from(key).is_err() {
                let msg = format!("unknown field `{key}`");
                warn(key, ErrorKey::UnknownField, &msg);
            }
        }
    }

    fn capital_of(&self) -> Option<&str> {
        if self.is_county_capital {
            self.parent.as_deref()
        } else {
            None
        }
    }
}
