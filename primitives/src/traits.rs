use crate::{account_key::AccountKey, identity_id::IdentityId};

use frame_support::{dispatch::DispatchError, traits::Currency};
use sp_std::result;

#[allow(missing_docs)]
pub trait IdentityCurrency<AccountId>: Currency<AccountId> {
    fn withdraw_identity_balance(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::NegativeImbalance, DispatchError>;

    fn charge_fee_to_identity(who: &AccountKey) -> Option<IdentityId>;

    /// Mints `value` to the free balance of `who`.
    ///
    /// If `who` doesn't exist, nothing is done and an `Err` returned.
    fn deposit_into_existing_identity(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::PositiveImbalance, DispatchError>;

    /// Similar to deposit_creating, only accepts a `NegativeImbalance` and returns nothing on
    /// success.
    fn resolve_into_existing_identity(
        who: &IdentityId,
        value: Self::NegativeImbalance,
    ) -> result::Result<(), Self::NegativeImbalance>;
}

/// A currency that has a block rewards reserve.
pub trait BlockRewardsReserveCurrency<Balance, NegativeImbalance> {
    /// An instance of `Drop` for positive imbalance.
    fn drop_positive_imbalance(amount: Balance);
    /// An instance of `Drop` for negative imbalance.
    fn drop_negative_imbalance(amount: Balance);
    /// Issues a given amount of currency from the block rewards reserve if possible.
    fn issue_using_block_rewards_reserve(amount: Balance) -> NegativeImbalance;
    /// Returns the balance of the block rewards reserve.
    fn block_rewards_reserve_balance() -> Balance;
}
