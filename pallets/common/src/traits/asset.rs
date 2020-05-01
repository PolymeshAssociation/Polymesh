use frame_support::dispatch::DispatchResult;
use polymesh_primitives::{IdentityId, Ticker};

/// This trait is used to call functions that accept transfer of a ticker or token ownership
pub trait AcceptTransfer {
    /// Accept and process a ticker transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current ticker owner
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;
    /// Accept and process a token ownership transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current token owner
    fn accept_token_ownership_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;
}

pub trait Trait<V, U> {
    fn total_supply(ticker: &Ticker) -> V;
    fn balance(ticker: &Ticker, did: IdentityId) -> V;
    fn _mint_from_sto(
        ticker: &Ticker,
        caller: U,
        sender_did: IdentityId,
        tokens_purchased: V,
    ) -> DispatchResult;
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool;
    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: u64) -> V;
}
