use anyhow::Result;
use imperator_tiger::GAME_CONSTS;
use tiger_bin_shared::tiger;

fn main() -> Result<()> {
    tiger(GAME_CONSTS, env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_NAME"))
}
