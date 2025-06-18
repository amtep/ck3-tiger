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
use crate::validate::validate_duration;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CompanyType {}
#[derive(Clone, Debug)]
pub struct DynamicCompanyName {}
#[derive(Clone, Debug)]
pub struct CompanyCharterType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CompanyType, CompanyType::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DynamicCompanyName, DynamicCompanyName::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CompanyCharterType, CompanyCharterType::add)
}

impl CompanyType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CompanyType, key, block, Box::new(Self {}));
    }
}

impl DynamicCompanyName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynamicCompanyName, key, block, Box::new(Self {}));
    }
}

impl CompanyCharterType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CompanyCharterType, key, block, Box::new(Self {}));
    }
}

impl DbKind for CompanyType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("icon", Item::File);
        vd.field_item("background", Item::File);

        vd.field_bool("flavored_company");
        vd.field_bool("uses_dynamic_naming");
        vd.field_list_items("preferred_headquarters", Item::StateRegion);

        vd.field_item("replaces_company", Item::CompanyType);
        vd.field_list_items("possible_prestige_goods", Item::PrestigeGoods);
        vd.field_trigger_rooted("prestige_goods_trigger", Tooltipped::No, Scopes::Company);

        if block.field_value_is("uses_dynamic_naming", "yes") {
            vd.field_list_items("dynamic_company_type_names", Item::Localization);
        } else {
            vd.ban_field("dynamic_company_type_names", || "uses_dynamic_naming = yes");
        }

        vd.field_list_items("building_types", Item::BuildingType);
        vd.field_list_items("extension_building_types", Item::BuildingType);

        vd.field_trigger_rooted("potential", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("attainable", Tooltipped::Yes, Scopes::Country);
        vd.field_trigger_rooted("possible", Tooltipped::Yes, Scopes::Country);

        vd.field_validated_block("prosperity_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_trigger_rooted("ai_will_do", Tooltipped::No, Scopes::Country);
        vd.field_validated_block("ai_construction_targets", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_blocks(Item::BuildingType, |_, block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer("level");
                vd.field_trigger_rooted("state_trigger", Tooltipped::No, Scopes::State);
            });
        });
        vd.field_script_value_rooted("ai_weight", Scopes::Country);

        // undocumented

        vd.field_list_items("unlocking_principles", Item::Principle);
    }
}

impl DbKind for DynamicCompanyName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_bool("uses_plural_naming");
        vd.field_bool("use_for_flavored_companies");
        vd.field_script_value_rooted("weight", Scopes::Country);
    }
}

impl DbKind for CompanyCharterType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_type_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_effects");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // "industry" is undocumented
        vd.field_choice(
            "type",
            &["industry", "investment", "monopoly", "new_industry", "trade", "colonization"],
        );
        vd.field_item("icon", Item::File);
        // TODO: which scope is it?
        let mut sc = ScopeContext::new(Scopes::Company | Scopes::Country, key);
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_trigger_rooted("ai_possible", Tooltipped::No, Scopes::Company);
        vd.field_script_value_no_breakdown_rooted("ai_weight", Scopes::Company);

        // undocumented

        vd.field_bool("additional_input");
    }
}
