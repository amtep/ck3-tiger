use tiger_bin_shared::{GameConsts, PackageEnv};

pub const GAME_CONSTS: GameConsts = GameConsts {
    name: "Imperator Rome",
    name_short: "Imperator",
    version: "2.0.4",
    dir: "steamapps/common/ImperatorRome",
    app_id: "859580",
    signature_file: "game/events/000_johan_debug.txt",
    paradox_dir: "Imperator",
};

pub const PACKAGE_ENV: PackageEnv =
    PackageEnv { name: env!("CARGO_PKG_NAME"), version: env!("CARGO_PKG_VERSION") };
