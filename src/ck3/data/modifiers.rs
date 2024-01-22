#![allow(non_upper_case_globals)]

use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Modifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Modifier, Modifier::add)
}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // There are {key} and {key}_desc locas but both are optional
        data.mark_used(Item::Localization, key.as_str());
        let loca = format!("{key}_desc");
        data.mark_used(Item::Localization, &loca);

        // icon is also optional
        if let Some(icon) = vd.field_value("icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(icon, "NGameIcons|STATICMODIFIER_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}.dds");
                data.verify_exists_implied(Item::File, &pathname, icon);
            }
        }

        vd.field_bool("stacking");
        vd.field_bool("hide_effects");
        // `scale` is validated in `validate_call`
        vd.field_block("scale");
        validate_modifs(block, data, ModifKinds::all(), vd);
    }

    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _from: &Token,
        _from_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let mut vd = Validator::new(block, data);
        // docs say that the object the scale is applied to is `root`, but I suspect it's really `this`.
        // TODO: verify
        vd.field_validated_block("scale", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("value");
            vd.field_script_value("value", sc);
            vd.field_item("desc", Item::Localization);
            // Undocumented `display_mode`
            // TODO: get all possible values
            vd.field_choice("display_mode", &["scaled"]);
        });
        vd.no_warn_remaining();
    }

    fn validate_property_use(
        &self,
        _key: &Token,
        block: &Block,
        property: &Token,
        _caller: &str,
        data: &Everything,
    ) {
        let mut vd = Validator::new(block, data);
        // skip over the known fields
        vd.field("icon");
        vd.field("stacking");
        vd.field("hide_effects");

        // TODO: make validate_modifs explain why it expected this kind
        validate_modifs(block, data, get_modif_kinds(property.as_str()), vd);
    }
}

// LAST UPDATED CK3 VERSION 1.11.3
/// Get the modifier kinds from property name
/// See `effects.log` from the game data dumps.
fn get_modif_kinds(name: &str) -> ModifKinds {
    for substr in [
        "artifact_modifier",
        "character_modifier",
        "dynasty_modifier",
        "house_modifier",
        "house_unity_modifier",
    ] {
        if name.contains(substr) {
            return ModifKinds::Character;
        }
    }
    if name.contains("county_modifier") {
        return ModifKinds::County;
    }
    if name.contains("province_modifier") {
        return ModifKinds::Province;
    }
    if name.contains("scheme_modifier") {
        return ModifKinds::Scheme;
    }
    if name.contains("travel_plan_modifier") {
        return ModifKinds::TravelPlan;
    }

    ModifKinds::empty()
}
