use std::fmt::Formatter;

use crate::modif::ModifKinds;

pub fn display_fmt(mk: ModifKinds, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if mk.contains(ModifKinds::Character) {
        vec.push("character");
    }
    if mk.contains(ModifKinds::Country) {
        vec.push("country");
    }
    if mk.contains(ModifKinds::Province) {
        vec.push("province");
    }
    if mk.contains(ModifKinds::State) {
        vec.push("state");
    }
    write!(f, "{}", vec.join(", "))
}
