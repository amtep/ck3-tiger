use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Flavorization {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Flavorization, Flavorization::add)
}

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
        vd.field_choice("type", &["character", "title", "domicile"]);
        let flavor_type = block.get_field_value("type").map_or("none", Token::as_str);
        if flavor_type == "character" {
            vd.field_choice("gender", &["male", "female"]);
            vd.field_choice(
                "special",
                &[
                    "ruler_child",
                    "holder",
                    "queen_mother",
                    "head_of_faith",
                    "councillor",
                    "domicile",
                ],
            );
        } else {
            vd.ban_field("gender", || "type = character");
            vd.ban_field("special", || "type = character");
        }
        if flavor_type == "character" || flavor_type == "title" {
            vd.field_choice("tier", &["empire", "kingdom", "duchy", "county", "barony", "none"]);
        } else {
            vd.ban_field("tier", || "type = character or title");
        }
        vd.field_integer("priority");
        vd.advice_field("flavorization_rules", "Should be `flavourization_rules`");
        vd.field_validated_block("flavourization_rules", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("faction");
            vd.field_bool("only_independent");
            vd.field_bool("spouse_takes_title");
            vd.field_bool("only_holder");
            vd.field_bool("top_liege");
            vd.field_bool("only_vassals");
            vd.field_bool("ignore_top_liege_government");
        });
        vd.field_value("flag");
        vd.field_list_items("governments", Item::GovernmentType);
        vd.field_item("domicile_type", Item::DomicileType);
        vd.field_list_items("name_lists", Item::NameList);
        vd.field_list_items("heritages", Item::CultureHeritage);
        vd.field_list_items("faiths", Item::Faith);
        vd.field_list_items("religions", Item::Religion);
        vd.field_item("council_position", Item::CouncilPosition);
        vd.field_list_items("de_jure_liege", Item::Title);
        vd.field_item("holding", Item::HoldingType);
        vd.field_list_items("titles", Item::Title);

        vd.advice_field("top_liege", "moved into flavorization_rules");
        vd.advice_field("only_holder", "moved into flavorization_rules");
        vd.advice_field("only_independent", "moved into flavorization_rules");
        vd.advice_field("faction", "moved into flavorization_rules");
    }
}
