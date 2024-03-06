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
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CompanyType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CompanyType, CompanyType::add)
}

impl CompanyType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CompanyType, key, block, Box::new(Self {}));
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

        vd.field_item("replaces_company", Item::CompanyType);

        if block.field_value_is("uses_dynamic_naming", "yes") {
            vd.field_list_items("dynamic_company_type_names", Item::Localization);
        } else {
            vd.ban_field("dynamic_company_type_names", || "uses_dynamic_naming = yes");
        }

        vd.field_list_items("building_types", Item::BuildingType);

        vd.field_validated_key_block("potential", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("attainable", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_key_block("possible", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("prosperity_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        let mut sc = ScopeContext::new(Scopes::Country, key);
        vd.field_script_value("ai_weight", &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct DynamicCompanyName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DynamicCompanyName, DynamicCompanyName::add)
}

impl DynamicCompanyName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynamicCompanyName, key, block, Box::new(Self {}));
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
