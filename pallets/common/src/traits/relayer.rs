use crate::{traits::identity, CommonConfig};
use frame_support::dispatch::DispatchResult;
use frame_support::{decl_event, weights::Weight};
use polymesh_primitives::{Balance, EventDid};

pub trait WeightInfo {
    fn set_paying_key() -> Weight;
    fn accept_paying_key() -> Weight;
    fn remove_paying_key() -> Weight;
    fn update_polyx_limit() -> Weight;
}

/// This trait is used for checking if a key is used as a paying key by Relayer.
pub trait IdentityToRelayer<AccountId> {
    /// Ensure that `key` is not being used as a paying key for a user key.
    fn ensure_paying_key_is_unused(key: &AccountId) -> DispatchResult;
}

pub trait Config: CommonConfig + identity::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Authorization given for `paying_key` to `user_key`.
        ///
        /// (Caller DID, User Key, Paying Key, Initial POLYX limit, Auth ID)
        PayingKeyAuthorized(EventDid, AccountId, AccountId, Balance, u64),

        /// Accepted paying key.
        ///
        /// (Caller DID, User Key, Paying Key)
        AcceptedPayingKey(EventDid, AccountId, AccountId),

        /// Remove paying key.
        ///
        /// (Caller DID, User Key, Paying Key)
        RemovedPayingKey(EventDid, AccountId, AccountId),

        /// Update polyx limit
        ///
        /// (Caller DID, User Key, Paying Key, POLYX limit)
        UpdatePolyxLimit(EventDid, AccountId, AccountId, Balance),
    }
}
