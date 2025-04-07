use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::data::localization::Language;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct ScriptedLocalisation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::ScriptedLocalisation, ScriptedLocalisation::add)
}

impl ScriptedLocalisation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("defined_text") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::ScriptedLocalisation, name.clone(), block, Box::new(Self {}));
            } else {
                let msg = "missing `name` field";
                err(ErrorKey::FieldMissing).msg(msg).loc(block).push();
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `defined_text`";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }

    pub fn validate_loca_call(block: &Block, data: &Everything, lang: Option<Language>) {
        for block in block.get_field_blocks("text") {
            if let Some(key) = block.get_field_value("localization_key") {
                data.localization.verify_exists_lang(key, lang);
            }
            if let Some(block) = block.get_field_block("random_list") {
                for (key, block) in block.iter_definitions() {
                    if key.is_integer() {
                        if let Some(key) = block.get_field_value("localization_key") {
                            data.localization.verify_exists_lang(key, lang);
                        }
                    }
                }
            }
        }
    }
}

impl DbKind for ScriptedLocalisation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::all_but_none(), key);

        // The localization keys will be checked in validate_loca_call().
        // This way they only check for the language that actually uses them.

        vd.field_value("name");
        vd.multi_field_validated_block("text", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger_full("trigger", &mut sc, Tooltipped::No);
            vd.req_field_one_of(&["localization_key", "random_list"]);
            vd.field_value("localization_key");
            vd.field_validated_block("random_list", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_target("seed", &mut sc, Scopes::Value);
                for (_, block) in vd.integer_blocks() {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("localization_key");
                    vd.field_value("localization_key");
                }
            });
        });
    }
}
