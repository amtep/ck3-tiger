use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct VassalContract {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::VassalContract, VassalContract::add)
}

impl VassalContract {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::VassalContract, key, block, Box::new(Self {}));
    }
}

impl DbKind for VassalContract {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("obligation_levels") {
            for (key, block) in block.iter_definitions() {
                for token in block.get_field_values("flag") {
                    db.add_flag(Item::VassalContractFlag, token.clone());
                }
                if !key.is("default") {
                    db.add_flag(Item::VassalObligationLevel, key.clone());
                }
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        sc.define_name("liege", Scopes::Character, key);
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
        vd.field_value("icon"); // TODO
        vd.field_validated_block("is_shown", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

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
                vd.field_item("parent", Item::VassalObligationLevel);
                vd.field_list_numeric_exactly("position", 2);
                vd.field_item("icon", Item::File);
                vd.field_script_value("levies", &mut sc);
                vd.field_script_value("tax", &mut sc);
                vd.field_script_value("min_levies", &mut sc);
                vd.field_script_value("min_tax", &mut sc);
                vd.field_validated_sc("contribution_desc", &mut sc, validate_desc);
                vd.field_item("tax_contribution_postfix", Item::Localization);
                vd.field_item("levies_contribution_postfix", Item::Localization);
                vd.field_item("unclamped_contribution_label", Item::Localization);
                vd.field_item("min_contribution_label", Item::Localization);
                vd.field_integer("vassal_opinion");
                vd.multi_field_value("flag");
                vd.field_integer("score");
                vd.field_validated("color", validate_possibly_named_color);
                vd.field_script_value("ai_liege_desire", &mut sc);
                vd.field_script_value("ai_vassal_desire", &mut sc);
                vd.field_validated_block("liege_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.field_validated_block("vassal_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.field_validated_block("is_shown", |block, data| {
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_validated_block("is_valid", |block, data| {
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });

                vd.field_script_value("tax_factor", &mut sc);
                vd.field_script_value("levies_factor", &mut sc);
            });
        });
    }
}
