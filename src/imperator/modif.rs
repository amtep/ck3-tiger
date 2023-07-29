#![allow(non_upper_case_globals)]

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct ModifKinds: u16 {
        const NoneModifKind = 0x0001;
        const Character = 0x0002;
        const Country = 0x0004;
        const Province = 0x0008;
        const State = 0x0010;
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut vec = Vec::new();
        if self.contains(ModifKinds::NoneModifKind) {
            vec.push("none");
        }
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Country) {
            vec.push("country");
        }
        if self.contains(ModifKinds::Province) {
            vec.push("province");
        }
        if self.contains(ModifKinds::State) {
            vec.push("state");
        }
        write!(f, "{}", vec.join(", "))
    }
}
