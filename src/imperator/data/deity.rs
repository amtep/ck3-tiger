use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
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
pub struct Deity {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Deity, Deity::add)
}

impl Deity {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, block: Block) {
        // Changes the key from "deity_name" to "omen_name"
        if let Some(s) = key.strip_prefix("deity_") {
            let omen_string = "omen_".to_owned() + s.as_str();
            let token = Token::new(&omen_string, key.loc);
            db.add(Item::Deity, token, block, Box::new(Self {}));
            db.add_flag(Item::Deity, key);
        }
    }
}

impl DbKind for Deity {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let key_str = key.to_string();

        // Slice "deity_" off the front of the key
        if let Some(deity_key) = key_str.strip_prefix("deity_") {
            let loca = format!("omen_{deity_key}");
            let loca2 = format!("omen_{deity_key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.verify_exists_implied(Item::Localization, &loca2, key);
        }

        vd.field_validated_block("trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("allow_on_setup", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_value("icon");
        vd.field_validated_block("passive_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("omen", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_item("religion", Item::Religion);
        vd.field_item("deity_category", Item::DeityCategory);
        vd.field_validated_block("deification_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_activate", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
    }
}
