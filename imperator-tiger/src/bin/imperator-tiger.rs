use anyhow::Result;
use imperator_tiger::{GAME_CONSTS, PACKAGE_ENV};
use tiger_bin_shared::tiger;

fn main() -> Result<()> {
    tiger(GAME_CONSTS, PACKAGE_ENV)
}
