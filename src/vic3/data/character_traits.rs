use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
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
pub struct CharacterTrait {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CharacterTrait, CharacterTrait::add)
}

impl CharacterTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterTrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.req_field("type");
        vd.field_choice("type", &["personality", "skill", "condition"]);
        vd.req_field("texture");
        vd.field_item("texture", Item::File);

        vd.field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("command_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_validated_block("country_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        // undocumented
        vd.field_validated_block("agitator_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_validated_block("interest_group_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_script_value("weight", &mut sc);

        vd.field_list_items("replace", Item::CharacterTrait);
        vd.field_integer("value");
    }
}
