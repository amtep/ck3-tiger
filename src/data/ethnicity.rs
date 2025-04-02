use crate::block::Block;
use crate::data::genes::Gene;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::report::{Confidence, Severity};
use crate::token::Token;
use crate::validate::validate_numeric_range;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Ethnicity {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::modern(), Item::Ethnicity, Ethnicity::add)
}

impl Ethnicity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Ethnicity, key, block, Box::new(Self {}));
    }
}

impl DbKind for Ethnicity {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);
        vd.field_bool("visible");
        if !block.field_value_is("visible", "no") {
            if Game::is_ck3() {
                data.verify_exists(Item::Localization, key);
            } else if Game::is_vic3() {
                let loca = format!("ethnicity_{key}");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }
        vd.field_item("template", Item::Ethnicity);
        vd.field_item("using", Item::Culture);
        vd.unknown_block_fields(|key, block| {
            data.verify_exists(Item::GeneCategory, key);
            let mut vd = Validator::new(block, data);
            vd.set_max_severity(Severity::Warning);
            for (_, block) in vd.integer_blocks() {
                if let Some(token) = block.get_field_value("name") {
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(Severity::Warning);
                    vd.field_value("name");
                    Gene::verify_has_template(key.as_str(), token, data);
                    vd.field_validated_block("range", |block, data| {
                        validate_numeric_range(
                            block,
                            data,
                            0.0,
                            1.0,
                            Severity::Warning,
                            Confidence::Reasonable,
                        );
                    });
                    #[cfg(feature = "ck3")]
                    if Game::is_ck3() {
                        vd.field_list_items("traits", Item::GeneticConstraint);
                    }
                } else {
                    // for color genes
                    data.validate_use(Item::GeneCategory, key, block);
                }
            }
        });
    }
}
