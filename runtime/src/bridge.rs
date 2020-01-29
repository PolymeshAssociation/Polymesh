//! Bridge from Ethereum to Polymesh

use crate::{identity, multisig};
use codec::{Decode, Encode};
use frame_system::ensure_signed;
use primitives::{AccountId, IdentityId, Key, Signer};
use sp_core::H256;

type EthTxHash = H256;

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the set of bridge validators.
        Validators get(validators): multisig::Multisig;
        /// Confirmations of locked ETH.
        Confirmations get(confirmations): map (Signer, LockTx) => bool;
    }
}

/// The intended recipient of POLY exchanged from the locked ETH.
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockRecipient {
    Account(AccountId),
    Identity(IdentityId),
}

/// Data of a lock transaction on Ethereum.
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockTx {
    pub recipient: LockRecipient,
    pub eth_tx_hash: EthTxHash,
}

decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// Confirmation of minting POLY on Polymesh in return for the locked ETH on Ethereum.
        Bridged(LockTx)
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        pub fn confirm_lock_event(origin, lock_tx: LockTx) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::from(Key::try_from(sender.encode())?);
            <Confirmations>::insert((sender_signer, lock_tx), true);
            Ok(())
        }
    }
}
