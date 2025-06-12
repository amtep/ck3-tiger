use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct LawGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::LawGroup, LawGroup::add)
}

impl LawGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LawGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for LawGroup {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (key, block) in block.iter_definitions() {
            for token in block.get_field_values("flag") {
                db.add_flag(Item::LawFlag, token.clone());
            }
            for block in block.get_field_blocks("triggered_flag") {
                if let Some(token) = block.get_field_value("flag") {
                    db.add_flag(Item::LawFlag, token.clone());
                }
            }
            db.add(Item::Law, key.clone(), block.clone(), Box::new(Law {}));
        }
        for token in block.get_field_values("flag") {
            db.add_flag(Item::LawFlag, token.clone());
        }
    }

    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if let Some(token) = vd.field_value("default") {
            if block.get_field_block(token.as_str()).is_none() {
                let msg = "law not defined in this group";
                err(ErrorKey::MissingItem).msg(msg).loc(token).push();
            }
        }
        vd.field_bool("cumulative");

        vd.multi_field_value("flag");
        // The laws. They are validated in the Law class.
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct Law {}

impl Law {}

impl DbKind for Law {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_effects");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_effects_not_in_prev");
        data.mark_used(Item::Localization, &loca);

        vd.field_item("pass_phrase", Item::Localization);
        vd.field_item("confirmation_title", Item::Localization);
        vd.field_item("confirmation_button_text", Item::Localization);

        vd.field_trigger_rooted("can_keep", Tooltipped::No, Scopes::Character);
        vd.field_trigger_rooted("can_have", Tooltipped::No, Scopes::Character);
        vd.field_trigger_rooted("can_pass", Tooltipped::Yes, Scopes::Character);
        vd.field_trigger_rooted("should_start_with", Tooltipped::Yes, Scopes::Character);
        vd.field_trigger_rooted("can_title_have", Tooltipped::Yes, Scopes::LandedTitle);
        vd.field_trigger_rooted("should_show_for_title", Tooltipped::No, Scopes::LandedTitle);
        vd.field_trigger_rooted("can_remove_from_title", Tooltipped::Yes, Scopes::LandedTitle);

        vd.field_validated_block_rooted("pass_cost", Scopes::Character, validate_cost);
        vd.field_validated_block_rooted("revoke_cost", Scopes::Character, validate_cost);

        vd.multi_field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.multi_field_value("flag");
        vd.field_validated_block("triggered_flag", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("trigger");
            vd.req_field("flag");
            vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Character);
            vd.field_value("flag");
        });

        let title_law = block.has_key("can_title_have");
        let sc_builder = |key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            if title_law {
                sc.define_name("title", Scopes::LandedTitle, key);
            }
            sc
        };

        vd.field_effect_builder("on_pass", Tooltipped::Yes, sc_builder);
        vd.field_effect_builder("on_revoke", Tooltipped::Yes, sc_builder);

        vd.field_validated_block("succession", |block, data| {
            let mut vd = Validator::new(block, data);
            // "generate" and "player_heir" are undocumented
            vd.field_choice(
                "order_of_succession",
                &[
                    "inheritance",
                    "election",
                    "appointment",
                    "theocratic",
                    "company",
                    "generate",
                    "generate_from_template",
                    "player_heir",
                    "noble_family",
                ],
            );
            let order_of_succession =
                block.get_field_value("order_of_succession").map_or("none", Token::as_str);

            if order_of_succession == "inheritance" {
                vd.field_choice("title_division", &["partition", "single_heir"]);
            } else {
                vd.ban_field("title_division", || "order_of_succession = inheritance");
            }

            // TODO: children may only be used if title_division == partition
            vd.field_choice("traversal_order", &["children", "dynasty_house", "dynasty"]);
            vd.field_choice("rank", &["oldest", "youngest"]);
            if let Some(title_division) = block.get_field_value("title_division") {
                if let Some(traversal_order) = block.get_field_value("traversal_order") {
                    if title_division.is("partition") && !traversal_order.is("children") {
                        let msg = "partition is only for `traversal_order = children`";
                        err(ErrorKey::Validation).msg(msg).loc(title_division).push();
                    }
                }
            }

            if order_of_succession == "theocratic"
                || order_of_succession == "company"
                || order_of_succession == "generate"
            {
                vd.field_item("pool_character_config", Item::PoolSelector);
            } else {
                vd.ban_field("pool_character_config", || {
                    "theocratic, company, or generate succession"
                });
            }

            if order_of_succession == "election" {
                vd.field_item("election_type", Item::SuccessionElection);
            } else {
                vd.ban_field("election_type", || "order_of_succession = election");
            }

            if order_of_succession == "appointment" {
                vd.field_item("appointment_type", Item::SuccessionAppointment);
            } else {
                vd.ban_field("appointment_type", || "order_of_succession = appointment");
            }

            vd.field_choice(
                "gender_law",
                &["male_only", "male_preference", "equal", "female_preference", "female_only"],
            );
            vd.field_choice("faith", &["same_faith", "same_religion", "same_family"]);
            vd.field_bool("create_primary_tier_titles");
            vd.field_numeric("primary_heir_minimum_share");
            vd.field_bool("exclude_rulers");
            vd.field_bool("limit_to_courtiers");
        });

        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);

        // undocumented

        vd.field_bool("shown_in_encyclopedia");
        vd.field_integer("title_allegiance_opinion");
        vd.field_trigger_rooted("potential", Tooltipped::No, Scopes::Character);
        vd.field_trigger_rooted("requires_approve", Tooltipped::No, Scopes::Character);
        // TODO: should be Item::WidgetName, but the name used in vanilla (widget_clan_law) is not
        // recognized by the gui parser because it's hidden in a type declaration.
        vd.field_value("widget_name");
    }
}
