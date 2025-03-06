//! Dealing with which game we are validating

use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use bitflags::bitflags;

use crate::helpers::display_choices;

/// Records at runtime which game we are validating, in case there are multiple feature flags set.
static GAME: OnceLock<Game> = OnceLock::new();

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
    /// Will panic if called before [`Game::set`].
    #[allow(clippy::self_named_constructors)] // not a constructor
    #[allow(unreachable_code)]
    pub fn game() -> Game {
        #[cfg(all(feature = "ck3", not(feature = "vic3"), not(feature = "imperator")))]
        return Game::Ck3;
        #[cfg(all(feature = "vic3", not(feature = "ck3"), not(feature = "imperator")))]
        return Game::Vic3;
        #[cfg(all(feature = "imperator", not(feature = "ck3"), not(feature = "vic3")))]
        return Game::Imperator;
        *GAME.get().expect("internal error: don't know which game we are validating")
    }

    /// Convenience function indicating whether we are validating Crusader Kings 3 mods.
    pub(crate) fn is_ck3() -> bool {
        #[cfg(not(feature = "ck3"))]
        return false;
        #[cfg(all(feature = "ck3", not(feature = "vic3"), not(feature = "imperator")))]
        return true;
        #[cfg(all(feature = "ck3", any(feature = "vic3", feature = "imperator")))]
        return GAME.get() == Some(&Game::Ck3);
    }

    /// Convenience function indicating whether we are validating Victoria 3 mods.
    pub(crate) fn is_vic3() -> bool {
        #[cfg(not(feature = "vic3"))]
        return false;
        #[cfg(all(feature = "vic3", not(feature = "ck3"), not(feature = "imperator")))]
        return true;
        #[cfg(all(feature = "vic3", any(feature = "ck3", feature = "imperator")))]
        return GAME.get() == Some(&Game::Vic3);
    }

    /// Convenience function indicating whether we are validating Imperator: Rome mods.
    pub(crate) fn is_imperator() -> bool {
        #[cfg(not(feature = "imperator"))]
        return false;
        #[cfg(all(feature = "imperator", not(feature = "ck3"), not(feature = "vic3")))]
        return true;
        #[cfg(all(feature = "imperator", any(feature = "ck3", feature = "vic3")))]
        return GAME.get() == Some(&Game::Imperator);
    }
}

bitflags! {
    /// A set of bitflags to indicate for which game something is intended,
    /// independent of which game we are validating.
    ///
    /// This way, error messages about things being used in the wrong game can be given at runtime.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameFlags: u8 {
        const Ck3 = 0x01;
        const Vic3 = 0x02;
        const Imperator = 0x04;
    }
}

impl GameFlags {
    /// Get a [`GameFlags`] value representing the game being validated.
    /// Useful for checking with `.contains`.
    pub fn game() -> Self {
        // Unfortunately we have to translate between the types here.
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => GameFlags::Ck3,
            #[cfg(feature = "vic3")]
            Game::Vic3 => GameFlags::Vic3,
            #[cfg(feature = "imperator")]
            Game::Imperator => GameFlags::Imperator,
        }
    }
}

impl Display for GameFlags {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut vec = Vec::new();
        if self.contains(Self::Ck3) {
            vec.push("Crusader Kings 3");
        }
        if self.contains(Self::Vic3) {
            vec.push("Victoria 3");
        }
        if self.contains(Self::Imperator) {
            vec.push("Imperator: Rome");
        }
        display_choices(f, &vec, "and")
    }
}

impl From<Game> for GameFlags {
    /// Convert a [`Game`] into a [`GameFlags`] with just that game's flag set.
    fn from(game: Game) -> Self {
        match game {
            #[cfg(feature = "ck3")]
            Game::Ck3 => GameFlags::Ck3,
            #[cfg(feature = "vic3")]
            Game::Vic3 => GameFlags::Vic3,
            #[cfg(feature = "imperator")]
            Game::Imperator => GameFlags::Imperator,
        }
    }
}
