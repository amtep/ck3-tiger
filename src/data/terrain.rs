use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct Terrain {}

impl Terrain {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Terrain, key, block, Box::new(Self {}));
    }
}

impl DbKind for Terrain {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);

        if !key.is("sea") && !key.is("coastal_sea") {
            let modif = format!("{key}_advantage");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_construction_gold_cost");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_construction_piety_cost");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_construction_prestige_cost");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_holding_construction_gold_cost");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_holding_construction_piety_cost");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_holding_construction_prestige_cost");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_development_growth");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_development_growth_factor");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_supply_limit");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_supply_limit_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_min_combat_roll");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_max_combat_roll");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_levy_size");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_travel_danger");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_attrition_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_tax_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
            let modif = format!("{key}_cancel_negative_supply");
            data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        }

        vd.req_field("color");

        vd.field_numeric("movement_speed");
        vd.field_validated_block("color", validate_color);
        vd.field_validated_block("travel_danger_color", validate_color);
        vd.field_script_value("travel_danger_score", &mut sc);

        vd.field_validated_block("attacker_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Terrain, vd);
        });
        vd.field_validated_block("defender_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Terrain, vd);
        });
        vd.field_block("attacker_combat_effects"); // TODO
        vd.field_block("defender_combat_effects"); // TODO

        vd.field_numeric("combat_width");
        vd.field_bool("is_desert");
        vd.field_bool("is_jungle");
        vd.field_numeric("audio_parameter"); // TODO: ??

        vd.field_validated_block("province_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });
    }
}
