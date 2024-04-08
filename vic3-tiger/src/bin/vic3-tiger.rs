use anyhow::Result;
use tiger_bin_shared::{run, GameConsts, PackageEnv};

fn main() -> Result<()> {
    run(
        GameConsts {
            name: "Victoria 3",
            name_short: "Vic3",
            version: "1.6.0 (BLACKCURRANT)",
            dir: "steamapps/common/Victoria 3",
            app_id: "529340",
            signature_file: "game/events/titanic_events.txt",
        },
        PackageEnv { name: env!("CARGO_PKG_NAME"), version: env!("CARGO_PKG_VERSION") },
    )
}
