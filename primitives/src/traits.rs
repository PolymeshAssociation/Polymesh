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
}
