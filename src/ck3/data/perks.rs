use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
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
            data.verify_icon("NGameIcons|LIFESTYPE_TREE_BACKGROUND_PATH", token, ".dds");
        }
        vd.field_validated_block("position", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_numbers_exactly(2);
        });

        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|PERK_ICON_PATH", icon, ".dds");

        vd.field_item("lifestyle", Item::Lifestyle);
        // TODO: check for loops in perk parents
        for parent in vd.multi_field_value("parent") {
            data.verify_exists(Item::Perk, parent);
        }

        vd.field_trigger_rooted("can_be_picked", Tooltipped::Yes, Scopes::Character);
        vd.field_trigger_rooted("can_be_auto_selected", Tooltipped::No, Scopes::Character);
        vd.multi_field_item("trait", Item::Trait);
        vd.field_effect_rooted("effect", Tooltipped::Yes, Scopes::Character);

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
        vd.multi_field_validated_block("government_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("flag", Item::GovernmentFlag);
            vd.field_bool("invert_check");
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_script_value("auto_selection_weight", &mut sc);
    }
}
