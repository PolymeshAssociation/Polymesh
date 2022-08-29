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
//! The Confidential Asset module is one place to create the MERCAT security assets on the
//! Polymesh blockchain.
//!
//! ## Overview
//!
//! The following documentation covers the functionalities needed for a transfer of a confidential asset.
//! Part of this functionality (creating a confidential account and minting confidential assets) are
//! handled in this pallet, while the confidential transfer is handled by the
//! [settlement module](../pallet_settlement/index.html). These pallets call out to the
//! [MERCAT library]https://github.com/PolymathNetwork/cryptography) which is an implementation of the
//! [MERCAT whitepaper](https://info.polymath.network/hubfs/PDFs/Polymath-MERCAT-Whitepaper-Mediated-Encrypted-Reversible-SeCure-Asset-Transfers.pdf).
//!
//!
//! ### Terminology
//!
//! - The parties:
//!   - Sender: The party who is sending assets out of her account. We refer to her as Alice in the
//!     examples.
//!   - Receiver: The party who is receiving some assets to his account. We refer to him as Bob in
//!     the examples.
//!   - Mediator: Also known as the exchange, is the party that preforms compliance checks and
//!     approves/rejects the transaction. We refer to him as Charlie in the examples.
//!   - Issuer: The party that mints the assets into her account.
//!
//! - Phases of a successful transfer:
//!   - Mint/issue: deposit assets to an account.
//!   - Initialize a transfer: Alice indicates that she wants to transfer some assets to Bob.
//!   - Finalize a transfer: Bob acknowledges that he expects to receive assets from Alice.
//!   - Justify a transfer: Charlie approves the transfer.
//!   - Verification: the chain verifies all the data and updates the encrypted balance of Alice and Bob.
//!
//! - The workflow: the overall workflow is that at different phases of the transaction, each party
//!   generates a cryptographic proof in their wallet, and submits the proof to the chain for
//!   verification.
//!
//!   There are 8 different phases for a transaction
//!     1. Create a confidential asset using `create_confidential_asset()` dispatchable. Note that
//!        unlike `create_asset()`, the minting is performed separately (in step 3).
//!     2. Different parties can create confidential accounts to manage the confidential asset in
//!        their wallets and submit the proof of correctness to the chain.
//!        - The chain verifies the proofs and stores the confidential accounts on the chain.
//!        - NB - The parties can create their accounts in any order, but each can only create
//!               their account for the MERCAT asset AFTER the MERCAT asset is created in step 1.
//!     3. The issuer issues assets of a confidential asset using `mint_confidential_asset()`
//!        dispatchable and submits the proof of correctness to the chain.
//!        - The chain verifies the proofs and updates the encrypted balance on the chain.
//!        - NB - The issuer can mint an asset only after she has created an account for that asset
//!               in step 2.
//!     4. The mediator creates a venue and an instruction with a `settlement::ConfidentialLeg`.
//!     5. The sender initiates a transfer in her wallet and submits the proof of correctness to the
//!        chain using `settlement::authorize_confidential_instruction()` dispatchable.
//!     6. The receiver finalizes the transfer in her wallet and submits the proof of correctness to
//!        the chain using `settlement::authorize_confidential_instruction()` dispatchable.
//!     7. The mediator justifies the transfer in her wallet and submits the proof of correctness to
//!        the chain using `settlement::authorize_confidential_instruction()` dispatchable.
//!     8. Once all proofs of steps 4-7 are gathered, the chain verifies them and updates the
//!        encrypted balance of the Sender and the Receiver.
//!
//!     NB - The steps 4-7 must be performed sequentially by each party since they all need information
//!          from the chain that is only available after the previous party authorizes the
//!          instruction.
//!     
//!
//! ### Goals
//!
//! The main goal is to enable the confidential transfer of assets such that the amount and the
//! asset type of the transfer remain hidden from anyone who has access to the chain. But at the
//! same time, enable certain stakeholders (issuers, mediators, and auditors) to view both the asset
//! and transfer amount for reporting, compliance checking, and auditing purposes.
//!
//!
//! ## Limitations
//!
//! - In the current implementation, you can have only one confidential leg per instruction. This restriction
//!   might be lifted in future versions: CRYP-1333.
//!
//!
//! ## Implementation Details
//!
//! - The proofs should be base64-encoded before getting passed to the chain.
//!
//! ## Related Modules
//!
//! - [settlement module](../pallet_settlement/index.html): Handles both plain and confidential transfers.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use frame_system::ensure_signed;
use mercat::{
    account::{convert_asset_ids, AccountValidator},
    asset::AssetValidator,
    cryptography_core::AssetId,
    transaction::TransactionValidator,
    AccountCreatorVerifier, AssetTransactionVerifier, EncryptedAmount, EncryptedAssetId,
    EncryptionPubKey, InitializedAssetTx, JustifiedTransferTx, PubAccount, PubAccountTx,
    TransferTransactionVerifier,
};
use pallet_identity as identity;
use pallet_statistics::{self as statistics};
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, constants::currency::ONE_UNIT, identity::Trait as IdentityTrait,
    CommonTrait, Context,
};
use polymesh_primitives::{
    rng, AssetIdentifier, AssetName, AssetType, Base64Vec, FundingRoundName, IdentityId, Ticker,
};
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

        /// Contains the encryption key for a mercat mediator.
        pub MediatorMercatAccounts get(fn mediator_mercat_accounts):
            map hasher(twox_64_concat) IdentityId => EncryptionPubKeyWrapper;

        /// Contains the mercat accounts for an identity.
        /// (did, account_id) -> MercatAccount.
        pub MercatAccounts get(fn mercat_accounts):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
            => MercatAccount;

        /// Contains the encrypted balance of a mercat account.
        /// (did, account_id) -> EncryptedBalanceWrapper.
        pub MercatAccountBalance get(fn mercat_account_balance):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
            => EncryptedBalanceWrapper;

        /// Accumulates the encrypted pending balance for a mercat account.
        /// (did, account_id) -> EncryptedBalanceWrapper.
        PendingOutgoingBalance get(fn pending_outgoing_balance):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
            => EncryptedBalanceWrapper;

        /// Accumulates the encrypted incoming balance for a mercat account.
        /// (did, account_id) -> EncryptedBalanceWrapper.
        IncomingBalance get(fn incoming_balance):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
            => EncryptedBalanceWrapper;

        /// Accumulates the encrypted failed balance for a mercat account.
        /// (did, account_id) -> EncryptedBalanceWrapper.
        FailedOutgoingBalance get(fn failed_outgoing_balance):
            double_map hasher(twox_64_concat) IdentityId,
            hasher(blake2_128_concat) MercatAccountId
            => EncryptedBalanceWrapper;

        /// Stores the pending state for a given instruction.
        /// ((did, account_id, instruction_id)) -> EncryptedBalanceWrapper.
        pub TxPendingState get(fn mercat_tx_pending_state):
        map hasher(blake2_128_concat) (IdentityId, MercatAccountId, u64)
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
            AccountValidator.verify(&tx, &valid_asset_ids).map_err(|_| Error::<T>::InvalidAccountCreationProof)?;
            let wrapped_enc_asset_id = EncryptedAssetIdWrapper::from(tx.pub_account.enc_asset_id);
            let wrapped_enc_pub_key = EncryptionPubKeyWrapper::from(tx.pub_account.owner_enc_pub_key);
            let account_id = MercatAccountId(wrapped_enc_asset_id.0.clone());
            <MercatAccounts>::insert(&owner_id, &account_id, MercatAccount {
                encrypted_asset_id: wrapped_enc_asset_id,
                encryption_pub_key: wrapped_enc_pub_key,
            });
            let wrapped_enc_balance = EncryptedBalanceWrapper::from(tx.initial_balance);
            MercatAccountBalance::insert(&owner_id, &account_id, wrapped_enc_balance.clone());

            Self::deposit_event(RawEvent::AccountCreated(owner_id, account_id, wrapped_enc_balance));
            Ok(())
        }

        /// Stores mediator's public key.
        ///
        /// # Arguments
        /// * `public_key` the encryption public key, used during mercat operations.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        #[weight = 1_000_000_000]
        pub fn add_mediator_mercat_account(origin,
            public_key: EncryptionPubKeyWrapper,
        ) -> DispatchResult {
            let owner_acc = ensure_signed(origin)?;
            let owner_id = Context::current_identity_or::<Identity<T>>(&owner_acc)?;

            MediatorMercatAccounts::insert(&owner_id, &public_key);

            Self::deposit_event(RawEvent::MediatorAccountCreated(owner_id, public_key));
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

            if !T::NonConfidentialAsset::is_divisible(ticker) {
                ensure!(
                    // Non-divisible asset amounts must maintain a 6 decimal places of precision.
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
            let new_encrypted_balance = AssetValidator
                                        .verify_asset_transaction(
                                            total_supply.saturated_into::<u32>(),
                                            &asset_mint_proof,
                                            &Self::mercat_accounts(owner_did, &account_id).to_mercat::<T>()?,
                                            &Self::mercat_account_balance(owner_did, &account_id).to_mercat::<T>()?,
                                            &[]
                                        ).map_err(|_| Error::<T>::InvalidAccountMintProof)?;

            // Set the total supply (both encrypted and plain).
            MercatAccountBalance::insert(
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

        /// Resets the `FailedOutgoingBalance` and `IncomingBalance` accumulators for the caller's account.
        /// If successful, the account owner must use their current balance minus the sum of all unsettled outgoing
        /// balances as their pending balance.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `account_id` - the mercat account ID of the `origin`.
        ///
        /// # Errors
        /// - `BadOrigin` if not signed.
        ///
        /// # Weight
        /// `3_000_000_000`
        #[weight = 3_000_000_000]
        pub fn reset_ordering_state(
            origin,
            account_id: MercatAccountId,
        ) {
            let owner = ensure_signed(origin)?;
            let owner_did = Context::current_identity_or::<Identity<T>>(&owner)?;

            Self::reset_mercat_pending_state(&owner_did, &account_id);

            // Emit an event and include account's current balance for account owner's information.
            let current_balance =
                Self::mercat_account_balance(owner_did, &account_id);
            Self::deposit_event(RawEvent::ResetConfidentialAccountOrderingState(
                owner_did,
                account_id,
                current_balance
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    /// Reset `IncomingBalance` and `FailedOutgoingBalance` accumulators, by doing this
    /// we are no longer keeping track of the old/settled transactions.
    fn reset_mercat_pending_state(owner_did: &IdentityId, account_id: &MercatAccountId) {
        if IncomingBalance::contains_key(owner_did, account_id) {
            IncomingBalance::remove(owner_did, account_id);
        }

        if FailedOutgoingBalance::contains_key(owner_did, account_id) {
            FailedOutgoingBalance::remove(owner_did, account_id);
        }
    }

    /// Add the `amount` to the mercat account balance, and update the `IncomingBalance` accumulator.
    pub fn mercat_account_deposit_amount(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        let current_balance =
            Self::mercat_account_balance(owner_did, account_id).to_mercat::<T>()?;
        MercatAccountBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(current_balance + amount),
        );

        // Update the incoming balance accumulator.
        Self::add_incoming_balance(owner_did, account_id, amount)
    }

    /// Subtract the `amount` from the mercat account balance, and update the `PendingOutgoingBalance` accumulator.
    pub fn mercat_account_withdraw_amount(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        let current_balance =
            Self::mercat_account_balance(owner_did, account_id).to_mercat::<T>()?;
        MercatAccountBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(current_balance - amount),
        );

        // Update the pending balance accumulator.
        let pending_balance =
            Self::pending_outgoing_balance(owner_did, account_id).to_mercat::<T>()?;

        PendingOutgoingBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(pending_balance - amount),
        );
        Ok(())
    }

    /// Calculate account owner's (transaction sender's) pending balance, given the previous
    /// pending outgoing, incoming, and failed outgoing transaction amounts.
    pub fn get_pending_balance(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
    ) -> Result<EncryptedAmount, DispatchError> {
        let mut current_balance =
            Self::mercat_account_balance(owner_did, account_id).to_mercat::<T>()?;

        if PendingOutgoingBalance::contains_key(owner_did, account_id) {
            current_balance -=
                Self::pending_outgoing_balance(owner_did, account_id).to_mercat::<T>()?;
        }

        if IncomingBalance::contains_key(owner_did, account_id) {
            current_balance -= Self::incoming_balance(owner_did, account_id).to_mercat::<T>()?;
        }

        if FailedOutgoingBalance::contains_key(owner_did, account_id) {
            current_balance -=
                Self::failed_outgoing_balance(owner_did, account_id).to_mercat::<T>()?;
        }
        Ok(current_balance)
    }

    /// Add the amount to the account's pending outgoing accumulator.
    pub fn add_pending_outgoing_balance(
        owner_did: &IdentityId,
        account_id: &EncryptedAssetId,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        let wrapped_enc_asset_id = EncryptedAssetIdWrapper::from(account_id.clone());
        let account_id = MercatAccountId(wrapped_enc_asset_id.0);

        let pending_balances =
            if PendingOutgoingBalance::contains_key(owner_did, account_id.clone()) {
                let pending_balance = Self::pending_outgoing_balance(owner_did, account_id.clone())
                    .to_mercat::<T>()?;
                pending_balance + amount
            } else {
                amount
            };

        PendingOutgoingBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(pending_balances),
        );

        Ok(())
    }

    /// Add the amount to the account's incoming balance accumulator.
    fn add_incoming_balance(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        let incoming_balances = if IncomingBalance::contains_key(owner_did, account_id) {
            let previous_balance =
                Self::incoming_balance(owner_did, account_id).to_mercat::<T>()?;
            previous_balance + amount
        } else {
            amount
        };

        IncomingBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(incoming_balances),
        );
        Ok(())
    }

    /// Subtracts the `amount` from the pending balance counter and adds it to the failed balance counter,
    /// i.e. transition the settlement amount from the pending outgoing accumulator to the failed outgoing
    /// accumulator.
    pub fn add_failed_outgoing_balance(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        let failed_balances = if FailedOutgoingBalance::contains_key(owner_did, account_id) {
            let failed_balance =
                Self::failed_outgoing_balance(owner_did, account_id).to_mercat::<T>()?;
            failed_balance + amount
        } else {
            amount
        };

        FailedOutgoingBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(failed_balances),
        );

        // We never reset or remove the pending outgoing accumulator.
        // The settlement state transition works in a way that this amount has definitely been added to the
        // pending state by this point.
        let pending_balance =
            Self::pending_outgoing_balance(owner_did, account_id).to_mercat::<T>()?;
        PendingOutgoingBalance::insert(
            owner_did,
            account_id,
            EncryptedBalanceWrapper::from(pending_balance - amount),
        );

        Ok(())
    }

    /// Store the current pending state for the given instruction_id. This is used at the time of transaction
    /// validation by the network validators, and can be removed, using the `remove_tx_pending_state()` API,
    /// as soon as the transaction is settled.
    pub fn set_tx_pending_state(
        owner_did: &IdentityId,
        account_id: &EncryptedAssetId,
        instruction_id: u64,
    ) -> DispatchResult {
        let wrapped_enc_asset_id = EncryptedAssetIdWrapper::from(account_id.clone());
        let account_id = MercatAccountId(wrapped_enc_asset_id.0);

        let pending_state = Self::get_pending_balance(owner_did, &account_id)?;
        TxPendingState::insert(
            (owner_did, account_id, instruction_id),
            EncryptedBalanceWrapper::from(pending_state),
        );
        Ok(())
    }

    /// Once the transaction is settled remove its pending state.
    pub fn remove_tx_pending_state(
        owner_did: &IdentityId,
        account_id: &MercatAccountId,
        instruction_id: u64,
    ) {
        TxPendingState::remove((owner_did, account_id, instruction_id));
    }

    /// Transfers an asset from one identity's portfolio to another.
    pub fn base_confidential_transfer(
        from_did: &IdentityId,
        from_account_id: &MercatAccountId,
        to_did: &IdentityId,
        to_account_id: &MercatAccountId,
        tx_data: &JustifiedTransferTx,
        instruction_id: u64,
    ) -> DispatchResult {
        // Read the mercat_accounts from the confidential-asset pallet.
        let from_mercat_pending_state =
            Self::mercat_tx_pending_state((from_did, from_account_id, instruction_id))
                .to_mercat::<T>()?;

        // Get receiver's account.
        let to_mercat = Self::mercat_accounts(to_did, to_account_id).to_mercat::<T>()?;

        // Get sender's account.
        let from_mercat = Self::mercat_accounts(from_did, from_account_id).to_mercat::<T>()?;

        // Verify the proofs.
        let mut rng = rng::Rng::default();
        let _ = TransactionValidator
            .verify_transaction(
                tx_data,
                &from_mercat,
                &from_mercat_pending_state,
                &to_mercat,
                &[],
                &mut rng,
            )
            .map_err(|_| {
                // Upon transaction validation failure, update the failed outgoing accumulator.
                let _ = Self::add_failed_outgoing_balance(
                    from_did,
                    from_account_id,
                    tx_data
                        .finalized_data
                        .init_data
                        .memo
                        .enc_amount_using_sender,
                );

                // There's no need to keep a copy of the pending state anymore.
                Self::remove_tx_pending_state(from_did, from_account_id, instruction_id);

                Error::<T>::ConfidentialTransferValidationFailure
            })?;

        // Note that storage failures could be a problem, if only some of the storage is updated.
        let _ = Self::mercat_account_withdraw_amount(
            from_did,
            from_account_id,
            tx_data
                .finalized_data
                .init_data
                .memo
                .enc_amount_using_sender,
        )?;

        let _ = Self::mercat_account_deposit_amount(
            to_did,
            to_account_id,
            tx_data
                .finalized_data
                .init_data
                .memo
                .enc_amount_using_receiver,
        )?;

        // There's no need to keep a copy of the pending state anymore.
        Self::remove_tx_pending_state(from_did, from_account_id, instruction_id);

        Ok(())
    }
}

