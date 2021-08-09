use crate::{traits::identity, CommonConfig};
use frame_support::{decl_event, weights::Weight};
use polymesh_primitives::{Balance, EventDid};
use sp_runtime::transaction_validity::InvalidTransaction;

pub trait WeightInfo {
    fn set_paying_key() -> Weight;
    fn accept_paying_key() -> Weight;
    fn remove_paying_key() -> Weight;
    fn update_polyx_limit() -> Weight;
    fn increase_polyx_limit() -> Weight;
    fn decrease_polyx_limit() -> Weight;
}

pub trait SubsidiserTrait<AccountId> {
    /// Check if a `user_key` has a subsidiser and that the subsidy can pay the `fee`.
    fn check_subsidy(
        user_key: &AccountId,
        fee: Balance,
        pallet: Option<&[u8]>,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
    /// Debit `fee` from the remaining balance of the subsidy for `user_key`.
    fn debit_subsidy(
        user_key: &AccountId,
        fee: Balance,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
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
        AuthorizedPayingKey(EventDid, AccountId, AccountId, Balance, u64),

        /// Accepted paying key.
        ///
        /// (Caller DID, User Key, Paying Key)
        AcceptedPayingKey(EventDid, AccountId, AccountId),

        /// Removed paying key.
        ///
        /// (Caller DID, User Key, Paying Key)
        RemovedPayingKey(EventDid, AccountId, AccountId),

        /// Updated polyx limit.
        ///
        /// (Caller DID, User Key, Paying Key, POLYX limit, old remaining POLYX)
        UpdatedPolyxLimit(EventDid, AccountId, AccountId, Balance, Balance),
    }
}
