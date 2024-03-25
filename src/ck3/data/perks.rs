use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Perk {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Perk, Perk::add)
}

impl Perk {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Perk, key, block, Box::new(Self {}));
    }
}

impl DbKind for Perk {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(tree) = block.get_field_value("tree") {
            db.add_flag(Item::PerkTree, tree.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if let Some(token) = vd.field_value("tree") {
            let loca = format!("{token}_name");
            data.verify_exists_implied(Item::Localization, &loca, token);
            let pathname = format!("gfx/interface/icons/lifestyle_tree_backgrounds/{token}.dds");
            data.verify_exists_implied(Item::File, &pathname, token);
        }
        vd.field_validated_block("position", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_numbers_exactly(2);
        });

        let icon = vd.field_value("icon").unwrap_or(key);
        if let Some(icon_path) = data.get_defined_string_warn(key, "NGameIcons|PERK_ICON_PATH") {
            let pathname = format!("{icon_path}/{icon}.dds");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }

        vd.field_item("lifestyle", Item::Lifestyle);
        // TODO: check for loops in perk parents
        for parent in vd.multi_field_value("parent") {
            data.verify_exists(Item::Perk, parent);
        }

        vd.field_validated_block_rooted("can_be_picked", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block_rooted(
            "can_be_auto_selected",
            Scopes::Character,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_block_rooted("effect", Scopes::Character, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::Yes);
        });

        vd.multi_field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("doctrine_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("doctrine", Item::Doctrine);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("culture_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("parameter", Item::CultureParameter);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_script_value("auto_selection_weight", &mut sc);
    }
}
