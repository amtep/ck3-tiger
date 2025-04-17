use std::path::PathBuf;

use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile};
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Theme {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Theme, Theme::add)
}

impl Theme {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Theme, key, block, Box::new(Self {}));
    }
}

impl DbKind for Theme {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("category");
        vd.field_choice("category", THEME_CATEGORIES);
        if let Some(category) = block.get_field_value("category") {
            data.verify_exists(Item::Localization, category);
            let loca = format!("{category}_desc");
            data.verify_exists_implied(Item::Localization, &loca, category);

            vd.advice_field(
                "map_textures",
                "docs say map_textures, but it's papermap_textures_file",
            );
            if category.is("papermap_theme") {
                vd.field_item("papermap_textures_file", Item::File);
            } else {
                vd.ban_field("papermap_textures_file", || "category papermap_theme");
            }
        }

        vd.advice_field("skin", "docs say skin, but it's ui_skin");
        vd.field_validated_value("ui_skin", |_, mut vd| {
            vd.maybe_is("default");
            vd.item(Item::Skin);
        });

        vd.advice_field("theme_object", "docs say theme_object, but it's map_object");
        vd.multi_field_validated_block("map_object", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field_one_of(&["pdxmesh", "entity"]);
            vd.field_item("pdxmesh", Item::Pdxmesh);
            vd.field_item("entity", Item::Entity);
            vd.req_field("locator");
            vd.field_value("locator"); // TODO
        });

        vd.field_item("dlc", Item::Dlc);
    }
}

#[derive(Clone, Debug)]
pub struct Skin {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Vic3, Item::Skin, PdxEncoding::Utf8Bom, ".skin", LoadAsFile::Yes, Skin::add)
}

impl Skin {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(name) = block.get_field_value("name") {
            db.add(Item::Skin, name.clone(), block, Box::new(Self {}));
        } else {
            let msg = "skin file with no name field";
            warn(ErrorKey::FieldMissing).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for Skin {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_item("folder", Item::Directory);

        if let Some(folder) = block.get_field_value("folder") {
            for entry in data.fileset.get_files_under(&PathBuf::from(folder.as_str())) {
                let path = entry.path().to_string_lossy();
                // This slice should never fail, because of how get_files_under() works
                let base_path = format!("gfx/interface{}", &path[folder.as_str().len()..]);
                if !data.item_exists(Item::File, &base_path) {
                    let msg = format!("file {base_path} does not exist");
                    let info = "every skin file must override an existing interface file";
                    warn(ErrorKey::ExtraFile).msg(msg).info(info).loc(entry).push();
                }
            }
        }
    }
}

const THEME_CATEGORIES: &[&str] = &[
    "ui_skin_theme",
    "papermap_theme",
    "table_top_theme",
    "table_asset_1_theme",
    "table_asset_2_theme",
    "table_asset_3_theme",
    "table_asset_4_theme",
    "table_asset_cloth_theme",
    "building_sets_themes",
];
