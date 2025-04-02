#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString)]
#[strum(use_phf)]
pub enum Hoi4Datatype {
    Ace,
    Building,
    Character,
    Country,
    IndustrialOrg,
    LocalizationEnvironment,
    Operation,
    Province,
    PurchaseContract,
    SpecialProject,
    State,
    Terrain,
    UnitLeader,
}
