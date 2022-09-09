use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::desc::{validate_desc, validate_desc_map};
use crate::errorkey::ErrorKey;
use crate::errors::error_info;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Traits {
    traits: FnvHashMap<String, Trait>,
    groups: FnvHashSet<String>,
}

impl Traits {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.traits.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "trait");
            }
        }
        if let Some(token) = block.get_field_value("group") {
            self.groups.insert(token.to_string());
        }
        if let Some(token) = block.get_field_value("group_equivalence") {
            self.groups.insert(token.to_string());
        }
        self.traits
            .insert(key.to_string(), Trait::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.traits.contains_key(key) || self.groups.contains(key)
    }

    pub fn validate(&self, data: &Everything) {
        let mut vec = self.traits.values().collect::<Vec<&Trait>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
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

    fn validate_modifiers<'a>(block: &Block, data: &'a Everything, vd: &mut Validator<'a>) {
        validate_modifs(block, data, ModifKinds::Character, vd);
    }

    fn validate_culture_modifier(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("parameter"); // TODO: check cultural parameter exists
        Self::validate_modifiers(block, data, &mut vd);
        // vd.warn_remaining(); TODO: re-enable when modifs are complete
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
                data.verify_exists(Item::Trait, &token);
            }
        }

        // TODO: validate as trait = integer assignments
        vd.field_block("compatibility");

        vd.field_integer("minimum_age");
        vd.field_bool("education");
        vd.field_integer("ruler_designer_cost");
        vd.field_bool("shown_in_ruler_designer");
        vd.field_bool("add_commander_trait");
        vd.field_bool("fame");
        vd.field_bool("lifestyle");
        vd.field_bool("personality");
        vd.field_bool("health_trait");
        vd.field_bool("genetic");
        vd.field_bool("physical");
        vd.field_bool("good");
        vd.field_bool("immortal");
        vd.field_bool("can_have_children");
        vd.field_bool("enables_inbred");
        vd.field_value("group");
        vd.field_value("group_equivalence");
        vd.field_numeric("same_opinion");
        vd.field_numeric("same_opinion_if_same_faith");
        vd.field_numeric("opposite_opinion");
        vd.field_numeric("same_faith_opinion");
        vd.field_integer("level");
        vd.field_integer("inherit_chance");
        vd.field_integer("both_parent_has_trait_inherit_chance");
        vd.field_numeric("birth");
        vd.field_numeric("random_creation");
        vd.field_bool("can_inherit");
        vd.field_bool("inherit_from_real_father");
        vd.field_bool("blocks_from_claim_inheritance");
        vd.field_bool("incapacitating");
        vd.field_bool("disables_combat_leadership");
        vd.field_choice("parent_inheritance_sex", &["male", "female"]);
        vd.field_values("flag");
        vd.field_bool("shown_in_encyclopedia");

        Self::validate_modifiers(&self.block, data, &mut vd);
        // vd.warn_remaining(); TODO: re-enable when modifs are complete
    }
}
