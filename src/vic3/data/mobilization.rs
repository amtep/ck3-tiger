use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

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
        fn sc_builder(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("military_formation", Scopes::MilitaryFormation, key);
            sc
        }

        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("military_formation_type", &["army"]);
        vd.field_item("group", Item::MobilizationOptionGroup);

        vd.field_item("texture", Item::File);

        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_list_items("unlocking_laws", Item::LawType);
        vd.field_list_items("unlocking_principles", Item::Principle); // undocumented

        vd.field_trigger_builder("is_shown", Tooltipped::No, sc_builder);
        vd.field_trigger_builder("possible", Tooltipped::Yes, sc_builder);
        vd.field_trigger_builder("can_be_turned_off", Tooltipped::Yes, sc_builder);

        for field in &[
            "on_activate",
            "on_deactivate",
            "on_activate_while_mobilized",
            "on_deactivate_while_mobilized",
        ] {
            vd.field_effect_rooted(field, Tooltipped::Yes, Scopes::MilitaryFormation);
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
        vd.field_script_value_no_breakdown_builder("ai_weight", sc_builder);
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
