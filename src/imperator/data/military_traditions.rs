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
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MilitaryTraditionTree {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::MilitaryTraditionTree, MilitaryTraditionTree::add)
}

impl MilitaryTraditionTree {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for (key, block) in block.iter_definitions() {
            if !&["color", "image", "allow"].iter().any(|&v| key.is(v)) {
                db.add(
                    Item::MilitaryTradition,
                    key.clone(),
                    block.clone(),
                    Box::new(MilitaryTradition {}),
                );
            }
        }
        db.add(Item::MilitaryTraditionTree, key, block, Box::new(Self {}));
    }
}

impl DbKind for MilitaryTraditionTree {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("color", validate_color);
        vd.field_value("image");

        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        // The individual traditions. They are validated in the MilitaryTradition class.
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct MilitaryTradition {}

impl DbKind for MilitaryTradition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("icon");
        vd.field_item("enable_tactic", Item::CombatTactic);
        vd.field_item("enable_ability", Item::UnitAbility);
        vd.field_item("allow_unit_type", Item::Unit);

        vd.field_list_items("requires", Item::MilitaryTradition);

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("on_activate", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
    }
}
