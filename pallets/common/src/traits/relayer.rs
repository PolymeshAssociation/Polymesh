use crate::traits::identity;
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

pub trait SubsidiserTrait<AccountId, RuntimeCall> {
    /// Check if a `user_key` has a subsidiser and that the subsidy can pay the `fee`.
    fn check_subsidy(
        user_key: &AccountId,
        fee: Balance,
        call: Option<&RuntimeCall>,
    ) -> Result<Option<AccountId>, InvalidTransaction>;

    /// Debit `fee` from the remaining balance of the subsidy for `user_key`.
    fn debit_subsidy(
        user_key: &AccountId,
        fee: Balance,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
}

pub trait Config: frame_system::Config + identity::Config {
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    /// Subsidy pallet weights.
    type WeightInfo: WeightInfo;
    /// Subsidy call filter.
    type SubsidyCallFilter: frame_support::traits::Contains<Self::RuntimeCall>;
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
