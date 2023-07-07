use std::fmt::{Display, Formatter};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct ModifKinds: u8 {
        const Character = 0x01;
        const Province = 0x02;
        const County = 0x04;
        const Terrain = 0x08;
        const Culture = 0x10;
        const Scheme = 0x20;
        const TravelPlan = 0x40;
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut vec = Vec::new();
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Province) {
            vec.push("province");
        }
        if self.contains(ModifKinds::County) {
            vec.push("county");
        }
        if self.contains(ModifKinds::Terrain) {
            vec.push("terrain");
        }
        if self.contains(ModifKinds::Culture) {
            vec.push("culture");
        }
        if self.contains(ModifKinds::Scheme) {
            vec.push("scheme");
        }
        if self.contains(ModifKinds::TravelPlan) {
            vec.push("travel plan");
        }
        write!(f, "{}", vec.join(", "))
    }
}
