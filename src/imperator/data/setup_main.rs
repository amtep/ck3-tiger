use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;
use crate::modif::{validate_modifs, ModifKinds};
use crate::imperator::tables::misc::{DLC_IMPERATOR};

#[derive(Clone, Debug)]
pub struct SetupMain {}

impl SetupMain {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decision, key, block, Box::new(Self {}));
    }
}

impl DbKind for SetupMain {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("treasure_manager", |block, data| {
            validate_treasures(block, data);
        });
        vd.field_validated_block("family", |block, data| {
            validate_families(block, data);
        });
        vd.field_validated_block("diplomacy", |block, data| {
            validate_diplomacy(block, data);
        });
        vd.field_validated_block("provinces", |block, data| {
            validate_provinces(block, data);
        });
        vd.field_validated_block("road_network", |block, data| {
            validate_roads(block, data);
        });
        vd.field_validated_block("country", |block, data| {
            validate_countries(block, data);
        });
        vd.field_validated_block("trade", |block, data| {
            validate_trade(block, data);
        });
        vd.field_validated_block("provinces", |block, data| {
            validate_provinces(block, data);
        });
        vd.field_validated_block("great_work_manager", |block, data| {
            validate_great_works(block, data);
        });
    }
}

impl FileHandler<Block> for SetupMain {
    // TODO - Is this impl needed? Copied it from src/vic3/data/history.rs
    fn subpath(&self) -> PathBuf {
        PathBuf::from("setup/main/")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

fn validate_treasures(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("database", |block, data| {
        for (_, block) in vd.integer_blocks() {
            vd.field_item("key", Item::Localization);
            vd.choice(DLC_IMPERATOR);
            vd.field("icon"); // TODO - icon can be any icon declared in "gfx/interface/icons/treasures", how to check that?
            vd.multi_field_validated_block("state_modifier", |block, data| {
                validate_modifs(block, data, ModifKinds::Country | ModifKinds::Province | ModifKinds::State, vd);
            });
        }
    });
}

fn validate_families(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("families", |block, data| {
        for (_, block) in vd.integer_blocks() {
            vd.field_item("key", Item::Localization);
            vd.field_item("owner", Item::Country); // can be any 3 letter country tag declared in setup
            vd.field_item("culture", Item::Country);
            vd.field_integer("prestige");
            vd.field_integer("color");
        }
    });
}
fn validate_diplomacy(block: &Block, data: &Everything) {
    vd.multi_field_validated_block("dependency", |block, data| {
        vd.multi_field_item("member", Item::Country);
    });
    for field in &["defensive_league", "dependency", "guarantee", "alliance"] {

    }
    vd.multi_field_validated_block("defensive_league", |block, data| {
        vd.field_item("first", Item::Country);
        vd.field_item("second", Item::Country);
        vd.field_item("subject_type", Item::SubjectType);
    });
}
fn validate_provinces(block: &Block, data: &Everything) {
    // todo
}
fn validate_roads(block: &Block, data: &Everything) {
    // todo
}
fn validate_countries(block: &Block, data: &Everything) {
    // todo
}
fn validate_trade(block: &Block, data: &Everything) {
    // todo
}
fn validate_provinces(block: &Block, data: &Everything) {
    // todo
}
fn validate_great_works(block: &Block, data: &Everything) {
    // todo
}