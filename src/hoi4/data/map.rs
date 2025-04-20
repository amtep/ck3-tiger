use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Continents {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Continent, Continents::add)
}

impl Continents {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("continents") {
            for continent in block.iter_values_warn() {
                db.add_flag(Item::Continent, continent.clone());
            }
        } else {
            warn(ErrorKey::UnknownField)
                .msg("unexpected key")
                .info("only `continents` list should be defined here")
                .loc(key)
                .push();
        }
        db.set_flag_validator(Item::Continent, |flag, data| {
            let adj = format!("{flag}_adj");
            data.verify_exists(Item::Localization, flag);
            data.verify_exists_implied(Item::Localization, &adj, flag);
        });
    }
}

#[derive(Clone, Debug)]
pub struct AdjacencyRule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::AdjacencyRule, AdjacencyRule::add)
}

impl AdjacencyRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("adjacency_rule") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::AdjacencyRule, name.clone(), block, Box::new(Self {}));
            } else {
                warn(ErrorKey::FieldMissing)
                    .msg("missing `name` field in adjacency_rule")
                    .loc(key)
                    .push();
            }
        } else {
            warn(ErrorKey::UnknownField)
                .msg("unexpected key")
                .info("only `adjacency_rule` is a valid key here")
                .loc(key)
                .push();
        }
    }
}

impl DbKind for AdjacencyRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_adjacency_rule(key, block, data);
    }
}

fn validate_adjacency_rule(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_item("name", Item::Localization);

    for state in ["contested", "friend", "enemy", "neutral"] {
        vd.field_validated_block(state, |block, data| {
            let mut vd = Validator::new(block, data);
            for kind in ["army", "navy", "submarine", "trade"] {
                vd.field_bool(kind);
            }
        });
    }

    vd.field_list_items("required_provinces", Item::Province);
    vd.field_trigger("is_disabled", Scopes::Country, Tooltipped::Inner);
    vd.field_item("icon", Item::Province);
    vd.field_list_numeric_exactly("offset", 3);

    vd.field_trigger("is_enemy", Scopes::Country, Tooltipped::No);
    vd.field_trigger("is_friend", Scopes::Country, Tooltipped::No);
    vd.field_trigger("is_neutral", Scopes::Country, Tooltipped::No);
}

#[derive(Clone, Debug)]
pub struct StrategicRegion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::StrategicRegion, StrategicRegion::add)
}

impl StrategicRegion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("strategic_region") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::StrategicRegion, id.clone(), block, Box::new(Self {}));
            } else {
                warn(ErrorKey::FieldMissing)
                    .msg("missing `id` field in strategic_region")
                    .loc(key)
                    .push();
            }
        } else {
            warn(ErrorKey::UnknownField)
                .msg("unexpected key")
                .info("only `strategic_region` is a valid key here")
                .loc(key)
                .push();
        }
    }
}

impl DbKind for StrategicRegion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_strategic_region(key, block, data);
    }
}

fn validate_strategic_region(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field("id");
    vd.field_item("name", Item::Localization);
    vd.field_list_items("provinces", Item::Province);
    vd.field_validated_block("weather", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.multi_field_validated_block("period", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_numeric_exactly("between", 2); // TODO: verify date
            vd.field_list_numeric_exactly("temperature", 2);
            for w in [
                "no_phenomenon",
                "rain_light",
                "rain_heavy",
                "snow",
                "blizzard",
                "arctic_water",
                "mud",
                "sandstorm",
            ] {
                vd.field_numeric(w);
            }
            vd.field_numeric("min_snow_level");
        });
    });
}

#[derive(Clone, Debug)]
pub struct SupplyArea {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::SupplyArea, SupplyArea::add)
}

impl SupplyArea {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("supply_area") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::SupplyArea, id.clone(), block, Box::new(Self {}));
            } else {
                warn(ErrorKey::FieldMissing)
                    .msg("missing `id` field in supply_area")
                    .loc(key)
                    .push();
            }
        } else {
            warn(ErrorKey::UnknownField)
                .msg("unexpected key")
                .info("only `supply_area` is a valid key here")
                .loc(key)
                .push();
        }
    }
}

impl DbKind for SupplyArea {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_supply_area(key, block, data);
    }
}

fn validate_supply_area(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field("id");
    vd.field_item("name", Item::Localization);
    vd.field_integer("value");
    vd.field_list_items("states", Item::State);
}
