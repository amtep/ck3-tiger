use std::fmt::Formatter;

use crate::modif::ModifKinds;

pub fn display_fmt(mk: ModifKinds, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if mk.contains(ModifKinds::Battle) {
        vec.push("battle");
    }
    if mk.contains(ModifKinds::Building) {
        vec.push("building");
    }
    if mk.contains(ModifKinds::Character) {
        vec.push("character");
    }
    if mk.contains(ModifKinds::Country) {
        vec.push("country");
    }
    if mk.contains(ModifKinds::InterestGroup) {
        vec.push("interest group");
    }
    if mk.contains(ModifKinds::Market) {
        vec.push("market");
    }
    if mk.contains(ModifKinds::PoliticalMovement) {
        vec.push("political movement");
    }
    if mk.contains(ModifKinds::State) {
        vec.push("state");
    }
    if mk.contains(ModifKinds::Tariff) {
        vec.push("tariff");
    }
    if mk.contains(ModifKinds::Tax) {
        vec.push("tax");
    }
    if mk.contains(ModifKinds::Unit) {
        vec.push("unit");
    }
    if mk.contains(ModifKinds::Goods) {
        vec.push("goods");
    }
    if mk.contains(ModifKinds::MilitaryFormation) {
        vec.push("military formation");
    }
    write!(f, "{}", vec.join(", "))
}
