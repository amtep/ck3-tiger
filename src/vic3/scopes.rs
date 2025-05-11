use crate::everything::Everything;
use crate::scopes::Scopes;

pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    use crate::item::Item;
    if scopes == Scopes::Building && data.item_exists(Item::BuildingType, arg) {
        return Some("b");
    }
    if scopes == Scopes::BuildingType && data.item_exists(Item::BuildingType, arg) {
        return Some("bt");
    }
    if scopes == Scopes::Country && data.item_exists(Item::Country, arg) {
        return Some("c");
    }
    if scopes == Scopes::CountryDefinition && data.item_exists(Item::Country, arg) {
        return Some("cd");
    }
    if scopes == Scopes::CompanyType && data.item_exists(Item::CompanyType, arg) {
        return Some("company_type");
    }
    if scopes == Scopes::Culture && data.item_exists(Item::Culture, arg) {
        return Some("cu");
    }
    if scopes == Scopes::Flag {
        return Some("flag");
    }
    if scopes == Scopes::Ideology && data.item_exists(Item::Ideology, arg) {
        return Some("i");
    }
    if scopes == Scopes::InterestGroup && data.item_exists(Item::InterestGroup, arg) {
        return Some("ig");
    }
    if scopes == Scopes::InterestGroupTrait && data.item_exists(Item::InterestGroupTrait, arg) {
        return Some("ig_trait");
    }
    if scopes == Scopes::InterestGroupType && data.item_exists(Item::InterestGroup, arg) {
        return Some("ig_type");
    }
    if scopes == Scopes::Institution && data.item_exists(Item::Institution, arg) {
        return Some("institution");
    }
    if scopes == Scopes::JournalEntry && data.item_exists(Item::JournalEntry, arg) {
        return Some("je");
    }
    if scopes == Scopes::LawType && data.item_exists(Item::LawType, arg) {
        return Some("law_type");
    }
    if scopes == Scopes::MarketGoods && data.item_exists(Item::Goods, arg) {
        return Some("mg");
    }
    if scopes == Scopes::MobilizationOption && data.item_exists(Item::MobilizationOption, arg) {
        return Some("mobilization_option");
    }
    if scopes == Scopes::Decree && data.item_exists(Item::Decree, arg) {
        return Some("nf");
    }
    if scopes == Scopes::Province && data.item_exists(Item::Province, arg) {
        return Some("p");
    }
    if scopes == Scopes::PopType && data.item_exists(Item::PopType, arg) {
        return Some("pop_type");
    }
    if scopes == Scopes::Party && data.item_exists(Item::Party, arg) {
        return Some("py");
    }
    if scopes == Scopes::Religion && data.item_exists(Item::Religion, arg) {
        return Some("rel");
    }
    if scopes == Scopes::StateRegion && data.item_exists(Item::StateRegion, arg) {
        return Some("s");
    }
    if scopes == Scopes::StrategicRegion && data.item_exists(Item::StrategicRegion, arg) {
        return Some("sr");
    }
    if scopes == Scopes::CombatUnitType && data.item_exists(Item::CombatUnit, arg) {
        return Some("unit_type");
    }
    None
}
