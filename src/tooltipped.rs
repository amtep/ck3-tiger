#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Tooltipped {
    No,
    Yes,
    /// for triggers
    FailuresOnly,
    /// for effects
    Past,
}

impl Tooltipped {
    pub fn is_tooltipped(self) -> bool {
        !matches!(self, Tooltipped::No)
    }
}
