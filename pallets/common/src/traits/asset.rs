use frame_support::dispatch::DispatchResult;
use polymesh_primitives::IdentityId;

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
