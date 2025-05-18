use crate::block::Block;
use crate::ck3::validate::validate_cost;
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
pub struct CourtType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CourtType, CourtType::add)
}

impl CourtType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtType, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_tooltip_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_bool("default");
        vd.field_item("background", Item::File);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("is_valid", Tooltipped::Yes, &mut sc);

        vd.multi_field_validated_block("level_perk", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("court_grandeur");
            validate_court_modifiers(vd);
        });

        vd.multi_field_validated_block("time_perk", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("required_months_in_court");
            validate_court_modifiers(vd);
        });

        vd.field_validated_block_sc("cost", &mut sc, validate_cost);

        vd.field_script_value("ai_will_do", &mut sc);
    }
}

fn validate_court_modifiers(mut vd: Validator) {
    vd.field_validated_block("owner_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_item("owner_modifier_description", Item::Localization);

    vd.field_validated_block("courtier_guest_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_item("courtier_guest_modifier_description", Item::Localization);
}
