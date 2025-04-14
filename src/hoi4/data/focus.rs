use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct NationalFocusTree {}
#[derive(Clone, Debug)]
pub struct NationalFocus {}
#[derive(Clone, Debug)]
pub struct NationalFocusStyle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::NationalFocusTree, NationalFocusTree::add)
}

impl NationalFocusTree {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("focus_tree") {
            if let Some(id) = block.get_field_value("id") {
                for block in block.get_field_blocks("focus") {
                    if let Some(id) = block.get_field_value("id") {
                        db.add(
                            Item::NationalFocus,
                            id.clone(),
                            block.clone(),
                            Box::new(NationalFocus {}),
                        );
                    } else {
                        let msg = "focus without id";
                        err(ErrorKey::FieldMissing).msg(msg).loc(block).push();
                    }
                }
                db.add(Item::NationalFocusTree, id.clone(), block, Box::new(Self {}));
            } else {
                let msg = "focus tree without id";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else if key.is("shared_focus") || key.is("joint_focus") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::NationalFocus, id.clone(), block, Box::new(NationalFocus {}));
            } else {
                let msg = "focus without id";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else if key.is("style") {
            if let Some(name) = block.get_field_value("name") {
                db.add(
                    Item::NationalFocusStyle,
                    name.clone(),
                    block,
                    Box::new(NationalFocusStyle {}),
                );
            } else {
                let msg = "style without name";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for NationalFocusTree {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("id");

        let mut sc = ScopeContext::new(Scopes::Country, key);
        vd.field_validated_block_sc("country", &mut sc, validate_modifiers_with_base);

        vd.field_bool("default");

        for field in &["initial_show_position", "continuous_focus_position"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer("x");
                vd.field_integer("y");
            });
        }

        vd.multi_field_item("shared_focus", Item::NationalFocus);

        vd.multi_field("focus"); // validated by NationalFocus item
    }
}

impl DbKind for NationalFocus {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("id");
        vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(value) => {
                data.verify_exists(Item::Sprite, value);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_item("value", Item::Sprite);
                vd.field_trigger_full("trigger", Scopes::Country, Tooltipped::No);
            }
        });

        vd.field_trigger_full("allow_branch", Scopes::Country, Tooltipped::Yes);
        for field in &["mutually_exclusive", "prerequisite"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.multi_field_item("focus", Item::NationalFocus);
            });
        }
        vd.field_integer("x");
        vd.field_integer("y");
        vd.field_item("relative_position_id", Item::NationalFocus);
        vd.field_integer("cost");
        vd.field_trigger_full("bypass", Scopes::Country, Tooltipped::Yes);
        vd.field_trigger_full("available", Scopes::Country, Tooltipped::Yes);

        vd.field_list_choice("search_filters", SEARCH_FILTERS);
        vd.field_effect_full("completion_reward", Scopes::Country, Tooltipped::Yes);
    }
}

impl DbKind for NationalFocusStyle {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_bool("default");
        vd.field_item("unavailable", Item::Sprite);
        vd.field_item("completed", Item::Sprite);
        vd.field_item("available", Item::Sprite);
        vd.field_item("current", Item::Sprite);
    }
}

// LAST UPDATED HOI4 VERSION 1.16
// Taken from localisation/english/focus_filter_tag_l_english.yml
pub const SEARCH_FILTERS: &[&str] = &[
    "FOCUS_FILTER_AIR_XP",
    "FOCUS_FILTER_ANNEXATION",
    "FOCUS_FILTER_ARMY_XP",
    "FOCUS_FILTER_BALANCE_OF_POWER",
    "FOCUS_FILTER_CHI_INFLATION",
    "FOCUS_FILTER_FOLKHEMMET",
    "FOCUS_FILTER_FRA_OCCUPATION_COST",
    "FOCUS_FILTER_FRA_POLITICAL_VIOLENCE",
    "FOCUS_FILTER_GRE_DEBT_TO_IFC",
    "FOCUS_FILTER_HISTORICAL",
    "FOCUS_FILTER_INDUSTRY",
    "FOCUS_FILTER_INNER_CIRCLE",
    "FOCUS_FILTER_INTERNAL_AFFAIRS",
    "FOCUS_FILTER_INTERNATIONAL_TRADE",
    "FOCUS_FILTER_ITA_MISSIOLINI",
    "FOCUS_FILTER_MANPOWER",
    "FOCUS_FILTER_MEX_CAUDILLO_REBELLION",
    "FOCUS_FILTER_MEX_CHURCH_AUTHORITY",
    "FOCUS_FILTER_MILITARY_CHARACTER",
    "FOCUS_FILTER_NAVY_XP",
    "FOCUS_FILTER_POLITICAL",
    "FOCUS_FILTER_POLITICAL_CHARACTER",
    "FOCUS_FILTER_PROPAGANDA",
    "FOCUS_FILTER_RESEARCH",
    "FOCUS_FILTER_SOV_POLITICAL_PARANOIA",
    "FOCUS_FILTER_SPA_CARLIST_UPRISING",
    "FOCUS_FILTER_SPA_CIVIL_WAR",
    "FOCUS_FILTER_STABILITY",
    "FOCUS_FILTER_SWI_MILITARY_READINESS",
    "FOCUS_FILTER_TFV_AUTONOMY",
    "FOCUS_FILTER_TUR_KEMALISM",
    "FOCUS_FILTER_TUR_KURDISTAN",
    "FOCUS_FILTER_TUR_TRADITIONALISM",
    "FOCUS_FILTER_USA_CONGRESS",
    "FOCUS_FILTER_WAR_SUPPORT",
];
