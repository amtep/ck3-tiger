use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::localization::Language;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

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
                validate_localization_key(key, data, lang);
            }
            if let Some(block) = block.get_field_block("random_list") {
                for (key, block) in block.iter_definitions() {
                    if key.is_integer() {
                        if let Some(key) = block.get_field_value("localization_key") {
                            validate_localization_key(key, data, lang);
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
            vd.field_trigger("trigger", &mut sc, Tooltipped::No);
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

fn validate_localization_key(key: &Token, data: &Everything, lang: Option<Language>) {
    let v = key.split('|');
    match v.len() {
        1 => data.localization.verify_exists_lang(key, lang),
        2 => {
            let [ref format, ref value] = v[..] else { unreachable!() };
            // The formats are described in documentation/loc_formatter_documentation.md
            match format.as_str() {
                "character_name" | "advisor_desc" | "country_leader_desc" => {
                    data.verify_exists(Item::Character, value);
                }
                "country_culture" => (), // TODO (no examples)
                "idea_name" | "idea_desc" => data.verify_exists(Item::Idea, value),
                "tech_effect" => data.verify_exists(Item::Technology, value),
                "building_state_modifier" => data.verify_exists(Item::Building, value),
                _ => {
                    let msg = "unknown format {format}";
                    warn(ErrorKey::Localization).msg(msg).loc(key).push();
                }
            }
        }
        _ => {
            let msg = "could not parse format of localization key {key}";
            warn(ErrorKey::Localization).msg(msg).loc(key).push();
        }
    }
}
