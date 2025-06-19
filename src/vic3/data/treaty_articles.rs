use crate::block::Block;
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
pub struct TreatyArticle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::TreatyArticle, TreatyArticle::add)
}

impl TreatyArticle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TreatyArticle, key, block, Box::new(Self {}));
    }
}

impl DbKind for TreatyArticle {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        data.verify_exists_implied(Item::Localization, &format!("{key}_desc"), key);
        data.verify_exists_implied(Item::Localization, &format!("{key}_effects_desc"), key);
        data.verify_exists_implied(Item::Localization, &format!("{key}_article_short_desc"), key);

        vd.req_field("kind");
        vd.field_choice("kind", &["directed", "mutual"]);
        let is_directed = block.get_field_value("kind").is_some_and(|k| k.is("directed"));

        vd.field_integer_range("cost", 0..);
        vd.field_numeric("relations_progress_per_day");
        vd.field_integer_range("relations_improvement_max", 0..);
        vd.field_integer_range("relations_improvement_min", 0..);

        vd.field_item("icon", Item::File);

        let flags = &[
            "is_alliance",
            "is_defensive_pact",
            "is_guarantee_independence",
            "is_support_independence",
            "is_host_power_bloc_embassy",
            "is_offer_embassy",
            "is_investment_rights",
            "is_join_power_bloc",
            "is_trade_privilege",
            "is_military_access",
            "is_military_assistance",
            "is_transit_rights",
            "is_non_colonization_agreement",
            "is_goods_transfer",
            "is_money_transfer",
            "is_monopoly_for_company",
            "is_prohibit_goods_trade_with_world_market",
            "is_no_tariffs",
            "is_no_subventions",
            "is_take_on_debt",
            "is_treaty_port",
            "is_law_commitment",
            "can_be_renegotiated",
            "can_be_enforced",
            "causes_state_transfer",
            "recipient_pays_maintenance",
        ];
        vd.field_list_choice("flags", flags);

        vd.field_choice(
            "usage_limit",
            &["once_per_treaty", "once_per_side", "once_per_side_with_same_inputs"],
        );

        vd.field_choice("maintenance_paid_by", &["target_country", "source_country"]);

        let required_inputs = &[
            "quantity",
            "goods",
            "state",
            "strategic_region",
            "company",
            "building_type",
            "law_type",
            "country",
        ];
        vd.field_list_choice("required_inputs", required_inputs);

        let required_inputs = block.get_field_list("required_inputs");
        let required_inputs = required_inputs.as_ref().map_or(&[][..], Vec::as_slice);
        for input in required_inputs {
            if input.is("quantity") {
                fn build_quantity_sc(key: &Token, is_directed: bool) -> ScopeContext {
                    let mut sc = build_article_sc(key, is_directed);
                    sc.define_name("other_country", Scopes::Country, key);
                    sc
                }

                vd.field_script_value_builder("quantity_min_value", |key| {
                    build_quantity_sc(key, is_directed)
                });
                vd.field_script_value_builder("quantity_max_value", |key| {
                    build_quantity_sc(key, is_directed)
                });
            } else {
                let valid_trigger = format!("{input}_valid_trigger");
                vd.field_trigger_builder(&valid_trigger, Tooltipped::No, |key| {
                    build_input_sc(key, input)
                });
            }
        }

