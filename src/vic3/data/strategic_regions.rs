use fnv::FnvHashMap;

use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct StrategicRegion {}

impl StrategicRegion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::StrategicRegion, key, block, Box::new(Self {}));
    }

    pub fn crosscheck(data: &Everything) {
        // Each state must be part of one and only one stategic region.
        let mut seen = FnvHashMap::default();
        for (key, block, _) in data.database.iter_itype(Item::StrategicRegion) {
            if let Some(vec) = block.get_field_list("states") {
                for token in vec {
                    if let Some(&other) = seen.get(token.as_str()) {
                        let msg =
                            format!("state {token} is part of more than one strategic region");
                        err(ErrorKey::Conflict)
                            .strong()
                            .msg(msg)
                            .loc(key)
                            .loc(other, "the other one")
                            .push();
                    } else {
                        seen.insert(token.to_string(), key);
                    }
                }
            }
        }
        for (key, _, _) in data.database.iter_itype(Item::StateRegion) {
            if !seen.contains_key(key.as_str()) {
                let msg = format!("state {key} is not part of any strategic region");
                err(ErrorKey::Validation).strong().msg(msg).loc(key).push();
            }
        }
    }
}

impl DbKind for StrategicRegion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        // TODO capital_province and map_color are required... but not for sea regions
        // TODO check that capital province is in region
        vd.field_item("capital_province", Item::Province);
        vd.field_validated("map_color", validate_possibly_named_color);
        vd.field_list_items("states", Item::StateRegion);
        vd.field_value("graphical_culture"); // TODO
    }
}
