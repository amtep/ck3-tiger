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
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct FlagDefinition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::FlagDefinition, FlagDefinition::add)
}

impl FlagDefinition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let block = block.condense_tag("list");
        db.add(Item::FlagDefinition, key, block, Box::new(Self {}));
    }
}

impl DbKind for FlagDefinition {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("includes", Item::FlagDefinition);
        vd.multi_field_validated_block("flag_definition", validate_flag_definition);
    }
}

fn validate_flag_definition(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    for field in &["coa", "coa_with_overlord_canton", "subject_canton", "revolutionary_canton"] {
        if let Some(token) = vd.field_value(field) {
            if let Some((_, list)) = token.split_once('"') {
                data.verify_exists(Item::CoaTemplateList, &list);
            } else {
                data.verify_exists(Item::Coa, token);
            }
        }
    }
    vd.field_bool("allow_overlord_canton");
    vd.field_list_precise_numeric_exactly("overlord_canton_offset", 2);
    vd.field_list_precise_numeric_exactly("overlord_canton_scale", 2);
    vd.field_bool("allow_revolutionary_indicator");

    vd.field_integer("priority");
    vd.field_validated_key_block("trigger", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::CountryDefinition, key);
        sc.define_name("target", Scopes::Country, key);
        sc.define_name("initiator", Scopes::Country, key);
        sc.define_name("actor", Scopes::Country, key);
        sc.define_name("overlord", Scopes::Country, key);
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
}
