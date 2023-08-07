use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target;

pub fn validate_remove_subunit_loyalty(
    _key: &Token,
    value: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    if !value.is("yes") {
        validate_target(value, data, sc, Scopes::SubUnit);
    }
}