        vd.field_list_items("mutual_exclusions", Item::TreatyArticle);
        vd.field_list_items("unlocked_by_technologies", Item::Technology);
        vd.field_list_items("automatically_support", Item::DiplomaticPlay);
        vd.field_validated_key_block("non_fulfillment", |key, block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("consequences", &["none", "withdraw", "freeze"]);
            let is_consequences_none =
                block.get_field_value("consequences").is_none_or(|t| t.is("none"));
            if is_consequences_none {
                vd.ban_field("max_consecutive_contraventions", || "withdraw/freeze");
                vd.ban_field("conditions", || "withdraw/freeze");
            } else if block
                .get_field_block("conditions")
                .is_none_or(|b| b.num_items() == 0 || b.field_value_is("always", "no"))
            {
                let msg = "at least one of the conditions triggers must be non-empty";
                err(ErrorKey::Validation).msg(msg).loc(key).push();
            }

            vd.field_integer_range("max_consecutive_contraventions", 0..);
            vd.field_validated_block("conditions", |block, data| {
                let mut vd = Validator::new(block, data);
                for interval in &["weekly", "monthly", "yearly"] {
                    vd.field_trigger_builder(interval, Tooltipped::No, |key| {
                        let mut sc = ScopeContext::new(Scopes::Country, key);
                        sc.define_name("article", Scopes::TreatyArticle, key);
                        sc
                    });
                }
            });
        });

        if !is_directed {
            vd.ban_field("source_modifier", || "directed");
            vd.ban_field("target_modifier", || "directed");
        }

        for modifier in &["source_modifier", "target_modifier", "mutual_modifier"] {
            vd.field_validated_block(modifier, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::all(), vd);
            });
        }

        vd.field_trigger_rooted("visible", Tooltipped::Yes, Scopes::Country);
        vd.field_trigger_builder("possible", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("other_country", Scopes::Country, key);
            sc
        });
        vd.field_trigger_builder("can_ratify", Tooltipped::Yes, |key| {
            build_article_treaty_sc(key, is_directed)
        });

        for active in &["on_entry_into_force", "on_enforced"] {
            vd.field_effect_builder(active, Tooltipped::Yes, |key| {
                let mut sc = ScopeContext::new(Scopes::None, key);
                sc.define_name("treaty_options", Scopes::TreatyOptions, key);
                sc.define_name("article_options", Scopes::TreatyArticleOptions, key);
                sc
            });
        }

        vd.field_trigger_builder("can_withdraw", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("withdrawing_country", Scopes::Country, key);
            sc.define_name("non_withdrawing_country", Scopes::Country, key);

            if is_directed {
                sc.define_name("source_country", Scopes::Country, key);
                sc.define_name("target_country", Scopes::Country, key);
            } else {
                sc.define_name("first_country", Scopes::Country, key);
                sc.define_name("second_country", Scopes::Country, key);
            }
            sc
        });

        for deactive in &["on_withdrawal", "on_break"] {
            vd.field_effect_builder(deactive, Tooltipped::Yes, |key| {
                let mut sc = ScopeContext::new(Scopes::None, key);
                sc.define_name("treaty_options", Scopes::TreatyOptions, key);
                sc.define_name("article", Scopes::TreatyArticle, key);
                sc.define_name("withdrawing_country", Scopes::Country, key);
                sc.define_name("non_withdrawing_country", Scopes::Country, key);
                sc
            });
        }

        vd.field_validated_block("ai", |block, data| {
            validate_ai(block, data, required_inputs, is_directed);
        });

        vd.field_validated_block("wargoal", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("execution_priority");
            let contestion_types = &[
                "control_target_state",
                "control_target_country_capital",
                "control_any_target_country_state",
                "control_own_state",
                "control_own_capital",
                "control_all_own_states",
                "control_all_target_country_claims",
            ];
            vd.field_choice("contestion_type", contestion_types);

            for script_value in &["maneuvers", "infamy"] {
                vd.field_script_value_builder(script_value, |key| {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    sc.define_name("target_country", Scopes::Country, key);
                    for input in required_inputs {
                        match input.as_str() {
                            "quantity" => sc.define_name("quantity", Scopes::Value, key),
                            "goods" => {
                                sc.define_name("goods", Scopes::Goods, key);
                                sc.define_name("market_goods", Scopes::MarketGoods, key);
                            }
                            "state" => sc.define_name("state", Scopes::State, key),
                            "strategic_region" => {
                                sc.define_name("region", Scopes::StrategicRegion, key);
                            }
                            "company" => sc.define_name("company", Scopes::Company, key),
                            "country" => sc.define_name("country", Scopes::Country, key),
                            // TODO: verify whether X or X_type scopes
                            "building_type" => sc.define_name("building", Scopes::Building, key),
                            "law_type" => sc.define_name("law", Scopes::Law, key),
                            _ => unreachable!(),
                        }
                    }
                    sc
                });
            }
        });
    }
}

fn build_input_sc(key: &Token, input: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("other_country", Scopes::Country, key);
    sc.define_name("article", Scopes::TreatyArticle, key);
    let input_scope = match input.as_str() {
        "goods" => Scopes::Goods,
        "state" => Scopes::State,
        "strategic_region" => Scopes::StrategicRegion,
        "company" => Scopes::Company,
        "building_type" => Scopes::BuildingType,
        "law_type" => Scopes::LawType,
        "country" => Scopes::Country,
        _ => unreachable!(),
    };
    sc.define_name("input", input_scope, key);

    if input.is("goods") {
        sc.define_name("market_goods", Scopes::MarketGoods, key);
    }
    sc
}

fn build_article_sc(key: &Token, is_directed: bool) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("article", Scopes::TreatyArticle, key);

    if is_directed {
        sc.define_name("source_country", Scopes::Country, key);
        sc.define_name("target_country", Scopes::Country, key);
    } else {
        sc.define_name("first_country", Scopes::Country, key);
        sc.define_name("second_country", Scopes::Country, key);
    }
    sc
}

fn build_article_treaty_sc(key: &Token, is_directed: bool) -> ScopeContext {
    let mut sc = build_article_sc(key, is_directed);
    sc.define_name("treaty", Scopes::Treaty, key);
    sc
}

fn validate_ai(block: &Block, data: &Everything, required_inputs: &[Token], is_directed: bool) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value_rooted("evaluation_chance", Scopes::Country);

    for input in required_inputs {
        if input.is("quantity") {
            vd.field_script_value_builder("quantity_input_value", |key| {
                let mut sc = ScopeContext::new(Scopes::Country, key);
                sc.define_name("other_country", Scopes::Country, key);
                sc.define_name("article", Scopes::TreatyArticle, key);
                sc
            });
        } else {
            let filter = format!("{input}_input_filter");
            vd.field_trigger_builder(&filter, Tooltipped::No, |key| build_input_sc(key, input));
        }
    }

    vd.field_list_choice("article_ai_usage", &["offer", "request"]);
    let categories = &[
        "economy",
        "trade",
        "military",
        "military_defense",
        "ideology",
        "expansion",
        "power_bloc",
        "other",
        "none",
    ];
    vd.field_list_choice("treaty_categories", categories);
    vd.field_integer("combined_acceptance_cap_max");
    vd.field_integer("combined_acceptance_cap_min");
    vd.field_script_value_builder("inherent_accept_score", |key| {
        build_article_sc(key, is_directed)
    });
    vd.field_script_value_builder("contextual_accept_score", |key| {
        build_article_treaty_sc(key, is_directed)
    });

    vd.field_script_value_builder("wargoal_score_multiplier", |key| {
        build_article_sc(key, is_directed)
    });
}
