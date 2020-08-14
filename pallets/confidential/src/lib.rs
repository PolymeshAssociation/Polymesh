// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(box_syntax)]

use polymesh_common_utilities::{identity::Trait as IdentityTrait, Context};
use polymesh_primitives::{IdentityId, Ticker};
use polymesh_primitives_derive::{SliceU8StrongTyped, VecU8StrongTyped};

use pallet_identity as identity;

use bulletproofs::RangeProof;
use cryptography::asset_proofs::range_proof::{
    prove_within_range, verify_within_range, InRangeProof,
};
use cryptography::mercat::{InitializedAssetTx, JustifiedAssetTx};
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};

use codec::{Decode, Encode};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

pub mod rng;
pub use rng::native_rng;

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, SliceU8StrongTyped)]
pub struct RangeProofInitialMessageWrapper(pub [u8; 32]);

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, VecU8StrongTyped)]
pub struct RangeProofFinalResponseWrapper(pub Vec<u8>);

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TickerRangeProof {
    // pub proof: RangeProofWrapper,
    pub initial_message: RangeProofInitialMessageWrapper,
    // pub committed_value: CompressedRistrettoWrapper,
    pub final_response: RangeProofFinalResponseWrapper,
    pub max_two_exp: u32,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TransactionId(pub u32);

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
// Todo: try sp_debug_derive::RuntimeDebug instead to only add Debug when it's needed.
pub struct MercatAssetTransactionWrapper(pub InitializedAssetTx);
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct TickerAssetTransaction {
    pub mercat_transaction: MercatAssetTransactionWrapper,
}

// #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
// pub struct MercatJustifiedAssetTransactionWrapper(pub JustifiedAssetTx);
// #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
// pub struct TickerJustifiedAssetTransaction {
//     pub mercat_transaction: MercatJustifiedAssetTransactionWrapper,
// }

pub trait Trait: frame_system::Trait + IdentityTrait {
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

type Identity<T> = identity::Module<T>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct ProverTickerKey {
    prover: IdentityId,
    ticker: Ticker,
}

decl_storage! {
    trait Store for Module<T: Trait> as Confidential {
        /// Number of investor per asset.
        pub RangeProofs get(fn range_proof): double_map hasher(twox_64_concat) IdentityId, hasher(blake2_128_concat) ProverTickerKey => Option<TickerRangeProof>;

        pub RangeProofVerifications get(fn range_proof_verification): double_map hasher(blake2_128_concat) (IdentityId, Ticker), hasher(twox_64_concat) IdentityId => bool;

        /// Store an asset issuance transaction.
        pub AssetTransactions get(fn asset_transaction): double_map hasher(blake2_128_concat) (IdentityId, Ticker), hasher(twox_64_concat) TransactionId => Option<TickerAssetTransaction>;

        /// This is the reason I think having a different type for mediator is unnecessary. Nothing new is happening here.
        // pub JustifiedAssetTransaction get(fn justified_asset_transaction): double_map hasher(blake2_128_concat) (IdentityId, Ticker), hasher(twox_64_concat) TransactionId => Option<TickerJustifiedAssetTransaction>

        pub ValidatedAssetTransactions get(fn asset_transaction_validation): double_map hasher(blake2_128_concat) (IdentityId, Ticker), hasher(twox_64_concat) TransactionId => bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 8_000_000_000]
        pub fn add_range_proof(origin,
            target_id: IdentityId,
            ticker: Ticker,
            secret_value: u64,
        ) -> DispatchResult
        {
            let prover_acc = ensure_signed(origin)?;
            let prover = Context::current_identity_or::<Identity<T>>(&prover_acc)?;

            // Create proof
            let mut rng = rng::Rng::default();
            let rand_blind = Scalar::random(&mut rng);
            let in_range_proof = prove_within_range( secret_value, rand_blind, 32, &mut rng)
                .map_err(|e| {
                    debug::error!("Confidential error: {:?}", e);
                    Error::<T>::InvalidRangeProof
                })?;

            let ticker_range_proof = TickerRangeProof {
                initial_message: RangeProofInitialMessageWrapper::from(in_range_proof.init.as_bytes().as_ref()),
                final_response: in_range_proof.response.to_bytes().into(),
                max_two_exp: 32,
            };
            let prover_ticker_key = ProverTickerKey { prover, ticker };
            <RangeProofs>::insert(&target_id, &prover_ticker_key, ticker_range_proof);
            Ok(())
        }

        #[weight = 6_000_000_000]
        pub fn add_verify_range_proof(origin,
            target: IdentityId,
            prover: IdentityId,
            ticker: Ticker) -> DispatchResult
        {
            let verifier = ensure_signed(origin)?;
            let verifier_id = Context::current_identity_or::<Identity<T>>(&verifier)?;

            Self::verify_range_proof(target, prover, ticker)?;

            // This is never mapping to false.
            <RangeProofVerifications>::insert((target, ticker), verifier_id, true);
            Ok(())
        }

        // Todo change this to use an anonymized ticker id.
        // Todo take in the mediator's id, so Mesh knows who can mediate this.
        #[weight = 6_000_000_000]
        pub fn add_asset_issuance_transaction(origin, ticker: Ticker, tx_id: TransactionId, conf_tx_data: InitializedAssetTx) -> DispatchResult {
            // Todo How can we make sure the `conf_tx_data` is signed? Is this call taking care of that?
            // Todo We might need to check that the caller is in fact allowed to issue assets to itself.
            let owner = ensure_signed(origin)?;
            let owner_id = Context::current_identity_or::<Identity<T>>(&owner)?;

            // Blindly put the transaction on the chain.
            let temp = TickerAssetTransaction{ mercat_transaction: MercatAssetTransactionWrapper(conf_tx_data)};
            <AssetTransactions>::insert((owner_id, ticker), tx_id, temp);

            // Todo do we need to deposit an event here? who would catch it?
            // Self::deposit_event(RawEvent::AssetFrozen(sender_did, ticker));

            Ok(())
        }

        // Todo Add the mediator function here.

        // Validators need to have a way of retrieving issuer and mediator's encryption public keys given their IdentityId.
        #[weight = 6_000_000_000]
        pub fn validate_asset_issuance_transaction(origin, issuer: IdentityId, ticker: Ticker, tx_id: TransactionId, _mediator: IdentityId) -> DispatchResult {
            let _verifier = ensure_signed(origin)?;

            // Todo We might need to check that the caller is in fact allowed to issue assets to itself.
            // By this point validator is convinced that the mediator authorized the transaction.
            // Check the proofs on the transaction.
            Self::verify_asset_transaction(issuer, ticker, tx_id.clone())?;

            // For now, if the transaction fails to verify, this logic will throw an error rather than saving the failure to storage.
            <ValidatedAssetTransactions>::insert((issuer, ticker), tx_id, true);

            // Todo do we need to deposit an event here? who would catch it?

            Ok(())
        }
    }
}

decl_event! {
    // Who's calling these? Don't seem to be accessible on polkadot.js.
    pub enum Event
    {
        // Todo these RangeProof events have to go.
        RangeProofAdded(IdentityId, Ticker, TickerRangeProof),
        RangeProofVerified(IdentityId, IdentityId, Ticker),

        // Todo I don't know what I'm doing here...
        AssetTransactionAdded(IdentityId, Ticker, TransactionId, TickerAssetTransaction),
        AssetTransactionVerified(IdentityId, Ticker, TransactionId ),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        ///
        MissingRangeProof,
        ///
        InvalidRangeProof,
        /// Could not find the asset transaction.
        NotFoundAssetTransaction,
    }
}

impl<T: Trait> Module<T> {
    pub fn verify_range_proof(
        target: IdentityId,
        prover: IdentityId,
        ticker: Ticker,
    ) -> DispatchResult {
        let prover_ticker_key = ProverTickerKey { prover, ticker };
        let mut rng = rng::Rng::default();

        let trp = Self::range_proof(&target, &prover_ticker_key)
            .ok_or_else(|| Error::<T>::MissingRangeProof)?;

        let init = CompressedRistretto::from_slice(trp.initial_message.as_slice());
        let response = RangeProof::from_bytes(trp.final_response.as_slice())
            .map_err(|_| Error::<T>::InvalidRangeProof)?;
        let proof = InRangeProof {
            init,
            response,
            range: 32,
        };

        verify_within_range(&proof, &mut rng).map_err(|_| Error::<T>::InvalidRangeProof.into())
    }

    pub fn verify_asset_transaction(
        issuer: IdentityId,
        ticker: Ticker,
        tx_id: TransactionId,
    ) -> DispatchResult {
        let asset_tx = Self::asset_transaction((issuer, ticker), tx_id)
            .ok_or_else(|| Error::<T>::NotFoundAssetTransaction)?;

        // This is cheating!
        let _justified_asset_transaction = JustifiedAssetTx {
            init_data: asset_tx.mercat_transaction.0,
        };

        // We need to have a way of storing and retrieving PubAccount and balances before we do this.
        // let validator = AssetValidator {};
        // validator.verify_asset_transaction(_justified_asset_transaction, issuer_account, issuer_init_balance, mediator_pub_key, &[]);

        Ok(())
    }
}
