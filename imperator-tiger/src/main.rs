use anyhow::Result;
use tiger_bin_shared::{run, GameConsts, PackageEnv};

fn main() -> Result<()> {
    run(
        GameConsts {
            name: "Imperator Rome",
            name_short: "Imperator",
            version: "2.0.4",
            dir: "steamapps/common/ImperatorRome",
            app_id: "859580",
            signature_file: "game/events/000_johan_debug.txt",
        },
        PackageEnv { name: env!("CARGO_PKG_NAME"), version: env!("CARGO_PKG_VERSION") },
    )
}
