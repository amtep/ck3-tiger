use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Ability {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Ability, Ability::add)
}

impl Ability {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("ability") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::Ability, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `ability`";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(&key).push();
        }
    }
}

impl DbKind for Ability {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.req_field("desc");

        vd.field_item("name", Item::Localization);
        vd.field_item("desc", Item::Localization);
        vd.field_item("icon", Item::Sprite);
        vd.field_item("sound_effect", Item::SoundEffect);

        vd.field_choice("type", &["army_leader"]); // TODO other choices

        vd.field_numeric("cost");
        vd.field_integer("duration");
        vd.field_integer("cooldown");
        vd.field_bool("cancelable");

        vd.field_trigger_rooted("allowed", Tooltipped::Yes, Scopes::Character);
        vd.field_effect_rooted("one_time_effect", Tooltipped::Yes, Scopes::Character);

        vd.field_validated_block("unit_modifiers", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::UnitLeader | ModifKinds::Army, vd);
        });

        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
