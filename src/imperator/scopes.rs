use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;

pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    // TODO: - imperator - add this when Item::Family exists
    // if scopes == Scopes::Family && data.item_exists(Item::Family, arg) {
    //     return Some("fam");
    // }
    // TODO: - imperator - add this when Item::Character exists
    // if scopes == Scopes::Character && data.item_exists(Item::Character, arg) {
    //     return Some("char");
    // }
    if scopes == Scopes::Party && data.item_exists(Item::PartyType, arg) {
        return Some("party");
    }
    if scopes == Scopes::Treasure && data.item_exists(Item::Treasure, arg) {
        return Some("treasure");
    }
    if scopes == Scopes::Region && data.item_exists(Item::Region, arg) {
        return Some("region");
    }
    if scopes == Scopes::Area && data.item_exists(Item::Area, arg) {
        return Some("area");
    }
    if scopes == Scopes::Culture && data.item_exists(Item::Culture, arg) {
        return Some("culture");
    }
    if scopes == Scopes::Deity && data.item_exists(Item::Deity, arg) {
        return Some("deity");
    }
    if scopes == Scopes::Country {
        return Some("c");
    }
    if scopes == Scopes::Religion && data.item_exists(Item::Religion, arg) {
        return Some("religion");
    }
    if scopes == Scopes::Flag {
        return Some("flag");
    }
    if scopes == Scopes::Province && data.item_exists(Item::Province, arg) {
        return Some("p");
    }
    None
}
