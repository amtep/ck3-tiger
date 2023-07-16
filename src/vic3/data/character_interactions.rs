use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_duration;

#[derive(Clone, Debug)]
pub struct CharacterInteraction {}

impl CharacterInteraction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterInteraction, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterInteraction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("actor", Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_item("icon", Item::File);
        vd.field_item("clicksound", Item::Sound);

        vd.field_validated_block("potential", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_bool("show_requirements");
        vd.field_bool("show_confirmation_box");
        vd.field_script_value("ai_chance", &mut sc);
    }
}
