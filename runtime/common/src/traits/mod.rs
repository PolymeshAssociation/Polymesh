use codec::{Codec, Decode, Encode};
use runtime_primitives::traits::{CheckedSub, MaybeSerializeDebug, Member, SimpleArithmetic};
use srml_support::{
    traits::{LockIdentifier, WithdrawReasons},
    Parameter,
};

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}

pub trait CommonTrait: system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + CheckedSub
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + From<u128>
        + From<Self::BlockNumber>;

    type CreationFee;

    // From Currency
    // type Balance: SimpleArithmetic + Codec + Copy + MaybeSerializeDebug + Default;

    // type Currency: Currency<Self::AccountId>;

    // pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
    // pub type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
}

pub mod balances;
pub mod identity;
