use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct AccoladeIcon {}

impl AccoladeIcon {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AccoladeIcon, key, block, Box::new(Self {}));
    }
}

impl DbKind for AccoladeIcon {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::AccoladeType, key.clone());
        sc.define_name("accolade", key.clone(), Scopes::Accolade);

        if let Some(token) = vd.field_value("texture") {
            let pathname = format!("gfx/interface/icons/knight_badge/icons/{token}");
            data.verify_exists_implied(Item::File, &pathname, token);
        }

        vd.field_validated_block("potential", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct AccoladeName {}

impl AccoladeName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AccoladeName, key, block, Box::new(Self {}));
    }
}

impl DbKind for AccoladeName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        sc.define_name("owner", key.clone(), Scopes::Character);
        sc.define_name("accolade_type", key.clone(), Scopes::AccoladeType);

        vd.req_field("key");
        vd.req_field("num_options");

        vd.field_item("key", Item::Localization);
        vd.field_integer("num_options");
        if let Some(key) = block.get_field_value("key") {
            if let Some(n) = block.get_field_integer("num_options") {
                data.localization.verify_key_has_options(key, n);
            }
        }

        let mut count = 0;
        vd.field_validated_bvs("option", |bv, data| {
            count += 1;
            validate_desc(bv, data, &mut sc);
        });

        if let Some(n) = block.get_field_integer("num_options") {
            if count != n {
                let msg = format!("expected {n} `option` blocks, found {count}");
                error(block, ErrorKey::Validation, &msg);
            }
        }

        vd.field_validated_block("potential", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_script_value("weight", &mut sc);
    }
}
