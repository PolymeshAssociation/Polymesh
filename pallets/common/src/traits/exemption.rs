use polymesh_primitives::{IdentityId, Ticker};

pub trait Trait {
    fn is_exempted(ticker: &Ticker, tm: u16, did: IdentityId) -> bool;
}
