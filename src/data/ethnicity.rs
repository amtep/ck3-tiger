use crate::block::validator::Validator;
use crate::block::Block;
use crate::data::genes::Gene;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Ethnicity {}

impl Ethnicity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Ethnicity, key, block, Box::new(Self {}));
    }
}

impl DbKind for Ethnicity {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_bool("visible");
        vd.field_item("template", Item::Ethnicity);
        vd.field_item("using", Item::Culture);
        for (key, block) in vd.unknown_block_fields() {
            data.verify_exists(Item::GeneCategory, key);
            let mut vd = Validator::new(block, data);
            for (_, block) in vd.integer_blocks() {
                if let Some(token) = block.get_field_value("name") {
                    let mut vd = Validator::new(block, data);
                    vd.field_value("name");
                    Gene::verify_has_template(key.as_str(), token, data);
                    vd.field_validated_blocks("range", |block, data| {
                        let mut vd = Validator::new(block, data);
                        // TODO: verify range 0.0 - 1.0
                        vd.req_tokens_numbers_exactly(2);
                    });
                    vd.field_list_items("traits", Item::GeneticConstraint);
                } else {
                    // for color genes
                    data.validate_use(Item::GeneCategory, key, block);
                }
            }
        }
    }
}
