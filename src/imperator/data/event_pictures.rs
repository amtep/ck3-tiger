use crate::block::Block;
use crate::block::BV;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::report;
use crate::report::ErrorKey;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;
use crate::Severity;

#[derive(Clone, Debug)]
pub struct EventPicture {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::EventPicture, EventPicture::add)
}

impl EventPicture {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventPicture, key, block, Box::new(Self {}));
    }
}

impl DbKind for EventPicture {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_item("theme", Item::EventTheme);

        vd.multi_field_validated("picture", |bv, data| match bv {
            BV::Value(t) => verify_exists_or_empty(data, t, Severity::Error),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_item("texture", Item::File);
                vd.field_validated_block("trigger", |block, data| {
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });
            }
        });
    }
}

pub fn verify_exists_or_empty(data: &Everything, file: &Token, max_sev: Severity) {
    let file_str = file.as_str();
    if file_str.is_empty() {
        return;
    }
    data.fileset.mark_used(&file_str.replace("//", "/"));
    if !data.fileset.exists(file_str) {
        let msg = format!("file {file_str} does not exist");
        report(ErrorKey::MissingFile, Item::File.severity().at_most(max_sev))
            .msg(msg)
            .loc(file)
            .push();
    }
}
