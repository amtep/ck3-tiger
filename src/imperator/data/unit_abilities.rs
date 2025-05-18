use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct UnitAbility {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::UnitAbility, UnitAbility::add)
}

impl UnitAbility {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::UnitAbility, key, block, Box::new(Self {}));
    }
}

impl DbKind for UnitAbility {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Unit, key);
        sc.define_name("province", Scopes::Province, key);

        vd.field_bool("enable");
        vd.field_bool("toggle");
        vd.field_bool("army_only");
        vd.field_bool("navy_only");
        vd.field_bool("map");
        vd.field_bool("is_road_building");
        vd.field_bool("cancel_on_combat_end");
        vd.field_bool("is_desecrate");
        vd.field_bool("confirm");

        vd.field_numeric("duration");

        vd.field_item("soundeffect", Item::Sound);

        vd.field_choice("idle_entity_state", &["recruiting", "army_drill", "raiding"]);
        vd.field_choice("move_entity_state", &["build_road", "force_march"]);

        vd.field_list("available_states");

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_trigger("hidden", Tooltipped::No, &mut sc);
        vd.field_trigger("allow", Tooltipped::Yes, &mut sc);
        vd.field_trigger("ai_will_revoke", Tooltipped::Yes, &mut sc);
        vd.field_trigger("finished_when", Tooltipped::Yes, &mut sc);
        vd.field_effect("start_effect", Tooltipped::Yes, &mut sc);
        vd.field_effect("finish_effect", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_entering_province", Tooltipped::Yes, &mut sc);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
