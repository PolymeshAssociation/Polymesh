use polymesh_primitives::{IdentityId, Ticker};

use core::result::Result;

pub trait Trait<Balance> {
    fn verify_restriction(
        ticker: &Ticker,
        from_id: Option<IdentityId>,
        to_id: Option<IdentityId>,
        _value: Balance,
    ) -> Result<u8, &'static str>;
}
