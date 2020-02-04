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
