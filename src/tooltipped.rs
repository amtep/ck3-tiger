#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Tooltipped {
    No,
    Yes,
    Negated,             // for triggers
    FailuresOnly,        // for triggers
    NegatedFailuresOnly, // for triggers
    Past,                // for effects
}

impl Tooltipped {
    pub fn is_tooltipped(self) -> bool {
        !matches!(self, Tooltipped::No)
    }

    pub fn is_failures_only(self) -> bool {
        matches!(
            self,
            Tooltipped::FailuresOnly | Tooltipped::NegatedFailuresOnly
        )
    }

    pub fn no_longer_failures_only(self) -> Self {
        match self {
            Tooltipped::FailuresOnly => Tooltipped::Yes,
            Tooltipped::NegatedFailuresOnly => Tooltipped::Negated,
            other => other,
        }
    }

    pub fn negated(self) -> Self {
        match self {
            Tooltipped::No => Tooltipped::No,
            Tooltipped::Yes | Tooltipped::Past => Tooltipped::Negated,
            Tooltipped::Negated => Tooltipped::Yes,
            Tooltipped::FailuresOnly => Tooltipped::NegatedFailuresOnly,
            Tooltipped::NegatedFailuresOnly => Tooltipped::FailuresOnly,
        }
    }
}
