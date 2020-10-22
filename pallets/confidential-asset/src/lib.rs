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
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use cryptography::{
    mercat::{
        account::{convert_asset_ids, AccountValidator},
        asset::AssetValidator,
        AccountCreatorVerifier, AssetTransactionVerifier, EncryptedAmount, EncryptedAssetId,
        EncryptionPubKey, InitializedAssetTx, PubAccount, PubAccountTx,
    },
    AssetId,
};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::{DispatchResult, DispatchError}, ensure,
};
use frame_system::ensure_signed;
use pallet_identity as identity;
use pallet_statistics::{self as statistics};
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, constants::currency::ONE_UNIT, identity::Trait as IdentityTrait,
    CommonTrait, Context,
};
use polymesh_primitives::{
    AssetIdentifier, AssetName, AssetType, Base64Vec, FundingRoundName, IdentityId, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_runtime::{traits::Zero, SaturatedConversion};
use sp_std::{
    convert::{From, TryFrom},
    prelude::*,
};

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + IdentityTrait + statistics::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type NonConfidentialAsset: AssetTrait<Self::Balance, Self::AccountId>;
}

/// Wrapper for Elgamal Encryption keys that correspond to `EncryptionPubKey`.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct EncryptionPubKeyWrapper(pub Base64Vec);

impl From<EncryptionPubKey> for EncryptionPubKeyWrapper {
    fn from(key: EncryptionPubKey) -> Self {
        Self(Base64Vec::new(key.encode()))
    }
}

impl EncryptionPubKeyWrapper {
    /// Unwraps the value so that it can be passed to mercat library.
    pub fn to_mercat<T: Trait>(&self) -> Result<EncryptionPubKey, DispatchError> {
        let mut data: &[u8] = &self.0.decode()?;
        EncryptionPubKey::decode(&mut data).map_err(|_| Error::<T>::UnwrapMercatDataError.into())
    }
}

/// Wrapper for Ciphertexts that correspond to `EncryptedAssetId`.
/// This is needed since `mercat::asset_proofs::elgamal_encryption::CipherText` implements
/// Encode and Decode, instead of deriving them. As a result, the `EncodeLike` operator is
/// not automatically implemented.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct EncryptedAssetIdWrapper(pub Base64Vec);

impl From<EncryptedAssetId> for EncryptedAssetIdWrapper {
    fn from(asset_id: EncryptedAssetId) -> Self {
        Self(Base64Vec::new(asset_id.encode()))
    }
}

impl EncryptedAssetIdWrapper {
    /// Unwraps the value so that it can be passed to mercat library.
    pub fn to_mercat<T: Trait>(&self) -> Result<EncryptedAssetId, DispatchError> {
        let mut data: &[u8] = &self.0.decode()?;
        EncryptedAssetId::decode(&mut data).map_err(|_| Error::<T>::UnwrapMercatDataError.into())
    }
}

/// Created for better code readability. Its content are the same as the `pallet_confidential_asset::EncryptedAssetIdWrapper`.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct MercatAccountId(pub Base64Vec);

/// Wrapper for Ciphertexts that correspond to EncryptedBalance.
/// This is needed since `mercat::asset_proofs::elgamal_encryption::CipherText` implements
/// Encode and Decode, instead of deriving them. As a result, the EncodeLike operator is
/// not automatically implemented.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct EncryptedBalanceWrapper(pub Base64Vec);

impl From<EncryptedAmount> for EncryptedBalanceWrapper {
    fn from(amount: EncryptedAmount) -> Self {
        Self(Base64Vec::from(Base64Vec::new(amount.encode())))
    }
}

impl EncryptedBalanceWrapper {
    /// Unwraps the value so that it can be passed to mercat library.
    pub fn to_mercat<T: Trait>(&self) -> Result<EncryptedAmount, DispatchError> {
        let mut data: &[u8] = &self.0.decode()?;
        EncryptedAmount::decode(&mut data).map_err(|_| Error::<T>::UnwrapMercatDataError.into())
    }
}

/// A mercat account consists of the public key that is used for encryption purposes and the
/// encrypted asset id. The encrypted asset id also acts as the unique identifier of this
/// struct.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct MercatAccount {
    pub encrypted_asset_id: EncryptedAssetIdWrapper,
    pub encryption_pub_key: EncryptionPubKeyWrapper,
}

impl MercatAccount {
    /// Unwraps the value so that it can be passed to mercat library.
    pub fn to_mercat<T: Trait>(&self) -> Result<PubAccount, DispatchError> {
        Ok(PubAccount {
            enc_asset_id: self.encrypted_asset_id.to_mercat::<T>()?,
            owner_enc_pub_key: self.encryption_pub_key.to_mercat::<T>()?,
        })
    }
}

/// Wrapper for the mercat account proof that correspond to base64 encoding of `PubAccountTx`.
/// Since this is received as input from user and is a binary data, the `Vec<u8>` will be a base64 encoded.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct PubAccountTxWrapper(pub Base64Vec);

