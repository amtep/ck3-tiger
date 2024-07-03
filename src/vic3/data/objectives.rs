use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Objective {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Objective, Objective::add)
}

impl Objective {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Objective, key, block, Box::new(Self {}));
    }
}

impl DbKind for Objective {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.replaced_field("background", "enabled_background and disabled_background");
        vd.field_item("enabled_background", Item::File);
        vd.field_item("disabled_background", Item::File);

        vd.field_list_items("recommended_tags", Item::Country);
        vd.field_bool("available_for_all_countries");
        vd.field_list_items("recommended_game_rules", Item::GameRuleSetting); // undocumented
        vd.field_list_items("objective_subgoals", Item::ObjectiveSubgoal);
        // TODO: check if it's in the list above?
        vd.field_item("final_subgoal", Item::ObjectiveSubgoal); // undocumented

        vd.field_validated_key_block("on_start", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_item("required_dlc_feature", Item::DlcFeature);
    }
}

#[derive(Clone, Debug)]
pub struct ObjectiveSubgoal {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ObjectiveSubgoal, ObjectiveSubgoal::add)
}

impl ObjectiveSubgoal {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ObjectiveSubgoal, key, block, Box::new(Self {}));
    }
}

impl DbKind for ObjectiveSubgoal {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("category", Item::ObjectiveSubgoalCategory);
        vd.field_bool("is_repeatable");

        vd.field_list_items("unlocking_subgoals_all", Item::ObjectiveSubgoal);
        vd.field_list_items("unlocking_subgoals_any", Item::ObjectiveSubgoal);
        vd.field_list_items("blocking_subgoals_none_of", Item::ObjectiveSubgoal);
        vd.field_list_items("blocking_subgoals_not_all", Item::ObjectiveSubgoal);

        vd.field_validated_key_block("trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_script_value_rooted("chance", Scopes::Country);
        vd.field_validated_key_block("on_start", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct ObjectiveSubgoalCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ObjectiveSubgoalCategory, ObjectiveSubgoalCategory::add)
}

impl ObjectiveSubgoalCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ObjectiveSubgoalCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for ObjectiveSubgoalCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_bool("is_exclusive");
        vd.field_bool("is_repeatable");
    }
}
