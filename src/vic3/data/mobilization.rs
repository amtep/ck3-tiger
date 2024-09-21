use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::{Builder, Validator};

#[derive(Clone, Debug)]
pub struct MobilizationOption {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MobilizationOption, MobilizationOption::add)
}

impl MobilizationOption {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MobilizationOption, key, block, Box::new(Self {}));
    }
}

impl DbKind for MobilizationOption {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let sc_builder: &Builder = &|key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("military_formation", Scopes::MilitaryFormation, key);
            sc
        };

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("military_formation_type", &["army"]);
        vd.field_item("group", Item::MobilizationOptionGroup);

        vd.field_item("texture", Item::File);

        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_list_items("unlocking_laws", Item::LawType);
        vd.field_list_items("unlocking_principles", Item::Principle); // undocumented

        vd.field_validated_key_block("is_shown", |key, block, data| {
            validate_trigger(block, data, &mut sc_builder(key), Tooltipped::No);
        });
        vd.field_validated_key_block("possible", |key, block, data| {
            validate_trigger(block, data, &mut sc_builder(key), Tooltipped::Yes);
        });
        vd.field_validated_key_block("can_be_turned_off", |key, block, data| {
            validate_trigger(block, data, &mut sc_builder(key), Tooltipped::Yes);
        });

        for field in &[
            "on_activate",
            "on_deactivate",
            "on_activate_while_mobilized",
            "on_deactivate_while_mobilized",
        ] {
            vd.field_validated_key_block(field, |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::MilitaryFormation, key);
                validate_effect(block, data, &mut sc, Tooltipped::Yes);
            });
        }

        vd.field_validated_block("upkeep_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_validated_block("upkeep_modifier_unscaled", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_validated_block("unit_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        // Docs say it's Militaryformation, but the only example in vanilla contradicts that.
        vd.field_script_value_full("ai_weight", sc_builder, false);
    }
}

#[derive(Clone, Debug)]
pub struct MobilizationOptionGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MobilizationOptionGroup, MobilizationOptionGroup::add)
}

impl MobilizationOptionGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MobilizationOptionGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for MobilizationOptionGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_numeric("weight");
    }
}
