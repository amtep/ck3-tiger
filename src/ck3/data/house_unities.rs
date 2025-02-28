use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct HouseUnity {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HouseUnity, HouseUnity::add)
}

impl HouseUnity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HouseUnity, key, block, Box::new(Self {}));
    }
}

impl DbKind for HouseUnity {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (token, block) in block.iter_definitions() {
            db.add_flag(Item::HouseUnityStage, token.clone());

            if let Some(block) = block.get_field_block("parameters") {
                for (token, _) in block.iter_assignments() {
                    db.add_flag(Item::HouseUnityParameter, token.clone());
                }
            }
        }
    }

    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("default_value");
        vd.field_integer("default_value");
        vd.field_integer("min_value");

        vd.unknown_block_fields(|token, block| {
            validate_stage(token, block, data);
        });
    }
}

fn validate_stage(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    data.verify_exists(Item::Localization, key);
    let loca = format!("{key}_desc");
    data.verify_exists_implied(Item::Localization, &loca, key);

    vd.req_field("points");
    vd.field_integer("points");

    let icon = vd.field_value("icon").unwrap_or(key);
    data.verify_icon("NGameIcons|HOUSE_UNITY_STAGE_ICON_PATH", icon, ".dds");

    // progress bar
    if let Some(progress_bar_path) =
        data.get_defined_string_warn(key, "NGameIcons|HOUSE_UNITY_STAGE_PROGRESS_BAR_PATH")
    {
        let pathname = format!("{progress_bar_path}/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
    }

    // background
    if let Some(background_path) =
        data.get_defined_string_warn(key, "NGameIcons|HOUSE_UNITY_STAGE_BACKGROUND_PATH")
    {
        let pathname = format!("{background_path}/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
    }

    vd.field_validated_block("parameters", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            ValueValidator::new(value, data).bool();
            let loca = format!("house_unity_parameter_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
        });
    });

    vd.field_validated_block("modifiers", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });

    vd.field_validated_block_rooted("on_start", Scopes::DynastyHouse, |block, data, sc| {
        validate_effect(block, data, sc, Tooltipped::No);
    });

    vd.field_validated_block_rooted("on_end", Scopes::DynastyHouse, |block, data, sc| {
        validate_effect(block, data, sc, Tooltipped::No);
    });

    vd.field_list_items("decisions", Item::Decision);

    // Undocumented
    vd.field_item("succession_law_flag", Item::LawFlag);
}
