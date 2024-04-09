use anyhow::Result;
use tiger_bin_shared::auto;
use vic3_tiger::GAME_CONSTS;

fn main() -> Result<()> {
    auto(GAME_CONSTS)
}
