use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Flavorization {}

impl Flavorization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Flavorization, key, block, Box::new(Self {}));
    }
}

impl DbKind for Flavorization {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.req_field("type");
        vd.field_choice("type", &["character", "title"]);
        vd.field_choice("gender", &["male", "female"]);
        vd.field_choice(
            "special",
            &[
                "ruler_child",
                "holder",
                "queen_mother",
                "head_of_faith",
                "councillor",
            ],
        );
        vd.field_choice("tier", &["empire", "kingdom", "duchy", "county", "barony"]);
        vd.field_integer("priority");
        vd.field_list_items("name_lists", Item::NameList);
        vd.field_list_items("heritages", Item::CultureHeritage);
        vd.field_list_items("governments", Item::GovernmentType);
        vd.field_list_items("faiths", Item::Faith);
        vd.field_list_items("religions", Item::Religion);
        vd.field_list_items("titles", Item::Title);
        vd.field_list_items("de_jure_liege", Item::Title);
        vd.field_item("council_position", Item::CouncilPosition);
        vd.field_item("holding", Item::Holding);
        vd.field_bool("top_liege");
        vd.field_bool("only_holder");
        vd.field_bool("only_independent");
        vd.field_bool("faction");
    }
}
