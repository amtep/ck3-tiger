#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Tooltipped {
    No,
    Yes,
    Negated, // for triggers
    Past,    // for effects
}

impl Tooltipped {
    pub fn from_effect(self) -> Self {
        match self {
            Tooltipped::Past => Tooltipped::Yes,
            other => other,
        }
    }

    pub fn is_tooltipped(self) -> bool {
        !matches!(self, Tooltipped::No)
    }

    pub fn negated(self) -> Self {
        match self {
            Tooltipped::No => Tooltipped::No,
            Tooltipped::Yes | Tooltipped::Past => Tooltipped::Negated,
            Tooltipped::Negated => Tooltipped::Yes,
        }
    }
}
