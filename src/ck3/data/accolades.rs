use crate::block::Block;
use crate::ck3::data::maa::{validate_terrain_bonus, validate_winter_bonus};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{error, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct AccoladeIcon {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::AccoladeIcon, AccoladeIcon::add)
}

impl AccoladeIcon {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AccoladeIcon, key, block, Box::new(Self {}));
    }
}

impl DbKind for AccoladeIcon {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::AccoladeType, key);
        sc.define_name("accolade", Scopes::Accolade, key);

        if let Some(token) = vd.field_value("texture") {
            let pathname = format!("gfx/interface/icons/knight_badge/icons/{token}");
            data.verify_exists_implied(Item::File, &pathname, token);
        }

        vd.field_validated_block("potential", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct AccoladeName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::AccoladeName, AccoladeName::add)
}

impl AccoladeName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AccoladeName, key, block, Box::new(Self {}));
    }
}

impl DbKind for AccoladeName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("owner", Scopes::Character, key);
        sc.define_name("accolade_type", Scopes::AccoladeType, key);

        vd.req_field("key");
        vd.req_field("num_options");

        vd.field_item("key", Item::Localization);
        vd.field_integer("num_options");
        if let Some(key) = block.get_field_value("key") {
            if let Some(n) = block.get_field_integer("num_options") {
                data.localization.verify_key_has_options(key.as_str(), key, n, "OPTION_");
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
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_script_value("weight", &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct AccoladeType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::AccoladeType, AccoladeType::add)
}

impl AccoladeType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(vec) = block.get_field_list("accolade_categories") {
            for token in vec {
                db.add_flag(Item::AccoladeCategory, token);
            }
        }
        if let Some(block) = block.get_field_block("ranks") {
            for (key, block) in block.iter_definitions() {
                if key.is_integer() {
                    if let Some(vec) = block.get_field_list("accolade_parameters") {
                        for token in vec {
                            db.add_flag(Item::AccoladeParameter, token);
                        }
                    }
                }
            }
        }
        db.add(Item::AccoladeType, key, block, Box::new(Self {}));
    }
}

impl DbKind for AccoladeType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_modifier");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("adjective", Item::Localization);
        vd.field_item("noun", Item::Localization);
        vd.field_list("accolade_categories");

        vd.field_validated_block("potential", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        sc.define_name("owner", Scopes::Character, key);
        vd.field_script_value("weight", &mut sc);

        vd.field_validated_block("ranks", |block, data| {
            let mut vd = Validator::new(block, data);
            for (_, block) in vd.integer_blocks() {
                let mut vd = Validator::new(block, data);
                vd.field_validated_block("liege_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.field_validated_block("knight_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.field_validated_block("knight_army_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
                vd.field_list_items("men_at_arms", Item::MenAtArms);
                vd.field_validated_block("terrain_bonus", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_block_fields(|key, block| {
                        data.verify_exists(Item::MenAtArms, key);
                        validate_terrain_bonus(block, data);
                    });
                });
                vd.field_validated_block("winter_bonus", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_block_fields(|key, block| {
                        data.verify_exists(Item::MenAtArms, key);
                        validate_winter_bonus(block, data);
                    });
                });
                vd.field_list_items("accolade_parameters", Item::Localization);
            }
        });
    }
}
