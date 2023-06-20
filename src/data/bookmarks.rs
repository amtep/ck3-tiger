use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct BookmarkGroup {}

impl BookmarkGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BookmarkGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for BookmarkGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_date("default_start_date");
    }
}

#[derive(Clone, Debug)]
pub struct Bookmark {}

impl Bookmark {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Bookmark, key, block, Box::new(Self {}));
    }
}

impl DbKind for Bookmark {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::None, key.clone());
        vd.field_date("start_date");
        vd.field_bool("is_playable");
        vd.field_bool("recommended");
        vd.field_bool("test_default");
        vd.field_item("group", Item::BookmarkGroup);
        vd.field_script_value("weight", &mut sc);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_blocks("character", |block, data| {
            validate_bookmark_character(block, data, true);
        });
    }
}

fn validate_bookmark_character(block: &Block, data: &Everything, toplevel: bool) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("tutorial");
    vd.field_bool("test_default");
    vd.field_bool("display");
    if block.field_value_is("display", "no") {
        vd.field_value("name");
    } else {
        vd.field_item("name", Item::Localization);
    }
    if toplevel {
        if let Some(token) = block.get_field_value("name") {
            let loca = format!("{token}_desc");
            data.verify_exists_implied(Item::Localization, &loca, token);
        }
    } else {
        vd.field_item("relation", Item::Localization);
    }
    vd.field_item("dynasty", Item::Dynasty);
    vd.field_item("dynasty_house", Item::House);
    vd.field_integer("dynasty_splendor_level");
    vd.field_choice("type", &["male", "female", "boy", "girl"]);
    vd.field_date("birth");
    vd.field_item("title", Item::Title);
    vd.field_item("title_text_override", Item::Localization);
    vd.field_item("government", Item::GovernmentType);
    vd.field_item("culture", Item::Culture);
    vd.field_item("religion", Item::Faith);
    vd.field_item("difficulty", Item::Localization);
    vd.field_item("history_id", Item::Character);
    vd.field_item("animation", Item::PortraitAnimation);
    vd.field_validated_block("position", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_tokens_integers_exactly(2);
    });
    vd.field_validated_blocks("character", |block, data| {
        validate_bookmark_character(block, data, false);
    });
}
