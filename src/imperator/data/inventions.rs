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
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct InventionGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::InventionGroup, InventionGroup::add)
}

impl InventionGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for (key, block) in block.iter_definitions() {
            if !&["technology", "color"].iter().any(|&v| key.is(v)) {
                db.add(Item::Invention, key.clone(), block.clone(), Box::new(Invention {}));
            }
        }
        db.add(Item::InventionGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for InventionGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("technology", Item::TechnologyTable);
        vd.field_validated_block("color", validate_color);

        // The inventions. They are validated in the Invention class.
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct Invention {}

impl DbKind for Invention {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_bool("keystone");
        vd.field_choice("icon_override", &["gw_icon", "war"]);

        // requires = { agressive_expansion_impact_inv_5 }
        vd.field_list_items("requires", Item::Invention);
        vd.field_list_items("requires_or", Item::Invention);

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
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

        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