impl From<PubAccountTx> for PubAccountTxWrapper {
    fn from(tx: PubAccountTx) -> Self {
        Self(Base64Vec::new(tx.encode()))
    }
}

impl PubAccountTxWrapper {
    /// Unwraps the value so that it can be passed to mercat library.
    pub fn to_mercat<T: Trait>(&self) -> Result<PubAccountTx, DispatchError> {
        let mut data: &[u8] = &self.0.decode()?;
        PubAccountTx::decode(&mut data).map_err(|_| Error::<T>::UnwrapMercatDataError.into())
    }
}

/// Wrapper for the asset issuance proof that correspond to `InitializedAssetTx`.
/// Since this is received as input from user and is a binary data, the `Vec<u8>` will be a base64 encoded.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct InitializedAssetTxWrapper(pub Base64Vec);

impl From<InitializedAssetTx> for InitializedAssetTxWrapper {
    fn from(tx: InitializedAssetTx) -> Self {
        Self(Base64Vec::new(tx.encode()))
    }
}

impl InitializedAssetTxWrapper {
    /// Unwraps the value so that it can be passed to mercat library.
    pub fn to_mercat<T: Trait>(&self) -> Result<InitializedAssetTx, DispatchError> {
        let mut data: &[u8] = &self.0.decode()?;
        InitializedAssetTx::decode(&mut data).map_err(|_| Error::<T>::UnwrapMercatDataError.into())
    }
}

