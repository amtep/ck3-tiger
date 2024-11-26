use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
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

        vd.field_validated_key_block("creation_trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_script_value_rooted("creation_weight", Scopes::Country);
        vd.field_validated_key_block("on_created", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_block("culture_selection_trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("country", Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key("culture_selection_weight", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });

        vd.field_validated_key_block("religion_selection_trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Religion, key);
            sc.define_name("country", Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key("religion_selection_weight", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Religion, key);
            sc.define_name("country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });

        vd.field_validated_key_block("character_support_trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key("character_support_weight", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            validate_script_value(bv, data, &mut sc);
        });

        vd.field_validated_key_block("pop_support_trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_list_items("pop_support_factors", Item::PoliticalMovementPopSupport);
        vd.field_validated_key("pop_support_weight", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            validate_script_value(bv, data, &mut sc);
        });

        for field in &["revolution", "secession"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_validated_key_block("possible", |key, block, data| {
                    let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
                    sc.define_name("clout", Scopes::Value, key);
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_validated_key("weight", |key, bv, data| {
                    let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
                    sc.define_name("clout", Scopes::Value, key);
                    validate_script_value(bv, data, &mut sc);
                });
                vd.field_validated_block("forced_tags", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_block_fields(|key, block| {
                        data.verify_exists(Item::Country, key);
                        let mut vd = Validator::new(block, data);
                        vd.field_validated_key_block("trigger", |key, block, data| {
                            let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
                            validate_trigger(block, data, &mut sc, Tooltipped::No);
                        });
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
        vd.field_validated_key("additional_radicalism_factors", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("religion", Scopes::Religion, key);
            validate_script_value(bv, data, &mut sc);
        });

        // undocumented

        vd.field_validated_key_block("disband_trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_disbanded", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
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

        vd.field_validated_key_block("visible", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::PoliticalMovement, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}
