use codec::{Codec, Decode, Encode};
use frame_support::{
    traits::{Get, LockIdentifier, WithdrawReasons},
    Parameter,
};
use sp_arithmetic::traits::{CheckedSub, Saturating, SimpleArithmetic};
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};
use sp_std::fmt::Debug;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}

pub trait BlockRewardsReserveTrait<B> {
    fn drop_positive_imbalance(amount: B);
    fn drop_negative_imbalance(amount: B);
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
        + Debug
        + From<u128>
        + From<Self::BlockNumber>;

    /// The fee required to create an account.
    type CreationFee: Get<Self::Balance>;

    type AcceptTransferTarget: asset::AcceptTransfer;

    type BlockRewardsReserve: BlockRewardsReserveTrait<Self::Balance>;
}

pub mod imbalances;
pub use imbalances::{NegativeImbalance, PositiveImbalance};

pub mod asset;
pub mod balances;
pub mod group;
pub mod identity;
pub mod multisig;
