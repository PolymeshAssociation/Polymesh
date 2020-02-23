use crate::traits::{identity::IdentityTrait, CommonTrait, NegativeImbalance};

use codec::{Decode, Encode};
use frame_support::{
    decl_event,
    dispatch::DispatchError,
    traits::{ExistenceRequirement, Get, OnFreeBalanceZero, OnUnbalanced, WithdrawReasons},
};
use frame_system::{self as system, OnNewAccount};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Memo(pub [u8; 32]);

impl Default for Memo {
    fn default() -> Self {
        Memo([0u8; 32])
    }
}

decl_event!(
    pub enum Event<T> where
    <T as system::Trait>::AccountId,
    <T as CommonTrait>::Balance
    {
        /// A new account was created.
        NewAccount(AccountId, Balance),
        /// An account was reaped.
        ReapedAccount(AccountId),
        /// Transfer succeeded (from, to, value, fees).
        Transfer(AccountId, AccountId, Balance, Balance),
        /// Transfer succeded with a memo.
        TransferWithMemo(AccountId, AccountId, Balance, Balance, Memo),
    }
);

pub trait Trait: CommonTrait {
    /// has been reduced to zero.
    ///
    /// Gives a chance to clean up resources associated with the given account.
    type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

    /// Handler for when a new account is created.
    type OnNewAccount: OnNewAccount<Self::AccountId>;

    /// Handler for the unbalanced reduction when taking fees associated with balance
    /// transfer (which may also include account creation).
    type TransferPayment: OnUnbalanced<NegativeImbalance<Self>>;

    /// Handler for the unbalanced reduction when removing a dust account.
    type DustRemoval: OnUnbalanced<NegativeImbalance<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// The minimum amount required to keep an account open.
    type ExistentialDeposit: Get<<Self as CommonTrait>::Balance>;

    /// The fee required to make a transfer.
    type TransferFee: Get<<Self as CommonTrait>::Balance>;

    /// Used to charge fee to identity rather than user directly
    type Identity: IdentityTrait;
}

pub trait BalancesTrait<A, B, NI> {
    fn withdraw(
        who: &A,
        value: B,
        reasons: WithdrawReasons,
        _liveness: ExistenceRequirement,
    ) -> sp_std::result::Result<NI, DispatchError>;
}
