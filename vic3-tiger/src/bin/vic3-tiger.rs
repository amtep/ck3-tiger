use anyhow::Result;
use tiger_bin_shared::tiger;
use vic3_tiger::{GAME_CONSTS, PACKAGE_ENV};

fn main() -> Result<()> {
    tiger(GAME_CONSTS, PACKAGE_ENV)
}
