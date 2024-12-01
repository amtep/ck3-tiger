use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::{Builder, Validator};

#[derive(Clone, Debug)]
pub struct LawType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::LawType, LawType::add)
}

impl LawType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LawType, key, block, Box::new(Self {}));
    }
}

impl DbKind for LawType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("group", Item::LawGroup);
        vd.field_item("icon", Item::File);

        vd.field_numeric("progressiveness");
        vd.field_bool("limited_to_frontier_colonization"); // undocumented

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_validated_block("acceptance_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_validated_block_rooted("is_visible", Scopes::Country, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block_rooted("can_enact", Scopes::Country, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block_rooted("on_enact", Scopes::Country, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::Yes);
        });
        // TODO: what is the difference between on_enact and on_activate? Are they both valid?
        vd.field_validated_block_rooted("on_activate", Scopes::Country, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block_rooted("on_deactivate", Scopes::Country, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_key_block("religious_acceptance_rule", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Religion, key);
            sc.define_name("country", Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("cultural_acceptance_rule", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("country", Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_list_items("possible_political_movements", Item::LawType);
        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_list_items("unlocking_laws", Item::LawType);
        vd.field_list_items("disallowing_laws", Item::LawType);
        vd.field_script_value_rooted("pop_support", Scopes::Pop);

        vd.field_item("institution", Item::Institution);
        vd.field_validated_block("institution_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_list_items("build_from_investment_pool", Item::BuildingGroup);
        vd.field_script_value_rooted("revolution_state_weight", Scopes::State);

        for field in &[
            "tax_modifier_very_low",
            "tax_modifier_low",
            "tax_modifier_medium",
            "tax_modifier_high",
            "tax_modifier_very_high",
        ] {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::all(), vd);
            });
        }

        for field in &[
            "tariff_modifier_no_priority",
            "tariff_modifier_export_priority",
            "tariff_modifier_import_priority",
        ] {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::all(), vd);
            });
        }

        vd.field_validated_key_block("ai_will_do", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key("ai_enact_weight_modifier", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("law", Scopes::LawType, key);
            validate_script_value(bv, data, &mut sc);
        });

        let sc_impose: &Builder = &|key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("initiator", Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            sc.define_name("law", Scopes::LawType, key);
            sc
        };
        vd.field_trigger_full("can_impose", sc_impose, Tooltipped::Yes);
        vd.field_effect_full("on_impose", sc_impose, Tooltipped::Yes);

        let sc_impose_chance: &Builder = &|key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            sc.define_name("law", Scopes::LawType, key);
            sc
        };
        vd.field_script_value_full("ai_impose_chance", sc_impose_chance, false);
    }
}

#[derive(Clone, Debug)]
pub struct LawGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::LawGroup, LawGroup::add)
}

impl LawGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LawGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for LawGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("law_group_category", &["power_structure", "economy", "human_rights"]);
        vd.field_integer("base_enactment_days");
        vd.field_numeric("enactment_approval_mult");

        vd.field_numeric("progressive_movement_chance");
        vd.field_numeric("regressive_movement_chance");

        vd.field_validated_block_rooted(
            "change_allowed_trigger",
            Scopes::Country,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::Yes);
            },
        );

        // undocumented

        vd.field_bool("affected_by_regime_change");
        vd.field_item("linked_social_hierarchy", Item::SocialHierarchy);
    }
}
