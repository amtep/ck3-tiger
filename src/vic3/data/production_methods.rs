use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ProductionMethod {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ProductionMethod, ProductionMethod::add)
}

impl ProductionMethod {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProductionMethod, key, block, Box::new(Self {}));
    }
}

impl DbKind for ProductionMethod {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("texture", Item::File);
        vd.field_bool("is_default");
        vd.field_bool("low_pop_method");

        vd.field_validated_block("country_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::all());
        });
        vd.field_validated_block("state_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::all());
        });
        vd.field_validated_block("building_modifiers", |block, data| {
            validate_modifier_block(block, data, ModifKinds::all());
        });
        vd.field_list_items("timed_modifiers", Item::Modifier);

        vd.field_list_items("unlocking_laws", Item::LawType);
        vd.field_list_items("disallowing_laws", Item::LawType);
        vd.field_list_items("unlocking_religions", Item::Religion);
        vd.field_list_items("disallowing_religions", Item::Religion);
        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_list_items("unlocking_production_methods", Item::ProductionMethod);
        vd.field_list_items("unlocking_global_technologies", Item::Technology);
        vd.field_list_items("unlocking_principles", Item::Principle);
        vd.multi_field_item("unlocking_identity", Item::PowerBlocIdentity);

        // TODO: verify ai_weight and ai_value; do both work?
        // Only ai_value is used in vanilla (once). ai_weight is not used.
        vd.field_numeric("ai_weight");
        vd.field_numeric("ai_value");
        vd.field_numeric("pollution_generation");

        // TODO: comment says "needs to match city_graphics_data.settings color array"
        vd.field_integer("city_lights_color_index");

        // undocumented

        vd.field_item("replacement_if_valid", Item::ProductionMethod);
        vd.field_bool("is_hidden_when_unavailable");
        vd.multi_field_item("required_input_goods", Item::Goods);
    }
}

fn validate_modifier_block(block: &Block, data: &Everything, kinds: ModifKinds) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("workforce_scaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
    vd.field_validated_block("level_scaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
    vd.field_validated_block("throughput_scaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
    vd.field_validated_block("unscaled", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, kinds, vd);
    });
}

#[derive(Clone, Debug)]
pub struct ProductionMethodGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ProductionMethodGroup, ProductionMethodGroup::add)
}

impl ProductionMethodGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProductionMethodGroup, key, block, Box::new(Self {}));
    }

    #[allow(clippy::unused_self)]
    pub fn contains_production_method(
        &self,
        contains_pm: &Token,
        block: &Block,
        _data: &Everything,
    ) -> bool {
        if let Some(pms) = block.get_field_list("production_methods") {
            return pms.iter().any(|pm| pm == contains_pm);
        }
        false
    }
}

impl DbKind for ProductionMethodGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        if !key.is("pmg_dummy") {
            vd.req_field("texture");
        }
        vd.field_item("texture", Item::File);
        vd.field_bool("is_hidden_when_unavailable");

        vd.req_field("production_methods");
        vd.field_list_items("production_methods", Item::ProductionMethod);

        vd.field_choice("ai_selection", &["most_profitable", "most_productive"]);
    }
}
