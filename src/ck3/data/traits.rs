use std::path::{Path, PathBuf};

use fnv::{FnvHashMap, FnvHashSet};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::{validate_desc, validate_desc_map};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::scriptvalue::validate_scriptvalue;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug, Default)]
pub struct Traits {
    traits: FnvHashMap<String, Trait>,
    groups: FnvHashSet<String>,
    tracks: FnvHashSet<String>,
    constraints: FnvHashSet<String>,
    flags: FnvHashSet<String>,
}

impl Traits {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.traits.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "trait");
            }
        }
        if let Some(token) = block.get_field_value("group") {
            self.groups.insert(token.to_string());
        }
        for token in block.get_field_values("flag") {
            self.flags.insert(token.to_string());
        }
        for field in &[
            "genetic_constraint_all",
            "genetic_constraint_men",
            "genetic_constraint_women",
        ] {
            if let Some(token) = block.get_field_value(field) {
                self.constraints.insert(token.to_string());
            }
        }
        if let Some(token) = block.get_field_value("group_equivalence") {
            self.groups.insert(token.to_string());
        }
        if block.has_key("track") {
            self.tracks.insert(key.to_string());
        }
        if let Some(block) = block.get_field_block("tracks") {
            for (key, _) in block.iter_definitions() {
                self.tracks.insert(key.to_string());
            }
        }
        self.traits.insert(key.to_string(), Trait::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.traits.contains_key(key) || self.groups.contains(key)
    }

    pub fn constraint_exists(&self, key: &str) -> bool {
        self.constraints.contains(key)
    }

    pub fn flag_exists(&self, key: &str) -> bool {
        self.flags.contains(key)
    }

    pub fn track_exists(&self, key: &str) -> bool {
        self.tracks.contains(key)
    }

    // Is the trait itself a track? Different than a trait having multiple tracks
    pub fn has_track(&self, key: &str) -> bool {
        self.traits.get(key).map_or(false, |t| t.has_track)
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

        let Some(mut block) = PdxFile::read(entry, fullpath) else { return; };
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Trait {
    key: Token,
    block: Block,
    has_track: bool,
}

impl Trait {
    pub fn new(key: Token, block: Block) -> Self {
        let has_track = block.has_key("track") || block.has_key("tracks");
        Self {
            key,
            block,
            has_track,
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Character, &self.key);

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("trait_{}", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("trait_{}_desc", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if !vd.field_validated("icon", |bv, data| {
            validate_desc_map(bv, data, &mut sc, |name, data| {
                if let Some(icon_path) =
                    data.get_defined_string_warn(&self.key, "NGameIcons|TRAIT_ICON_PATH")
                {
                    let path = format!("{icon_path}/{name}");
                    data.fileset.verify_exists_implied(&path, name);
                }
            });
        }) {
            if let Some(icon_path) =
                data.get_defined_string_warn(&self.key, "NGameIcons|TRAIT_ICON_PATH")
            {
                let path = format!("{icon_path}/{}.dds", self.key);
                data.fileset.verify_exists_implied(&path, &self.key);
            }
        }

        vd.field_item("category", Item::TraitCategory);
        vd.field_validated_blocks("culture_modifier", validate_culture_modifier);
        vd.field_validated_blocks("faith_modifier", validate_faith_modifier);
        vd.field_item("culture_succession_prio", Item::CultureParameter);
        vd.field_validated_blocks("triggered_opinion", validate_triggered_opinion);

        vd.field_validated_block("tracks", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, block) in vd.unknown_block_fields() {
                validate_trait_track(key, block, data, key);
            }
        });
        vd.field_validated_key_block("track", |key, block, data| {
            validate_trait_track(&self.key, block, data, key);
        });

        vd.field_list_items("opposites", Item::Trait);
        if let Some(tokens) = self.block.get_field_list("opposites") {
            for token in tokens {
                data.verify_exists(Item::Trait, &token);
            }
        }

        vd.field_validated_block("compatibility", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, _) in vd.unknown_value_fields() {
                data.verify_exists(Item::Trait, key);
            }
        });

        vd.field_validated_block("potential", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_integer("minimum_age");
        vd.field_integer("maximum_age");
        vd.field_choice("valid_sex", &["all", "male", "female"]);
        vd.replaced_field("education", "`category = education`");
        vd.replaced_field("childhood", "`category = childhood`");
        vd.field_integer("ruler_designer_cost");
        vd.field_bool("shown_in_ruler_designer");
        vd.field_bool("add_commander_trait");
        vd.replaced_field("fame", "`category = fame`");
        vd.replaced_field("lifestyle", "`category = lifestyle`");
        vd.replaced_field("personality", "`category = personality`");
        vd.replaced_field("health_trait", "`category = health`");
        vd.replaced_field("commander", "`category = commander`");
        vd.replaced_field("court_type_trait", "`category = court_type`");
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
        vd.field_integer_range("inherit_chance", 0, 100);
        vd.field_integer_range("both_parent_has_trait_inherit_chance", 0, 100);
        vd.field_numeric_range("birth", 0.0, 1.0);
        vd.field_numeric_range("random_creation", 0.0, 1.0);
        vd.field_bool("can_inherit");
        vd.field_bool("inherit_from_real_father");
        vd.replaced_field(
            "blocks_from_claim_inheritance",
            "`claim_inheritance_blocker = all`",
        );
        vd.replaced_field(
            "blocks_from_claim_inheritance_from_dynasty",
            "`claim_inheritance_blocker = dynasty`",
        );
        vd.field_bool("incapacitating");
        vd.field_bool("disables_combat_leadership");
        vd.field_choice("parent_inheritance_sex", &["male", "female", "all"]);
        vd.field_choice("child_inheritance_sex", &["male", "female", "all"]);
        for _token in vd.field_values("flag") {
            // These are optional
            // let loca = format!("TRAIT_FLAG_DESC_{token}");
            // data.verify_exists_implied(Item::Localization, &loca, token);
        }
        vd.field_bool("shown_in_encyclopedia");

        vd.field_choice("inheritance_blocker", &["none", "dynasty", "all"]);
        vd.field_choice("claim_inheritance_blocker", &["none", "dynasty", "all"]);
        vd.field_choice("bastard", &["none", "illegitimate", "legitimate"]);

        // The ethnicity files refer to these
        vd.field_value("genetic_constraint_all");
        vd.field_value("genetic_constraint_men");
        vd.field_value("genetic_constraint_women");

        vd.field_numeric("portrait_extremity_shift");
        vd.field_numeric("ugliness_portrait_extremity_shift");
        vd.field_block("portrait_pose"); // TODO

        vd.field_list_items("trait_exclusive_if_realm_contains", Item::Terrain);
        vd.replaced_field("trait_winter_exclusive", "`category = winter_commander`");

        validate_modifs(&self.block, data, ModifKinds::Character, vd);
    }
}