decl_event! {
    pub enum Event<T> where Balance = <T as CommonTrait>::Balance,
    {
        /// Event for creation of a Mediator Mercat account.
        /// caller DID/ owner DID and encryption public key
        MediatorAccountCreated(IdentityId, EncryptionPubKeyWrapper),

        /// Event for creation of a Mercat account.
        /// caller DID/ owner DID, mercat account id (which is equal to encrypted asset ID), encrypted balance
        AccountCreated(IdentityId, MercatAccountId, EncryptedBalanceWrapper),

        /// Event for creation of a confidential asset.
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type, beneficiary DID
        ConfidentialAssetCreated(IdentityId, Ticker, Balance, bool, AssetType, IdentityId),

        /// Event for resetting the ordering state.
        /// caller DID/ owner DID, mercat account id, current encrypted account balance
        ResetConfidentialAccountOrderingState(IdentityId, MercatAccountId, EncryptedBalanceWrapper),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The MERCAT account creation proofs are invalid.
        InvalidAccountCreationProof,

        /// Unwrapping the wrapped data types into mercat data types has failed.
        UnwrapMercatDataError,

        /// The MERCAT asset issuance proofs are invalid.
        InvalidAccountMintProof,

        /// The provided total supply of a confidential asset is invalid.
        InvalidTotalSupply,

        /// The balance values does not fit `u32`.
        TotalSupplyAboveU32Limit,

        /// The user is not authorized.
        Unauthorized,

        /// The provided asset is not among the set of valid asset ids.
        UnknownConfidentialAsset,

        /// After registering the confidential asset, its total supply can change once from zero to a positive value.
        CanSetTotalSupplyOnlyOnce,

        /// A confidential asset's total supply must be positive.
        TotalSupplyMustBePositive,

        /// Insufficient mercat authorizations are provided.
        InsufficientMercatAuthorizations,

        /// Confidential transfer's proofs are invalid.
        ConfidentialTransferValidationFailure,
    }
}
