
use frame_support::{
    traits::{ Currency },
    dispatch::DispatchError, 
};
use sp_std::{result};
use crate::key;
use key::Key;
use crate::identity_id;
use identity_id::IdentityId;
use crate::AccountId;

pub trait IdentityCurrency<AccountId>: Currency<AccountId> {

    fn withdraw_identity_balance(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::NegativeImbalance, DispatchError>;

    fn charge_fee_to_identity(who: &Key) -> Option<IdentityId>;

}