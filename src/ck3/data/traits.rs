use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::{validate_desc, validate_desc_map};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct Traits {
    traits: FnvHashMap<String, Trait>,
    groups: FnvHashMap<String, Token>,
    tracks: FnvHashMap<String, Token>,
    constraints: FnvHashMap<String, Token>,
    flags: FnvHashMap<String, Token>,
}

impl Traits {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.traits.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "trait");
            }
        }
        if let Some(token) = block.get_field_value("group") {
            self.groups.insert(token.to_string(), token.clone());
        }
        for token in block.get_field_values("flag") {
            self.flags.insert(token.to_string(), token.clone());
        }
        for field in
            &["genetic_constraint_all", "genetic_constraint_men", "genetic_constraint_women"]
        {
            if let Some(token) = block.get_field_value(field) {
                self.constraints.insert(token.to_string(), token.clone());
            }
        }
        if let Some(token) = block.get_field_value("group_equivalence") {
            self.groups.insert(token.to_string(), token.clone());
        }
        if block.has_key("track") {
            self.tracks.insert(key.to_string(), key.clone());
        }
        if let Some(block) = block.get_field_block("tracks") {
            for (key, _) in block.iter_definitions() {
                self.tracks.insert(key.to_string(), key.clone());
            }
        }
        self.traits.insert(key.to_string(), Trait::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.traits.contains_key(key) || self.groups.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.traits.values().map(|item| &item.key).chain(self.groups.values())
    }

    pub fn constraint_exists(&self, key: &str) -> bool {
        self.constraints.contains_key(key)
    }

    pub fn iter_constraint_keys(&self) -> impl Iterator<Item = &Token> {
        self.constraints.values()
    }

    pub fn flag_exists(&self, key: &str) -> bool {
        self.flags.contains_key(key)
    }

    pub fn iter_flag_keys(&self) -> impl Iterator<Item = &Token> {
        self.flags.values()
    }

    pub fn track_exists(&self, key: &str) -> bool {
        self.tracks.contains_key(key)
    }

    pub fn iter_track_keys(&self) -> impl Iterator<Item = &Token> {
        self.tracks.values()
    }

    // Is the trait itself a track? Different than a trait having multiple tracks
    pub fn has_track(&self, key: &str) -> bool {
        self.traits.get(key).map_or(false, |t| t.has_track)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.traits.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Traits {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/traits")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
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
        Self { key, block, has_track }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Character, &self.key);

        let genetic = self.block.field_value_is("genetic", "yes");

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("trait_{}", self.key);
            data.verify_exists_implied(Item::Localization, &loca, &self.key);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("trait_{}_desc", self.key);
            data.verify_exists_implied(Item::Localization, &loca, &self.key);
        }

        if !vd.field_validated("icon", |bv, data| {
            validate_desc_map(bv, data, &mut sc, |name, data, _| {
                if let Some(icon_path) =
                    data.get_defined_string_warn(&self.key, "NGameIcons|TRAIT_ICON_PATH")
                {
                    let path = format!("{icon_path}/{name}");
                    data.verify_exists_implied(Item::File, &path, name);
                }
            });
        }) {
            if let Some(icon_path) =
                data.get_defined_string_warn(&self.key, "NGameIcons|TRAIT_ICON_PATH")
            {
                let path = format!("{icon_path}/{}.dds", self.key);
                data.verify_exists_implied(Item::File, &path, &self.key);
            }
        }

        vd.field_item("category", Item::TraitCategory);
        vd.multi_field_validated_block("culture_modifier", validate_culture_modifier);
        vd.multi_field_validated_block("faith_modifier", validate_faith_modifier);
        vd.field_item("culture_succession_prio", Item::CultureParameter);
        vd.multi_field_validated_block("triggered_opinion", validate_triggered_opinion);

        vd.field_validated_block("tracks", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                validate_trait_track(key, block, data, key);
            });
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
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Trait, key);
                value.expect_number();
            });
        });

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
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
        vd.advice_field("can_inherit", "no longer used");
        vd.field_bool("inherit_from_real_mother");
        vd.field_bool("inherit_from_real_father");
        vd.field_choice("parent_inheritance_sex", &["male", "female", "all"]);
        vd.field_choice("child_inheritance_sex", &["male", "female", "all"]);
        if genetic {
            vd.field_numeric_range("birth", 0.0, 1.0);
            vd.field_numeric_range("random_creation", 0.0, 1.0);
        } else {
            vd.ban_field("birth", || "genetic = yes");
            vd.ban_field("random_creation", || "genetic = yes");
        }
        vd.replaced_field("blocks_from_claim_inheritance", "`claim_inheritance_blocker = all`");
        vd.replaced_field(
            "blocks_from_claim_inheritance_from_dynasty",
            "`claim_inheritance_blocker = dynasty`",
        );
        vd.field_bool("incapacitating");
        vd.field_bool("disables_combat_leadership");
        for token in vd.multi_field_value("flag") {
            // These are optional
            let loca = format!("TRAIT_FLAG_DESC_{token}");
            data.mark_used(Item::Localization, &loca);
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
    vd.unknown_block_fields(|key, block| {
        let mut sc = ScopeContext::new(Scopes::None, warn_key);
        validate_script_value(&BV::Value(key.clone()), data, &mut sc);
        if let Some(xp) = key.get_integer() {
            // LAST UPDATED CK3 VERSION 1.9.2.1
            if xp > 100 {
                let msg = "trait xp only goes up to 100";
                err(ErrorKey::Range).strong().msg(msg).loc(key).push();
            }
        }

        let mut vd = Validator::new(block, data);
        vd.multi_field_validated_block("culture_modifier", validate_culture_modifier);
        vd.multi_field_validated_block("faith_modifier", validate_faith_modifier);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
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
