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
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct OldCombatUnit {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::OldCombatUnit, OldCombatUnit::add)
}

impl OldCombatUnit {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::OldCombatUnit, key, block, Box::new(Self {}));
    }
}

impl DbKind for OldCombatUnit {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_integer("max_manpower");
        vd.field_choice("type", &["army", "navy"]);

        vd.field_item("icon", Item::File);

        vd.multi_field_validated_key_block("combat_unit_image", |key, block, data| {
            let mut vd = Validator::new(block, data);
            let mut sc = ScopeContext::new(Scopes::CombatUnit, key);
            vd.field_validated_block("trigger", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_item("texture", Item::File);
        });
    }
}

#[derive(Clone, Debug)]
pub struct CombatUnitExperienceLevel {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CombatUnitExperienceLevel, CombatUnitExperienceLevel::add)
}

impl CombatUnitExperienceLevel {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatUnitExperienceLevel, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatUnitExperienceLevel {
    // This whole item type is undocumented.
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        // TODO: is there a requirement for levels to be consecutive?
        vd.field_integer("level");
        vd.field_item("icon", Item::File);
        vd.field_integer("needed_experience");
        vd.field_validated_block("unit_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Unit, vd);
        });
    }
}
