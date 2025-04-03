use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Achievement {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::Achievement, Achievement::add)
}

impl Achievement {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Achievement, key, block, Box::new(Self {}));
    }
}

impl DbKind for Achievement {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("ACHIEVEMENT_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("ACHIEVEMENT_DESC_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_key_block("possible", |key, block, data| {
            let mut sc = ScopeContext::new(achievement_scope(), key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_block("happened", |key, block, data| {
            let mut sc = ScopeContext::new(achievement_scope(), key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
    }
}

fn achievement_scope() -> Scopes {
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => Scopes::Character,
        #[cfg(feature = "vic3")]
        Game::Vic3 => Scopes::Country,
        #[cfg(feature = "imperator")]
        Game::Imperator => Scopes::Country,
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => Scopes::Country,
    }
}

#[cfg(feature = "jomini")]
#[derive(Clone, Debug)]
pub struct AchievementGroup {}

#[cfg(feature = "jomini")]
inventory::submit! {
    ItemLoader::Normal(GameFlags::jomini(), Item::AchievementGroup, AchievementGroup::add)
}

#[cfg(feature = "jomini")]
impl AchievementGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("group") {
            if let Some(name) = block.get_field_value("name").cloned() {
                db.add(Item::AchievementGroup, name, block, Box::new(Self {}));
            } else {
                let msg = "group missing `name` field";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = "unknown key in achievement groups";
            let info = "expected only `group`";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

#[cfg(feature = "jomini")]
impl DbKind for AchievementGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("ACHIEVEMENT_GROUP_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("name");
        vd.field_list_items("order", Item::Achievement);
    }
}