fn validate_culture_modifier(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_item("parameter", Item::CultureParameter);
    validate_modifs(block, data, ModifKinds::Character, vd);
}

fn validate_faith_modifier(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_item("parameter", Item::DoctrineParameter);
    validate_modifs(block, data, ModifKinds::Character, vd);
}

fn validate_triggered_opinion(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_item("opinion_modifier", Item::OpinionModifier);
    vd.field_item("parameter", Item::DoctrineParameter);
    vd.field_bool("check_missing");
    vd.field_bool("same_faith");
    vd.field_bool("same_dynasty");
    vd.field_bool("ignore_opinion_value_if_same_trait");
    vd.field_bool("male_only");
    vd.field_bool("female_only");
}

fn validate_trait_track(key: &Token, block: &Block, data: &Everything, warn_key: &Token) {
    let mut vd = Validator::new(block, data);
    for (key, block) in vd.unknown_block_fields() {
        let mut sc = ScopeContext::new(Scopes::None, warn_key);
        validate_scriptvalue(&BV::Value(key.clone()), data, &mut sc);

        let mut vd = Validator::new(block, data);
        vd.field_validated_blocks("culture_modifier", validate_culture_modifier);
        vd.field_validated_blocks("faith_modifier", validate_faith_modifier);
        validate_modifs(block, data, ModifKinds::Character, vd);
    }
    // let modif = format!("{key}_xp_degradation_mult");
    // data.verify_exists_implied(Item::ModifierFormat, &modif, warn_key);
    // let modif = format!("{key}_xp_gain_mult");
    // data.verify_exists_implied(Item::ModifierFormat, &modif, warn_key);
    // let modif = format!("{key}_xp_loss_mult");
    // data.verify_exists_implied(Item::ModifierFormat, &modif, warn_key);

    let loca = format!("trait_track_{key}");
    data.verify_exists_implied(Item::Localization, &loca, warn_key);
    let loca = format!("trait_track_{key}_desc");
    data.verify_exists_implied(Item::Localization, &loca, warn_key);
}