type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait> as ConfidentialAsset {

        /// Contains the mercat accounts for an identity.
        pub MercatAccounts get(fn mercat_accounts):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
            => MercatAccount;

        /// Contains the encrypted balance of a mercat account.
        pub MercatAccountBalance get(fn mercat_account_balance):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
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
            tx: PubAccountTxWrapper,
        ) -> DispatchResult {
            let owner_acc = ensure_signed(origin)?;
            let owner_id = Context::current_identity_or::<Identity<T>>(&owner_acc)?;
            let tx = tx.to_mercat::<T>()?;

            let valid_asset_ids = convert_asset_ids(Self::confidential_tickers());
            AccountValidator{}.verify(&tx, &valid_asset_ids).map_err(|_| Error::<T>::InvalidAccountCreationProof)?;
            let wrapped_enc_asset_id = EncryptedAssetIdWrapper::from(tx.pub_account.enc_asset_id);
            let wrapped_enc_pub_key = EncryptionPubKeyWrapper::from(tx.pub_account.owner_enc_pub_key);
            let account_id = MercatAccountId(wrapped_enc_asset_id.0.clone());
            <MercatAccounts>::insert(&owner_id, &account_id, MercatAccount {
                encrypted_asset_id: wrapped_enc_asset_id,
                encryption_pub_key: wrapped_enc_pub_key,
            });
            let wrapped_enc_balance = EncryptedBalanceWrapper::from(tx.initial_balance);
            <MercatAccountBalance>::insert(&owner_id, &account_id, wrapped_enc_balance.clone());

            Self::deposit_event(RawEvent::AccountCreated(owner_id, account_id, wrapped_enc_balance));
            Ok(())
        }

        /// Initializes a new confidential security token.
        /// Makes the initiating account the owner of the security token
        /// & the balance of the owner is set to total zero. To set to total supply, `mint_confidential_asset` should
        /// be called after a successful call of this function.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `name` - the name of the token.
        /// * `ticker` - the ticker symbol of the token.
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
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<AssetIdentifier>,
            funding_round: Option<FundingRoundName>,
        ) -> DispatchResult {
            let primary_owner = ensure_signed(origin)?;
            let primary_owner_did = Context::current_identity_or::<Identity<T>>(&primary_owner)?;

            T::NonConfidentialAsset::base_create_asset(primary_owner_did, name, ticker, Zero::zero(), divisible, asset_type.clone(), identifiers, funding_round, true)?;

            // Append the ticker to the list of confidential tickers.
            <ConfidentialTickers>::append(AssetId { id: ticker.as_bytes().clone() });

            Self::deposit_event(RawEvent::ConfidentialAssetCreated(
                primary_owner_did,
                ticker,
                Zero::zero(),
                divisible,
                asset_type,
                primary_owner_did,
            ));
            Ok(())
        }

        /// Verifies the proof of the asset minting, `asset_mint_proof`. If successful, it sets the total
        /// balance of the owner to `total_supply`. This function should only be called once with a non-zero total supply,
        /// after `create_confidential_asset` is called.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` - the ticker symbol of the token.
        /// * `total_supply` - the total supply of the token.
        /// * `asset_mint_proof` - The proofs that the encrypted asset id is a valid ticker name and that the `total_supply` matches encrypted value.
        ///
        /// # Errors
        /// - `BadOrigin` if not signed.
        /// - `Unauthorized` if origin is not the owner of the asset.
        /// - `CanSetTotalSupplyOnlyOnce` if this function is called more than once.
        /// - `TotalSupplyMustBePositive` if total supply is zero.
        /// - `InvalidTotalSupply` if `total_supply` is not a multiply of unit.
        /// - `TotalSupplyAboveU32Limit` if `total_supply` exceeds the u32 limit. This is imposed by the MERCAT lib.
        /// - `UnknownConfidentialAsset` The ticker is not part of the set of confidential assets.
        /// - `InvalidAccountMintProof` if the proofs of ticker name and total supply are incorrect.
        ///
        /// # Weight
        /// `3_000_000_000`
        #[weight = 3_000_000_000]
        pub fn mint_confidential_asset(
            origin,
            ticker: Ticker,
            total_supply: T::Balance,
            asset_mint_proof: InitializedAssetTxWrapper,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            let owner_did = Context::current_identity_or::<Identity<T>>(&owner)?;

            // Only the owner of the asset can change its total supply.
            ensure!(
                T::NonConfidentialAsset::is_owner(&ticker, owner_did),
                Error::<T>::Unauthorized
            );

            // Current total supply must be zero.
            ensure!(
                T::NonConfidentialAsset::token_details(&ticker).total_supply == Zero::zero(),
                Error::<T>::CanSetTotalSupplyOnlyOnce
            );

            // New total supply must be positive.
            ensure!(
                total_supply != Zero::zero(),
                Error::<T>::TotalSupplyMustBePositive
            );

            ensure!(
                Self::confidential_tickers().contains(&AssetId {
                    id: *ticker.as_bytes(),
                }),
                Error::<T>::UnknownConfidentialAsset
            );

            if T::NonConfidentialAsset::is_divisible(ticker) {
                ensure!(
                    total_supply % ONE_UNIT.into() == 0.into(),
                    Error::<T>::InvalidTotalSupply
                );
            }

            // At the moment, mercat lib imposes that balances can be at most u32 integers.
            let max_balance_mercat = u32::MAX.saturated_into::<T::Balance>();
            ensure!(
                total_supply <= max_balance_mercat,
                Error::<T>::TotalSupplyAboveU32Limit
            );

            let asset_mint_proof = asset_mint_proof.to_mercat::<T>()?;
            let account_id = MercatAccountId(Base64Vec::new(asset_mint_proof.account_id.encode()));
            let new_encrypted_balance = AssetValidator{}
                                        .verify_asset_transaction(
                                            total_supply.saturated_into::<u32>(),
                                            &asset_mint_proof,
                                            &Self::mercat_accounts(owner_did, account_id.clone()).to_mercat::<T>()?,
                                            &Self::mercat_account_balance(owner_did, account_id.clone()).to_mercat::<T>()?,
                                            &[]
                                        ).map_err(|_| Error::<T>::InvalidAccountMintProof)?;

            // Set the total supply (both encrypted and plain)
            <MercatAccountBalance>::insert(
                &owner_did,
                &account_id,
                EncryptedBalanceWrapper::from(new_encrypted_balance),
            );

            // This will emit the total supply changed event.
            T::NonConfidentialAsset::unchecked_set_total_supply(owner_did, ticker, total_supply)?;

            // Update statistic info.
            <statistics::Module<T>>::update_transfer_stats(
                &ticker,
                None,
                Some(total_supply),
                total_supply,
            );

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn update_mercat_account_balance(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
        new_encrypted_balance: EncryptedAmount,
    ) {
        <MercatAccountBalance>::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(new_encrypted_balance),
        );
    }
}

decl_event! {
    pub enum Event<T> where Balance = <T as CommonTrait>::Balance,
    {
        /// Event for creation of a Mercat account.
        /// caller DID/ owner DID, mercat account id (which is equal to encrypted asset ID), encrypted balance
        AccountCreated(IdentityId, MercatAccountId, EncryptedBalanceWrapper),

        /// Event for creation of a confidential asset.
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type, beneficiary DID
        ConfidentialAssetCreated(IdentityId, Ticker, Balance, bool, AssetType, IdentityId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Mercat library has rejected the account creation proofs.
        InvalidAccountCreationProof,

        /// Error during the converting of wrapped data types into mercat data types.
        UnwrapMercatDataError,

        /// Mercat library has rejected the asset issuance proofs.
        InvalidAccountMintProof,

        /// Thrown when the total supply of a confidential asset is not divisible.
        InvalidTotalSupply,

        /// The balance values does not fit `u32`.
        TotalSupplyAboveU32Limit,

        /// The user is not authorized.
        Unauthorized,

        /// The provided asset is not among the set of valid asset ids.
        UnknownConfidentialAsset,

        /// After registering the confidential asset, its total supply can change once from zero to a positive value.
        CanSetTotalSupplyOnlyOnce,

        /// After registering the confidential asset, its total supply can change once from zero to a positive value.
        TotalSupplyMustBePositive,
    }
}
