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
pub struct DiarchyType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DiarchyType, DiarchyType::add)
}

impl DiarchyType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiarchyType, key, block, Box::new(Self {}));
    }
}

impl DbKind for DiarchyType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for block in block.get_field_blocks("power_level") {
            for token in block.get_field_values("parameter") {
                db.add_flag(Item::DiarchyParameter, token.clone());
            }
            for token in block.get_field_values("hidden_parameter") {
                db.add_flag(Item::DiarchyParameter, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key); // TODO scope type

        let loca = format!("{key}_diarchy_type");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_diarch_title");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("start", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("end", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_bool("succession");

        vd.field_script_value_rooted("candidate_score", Scopes::Character);
        // TODO: "Please avoid circular dependencies and don't use aptitude for mandate qualifications"
        vd.field_script_value_rooted("aptitude_score", Scopes::Character);
        vd.field_script_value_rooted("loyalty_score", Scopes::Character);

        vd.multi_field_item("mandate", Item::DiarchyMandate);

        vd.multi_field_validated_block("power_level", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("swing");
            vd.multi_field_item("parameter", Item::Localization);
            vd.multi_field_value("hidden_parameter"); // TODO: localization?
        });
        vd.field_script_value_rooted("swing_balance", Scopes::Character);

        vd.field_item("end_interaction", Item::CharacterInteraction);

        vd.multi_field_validated_block("liege_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            vd.field_script_value_rooted("scale", Scopes::Character);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("diarch_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            vd.field_script_value_rooted("scale", Scopes::Character);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
    }
}

#[derive(Clone, Debug)]
pub struct DiarchyMandate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DiarchyMandate, DiarchyMandate::add)
}

impl DiarchyMandate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiarchyMandate, key, block, Box::new(Self {}));
    }
}

impl DbKind for DiarchyMandate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_mandate");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_mandate_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_script_value_rooted("qualification_score", Scopes::Character);
        vd.field_script_value_rooted("ai_score", Scopes::Character);
    }
}
