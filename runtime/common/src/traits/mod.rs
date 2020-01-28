use polymesh_primitives::{ IdentityId, Signer };

use codec::{Codec, Decode, Encode};
use frame_support::{
    traits::{LockIdentifier, WithdrawReasons},
    dispatch::DispatchResult,
    Parameter,
};
use sp_runtime::traits::{
    CheckedSub, MaybeSerializeDeserialize, Member, Saturating, SimpleArithmetic,
};


#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}

pub trait CommonTrait: frame_system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + CheckedSub
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Saturating
        + From<u128>
        + From<Self::BlockNumber>;

    type CreationFee;

    // From Currency
    // type Balance: SimpleArithmetic + Codec + Copy + MaybeSerializeDebug + Default;

    // type Currency: Currency<Self::AccountId>;

    // pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
    // pub type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
}

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

/// This trait is used to add a signer to a multisig
pub trait AddSignerMultiSig {
    /// Accept and add a multisig signer
    ///
    /// # Arguments
    /// * `signer` did/key of the signer
    /// * `auth_id` Authorization id of the authorization created by the multisig
    fn accept_multisig_signer(signer: Signer, auth_id: u64) -> DispatchResult;
}



pub mod group;
pub mod balances;
pub mod identity;
