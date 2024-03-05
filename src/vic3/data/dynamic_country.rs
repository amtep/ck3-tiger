use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DynamicCountryMapColor {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DynamicCountryMapColor, DynamicCountryMapColor::add)
}

impl DynamicCountryMapColor {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynamicCountryMapColor, key, block, Box::new(Self {}));
    }
}

impl DbKind for DynamicCountryMapColor {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated("color", validate_possibly_named_color);

        vd.field_validated_key_block("possible", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct DynamicCountryName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DynamicCountryName, DynamicCountryName::add)
}

impl DynamicCountryName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynamicCountryName, key, block, Box::new(Self {}));
    }
}

impl DbKind for DynamicCountryName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !key.is("DEFAULT") {
            data.verify_exists(Item::Country, key);
        }

        vd.multi_field_validated_block("dynamic_country_name", validate_dynamic_country_name);
    }
}

fn validate_dynamic_country_name(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_item("name", Item::Localization);
    vd.field_item("adjective", Item::Localization);

    vd.field_bool("is_revolutionary");
    vd.field_bool("is_main_tag_only");
    vd.field_bool("use_overlord_prefix");
    vd.field_integer("priority");

    vd.field_validated_key_block("trigger", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("actor", Scopes::Country, key);
        sc.define_name("overlord", Scopes::Country, key); // TODO: verify
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
}
