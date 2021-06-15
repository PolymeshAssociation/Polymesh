use crate::{traits::identity, CommonTrait};
use frame_support::dispatch::DispatchResult;
use frame_support::{decl_event, weights::Weight};
use polymesh_primitives::{EventDid, IdentityId, Signatory};

pub trait WeightInfo {
    fn set_paying_key() -> Weight;
    fn accept_paying_key() -> Weight;
    fn remove_paying_key() -> Weight;
}

/// This trait is used to add a paying key to a user key.
pub trait RelayerSubTrait<AccountId> {
    /// Accepts and adds a paying key to a user key
    ///
    /// # Arguments
    /// * `signer` - DID/key of the signer.
    /// * `from` - The DID that sent the authorization.
    /// * `user_key` - The user key.
    /// * `paying_key` - The paying key.
    fn auth_accept_paying_key(
        signer: Signatory<AccountId>,
        from: IdentityId,
        user_key: AccountId,
        paying_key: AccountId,
    ) -> DispatchResult;

    /// Check if the signer can set `paying_key` for `user_key`
    ///
    /// # Arguments
    /// * `from` - The DID that sent the authorization.
    /// * `user_key` - The user key.
    /// * `paying_key` - The paying key.
    fn ensure_set_paying_key(
        from: IdentityId,
        user_key: &AccountId,
        paying_key: &AccountId,
    ) -> DispatchResult;
}

pub trait Trait: CommonTrait + identity::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Authorization given for `paying_key` to `user_key`.
        ///
        /// (Caller DID, User Key, Paying Key, auth id)
        PayingKeyAuthorized(EventDid, AccountId, AccountId, u64),

        /// Accepted paying key.
        ///
        /// (Caller DID, User Key, Paying Key)
        AcceptedPayingKey(EventDid, AccountId, AccountId),

        /// Remove paying key.
        ///
        /// (Caller DID, User Key, Paying Key)
        RemovedPayingKey(EventDid, AccountId, AccountId),

    }
}
