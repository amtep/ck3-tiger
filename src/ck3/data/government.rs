use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Government {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::GovernmentType, Government::add)
}

impl Government {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GovernmentType, key, block, Box::new(Self {}));
    }
}

impl DbKind for Government {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::GovernmentFlag, token.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        // let modif = format!("{key}_levy_contribution_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_levy_contribution_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_tax_contribution_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_tax_contribution_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_vassal_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_opinion_same_faith");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_adjective");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_realm");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        if block.has_key("vassal_contract") {
            let loca = format!("{key}_vassals_label");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_bool("create_cadet_branches");
        vd.field_bool("religious");
        vd.field_bool("rulers_should_have_dynasty");
        vd.field_bool("council");
        vd.field_integer("fallback");
        vd.field_bool("always_use_patronym");

        vd.field_validated_block("can_get_government", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_item("primary_holding", Item::HoldingType);
        vd.field_list_items("valid_holdings", Item::HoldingType);
        vd.field_list_items("required_county_holdings", Item::HoldingType);

        vd.field_list_items("primary_cultures", Item::Culture);
        vd.field_list_items("primary_heritages", Item::CultureHeritage);

        vd.field_list_items("preferred_religions", Item::Religion);

        vd.field_bool("court_generate_spouses");
        if let Some(token) = vd.field_value("court_generate_commanders") {
            if !token.is("yes") && !token.is("no") {
                token.expect_number();
            }
        }
        vd.field_bool("royal_court");
        vd.field_numeric("supply_limit_mult_for_others");
        vd.field_bool("affected_by_development");
        vd.field_bool("regiments_prestige_as_gold");

        vd.field_validated_block("prestige_opinion_override", |block, data| {
            let mut vd = Validator::new(block, data);
            for token in vd.values() {
                token.expect_number();
            }
        });

        vd.field_list_items("vassal_contract", Item::VassalContract);
        vd.field_item("house_unity", Item::HouseUnity);
        vd.field_item("tax_slot_type", Item::TaxSlotType); // undocumented
        vd.field_validated_block("ai", validate_ai);
        vd.multi_field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("color", validate_color);

        // undocumented

        vd.multi_field_value("flag");
        vd.field_bool("dynasty_named_realms");
        vd.field_script_value_rooted("opinion_of_liege", Scopes::Character);
        vd.field_validated_key("opinion_of_liege_desc", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("vassal", Scopes::Character, key);
            sc.define_name("liege", Scopes::Character, key);
            validate_desc(bv, data, &mut sc);
        });
    }
}

fn validate_ai(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("use_lifestyle");
    vd.field_bool("imprison");
    vd.field_bool("start_murders");
    vd.field_bool("arrange_marriage");
    vd.field_bool("use_goals");
    vd.field_bool("use_decisions");
    vd.field_bool("use_scripted_guis");
    vd.field_bool("use_legends");
    vd.field_bool("perform_religious_reformation");
}
