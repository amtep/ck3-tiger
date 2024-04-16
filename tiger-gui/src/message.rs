use crate::mods::Mod;

#[derive(Debug, Clone)]
pub enum Message {
    ShowResults(Mod),
}
