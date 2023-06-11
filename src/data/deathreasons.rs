use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct DeathReason {}

impl DeathReason {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DeathReason, key, block, Box::new(Self {}));
    }
}

impl DbKind for DeathReason {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());

        vd.field_bool("public_knowledge");
        vd.field_bool("natural");
        vd.field_integer("priority");
        vd.field_bool("default");

        if let Some(icon) = vd.field_value("icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(key, "NGameIcons|DEATH_REASON_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}");
                data.fileset.verify_exists_implied(&pathname, icon);
            }
        }

        vd.field_validated_block("natural_death_trigger", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_item("use_equipped_artifact_in_slot", Item::ArtifactSlot);
    }
}
