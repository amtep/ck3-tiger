use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::desc::{validate_desc, validate_desc_map};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Traits {
    traits: FnvHashMap<String, Trait>,
}

impl Traits {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.traits.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "trait");
            }
        }
        self.traits
            .insert(key.to_string(), Trait::new(key.clone(), block.clone()));
    }

    pub fn verify_exists(&self, item: &Token) {
        if !self.traits.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "trait not defined in common/traits/",
            );
        }
    }

    pub fn verify_exists_opt(&self, item: Option<&Token>) {
        if let Some(item) = item {
            self.verify_exists(item);
        }
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.traits.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Traits {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/traits")
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

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Trait {
    key: Token,
    block: Block,
}

impl Trait {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    // TODO: move these to Modifiers when we load those.
    fn validate_modifiers(_data: &Everything, vd: &mut Validator) {
        vd.field_integer("diplomacy");
        vd.field_integer("martial");
        vd.field_integer("intrigue");
        vd.field_integer("stewardship");
        vd.field_integer("learning");
        vd.field_integer("prowess");
        vd.field_integer("diplomacy_no_portrait");
        vd.field_integer("martial_no_portrait");
        vd.field_integer("intrigue_no_portrait");
        vd.field_integer("stewardship_no_portrait");
        vd.field_integer("learning_no_portrait");
        vd.field_integer("prowess_no_portrait");
        vd.field_numeric("dead_loss_mult");
        vd.field_integer("attraction_opinion");
        vd.field_integer("ai_boldness");
        vd.field_integer("ai_energy");
        vd.field_integer("monthly_piety");
        vd.field_integer("advantage");

        // TODO: monthly_<lifestyle>_xp_gain_mult
    }

    fn validate_culture_modifier(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("parameter"); // TODO: check cultural parameter exists
        Self::validate_modifiers(data, &mut vd);
        vd.warn_remaining();
    }

    fn validate_triggered_opinion(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("opinion_modifier"); // TODO: validate
        vd.field_value("parameter"); // TODO: check doctrine parameter exists
        vd.field_bool("check_missing");
        vd.field_bool("same_faith");
        vd.field_bool("same_dynasty");
        vd.field_bool("ignore_opinion_value_if_same_trait");
        vd.field_bool("male_only");
        vd.field_bool("female_only");
        vd.warn_remaining();
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        if let Some(bv) = vd.field("name") {
            validate_desc(bv, data);
        } else {
            let loca = format!("trait_{}", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if let Some(bv) = vd.field("desc") {
            validate_desc(bv, data);
        } else {
            let loca = format!("trait_{}_desc", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if let Some(bv) = vd.field("icon") {
            validate_desc_map(bv, data, |name, data| {
                let path = format!("gfx/interface/icons/traits/{}", name);
                data.fileset.verify_exists_implied(&path, name);
            });
        } else {
            let path = format!("gfx/interface/icons/traits/{}.dds", self.key);
            data.fileset.verify_exists_implied(&path, &self.key);
        }

        vd.field_validated_blocks("culture_modifier", Self::validate_culture_modifier);
        vd.field_validated_blocks("triggered_opinion", Self::validate_triggered_opinion);

        vd.field_list("opposites");
        if let Some(tokens) = self.block.get_field_list("opposites") {
            for token in tokens {
                data.traits.verify_exists(&token);
            }
        }

        vd.field_integer("minimum_age");
        vd.field_bool("education");
        vd.field_integer("ruler_designer_cost");
        vd.field_bool("shown_in_ruler_designer");
        vd.field_bool("add_commander_trait");
        vd.field_value("group");

        Self::validate_modifiers(data, &mut vd);
        vd.warn_remaining();
    }
}
