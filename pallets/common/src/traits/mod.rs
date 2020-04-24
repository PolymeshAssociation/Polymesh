use codec::{Codec, Decode, Encode};
use frame_support::{
    traits::{LockIdentifier, WithdrawReasons},
    Parameter,
};
use polymesh_primitives::traits::BlockRewardsReserveCurrency;
use sp_arithmetic::traits::{AtLeast32Bit, CheckedSub, Saturating};
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

pub trait CommonTrait: frame_system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + AtLeast32Bit
        + CheckedSub
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Saturating
        + Debug
        + From<u128>
        + From<Self::BlockNumber>;

    type AcceptTransferTarget: asset::AcceptTransfer;

    type BlockRewardsReserve: BlockRewardsReserveCurrency<Self::Balance, NegativeImbalance<Self>>;
}

pub mod imbalances;
pub use imbalances::{NegativeImbalance, PositiveImbalance};

pub mod asset;
pub mod balances;
pub mod exemption;
pub mod general_tm;
pub mod governance_group;
pub mod group;
pub mod identity;
pub mod multisig;
