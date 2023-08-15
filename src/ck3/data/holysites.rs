use crate::block::Block;
use crate::ck3::data::titles::Tier;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{fatal, old_warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct HolySite {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HolySite, HolySite::add)
}

impl HolySite {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::HolySiteFlag, token.clone());
        }
        db.add(Item::HolySite, key, block, Box::new(Self {}));
    }
}

impl DbKind for HolySite {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("holy_site_{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("county");
        vd.field_item("county", Item::Title);
        vd.field_item("barony", Item::Title);

        if let Some(county) = block.get_field_value("county") {
            if Tier::try_from(county) != Ok(Tier::County) {
                old_warn(county, ErrorKey::TitleTier, "must be a county");
            }
            if let Some(barony) = block.get_field_value("barony") {
                if Tier::try_from(barony) != Ok(Tier::Barony) {
                    old_warn(barony, ErrorKey::TitleTier, "must be a barony");
                }
                if let Some(title) = data.titles.get(barony.as_str()) {
                    if title.parent.as_deref() != Some(county.as_str()) {
                        let msg = format!("barony not in specified county {county}");
                        fatal(ErrorKey::Crash).strong().msg(msg).loc(barony).push();
                    }
                }
            }
        }

        vd.multi_field_value("flag");

        vd.multi_field_validated_block("character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            if let Some(token) = vd.field_value("name") {
                data.verify_exists(Item::Localization, token);
            } else {
                let loca = format!("holy_site_{key}_effects");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        // undocumented

        vd.field_bool("is_active");
    }
}
