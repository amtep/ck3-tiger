use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

use crate::block::Block;
use crate::ck3::data::provinces::ProvId;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Clone, Debug, Default)]
pub struct Titles {
    titles: TigerHashMap<&'static str, Arc<Title>>,
    baronies: TigerHashMap<ProvId, Arc<Title>>,
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
        block: &Block,
        parent: Option<&'static str>,
        is_county_capital: bool,
    ) {
        if let Some(other) = self.titles.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "title");
            }
        }
        let title = Arc::new(Title::new(key.clone(), block.clone(), parent, is_county_capital));
        self.titles.insert(key.as_str(), Arc::clone(&title));

        let parent_tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        if parent_tier == Tier::Barony {
            if let Some(provid) = block.get_field_integer("province") {
                if let Ok(provid) = ProvId::try_from(provid) {
                    self.baronies.insert(provid, title);
                } else {
                    err(ErrorKey::Validation)
                        .msg("province id out of range")
                        .loc(block.get_field_value("province").unwrap())
                        .push();
                }
            } else {
                err(ErrorKey::Validation).msg("barony without province id").loc(&key).push();
            }
        }

        let mut is_county_capital = parent_tier == Tier::County;
        for (k, block) in block.iter_definitions() {
            if let Ok(tier) = Tier::try_from(k) {
                if tier >= parent_tier {
                    let msg = format!("can't put a {tier} inside a {parent_tier}");
                    err(ErrorKey::TitleTier).msg(msg).loc(k).push();
                }
                self.load_item(k.clone(), block, Some(key.as_str()), is_county_capital);
                is_county_capital = false;
            }
        }
        if is_county_capital {
            err(ErrorKey::Validation).msg("county with no baronies!").loc(key).push();
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.titles.values() {
            // Title blocks are nested, so the parent check is to avoid re-scanning subordinate titles.
            if item.parent.is_none() {
                registry.scan(&item.block);
            }
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.titles.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.titles.values().map(|item| &item.key)
    }

    pub fn get(&self, key: &str) -> Option<Arc<Title>> {
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

impl FileHandler<Block> for Titles {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/landed_titles")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if Tier::try_from(&key).is_ok() {
                self.load_item(key, &block, None, false);
            } else {
                warn(ErrorKey::Validation).msg("expected title").loc(key).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Title {
    key: Token,
    block: Block,
    pub tier: Tier,
    pub parent: Option<&'static str>,
    is_county_capital: bool, // for baronies
}

impl Title {
    pub fn new(
        key: Token,
        block: Block,
        parent: Option<&'static str>,
        is_county_capital: bool,
    ) -> Self {
        let tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        Self { key, block, tier, parent, is_county_capital }
    }

    pub fn validate(&self, data: &Everything) {
        // NOTE: There used to be a check that non-barony titles existed in the
        // title history, but that seems to be optional.
        data.verify_exists(Item::Localization, &self.key);
        let loca = format!("{}_adj", &self.key);
        if self.tier > Tier::Barony {
            data.verify_exists_implied(Item::Localization, &loca, &self.key);
        } else {
            data.mark_used(Item::Localization, &loca);
        }
        // The _pre is rarely defined even in vanilla
        let loca = format!("{}_pre", &self.key);
        data.mark_used(Item::Localization, &loca);
        let definite_form = self.block.field_value_is("definite_form", "yes");
        if definite_form {
            let loca = format!("{}_article", &self.key);
            data.mark_used(Item::Localization, &loca);
        }

        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Character, &self.key);

        vd.field_validated("color", validate_possibly_named_color);
        vd.advice_field("color2", "no longer used");
        if let Some(token) = vd.field_value("capital") {
            data.verify_exists(Item::Title, token);
            if Tier::try_from(token) != Ok(Tier::County) {
                let msg = "capital must be a county";
                err(ErrorKey::TitleTier).msg(msg).loc(token).push();
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
        vd.field_bool("require_landless");
        vd.field_bool("noble_family");
        vd.field_bool("can_use_nomadic_naming");

        vd.field_list_items("male_names", Item::Localization);
        vd.field_list_items("female_names", Item::Localization);

        if Tier::try_from(&self.key) == Ok(Tier::Barony) {
            // TODO: check that no two baronies have the same province
            vd.field_item("province", Item::Province);
        }

        vd.field_script_value_no_breakdown("ai_primary_priority", &mut sc);

        vd.field_trigger("can_create", Tooltipped::Yes, &mut sc);
        vd.field_trigger("can_create_on_parition", Tooltipped::No, &mut sc);
        vd.field_trigger("can_destroy", Tooltipped::Yes, &mut sc);

        vd.field_validated_block("cultural_names", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, token| {
                data.verify_exists(Item::NameList, key);
                data.verify_exists(Item::Localization, token);
                let loca = format!("{token}_adj");
                data.mark_used(Item::Localization, &loca);
                if definite_form {
                    let loca = format!("{token}_article");
                    data.mark_used(Item::Localization, &loca);
                }
            });
        });

        // The blocks are validated by the next level Title
        vd.unknown_block_fields(|key, _| {
            if Tier::try_from(key).is_err() {
                let msg = format!("unknown field `{key}`");
                warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
            }
        });
    }

    fn capital_of(&self) -> Option<&str> {
        if self.is_county_capital {
            self.parent
        } else {
            None
        }
    }
}
