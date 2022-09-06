use fnv::{FnvHashMap, FnvHashSet};
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
use crate::scopes::Scopes;
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

    pub fn verify_exists(&self, item: &Token) {
        if !self.traits.contains_key(item.as_str()) && !self.groups.contains(item.as_str()) {
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
        vd.field_integer("diplomacy_per_stress_level");
        vd.field_integer("martial_per_stress_level");
        vd.field_integer("intrigue_per_stress_level");
        vd.field_integer("stewardship_per_stress_level");
        vd.field_integer("learning_per_stress_level");
        vd.field_integer("prowess_per_stress_level");
        vd.field_numeric("dread_loss_mult");
        vd.field_numeric("dread_gain_mult");
        vd.field_numeric("dread_baseline_add");
        vd.field_integer("attraction_opinion");
        vd.field_integer("general_opinion");
        vd.field_integer("vassal_opinion");
        vd.field_numeric("same_culture_opinion");
        vd.field_integer("county_opinion_add");
        vd.field_script_value("ai_boldness", Scopes::None);
        vd.field_script_value("ai_compassion", Scopes::None);
        vd.field_script_value("ai_energy", Scopes::None);
        vd.field_script_value("ai_greed", Scopes::None);
        vd.field_script_value("ai_honor", Scopes::None);
        vd.field_script_value("ai_rationality", Scopes::None);
        vd.field_script_value("ai_sociability", Scopes::None);
        vd.field_script_value("ai_vengefulness", Scopes::None);
        vd.field_script_value("ai_zeal", Scopes::None);
        vd.field_numeric("monthly_income");
        vd.field_numeric("monthly_piety");
        vd.field_numeric("monthly_prestige");
        vd.field_numeric("monthly_dynasty_prestige");
        vd.field_numeric("monthly_piety_gain_mult");
        vd.field_numeric("monthly_prestige_gain_mult");
        vd.field_numeric("monthly_dynasty_prestige_mult");
        vd.field_numeric("monthly_lifestyle_xp_gain_mult");
        vd.field_numeric("monthly_county_control_change_add");
        vd.field_numeric("advantage");
        vd.field_numeric("attacker_advantage");
        vd.field_numeric("defender_advantage");
        vd.field_numeric("enemy_terrain_advantage");
        vd.field_numeric("controlled_province_advantage");
        vd.field_numeric("tolerance_advantage_mod");
        vd.field_numeric("hard_casualty_modifier");
        vd.field_numeric("enemy_hard_casualty_modifier");
        vd.field_numeric("counter_efficiency");
        vd.field_numeric("pursue_efficiency");
        vd.field_numeric("retreat_losses");
        vd.field_numeric("supply_duration");
        vd.field_numeric("movement_speed");
        vd.field_numeric("winter_movement_speed");
        vd.field_numeric("raid_speed");
        vd.field_integer("min_combat_roll");
        vd.field_integer("max_combat_roll");
        vd.field_numeric("hostile_county_attrition");
        vd.field_numeric("fertility");
        vd.field_numeric("health");
        vd.field_numeric("negate_health_penalty_add");
        vd.field_integer("years_of_fertility");
        vd.field_integer("life_expectancy");
        vd.field_integer("max_hostile_schemes_add");
        vd.field_integer("hostile_scheme_resistance_add");
        vd.field_integer("hostile_scheme_resistance_mult");
        vd.field_integer("hostile_scheme_power_add");
        vd.field_integer("owned_hostile_scheme_success_chance_add");
        vd.field_numeric("ai_war_cooldown");
        vd.field_numeric("ai_war_chance");
        vd.field_bool("no_prowess_loss_from_age");
        vd.field_numeric("stress_loss_mult");
        vd.field_numeric("stress_gain_mult");
        vd.field_numeric("knight_effectiveness_mult");
        vd.field_integer("positive_inactive_inheritance_chance");
        vd.field_integer("positive_random_genetic_chance");
        vd.field_integer("genetic_trait_strengthen_chance");
        vd.field_numeric("levy_size");

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
        vd.field_value("group");
        vd.field_value("group_equivalence");
        vd.field_numeric("same_opinion");
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
        vd.field_bool("no_water_crossing_penalty");
        vd.field_choice("parent_inheritance_sex", &["male", "female"]);
        vd.field_values("flag");

        Self::validate_modifiers(data, &mut vd);
        vd.warn_remaining();
    }
}
