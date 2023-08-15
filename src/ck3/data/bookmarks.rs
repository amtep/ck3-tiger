use crate::block::Block;
use crate::ck3::data::dna::validate_genes;
use crate::ck3::validate::validate_portrait_modifier_overrides;
use crate::context::ScopeContext;
use crate::date::Date;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, warn_info, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct BookmarkGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::BookmarkGroup, BookmarkGroup::add)
}

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

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Bookmark, Bookmark::add)
}

impl Bookmark {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Bookmark, key, block, Box::new(Self {}));
    }
}

impl DbKind for Bookmark {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        vd.req_field("start_date");
        vd.field_date("start_date");
        vd.field_bool("is_playable");
        vd.field_bool("recommended");
        vd.field_bool("test_default");
        vd.field_item("group", Item::BookmarkGroup);
        vd.field_script_value("weight", &mut sc);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let pathname = format!("gfx/interface/bookmarks/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
        let pathname = format!("gfx/interface/bookmarks/start_buttons/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
        let pathname = format!("gfx/interface/icons/bookmark_buttons/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);

        let start_date = block.get_field_date("start_date");
        vd.multi_field_validated_block("character", |block, data| {
            if let Some(name) = block.get_field_value("name") {
                let pathname = format!("gfx/interface/bookmarks/{key}_{name}.dds");
                data.verify_exists_implied(Item::File, &pathname, name);
            }
            validate_bookmark_character(block, data, true, start_date);
        });
    }
}

fn validate_bookmark_character(
    block: &Block,
    data: &Everything,
    toplevel: bool,
    start_date: Option<Date>,
) {
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
    if let Some(token) =
        block.get_field_value("dynasty_house").or_else(|| block.get_field_value("dynasty"))
    {
        if !data.item_exists(Item::Coa, token.as_str()) {
            let msg = format!("{} {token} not defined in {}", Item::Coa, Item::Coa.path());
            let info = "bookmark characters must have a defined coa or their shields will be blank";
            warn_info(token, ErrorKey::MissingItem, &msg, info);
        }
    }
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
    if let Some(start_date) = start_date {
        if let Some(id) = block.get_field_value("history_id") {
            let name = block.get_field_value("name");
            if data.item_exists(Item::Character, id.as_str()) {
                validate_bookmark_against_history(
                    block.get_field_value("dynasty"),
                    "dynasty",
                    start_date,
                    data.characters.get_dynasty(id, start_date, data),
                    name,
                );
                validate_bookmark_against_history(
                    block.get_field_value("dynasty_house"),
                    "house",
                    start_date,
                    data.characters.get_house(id, start_date),
                    name,
                );
                validate_bookmark_against_history(
                    block.get_field_value("culture"),
                    "culture",
                    start_date,
                    data.characters.get_culture(id, start_date),
                    name,
                );
                validate_bookmark_against_history(
                    block.get_field_value("faith"),
                    "faith",
                    start_date,
                    data.characters.get_faith(id, start_date),
                    name,
                );
            }
        }
    }
    vd.multi_field_validated_block("character", |block, data| {
        validate_bookmark_character(block, data, false, start_date);
    });
}

#[derive(Clone, Debug)]
pub struct BookmarkPortrait {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::BookmarkPortrait, PdxEncoding::Utf8OptionalBom, ".txt", false, BookmarkPortrait::add)
}

impl BookmarkPortrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BookmarkPortrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for BookmarkPortrait {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_choice("type", &["male", "female", "boy", "girl"]);
        vd.field_value("id"); // TODO
        vd.field_precise_numeric("age");
        vd.field_list_integers_exactly("entity", 2);
        vd.field_validated_block("genes", validate_genes);
        vd.field_validated_block("override", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block(
                "portrait_modifier_overrides",
                validate_portrait_modifier_overrides,
            );
        });
        vd.field_validated_block("tags", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                let mut vd = Validator::new(block, data);
                vd.field_value("hash");
                vd.field_bool("invert");
            }
        });
    }
}

fn validate_bookmark_against_history(
    field: Option<&Token>,
    desc: &str,
    date: Date,
    history: Option<&Token>,
    name: Option<&Token>,
) {
    if let Some(field) = field {
        if let Some(history) = history {
            if field != history {
                let msg = format!(
                    "{desc} is {field} in bookmark but {history} in character history at {date}"
                );
                warn(ErrorKey::Bookmarks)
                    .strong()
                    .msg(msg)
                    .loc_msg(field, "bookmark")
                    .loc(history, "history")
                    .opt_loc(name, "character")
                    .push();
            }
        } else {
            let msg = format!(
                "{desc} is {field} in bookmark but character has no {desc} in history at {date}"
            );
            warn(ErrorKey::Bookmarks).strong().msg(msg).loc(field).opt_loc(name, "bookmark").push();
        }
    }
}
