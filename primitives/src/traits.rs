use crate::identity_id;
use crate::key;
use crate::AccountId;
use frame_support::{dispatch::DispatchError, traits::Currency};
use identity_id::IdentityId;
use key::Key;
use sp_std::result;

pub trait IdentityCurrency<AccountId>: Currency<AccountId> {
    fn withdraw_identity_balance(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::NegativeImbalance, DispatchError>;

    fn charge_fee_to_identity(who: &Key) -> Option<IdentityId>;
}
