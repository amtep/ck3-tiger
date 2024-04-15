use std::path::PathBuf;

use crate::game::Game;

#[derive(Debug, Clone)]
pub enum Message {
    ShowResults(Game, PathBuf),
}
