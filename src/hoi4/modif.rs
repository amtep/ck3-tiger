use std::fmt::Formatter;

use crate::modif::ModifKinds;

// TODO

pub fn display_fmt(mk: ModifKinds, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if mk.contains(ModifKinds::Aggressive) {
        vec.push("aggressive");
    }
    if mk.contains(ModifKinds::Ai) {
        vec.push("ai");
    }
    if mk.contains(ModifKinds::Air) {
        vec.push("air");
    }
    if mk.contains(ModifKinds::Army) {
        vec.push("army");
    }
    if mk.contains(ModifKinds::Autonomy) {
        vec.push("autonomy");
    }
    if mk.contains(ModifKinds::Character) {
        vec.push("character");
    }
    if mk.contains(ModifKinds::Country) {
        vec.push("country");
    }
    if mk.contains(ModifKinds::Defensive) {
        vec.push("defensive");
    }
    if mk.contains(ModifKinds::GovernmentInExile) {
        vec.push("government in exile");
    }
    if mk.contains(ModifKinds::IntelligenceAgency) {
        vec.push("intelligence agency");
    }
    if mk.contains(ModifKinds::MilitaryAdvancements) {
        vec.push("military advancements");
    }
    if mk.contains(ModifKinds::Naval) {
        vec.push("naval");
    }
    if mk.contains(ModifKinds::Peace) {
        vec.push("peace");
    }
    if mk.contains(ModifKinds::Politics) {
        vec.push("politics");
    }
    if mk.contains(ModifKinds::Scientist) {
        vec.push("scientist");
    }
    if mk.contains(ModifKinds::State) {
        vec.push("state");
    }
    if mk.contains(ModifKinds::UnitLeader) {
        vec.push("unit leader");
    }
    if mk.contains(ModifKinds::WarProduction) {
        vec.push("war production");
    }
    write!(f, "{}", vec.join(", "))
}
