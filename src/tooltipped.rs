#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Tooltipped {
    No,
    Yes,
    FailuresOnly, // for triggers
    Past,         // for effects
}

impl Tooltipped {
    pub fn is_tooltipped(self) -> bool {
        !matches!(self, Tooltipped::No)
    }
}
