use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
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
        for (token, block) in block.iter_definitions() {
            db.add_flag(Item::HouseUnityStage, token.clone());

            if let Some(block) = block.get_field_block("parameters") {
                for (token, _) in block.iter_assignments() {
                    db.add_flag(Item::HouseUnityParameter, token.clone());
                }
            }
        }
        db.add(Item::HouseUnity, key, block, Box::new(Self {}));
    }
}

impl DbKind for HouseUnity {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("default_value");
        // TODO: Verify against `field_integer`
        vd.field_script_value_rooted("default_value", Scopes::None);
        vd.field_script_value_rooted("min_value", Scopes::None);

        for (token, block) in block.iter_definitions() {
            validate_stage(token, block, data);
        }
    }
}

fn validate_stage(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.req_field("points");
    vd.field_script_value_rooted("points", Scopes::None);

    if let Some(icon) = vd.field_value("icon") {
        if let Some(icon_path) =
            data.get_defined_string_warn(key, "gfx/interface/icons/currencies/house_unity")
        {
            let pathname = format!("{icon_path}/{icon}");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }
    }

    // TODO: Verify no character scope needed; otherwise use `_rooted`
    vd.field_validated_block("parameters", |block, data| {
        for (_, value) in block.iter_assignments_warn() {
            ValueValidator::new(value, data).bool();
        }
    });

    vd.field_validated_block_rooted("modifiers", Scopes::Character, |block, data, sc| {
        let mut vd = Validator::new(block, data);
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
