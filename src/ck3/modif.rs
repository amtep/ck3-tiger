use std::fmt::Formatter;

use crate::modif::ModifKinds;

pub fn display_fmt(mk: ModifKinds, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if mk.contains(ModifKinds::Character) {
        vec.push("character");
    }
    if mk.contains(ModifKinds::Province) {
        vec.push("province");
    }
    if mk.contains(ModifKinds::County) {
        vec.push("county");
    }
    if mk.contains(ModifKinds::Terrain) {
        vec.push("terrain");
    }
    if mk.contains(ModifKinds::Culture) {
        vec.push("culture");
    }
    if mk.contains(ModifKinds::Scheme) {
        vec.push("scheme");
    }
    if mk.contains(ModifKinds::TravelPlan) {
        vec.push("travel plan");
    }
    write!(f, "{}", vec.join(", "))
}
