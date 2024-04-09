use tiger_bin_shared::{GameConsts, PackageEnv};

pub const GAME_CONSTS: GameConsts = GameConsts {
    name: "Victoria 3",
    name_short: "Vic3",
    version: "1.6.0 (BLACKCURRANT)",
    dir: "steamapps/common/Victoria 3",
    app_id: "529340",
    signature_file: "game/events/titanic_events.txt",
    paradox_dir: "Victoria 3",
};

pub const PACKAGE_ENV: PackageEnv =
    PackageEnv { name: env!("CARGO_PKG_NAME"), version: env!("CARGO_PKG_VERSION") };
