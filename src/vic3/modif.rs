#![allow(non_upper_case_globals)]

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct ModifKinds: u16 {
        const NoneModifKind = 0x0001;
        const Battle = 0x0002;
        const Building = 0x0004;
        const Character = 0x0008;
        const Country = 0x0010;
        const Front = 0x0020;
        const InterestGroup = 0x0040;
        const Market = 0x0080;
        const PoliticalMovement = 0x0100;
        const State = 0x0200;
        const Tariff = 0x0400;
        const Tax = 0x0800;
        const Unit = 0x1000;
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut vec = Vec::new();
        if self.contains(ModifKinds::NoneModifKind) {
            vec.push("none");
        }
        if self.contains(ModifKinds::Battle) {
            vec.push("battle");
        }
        if self.contains(ModifKinds::Building) {
            vec.push("building");
        }
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Country) {
            vec.push("country");
        }
        if self.contains(ModifKinds::Front) {
            vec.push("front");
        }
        if self.contains(ModifKinds::InterestGroup) {
            vec.push("interest group");
        }
        if self.contains(ModifKinds::Market) {
            vec.push("market");
        }
        if self.contains(ModifKinds::PoliticalMovement) {
            vec.push("political movement");
        }
        if self.contains(ModifKinds::State) {
            vec.push("state");
        }
        if self.contains(ModifKinds::Tariff) {
            vec.push("tariff");
        }
        if self.contains(ModifKinds::Tax) {
            vec.push("tax");
        }
        if self.contains(ModifKinds::Unit) {
            vec.push("unit");
        }
        write!(f, "{}", vec.join(", "))
    }
}
