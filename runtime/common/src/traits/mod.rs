use codec::{Codec, Decode, Encode};
use frame_support::{
    traits::{LockIdentifier, WithdrawReasons},
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

pub mod group;
pub mod balances;
pub mod identity;
