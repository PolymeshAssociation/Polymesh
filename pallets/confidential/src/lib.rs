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

use polymesh_common_utilities::{identity::Trait as IdentityTrait, Context};
use polymesh_primitives::{AccountKey, IdentityId, Ticker};
use polymesh_primitives_derive::{SliceU8StrongTyped, VecU8StrongTyped};

use cryptography::asset_proofs::range_proof::{prove_within_range, verify_within_range};
use pallet_identity as identity;

use bulletproofs::RangeProof;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use rand;

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, SliceU8StrongTyped)]
pub struct CompressedRistrettoWrapper(pub [u8; 32]);

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, VecU8StrongTyped)]
pub struct RangeProofWrapper(pub Vec<u8>);

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TickerRangeProof {
    pub proof: RangeProofWrapper,
    pub committed_value: CompressedRistrettoWrapper,
    pub max_two_exp: u8,
}

pub trait Trait: frame_system::Trait + IdentityTrait {
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

type Identity<T> = identity::Module<T>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct ProberTickerKey {
    prover: IdentityId,
    ticker: Ticker,
}

decl_storage! {
    trait Store for Module<T: Trait> as Confidential {
        /// Number of investor per asset.
        pub RangeProofs get(fn range_proof): double_map hasher(twox_64_concat) IdentityId, hasher(blake2_128_concat) ProberTickerKey => Option<TickerRangeProof>;

        pub RangeProofVerifications get(fn range_proof_verification): double_map hasher(blake2_128_concat) (IdentityId, Ticker), hasher(twox_64_concat) IdentityId => bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn add_range_proof(origin,
            target_id: IdentityId,
            ticker: Ticker,
            secret_value: u64,
            max_two_exp: u8
        ) -> DispatchResult
        {
            let prover_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let prover = Context::current_identity_or::<Identity<T>>(&prover_key)?;

            // Create proof
            let mut rng = rand::thread_rng();
            let rand_blind = Scalar::random(&mut rng);
            let (proof, committed_value) = prove_within_range(secret_value, rand_blind, max_two_exp as usize)
                .map_err(|e| {
                    debug::error!("Confidential error: {:?}", e);
                    Error::<T>::InvalidRangeProof
                })?;

            let ticker_range_proof = TickerRangeProof {
                proof: proof.to_bytes().into(),
                committed_value: committed_value.to_bytes().into(),
                max_two_exp
            };
            let prover_ticker_key = ProberTickerKey {
                prover,
                ticker
            };
            <RangeProofs>::insert(&target_id, &prover_ticker_key, ticker_range_proof);
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn add_verify_range_proof(origin,
            target: IdentityId,
            prover: IdentityId,
            ticker: Ticker) -> DispatchResult
        {
            let verifier_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let verifier = Context::current_identity_or::<Identity<T>>(&verifier_key)?;

            Self::verify_range_proof(target, prover, ticker.clone())?;

            <RangeProofVerifications>::insert((target,ticker), verifier, true);
            Ok(())
        }
    }
}

decl_event! {
    pub enum Event
    {
        RangeProofAdded(IdentityId, Ticker, TickerRangeProof),
        RangeProofVerified(IdentityId, IdentityId, Ticker),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        ///
        MissingRangeProof,
        ///
        InvalidRangeProof,
    }
}

impl<T: Trait> Module<T> {
    pub fn verify_range_proof(
        target: IdentityId,
        prover: IdentityId,
        ticker: Ticker,
    ) -> DispatchResult {
        let prover_ticker_key = ProberTickerKey { prover, ticker };

        let trp = Self::range_proof(&target, &prover_ticker_key)
            .ok_or_else(|| Error::<T>::MissingRangeProof)?;

        let proof = RangeProof::from_bytes(trp.proof.as_slice())
            .map_err(|_| Error::<T>::InvalidRangeProof)?;

        let committed_value = CompressedRistretto::from_slice(trp.committed_value.as_slice());

        // Verify.
        ensure!(
            verify_within_range(proof, committed_value, trp.max_two_exp as usize),
            Error::<T>::InvalidRangeProof
        );

        Ok(())
    }

    /*
    pub fn x() {
        let mut rng = StdRng::from_seed(SEED_1);
        // Positive test: secret value within range [0, 2^32)
        let secret_value = 42u32;
        let rand_blind = Scalar::random(&mut rng);

        let (proof, initial_message) = prove_within_range(secret_value as u64, rand_blind, 32)
            .expect("This shouldn't happen.");
        assert!(verify_within_range(proof, initial_message, 32));

        // Make sure the second part of the elgamal encryption is the same as the commited value in the range proof.
        let w = CommitmentWitness::try_from((secret_value, rand_blind)).unwrap();
        let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
        let elg_pub = elg_secret.get_public_key();
        let cipher = elg_pub.encrypt(&w);
        assert_eq!(initial_message, cipher.y.compress());

        // Negative test: secret value outside the allowed range
        let large_secret_value: u64 = u64::from(u32::max_value()) + 3;
        let (bad_proof, bad_commitment) =
            prove_within_range(large_secret_value, rand_blind, 32).expect("This shouldn't happen.");
        assert!(!verify_within_range(bad_proof, bad_commitment, 32));
    }
    */
}
