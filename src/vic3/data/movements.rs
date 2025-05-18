use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PoliticalMovement {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PoliticalMovement, PoliticalMovement::add)
}

impl PoliticalMovement {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoliticalMovement, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoliticalMovement {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("category", Item::PoliticalMovementCategory);
        vd.field_item("ideology", Item::Ideology);
        vd.field_list_items("character_ideologies", Item::Ideology);

        vd.field_trigger_rooted("creation_trigger", Tooltipped::No, Scopes::Country);
        vd.field_script_value_rooted("creation_weight", Scopes::Country);
        vd.field_effect_rooted("on_created", Tooltipped::No, Scopes::PoliticalMovement);

        vd.field_trigger_builder("culture_selection_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("country", Scopes::Country, key);
            sc
        });
        vd.field_script_value_builder("culture_selection_weight", |key| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("country", Scopes::Country, key);
            sc
        });

        vd.field_trigger_builder("religion_selection_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Religion, key);
            sc.define_name("country", Scopes::Country, key);
            sc
        });
        vd.field_script_value_builder("religion_selection_weight", |key| {
            let mut sc = ScopeContext::new(Scopes::Religion, key);
            sc.define_name("country", Scopes::Country, key);
            sc
        });

        vd.field_trigger_builder("character_support_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            sc
        });
        vd.field_script_value_builder("character_support_weight", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            sc
        });

        vd.field_trigger_rooted(
            "can_pressure_interest_group",
            Tooltipped::No,
            Scopes::InterestGroup,
        );

        vd.field_trigger_builder("pop_support_trigger", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            sc
        });
        vd.field_list_items("pop_support_factors", Item::PoliticalMovementPopSupport);
        vd.field_script_value_builder("pop_support_weight", |key| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            sc
        });

        for field in &["revolution", "secession"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_trigger_builder("possible", Tooltipped::No, |key| {
                    let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
                    sc.define_name("clout", Scopes::Value, key);
                    sc
                });
                vd.field_script_value_builder("weight", |key| {
                    let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
                    sc.define_name("clout", Scopes::Value, key);
                    sc
                });
                vd.field_validated_block("forced_tags", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_block_fields(|key, block| {
                        data.verify_exists(Item::Country, key);
                        let mut vd = Validator::new(block, data);
                        vd.field_trigger_rooted(
                            "trigger",
                            Tooltipped::No,
                            Scopes::PoliticalMovement,
                        );
                        vd.field_script_value_rooted("weight", Scopes::PoliticalMovement);
                    });
                });
                vd.field_validated_key("state_weight", |key, bv, data| {
                    let mut sc = ScopeContext::new(Scopes::State, key);
                    sc.define_name("political_movement", Scopes::PoliticalMovement, key);
                    validate_script_value(bv, data, &mut sc);
                });
                vd.field_validated_key("target_fraction_of_states", |key, bv, data| {
                    let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
                    sc.define_name("clout", Scopes::Value, key);
                    validate_script_value(bv, data, &mut sc);
                });
            });
        }

        vd.field_script_value_rooted(
            "law_enactment_radicalism_multiplier",
            Scopes::PoliticalMovement,
        );
        vd.field_script_value_rooted("active_law_radicalism_multiplier", Scopes::PoliticalMovement);
        vd.field_validated_key("additional_radicalism_factors", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            validate_script_value(bv, data, &mut sc);
        });

        // undocumented

        vd.field_trigger_rooted("disband_trigger", Tooltipped::No, Scopes::Country);
        vd.field_effect_rooted("on_disbanded", Tooltipped::No, Scopes::Country);
    }
}

#[derive(Clone, Debug)]
pub struct PoliticalMovementCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PoliticalMovementCategory, PoliticalMovementCategory::add)
}

impl PoliticalMovementCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoliticalMovementCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoliticalMovementCategory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_bool("cultural_identity");
        vd.field_bool("religious_identity");
        vd.field_bool("minimum_support_is_within_identity");
        vd.field_numeric("minimum_support_to_create");
        vd.field_numeric("minimum_support_to_maintain");

        // undocumented

        vd.field_numeric("sorting_order");
    }
}

#[derive(Clone, Debug)]
pub struct PoliticalMovementPopSupport {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PoliticalMovementPopSupport, PoliticalMovementPopSupport::add)
}

impl PoliticalMovementPopSupport {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoliticalMovementPopSupport, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoliticalMovementPopSupport {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("name", Item::Localization);

        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::PoliticalMovement);
    }
}
