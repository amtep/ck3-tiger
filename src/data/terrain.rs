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
        let mut sc = ScopeContext::new_root(Scopes::None, key.clone());

        vd.req_field("color");

        vd.field_numeric("movement_speed");
        vd.field_validated_block("color", validate_color);
        vd.field_validated_block("travel_danger_color", validate_color);
        vd.field_script_value("travel_danger_score", &mut sc);

        vd.field_validated_block("attacker_modifier", |b, data| {
            validate_combat_modifier(b, data, &mut sc);
        });
        vd.field_validated_block("defender_modifier", |b, data| {
            validate_combat_modifier(b, data, &mut sc);
        });
        vd.field_block("attacker_combat_effects"); // TODO
        vd.field_block("defender_combat_effects"); // TODO

        vd.field_numeric("combat_width");
        vd.field_bool("is_desert");
        vd.field_bool("is_jungle");
        vd.field_numeric("audio_parameter"); // ??

        vd.field_validated_block("province_modifier", |b, data| {
            validate_province_modifier(b, data, &mut sc);
        });
    }
}

pub fn validate_combat_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let vd = Validator::new(block, data);
    validate_modifs(block, data, ModifKinds::Terrain, sc, vd);
}

pub fn validate_province_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let vd = Validator::new(block, data);
    validate_modifs(block, data, ModifKinds::Province, sc, vd);
}
