use anyhow::Result;
use tiger_bin_shared::tiger;
use vic3_tiger::GAME_CONSTS;

fn main() -> Result<()> {
    tiger(GAME_CONSTS, env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_NAME"))
}
