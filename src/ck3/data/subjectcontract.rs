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
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SubjectContract {}
#[derive(Clone, Debug)]
pub struct SubjectContractGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SubjectContract, SubjectContract::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SubjectContractGroup, SubjectContractGroup::add)
}

impl SubjectContract {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SubjectContract, key, block, Box::new(Self {}));
    }
}

impl SubjectContractGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SubjectContractGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for SubjectContract {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("obligation_levels") {
            for (key, block) in block.iter_definitions() {
                for token in block.get_field_values("flag") {
                    db.add_flag(Item::SubjectContractFlag, token.clone());
                }
                if !key.is("default") {
                    db.add_flag(Item::SubjectContractObligationLevel, key.clone());
                }
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        sc.define_name("liege", Scopes::Character, key);
        sc.define_name("subject", Scopes::Character, key);
        sc.define_name("vassal", Scopes::Character, key);
        sc.define_name("tax_slot", Scopes::TaxSlot, key);
        sc.define_name("tax_collector", Scopes::Character, key);

        vd.field_bool("uses_opinion_of_liege");
        if let Some(token) = block.get_field_value("uses_opinion_of_liege") {
            if token.is("yes") {
                sc.define_name("opinion_of_liege", Scopes::Value, token);
            }
        }

        vd.field_choice("display_mode", &["tree", "list", "radiobutton", "checkbox"]);
        vd.field_item("icon", Item::TextIcon);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);

        vd.field_validated_block("obligation_levels", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                if !key.is("default") {
                    data.verify_exists(Item::Localization, key);
                    let loca = format!("{key}_short");
                    data.verify_exists_implied(Item::Localization, &loca, key);
                    let loca = format!("{key}_desc");
                    data.mark_used(Item::Localization, &loca);
                }

                let mut vd = Validator::new(block, data);
                vd.field_bool("default");
                vd.field_item("parent", Item::SubjectContractObligationLevel);
                vd.field_list_numeric_exactly("position", 2);
                vd.field_item("icon", Item::File);
                vd.field_script_value("levies", &mut sc);
                vd.field_script_value("tax", &mut sc);
                vd.field_script_value("herd", &mut sc);
                vd.field_script_value("prestige", &mut sc);
                vd.field_script_value("min_levies", &mut sc);
                vd.field_script_value("min_tax", &mut sc);
                vd.field_script_value("min_herd", &mut sc);
                vd.field_validated_sc("contribution_desc", &mut sc, validate_desc);
                vd.field_item("tax_contribution_postfix", Item::Localization);
                vd.field_item("levies_contribution_postfix", Item::Localization);
                vd.field_item("herd_contribution_postfix", Item::Localization);
                vd.field_item("unclamped_contribution_label", Item::Localization);
                vd.field_item("min_contribution_label", Item::Localization);
                vd.advice_field("vassal_opinion", "replaced with `subject_opinion` in 1.16");
                vd.field_integer("subject_opinion");
                vd.multi_field_value("flag");
                vd.field_integer("score");
                vd.field_validated("color", validate_possibly_named_color);
                vd.field_script_value("ai_liege_desire", &mut sc);
                vd.advice_field("ai_vassal_desire", "replaced with `ai_subject_desire` in 1.16");
                vd.field_script_value("ai_subject_desire", &mut sc);
                vd.field_validated_block("liege_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.advice_field("vassal_modifier", "replaced with `subject_modifier` in 1.16");
                vd.field_validated_block("subject_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
                vd.field_trigger("is_valid", Tooltipped::Yes, &mut sc);
                vd.field_script_value("tax_factor", &mut sc);
                vd.field_script_value("levies_factor", &mut sc);
                vd.field_script_value("herd_factor", &mut sc);
            });
        });
    }
}

impl DbKind for SubjectContractGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.field_list_items("contracts", Item::SubjectContract);
        vd.field_value("modify_contract_layout");
        vd.field_bool("is_tributary");

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("suzerain", Scopes::Character, key);

        if block.get_field_bool("is_tributary").unwrap_or(false) {
            vd.field_trigger("is_valid_tributary_contract", Tooltipped::Yes, &mut sc);
            vd.field_trigger("tributary_can_break_free", Tooltipped::Yes, &mut sc);
            vd.field_item("suzerain_line_type", Item::LineType);
            vd.field_item("tributary_line_type", Item::LineType);
            vd.field_bool("should_show_as_suzerain_realm_name");
            vd.field_bool("should_show_as_suzerain_realm_color");
            vd.field_bool("tributary_heir_succession");
            vd.field_bool("suzerain_heir_succession");
        } else {
            vd.ban_field("is_valid_tributary_contract", || "is_tributary = yes");
            vd.ban_field("tributary_can_break_free", || "is_tributary = yes");
            vd.ban_field("suzerain_line_type", || "is_tributary = yes");
            vd.ban_field("tributary_line_type", || "is_tributary = yes");
            vd.ban_field("should_show_as_suzerain_realm_name", || "is_tributary = yes");
            vd.ban_field("should_show_as_suzerain_realm_color", || "is_tributary = yes");
            vd.ban_field("tributary_heir_succession", || "is_tributary = yes");
            vd.ban_field("suzerain_heir_succession", || "is_tributary = yes");
        }
    }
}
