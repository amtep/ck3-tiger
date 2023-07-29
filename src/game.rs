use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;

static GAME: OnceCell<Game> = OnceCell::new();

/// Enum specifying which game we are validating.
///
/// This enum is meant to be optimized away entirely when there is only one feature flag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Game {
    #[cfg(feature = "ck3")]
    Ck3,
    #[cfg(feature = "vic3")]
    Vic3,
    #[cfg(feature = "imperator")]
    Imperator,
}

impl Game {
    /// Decide which game we are validating. Should be called as early as possible.
    /// Returns an error if called more than once.
    pub fn set(game: Game) -> Result<()> {
        GAME.set(game).map_err(|_| anyhow!("tried to set game type twice"))?;
        Ok(())
    }

    /// Return which game we are validating. Should only be called after [`Game::set`].
    ///
    /// ## Panics
    /// Will panic if called before [`Game::Set`].
    #[allow(clippy::self_named_constructors)] // not a constructor
    pub fn game() -> Game {
        *GAME.get().expect("internal error: don't know which game we are validating")
    }

    /// Convenience function indicating whether we are validating Crusader Kings 3 mods.
    pub(crate) fn is_ck3() -> bool {
        #[cfg(not(feature = "ck3"))]
        return false;
        #[cfg(all(feature = "ck3", not(feature = "vic3"), not(feature = "imperator")))]
        return true;
        #[cfg(all(feature = "ck3", any(feature = "vic3", feature = "imperator")))]
        return GAME.get() == Some(Game::Ck3);
    }

    /// Convenience function indicating whether we are validating Victoria 3 mods.
    pub(crate) fn is_vic3() -> bool {
        #[cfg(not(feature = "vic3"))]
        return false;
        #[cfg(all(feature = "vic3", not(feature = "ck3"), not(feature = "imperator")))]
        return true;
        #[cfg(all(feature = "vic3", any(feature = "ck3", feature = "imperator")))]
        return GAME.get() == Some(Game::Vic3);
    }

    /// Convenience function indicating whether we are validating Imperator: Rome mods.
    pub(crate) fn is_imperator() -> bool {
        #[cfg(not(feature = "imperator"))]
        return false;
        #[cfg(all(feature = "imperator", not(feature = "ck3"), not(feature = "vic3")))]
        return true;
        #[cfg(all(feature = "imperator", any(feature = "ck3", feature = "vic3")))]
        return GAME.get() == Some(Game::Imperator);
    }
}
