use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DynastyLegacy {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DynastyLegacy, DynastyLegacy::add)
}

impl DynastyLegacy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynastyLegacy, key, block, Box::new(Self {}));
    }
}

impl DbKind for DynastyLegacy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        if let Some(path) = data.get_defined_string_warn(key, "NGameIcons|LEGACY_TRACK_ICON_PATH") {
            let pathname = format!("{path}/{key}.dds");
            data.verify_exists_implied(Item::File, &pathname, key);
        }
        if let Some(path) = data.get_defined_string_warn(key, "NGameIcons|LEGACY_ICON_PATH") {
            let pathname = format!("{path}/{key}.dds");
            data.verify_exists_implied(Item::File, &pathname, key);
        }

        vd.field_validated_block("is_shown", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct DynastyPerk {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DynastyPerk, DynastyPerk::add)
}

impl DynastyPerk {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynastyPerk, key, block, Box::new(Self {}));
    }
}

impl DbKind for DynastyPerk {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("legacy", Item::DynastyLegacy);

        vd.field_validated_block("can_be_picked", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.multi_field_validated_block("character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("doctrine_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            vd.field_item("doctrine", Item::Doctrine);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.multi_field_item("unlock_maa", Item::MenAtArms);
        vd.field_item("trait", Item::Trait);
        vd.field_validated_block("traits", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Trait, key);
                value.expect_integer();
            });
        });

        vd.field_script_value_no_breakdown("ai_chance", &mut sc);
    }
}
