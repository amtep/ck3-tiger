use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CombatTactic {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::CombatTactic, CombatTactic::add)
}

impl CombatTactic {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("phases") {
            for value in block.iter_values_warn() {
                db.add_flag(Item::CombatTacticPhase, value.clone());
            }
            db.set_flag_validator(Item::CombatTacticPhase, |flag, data| {
                data.verify_exists(Item::Localization, flag);
            });
        } else {
            db.add(Item::CombatTactic, key, block, Box::new(Self {}));
        }
    }
}

impl DbKind for CombatTactic {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::CombatTactic, key);

        vd.field_item("only_show_for", Item::CountryTag);

        vd.field_bool("is_attacker");
        vd.field_bool("active");

        vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Combatant);

        let mut sc = ScopeContext::new(Scopes::Combatant, key);
        vd.field_validated_block_sc("base", &mut sc, validate_modifiers_with_base);

        if let Some(picture) = vd.field_value("picture") {
            let pathname = format!("gfx/interface/landcombat/tactics/{picture}.dds");
            data.verify_exists_implied(Item::File, &pathname, picture);
        }

        vd.field_validated_value("phase", |_, mut vd| {
            vd.maybe_is("no");
            vd.item(Item::CombatTacticPhase);
        });
        vd.field_item("display_phase", Item::CombatTacticPhase);

        vd.field_item("countered_by", Item::CombatTactic);
        vd.field_numeric("attacker");
        vd.field_numeric("defender");
        vd.field_numeric("attacker_movement_speed");
        vd.field_numeric("defender_movement_speed");
        vd.field_numeric("attacker_org_damage_modifier");
        vd.field_numeric("defender_org_damage_modifier");
        vd.field_numeric("combat_width");
    }
}
