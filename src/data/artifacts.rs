use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct ArtifactSlot {}

impl ArtifactSlot {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(slot_type) = block.get_field_value("type") {
            db.add_flag(Item::ArtifactSlotType, slot_type.clone());
        }
        db.add(Item::ArtifactSlot, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactSlot {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);
        vd.field_item("type", Item::ArtifactSlotType);
        vd.field_choice("category", &["inventory", "court"]);
        if let Some(category) = block.get_field_value("category") {
            if category.is("inventory") {
                let icon = vd.field_value("icon").unwrap_or(key);
                if let Some(icon_path) =
                    data.get_defined_string_warn(key, "NGameIcons|INVENTORY_SLOT_ICON_PATH")
                {
                    let pathname = format!("{icon_path}/{icon}.dds");
                    data.verify_exists_implied(Item::File, &pathname, icon);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactType {}

impl ArtifactType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactType, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let loca = format!("artifact_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("slot", Item::ArtifactSlotType);
        vd.field_list_items("required_features", Item::ArtifactFeatureGroup);
        vd.field_list_items("optional_features", Item::ArtifactFeatureGroup);
        vd.field_bool("can_reforge");
        vd.field_item("default_visuals", Item::ArtifactVisual);
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactTemplate {}

impl ArtifactTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactTemplate, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        sc.define_name("artifact", key.clone(), Scopes::Artifact);

        vd.field_validated_block("can_equip", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_benefit", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_reforge", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_repair", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("fallback", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, &mut sc, vd);
        });

        vd.field_script_value("ai_score", &mut sc);
        vd.field_bool("unique");
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactVisual {}

impl ArtifactVisual {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactVisual, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactVisual {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        sc.define_name("artifact", key.clone(), Scopes::Artifact);

        vd.field_value("default_type"); // unused

        // These two are undocumented
        vd.field_value("pedestal"); // TODO
        vd.field_value("support_type"); // TODO

        let mut unconditional = false;
        vd.field_validated_bvs("icon", |bv, data| match bv {
            BV::Value(icon) => {
                unconditional = true;
                if let Some(icon_path) =
                    data.get_defined_string_warn(icon, "NGameIcons|ARTIFACT_ICON_PATH")
                {
                    let pathname = format!("{icon_path}/{icon}");
                    data.verify_exists_implied(Item::File, &pathname, icon);
                }
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                if !block.has_key("trigger") {
                    unconditional = true;
                }
                vd.field_validated_block("trigger", |block, data| {
                    validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_validated_value("reference", |_, icon, data| {
                    if let Some(icon_path) =
                        data.get_defined_string_warn(icon, "NGameIcons|ARTIFACT_ICON_PATH")
                    {
                        let pathname = format!("{icon_path}/{icon}");
                        data.verify_exists_implied(Item::File, &pathname, icon);
                    }
                });
            }
        });
        if !unconditional {
            let msg = "needs one icon without a trigger";
            warn(key, ErrorKey::Validation, msg);
        }

        unconditional = false;
        vd.field_validated_bvs("asset", |bv, data| match bv {
            BV::Value(asset) => {
                unconditional = true;
                data.verify_exists(Item::Asset, asset);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                if !block.has_key("trigger") {
                    unconditional = true;
                }
                vd.field_validated_block("trigger", |block, data| {
                    validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_validated_value("reference", |_, asset, data| {
                    data.verify_exists(Item::Asset, asset);
                });
            }
        });
        if !unconditional {
            let msg = "needs at least one asset without a trigger";
            warn(key, ErrorKey::Validation, msg);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactFeature {}

impl ArtifactFeature {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactFeature, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactFeature {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // TODO: it's not clear what the scope is for these triggers
        let mut sc = ScopeContext::new_unrooted(Scopes::Artifact | Scopes::Character, key.clone());
        sc.define_name("newly_created_artifact", key.clone(), Scopes::Artifact);
        sc.define_name("owner", key.clone(), Scopes::Character);
        sc.define_name("wealth", key.clone(), Scopes::Value);

        let loca = format!("feature_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("group", Item::ArtifactFeatureGroup);
        vd.field_script_value("weight", &mut sc);

        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactFeatureGroup {}

impl ArtifactFeatureGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactFeatureGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactFeatureGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut _vd = Validator::new(block, data);
    }
}
