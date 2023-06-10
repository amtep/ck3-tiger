use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct OpinionModifier {}

impl OpinionModifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::OpinionModifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for OpinionModifier {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // TODO: figure out when it is necessary to localize an opinion.
        // Opinions given from traits don't need to be localized, for one.
        // Maybe only ones used with add_opinion
        // data.verify_exists(Item::Localization, key);

        vd.field_integer("opinion");

        vd.field_bool("decaying");
        if block.get_field_bool("decaying").unwrap_or(false) {
            vd.field_integer("delay_days");
            vd.field_integer("delay_months");
            vd.field_integer("delay_years");
        } else {
            vd.ban_field("delay_days", || "decaying opinions");
            vd.ban_field("delay_months", || "decaying opinions");
            vd.ban_field("delay_years", || "decaying opinions");
        }

        vd.field_integer("days");
        vd.field_integer("months");
        vd.field_integer("years");

        vd.field_numeric("monthly_change");

        vd.field_bool("growing");
        vd.field_bool("stacking");

        vd.field_integer("min");
        vd.field_integer("max");

        vd.field_bool("imprisonment_reason");
        vd.field_bool("banish_reason");
        vd.field_bool("execute_reason");
        vd.field_bool("revoke_title_reason");
        vd.field_bool("divorce_reason");

        vd.field_bool("disable_non_aggression_pacts");
        vd.field_bool("non_aggression_pact");
        vd.field_bool("obedient");
        vd.field_bool("non_interference");
    }
}
