use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_duration;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CharacterInteraction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CharacterInteraction, CharacterInteraction::add)
}

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

        vd.field_trigger("potential", Tooltipped::No, &mut sc);
        vd.field_trigger("possible", Tooltipped::Yes, &mut sc);
        vd.field_effect("effect", Tooltipped::Yes, &mut sc);
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_bool("show_requirements");
        vd.field_bool("show_confirmation_box");
        vd.field_script_value("ai_chance", &mut sc);

        // undocumented

        vd.field_trigger_rooted("should_ai_evaluate", Tooltipped::No, Scopes::Country);
        vd.field_bool("ai_considers_exiles");
    }
}
