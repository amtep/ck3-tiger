use anyhow::Result;
use ck3_tiger::GAME_CONSTS;
use tiger_bin_shared::tiger;

fn main() -> Result<()> {
    tiger(GAME_CONSTS, env!("CARGO_PKG_VERSION"))
}
