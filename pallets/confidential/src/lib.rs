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

use codec::{Decode, Encode};
use confidential_identity_core::{
    asset_proofs::{
        bulletproofs::RangeProof,
        range_proof::{prove_within_range, verify_within_range, InRangeProof},
    },
    CompressedRistretto, Scalar,
};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    traits::Randomness, weights::Weight,
};
use pallet_identity as identity;
use polymesh_common_utilities::{asset::AssetFnTrait, identity::Config as IdentityConfig};
use polymesh_primitives::{IdentityId, Ticker};
use polymesh_primitives_derive::{SliceU8StrongTyped, VecU8StrongTyped};
use scale_info::TypeInfo;
use sp_std::prelude::*;

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[derive(
    Encode,
    Decode,
    TypeInfo,
    Clone,
    Default,
    PartialEq,
    Eq,
    SliceU8StrongTyped
)]
pub struct RangeProofInitialMessageWrapper(pub [u8; 32]);

#[derive(
    Encode,
    Decode,
    TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
    VecU8StrongTyped
)]
pub struct RangeProofFinalResponseWrapper(pub Vec<u8>);

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct TickerRangeProof {
    // pub proof: RangeProofWrapper,
    pub initial_message: RangeProofInitialMessageWrapper,
    // pub committed_value: CompressedRistrettoWrapper,
    pub final_response: RangeProofFinalResponseWrapper,
    pub max_two_exp: u32,
}

pub trait WeightInfo {
    fn add_range_proof() -> Weight;
    fn add_verify_range_proof() -> Weight;
}

pub trait Config: frame_system::Config + IdentityConfig {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    type Asset: AssetFnTrait<Self::AccountId, Self::Origin>;
    type WeightInfo: WeightInfo;

    /// Randomness source.
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
}

type Identity<T> = identity::Module<T>;

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct ProverTickerKey {
    prover: IdentityId,
    ticker: Ticker,
}

decl_storage! {
    trait Store for Module<T: Config> as Confidential {
        /// Number of investor per asset.
        pub RangeProofs get(fn range_proof): double_map hasher(identity) IdentityId, hasher(blake2_128_concat) ProverTickerKey => Option<TickerRangeProof>;

        pub RangeProofVerifications get(fn range_proof_verification): double_map hasher(blake2_128_concat) (IdentityId, Ticker), hasher(identity) IdentityId => bool;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = <T as Config>::WeightInfo::add_range_proof()]
        pub fn add_range_proof(origin, target_id: IdentityId, ticker: Ticker, secret_value: u64) {
            let prover = Identity::<T>::ensure_perms(origin)?;

            // Create proof
            let mut rng = Self::get_rng();
            let rand_blind = Scalar::random(&mut rng);
            let in_range_proof = prove_within_range( secret_value, rand_blind, 32, &mut rng)
                .map_err(|e| {
                    log::error!("Confidential error: {:?}", e);
                    Error::<T>::InvalidRangeProof
                })?;

            let ticker_range_proof = TickerRangeProof {
                initial_message: RangeProofInitialMessageWrapper::from(in_range_proof.init.as_bytes().as_ref()),
                final_response: in_range_proof.response.to_bytes().into(),
                max_two_exp: 32,
            };
            let prover_ticker_key = ProverTickerKey { prover, ticker };
            RangeProofs::insert(&target_id, &prover_ticker_key, ticker_range_proof);
        }

        #[weight = <T as Config>::WeightInfo::add_verify_range_proof()]
        pub fn add_verify_range_proof(origin, target: IdentityId, prover: IdentityId, ticker: Ticker) {
            let verifier_id = Identity::<T>::ensure_perms(origin)?;
            Self::verify_range_proof(target, prover, ticker)?;
            RangeProofVerifications::insert((target, ticker), verifier_id, true);
        }
    }
}

decl_event! {
    pub enum Event {
        RangeProofAdded(IdentityId, Ticker, TickerRangeProof),
        RangeProofVerified(IdentityId, IdentityId, Ticker),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        ///
        MissingRangeProof,
        ///
        InvalidRangeProof,
    }
}

impl<T: Config> Module<T> {
    pub fn verify_range_proof(
        target: IdentityId,
        prover: IdentityId,
        ticker: Ticker,
    ) -> DispatchResult {
        let prover_ticker_key = ProverTickerKey { prover, ticker };
        let mut rng = Self::get_rng();

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

    fn get_rng() -> Rng {
        // TODO:
        let (random_hash, _) = T::Randomness::random(b"TODO: add nonce.");
        let seed = <u64>::decode(&mut random_hash.as_ref())
            .expect("secure hashes should always be bigger than u64; qed");
        Rng::seed_from_u64(seed)
    }
}
