//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLY on Polymesh in return for permanently locked ERC20 POLY tokens.

use crate::{identity, multisig, runtime};
use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::{self as system, ensure_signed};
use primitives::{IdentityId, Key, Signer};
use sp_core::{H256, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: multisig::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Proposal: From<Call<Self>>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the set of bridge validators.
        Validators get(validators): T::AccountId;
        /// Confirmations of locked ERC20 tokens.
        Confirmations get(confirmations): map (Signer, BridgeTx<T::AccountId>) => bool;
        /// Correspondence between bridge transactions and multisig proposal IDs.
        BridgeTxProposals get(bridge_tx_proposals): map BridgeTx<T::AccountId> => u64;
    }
}

/// The intended recipient of POLY exchanged from the locked ERC20 tokens.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MintRecipient<AccountId> {
    Account(AccountId),
    Identity(IdentityId),
}

/// A unique lock-and-mint bridge transaction containing Ethereum transaction data and a bridge nonce.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BridgeTx<AccountId> {
    /// Bridge validator runtime nonce.
    pub nonce: U256,
    /// Recipient of POLY on Polymesh: the deposit address or identity.
    pub recipient: MintRecipient<AccountId>,
    /// Amount of tokens locked on Ethereum.
    pub value: U256,
    /// Ethereum token lock transaction hash.
    pub tx_hash: H256,
}

decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// Confirmation of minting POLY on Polymesh in return for the locked ERC20 tokens on
        /// Ethereum.
        Bridged(BridgeTx<AccountId>),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Proposes to change the address of the bridge validator multisig account.
        // pub fn change_validators(origin, account_id: T::AccountId) -> DispatchResult {
        //     let current_account_id = Self::validators();
        //     if current_account_id == Default::default() {
        //         <Validators<T>>::put(account_id);
        //     } else {
        //         let proposal = Box::new(Proposal::ChangeValidators(account_id));
        //         <multisig::Module<T>>::create_proposal_as_key(origin, current_account_id, proposal);
        //     }
        //     Ok(())
        // }

        /// Confirms a bridge transaction, which entails making a multisig proposal for the bridge
        /// transaction if the transaction is new or approving an existing proposal if the
        /// transaction has one.
        pub fn confirm_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let sender_signer = Signer::from(Key::try_from(sender.encode())?);
            let validators = Self::validators();
            ensure!(validators != Default::default(), "bridge validators not set");
            let proposal_id = Self::bridge_tx_proposals(bridge_tx.clone());
            if proposal_id == 0 {
                let proposal = <T as Trait>::Proposal::from(Call::<T>::insert_bridge_tx(bridge_tx));
//                let call = runtime::Call::Bridge(Call::insert_bridge_tx(bridge_tx));
//                <multisig::Module<T>>::create_proposal(validators, Box::new(call), sender_signer)?;
            } else {
                <multisig::Module<T>>::approve_as_key(origin, validators, proposal_id)?;
            }
            Ok(())
        }

        fn insert_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            <BridgeTxProposals<T>>::insert(bridge_tx, 42 /* FIXME */);
            Ok(())
        }
    }
}
