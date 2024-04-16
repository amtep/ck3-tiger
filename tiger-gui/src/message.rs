use crate::mods::Mod;
use crate::reports::Results;

#[derive(Debug, Clone)]
pub enum Message {
    ShowResults(Mod),
    ModScanned((Mod, Result<Results, String>)),
}
