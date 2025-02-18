use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Dlc {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3.union(GameFlags::Vic3), Item::Dlc, Dlc::add)
}

impl Dlc {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Dlc, key, block, Box::new(Self {}));
    }
}

impl DbKind for Dlc {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(name) = block.get_field_value("name") {
            db.add_flag(Item::DlcName, name.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        if Game::is_vic3() && block.get_field_bool("theme_provider").unwrap_or(false) {
            let path = format!("gfx/interface/icons/dlc_icons/{key}.dds");
            data.verify_exists_implied(Item::File, &path, key);
            let path = format!("gfx/interface/banners/dlc_banners/{key}.dds");
            data.verify_exists_implied(Item::File, &path, key);
        } else if Game::is_ck3() {
            let path = format!("gfx/interface/illustrations/dlc_event_decorations/{key}.dds");
            data.verify_exists_implied(Item::File, &path, key);
            let path = format!("gfx/interface/icons/dlc/{key}.dds");
            data.verify_exists_implied(Item::File, &path, key);
        }

        let path = format!("{key}.dlc");
        data.verify_exists_implied(Item::File, &path, key);
        if Game::is_vic3() {
            let path = format!("{key}.dlc.json");
            data.verify_exists_implied(Item::File, &path, key);
        }

        vd.req_field("name");
        vd.field_value("name");

        if Game::is_vic3() {
            vd.field_choice("type", &["minor", "major"]);
        } else if Game::is_ck3() {
            vd.field_choice("type", &["minor", "medium", "major"]);
        }

        vd.field_integer("priority");
        vd.field_value("steam_id");
        vd.field_value("msgr_id");

        if Game::is_vic3() {
            vd.field_bool("theme_provider");
        }

        if Game::is_ck3() {
            // Documented but not used
            vd.field_list_items("features", Item::Localization);
        }
    }
}
