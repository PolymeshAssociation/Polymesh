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

//! # Confidential Asset Module
//!
//! The Confidential Asset module is one place to create the MERCAT security tokens on the
//! Polymesh blockchain.

use codec::{Decode, Encode};
use cryptography::{
    mercat::{
        account::{convert_asset_ids, AccountValidator},
        AccountCreatorVerifier, EncryptionPubKey, PubAccountTx,
    },
    AssetId,
};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::ensure_signed;
use pallet_identity as identity;
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, balances::Trait as BalancesTrait, identity::Trait as IdentityTrait,
    CommonTrait, Context,
};
use polymesh_primitives::{
    AssetIdentifier, AssetName, AssetType, FundingRoundName, IdentifierType, IdentityId, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_std::{
    convert::{From, TryFrom},
    prelude::*,
};

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + IdentityTrait + BalancesTrait {
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Wrapper for Ciphertexts that correspond to `EncryptedAssetId`.
/// This is needed since `mercat::asset_proofs::elgamal_encryption::CipherText` implements
/// Encode and Decode, instead of deriving them. As a result, the `EncodeLike` operator is
/// not automatically implemented.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, VecU8StrongTyped, Default)]
pub struct EncryptedAssetIdWrapper(pub Vec<u8>);

/// Wrapper for Ciphertexts that correspond to EncryptedBalance.
/// This is needed since `mercat::asset_proofs::elgamal_encryption::CipherText` implements
/// Encode and Decode, instead of deriving them. As a result, the EncodeLike operator is
/// not automatically implemented.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, VecU8StrongTyped, Default)]
pub struct EncryptedBalanceWrapper(pub Vec<u8>);

/// A mercat account consists of the public key that is used for encryption purposes and the
/// encrypted asset id. The encrypted asset id also acts as the unique identifier of this
/// struct.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct MercatAccount {
    pub encrypted_asset_id: EncryptedAssetIdWrapper,
    pub encryption_pub_key: EncryptionPubKey,
}

type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait> as ConfidentialAsset {

        /// Contains the mercat accounts for an identity.
        pub MercatAccounts get(fn mercat_accounts):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) EncryptedAssetIdWrapper
            => MercatAccount;

        /// Contains the encrypted balance of a mercat account.
        pub MercatAccountBalance get(fn mercat_account_balance):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) EncryptedAssetIdWrapper
            => EncryptedBalanceWrapper;

        /// List of Tickers of the type ConfidentialAsset.
        /// Returns a list of confidential tickers.
        pub ConfidentialTickers get(fn confidential_tickers): Vec<AssetId>;
    }
}

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// Initialize the default event for this module
        fn deposit_event() = default;

        /// Verifies the proofs given the `tx` (transaction) for creating a mercat account.
        /// If all proofs pass, it stores the mercat account for the origin identity, sets the
        /// initial encrypted balance, and emits account creation event. The proofs for the
        /// transaction require that the encrypted asset id inside the `tx`
        /// (`tx.pub_account.enc_asset_id`) is a member of the list of all the confidential asset ids.
        ///
        /// # Arguments
        /// * `tx` The account creation transaction created by the mercat lib.
        ///
        /// # Errors
        /// * `InvalidAccountCreationProof` if the provided proofs fail to verify.
        /// * `BadOrigin` if `origin` isn't signed.
        #[weight = 1_000_000_000]
        pub fn validate_mercat_account(origin,
            tx: PubAccountTx,
        ) -> DispatchResult {
            let owner_acc = ensure_signed(origin)?;
            let owner_id = Context::current_identity_or::<Identity<T>>(&owner_acc)?;

            let valid_asset_ids = convert_asset_ids(Self::confidential_tickers());
            AccountValidator{}.verify(&tx, &valid_asset_ids).map_err(|_| Error::<T>::InvalidAccountCreationProof)?;
            let wrapped_enc_asset_id = EncryptedAssetIdWrapper::from(tx.pub_account.enc_asset_id.encode());
            <MercatAccounts>::insert(&owner_id, &wrapped_enc_asset_id, MercatAccount {
                encrypted_asset_id: wrapped_enc_asset_id.clone(),
                encryption_pub_key: tx.pub_account.owner_enc_pub_key,
            });
            let wrapped_enc_balance = EncryptedBalanceWrapper::from(tx.initial_balance.encode());
            <MercatAccountBalance>::insert(&owner_id, &wrapped_enc_asset_id, wrapped_enc_balance.clone());

            Self::deposit_event(RawEvent::AccountCreated(owner_id, wrapped_enc_asset_id, wrapped_enc_balance));
            Ok(())
        }

        /// Initializes a new confidential security token.
        /// Makes the initiating account the owner of the security token
        /// & the balance of the owner is set to total supply.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `name` - the name of the token.
        /// * `ticker` - the ticker symbol of the token.
        /// * `total_supply` - the total supply of the token.
        /// * `divisible` - a boolean to identify the divisibility status of the token.
        /// * `asset_type` - the asset type.
        /// * `identifiers` - a vector of asset identifiers.
        /// * `funding_round` - name of the funding round.
        ///
        /// # Errors
        /// - `TickerAlreadyRegistered` if the ticker was already registered, e.g., by `origin`.
        /// - `TickerRegistrationExpired` if the ticker's registration has expired.
        /// - `InvalidTotalSupply` if not `divisible` but `total_supply` is not a multiply of unit.
        /// - `TotalSupplyAboveLimit` if `total_supply` exceeds the limit.
        /// - `BadOrigin` if not signed.
        ///
        /// # Weight
        /// `3_000_000_000 + 20_000 * identifiers.len()`
        #[weight = 3_000_000_000 + 20_000 * u64::try_from(identifiers.len()).unwrap_or_default()]
        pub fn create_confidential_asset(
            origin,
            name: AssetName,
            ticker: Ticker,
            total_supply: T::Balance,
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<(IdentifierType, AssetIdentifier)>,
            funding_round: Option<FundingRoundName>,
        ) -> DispatchResult {
            let primary_owner = ensure_signed(origin)?;
            let primary_owner_did = Context::current_identity_or::<Identity<T>>(&primary_owner)?;

            T::Asset::base_create_asset(primary_owner_did, name, ticker, total_supply, divisible, asset_type.clone(), identifiers, funding_round, true)?;

            // Append the ticker to the list of confidential tickers.
            <ConfidentialTickers>::append(AssetId { id: ticker.as_bytes().clone() });

            // TODO(CRYP-160) : mint `total_supply` assets to the primary asset issuer here.

            Self::deposit_event(RawEvent::ConfidentialAssetCreated(
                primary_owner_did,
                ticker,
                total_supply,
                divisible,
                asset_type,
                primary_owner_did,
            ));
            Ok(())
        }

    }
}

decl_event! {
    pub enum Event<T> where Balance = <T as CommonTrait>::Balance,
    {
        /// Event for creation of a Mercat account.
        /// caller DID/ owner DID, encrypted asset ID, encrypted balance
        AccountCreated(IdentityId, EncryptedAssetIdWrapper, EncryptedBalanceWrapper),

        /// Event for creation of a confidential asset.
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type, beneficiary DID
        ConfidentialAssetCreated(IdentityId, Ticker, Balance, bool, AssetType, IdentityId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Mercat library has rejected the account creation proofs.
        InvalidAccountCreationProof,
    }
}
