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
//! - The proofs should be SCALE-encoded before getting passed to the chain.
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
    traits::Randomness,
    weights::Weight,
};
use mercat::{
    account::AccountValidator, asset::AssetValidator,
    confidential_identity_core::asset_proofs::Balance as MercatBalance,
    transaction::verify_initialized_transaction, AccountCreatorVerifier, AssetTransactionVerifier,
    CompressedEncryptionPubKey, EncryptedAmount, InitializedAssetTx, InitializedTransferTx,
    PubAccount, PubAccountTx,
};
use pallet_base::try_next_post;
use pallet_identity as identity;
use polymesh_common_utilities::{
    balances::Config as BalancesConfig, identity::Config as IdentityConfig,
};
use polymesh_primitives::{
    asset::{AssetName, AssetType},
    impl_checked_inc, Balance, IdentityId, Ticker,
};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, SaturatedConversion};
use sp_std::{convert::From, prelude::*};

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

type Identity<T> = identity::Module<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub trait WeightInfo {
    fn validate_mercat_account() -> Weight;
    fn add_mediator_mercat_account() -> Weight;
    fn create_confidential_asset() -> Weight;
    fn mint_confidential_asset() -> Weight;
    fn add_transaction() -> Weight;
    fn sender_affirm_transaction() -> Weight;
    fn receiver_affirm_transaction() -> Weight;
    fn mediator_affirm_transaction() -> Weight;
    fn execute_transaction() -> Weight;
    fn apply_incoming_balance() -> Weight;

    fn affirm_transaction(affirm: &AffirmLeg) -> Weight {
        match affirm.parity {
            ParityAffirmLeg::SenderProof(_) => Self::sender_affirm_transaction(),
            ParityAffirmLeg::ReceiverAffirm => Self::receiver_affirm_transaction(),
            ParityAffirmLeg::MediatorAffirm => Self::mediator_affirm_transaction(),
        }
    }
}

/// Mercat types are uploaded as bytes (hex).
/// This make it easy to copy paste the proofs from CLI tools.
macro_rules! impl_wrapper {
    ($wrapper:ident, $wrapped:ident) => {
        #[derive(Clone, Debug)]
        pub struct $wrapper($wrapped);

        impl scale_info::TypeInfo for $wrapper {
            type Identity = Self;
            fn type_info() -> scale_info::Type {
                scale_info::Type::builder()
                    .docs_always(&[concat!("A wrapper for ", stringify!($wrapped))])
                    .path(scale_info::Path::new(stringify!($wrapper), module_path!()))
                    .composite(scale_info::build::Fields::unnamed().field(|f| {
                        f.ty::<Vec<u8>>()
                            .docs_always(&[concat!("SCALE encoded ", stringify!($wrapped))])
                            .type_name("Vec<u8>")
                    }))
            }
        }

        impl codec::EncodeLike for $wrapper {}

        impl Encode for $wrapper {
            #[inline]
            fn size_hint(&self) -> usize {
                // Get the wrapped value's size and add 2 bytes (estimate of the Vec<u8> length encoding).
                self.0.size_hint() + 2
            }

            fn encode_to<W: codec::Output + ?Sized>(&self, dest: &mut W) {
                // Encode wrapped value as a `Vec<u8>`.
                let encoded = self.0.encode();
                // Encode the `Vec<u8>`.
                encoded.encode_to(dest);
            }
        }

        impl Decode for $wrapper {
            fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
                let encoded = <Vec<u8>>::decode(input)?;
                let wrapped = <$wrapped>::decode(&mut encoded.as_slice())?;

                Ok(Self(wrapped))
            }
        }

        impl PartialEq for $wrapper {
            fn eq(&self, other: &Self) -> bool {
                self.encode() == other.encode()
            }
        }

        impl Eq for $wrapper {}

        impl From<$wrapper> for $wrapped {
            fn from(data: $wrapper) -> Self {
                data.0
            }
        }

        impl core::ops::Deref for $wrapper {
            type Target = $wrapped;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$wrapped> for $wrapper {
            fn from(data: $wrapped) -> Self {
                Self(data)
            }
        }
    };
}

impl_wrapper!(EncryptedAmountWrapper, EncryptedAmount);

impl_wrapper!(PubAccountTxWrapper, PubAccountTx);
impl_wrapper!(InitializedAssetTxWrapper, InitializedAssetTx);

impl_wrapper!(InitializedTransferTxWrapper, InitializedTransferTx);

impl core::ops::AddAssign<EncryptedAmount> for EncryptedAmountWrapper {
    fn add_assign(&mut self, other: EncryptedAmount) {
        self.0 += other;
    }
}

impl core::ops::AddAssign for EncryptedAmountWrapper {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl core::ops::SubAssign<EncryptedAmount> for EncryptedAmountWrapper {
    fn sub_assign(&mut self, other: EncryptedAmount) {
        self.0 -= other;
    }
}

/// TODO: Import venue ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct VenueId(pub u64);

/// TODO: Import leg ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct TransactionLegId(pub u64);

/// The module's configuration trait.
pub trait Config:
    frame_system::Config + BalancesConfig + IdentityConfig + pallet_statistics::Config
{
    /// Pallet's events.
    type RuntimeEvent: From<Event> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    /// Randomness source.
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

    /// Confidential asset pallet weights.
    type WeightInfo: WeightInfo;
}

/// A mercat account consists of the public key that is used for encryption purposes.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct MercatAccount(CompressedEncryptionPubKey);

impl MercatAccount {
    pub fn from_pub_account(account: &PubAccount) -> Self {
        Self(account.owner_enc_pub_key.into())
    }

    pub fn pub_account(&self) -> Option<PubAccount> {
        self.0.into_public_key().map(|pub_key| PubAccount {
            owner_enc_pub_key: pub_key,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.0.into_public_key().is_some()
    }
}

impl From<PubAccount> for MercatAccount {
    fn from(data: PubAccount) -> Self {
        Self(data.owner_enc_pub_key.into())
    }
}

impl From<&PubAccount> for MercatAccount {
    fn from(data: &PubAccount) -> Self {
        Self(data.owner_enc_pub_key.into())
    }
}

/// Confidential transaction leg.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct TransactionLeg {
    /// Asset ticker.
    pub ticker: Ticker,
    /// Mercat account of the sender.
    pub sender: MercatAccount,
    /// Mercat account of the receiver.
    pub receiver: MercatAccount,
    /// Mediator.
    pub mediator: IdentityId,
}

impl TransactionLeg {
    /// Check if the sender/receiver accounts are valid.
    pub fn verify_accounts(&self) -> bool {
        self.sender.is_valid() && self.receiver.is_valid()
    }

    pub fn sender_account(&self) -> Option<PubAccount> {
        self.sender.pub_account()
    }

    pub fn receiver_account(&self) -> Option<PubAccount> {
        self.receiver.pub_account()
    }
}

/// This pending state is initialized from the sender's proof.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct LegPendingState {
    /// The sender's initial balance used to verify the sender's proof.
    pub sender_init_balance: EncryptedAmountWrapper,
    /// The transaction amount encrypted using the sender's public key.
    pub sender_amount: EncryptedAmountWrapper,
    /// The transaction amount encrypted using the receiver's public key.
    pub receiver_amount: EncryptedAmountWrapper,
}

/// Mercat sender proof.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct SenderProof(pub InitializedTransferTxWrapper);

/// Who is affirming the transaction leg.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum ParityAffirmLeg {
    SenderProof(InitializedTransferTxWrapper),
    ReceiverAffirm,
    MediatorAffirm,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct AffirmLeg {
    leg_id: TransactionLegId,
    parity: ParityAffirmLeg,
}

impl AffirmLeg {
    pub fn new_sender(leg_id: TransactionLegId, tx: InitializedTransferTx) -> Self {
        Self {
            leg_id,
            parity: ParityAffirmLeg::SenderProof(tx.into()),
        }
    }

    pub fn new_receiver(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            parity: ParityAffirmLeg::ReceiverAffirm,
        }
    }

    pub fn new_mediator(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            parity: ParityAffirmLeg::MediatorAffirm,
        }
    }
}

/// A global and unique confidential transaction ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct TransactionId(pub u64);
impl_checked_inc!(TransactionId);

/// Confidential asset details.
#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Debug)]
pub struct ConfidentialAssetDetails {
    /// Total supply of the asset.
    pub total_supply: Balance,
    /// Asset's owner DID.
    pub owner_did: IdentityId,
    /// Type of the asset.
    pub asset_type: AssetType,
}

decl_storage! {
    trait Store for Module<T: Config> as ConfidentialAsset {
        /// Details of the confidential asset.
        /// (ticker) -> ConfidentialAssetDetails
        pub Details get(fn confidential_asset_details): map hasher(blake2_128_concat) Ticker => Option<ConfidentialAssetDetails>;

        /// Contains the encryption key for a mercat mediator.
        pub MediatorMercatAccounts get(fn mediator_mercat_accounts):
            map hasher(twox_64_concat) IdentityId => Option<MercatAccount>;

        /// Records the did for a mercat account.
        /// account -> IdentityId.
        pub MercatAccountDid get(fn mercat_account_did):
            map hasher(blake2_128_concat) MercatAccount
            => Option<IdentityId>;

        /// Contains the encrypted balance of a mercat account.
        /// (account, ticker) -> EncryptedAmountWrapper.
        pub MercatAccountBalance get(fn mercat_account_balance):
            double_map hasher(blake2_128_concat) MercatAccount,
            hasher(blake2_128_concat) Ticker
            => Option<EncryptedAmountWrapper>;

        /// Accumulates the encrypted incoming balance for a mercat account.
        /// (account, ticker) -> EncryptedAmountWrapper.
        IncomingBalance get(fn incoming_balance):
            double_map hasher(blake2_128_concat) MercatAccount,
            hasher(blake2_128_concat) Ticker
            => Option<EncryptedAmountWrapper>;

        /// Stores the pending state for a given transaction.
        /// (transaction_id, leg_id) -> Option<LegPendingState>
        pub TxPendingState get(fn mercat_tx_pending_state):
            map hasher(blake2_128_concat) (TransactionId, TransactionLegId) => Option<LegPendingState>;

        /// Legs of a transaction. (transaction_id, leg_id) -> Leg
        pub TransactionLegs get(fn transaction_legs):
            double_map hasher(twox_64_concat) TransactionId, hasher(twox_64_concat) TransactionLegId => Option<TransactionLeg>;

        /// Number of affirmations pending before transaction is executed. transaction_id -> affirms_pending
        PendingAffirms get(fn affirms_pending): map hasher(twox_64_concat) TransactionId => Option<u32>;

        /// Track pending transaction affirmations.
        /// (counter_party, (transaction_id, leg_id)) -> Option<bool>
        UserAffirmations get(fn user_affirmations):
            double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) (TransactionId, TransactionLegId) => Option<bool>;

        /// The storage for mercat sender proofs.
        SenderProofs get(fn sender_proofs):
            double_map hasher(twox_64_concat) TransactionId, hasher(twox_64_concat) TransactionLegId => Option<SenderProof>;

        /// Number of transactions in the system (It's one more than the actual number)
        TransactionCounter get(fn transaction_counter) build(|_| TransactionId(1u64)): TransactionId;

        /// RngNonce - Nonce used as `subject` to `Randomness`.
        RngNonce get(fn rng_nonce): u64;
    }
}

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::RuntimeOrigin {
        type Error = Error<T>;

        /// Initialize the default event for this module
        fn deposit_event() = default;

        /// Verifies the proofs given the `tx` (transaction) for creating a mercat account.
        /// If all proofs pass, it stores the mercat account for the origin identity, sets the
        /// initial encrypted balance, and emits account creation event.
        ///
        /// # Arguments
        /// * `tx` The account creation transaction created by the mercat lib.
        ///
        /// # Errors
        /// * `InvalidAccountCreationProof` if the provided proofs fail to verify.
        /// * `BadOrigin` if `origin` isn't signed.
        #[weight = <T as Config>::WeightInfo::validate_mercat_account()]
        pub fn validate_mercat_account(origin,
            ticker: Ticker,
            tx: PubAccountTxWrapper,
        ) -> DispatchResult {
            let caller_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_validate_mercat_account(caller_did, ticker, tx)
        }

        /// Stores mediator's public key.
        ///
        /// # Arguments
        /// * `public_key` the encryption public key, used during mercat operations.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        #[weight = <T as Config>::WeightInfo::add_mediator_mercat_account()]
        pub fn add_mediator_mercat_account(origin,
            account: MercatAccount,
        ) -> DispatchResult {
            let caller_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_add_mediator_mercat_account(caller_did, account)
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
        #[weight = <T as Config>::WeightInfo::create_confidential_asset()]
        pub fn create_confidential_asset(
            origin,
            name: AssetName,
            ticker: Ticker,
            asset_type: AssetType,
        ) -> DispatchResult {
            let owner_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_create_confidential_asset(owner_did, name, ticker, asset_type)
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
        /// - `TotalSupplyAboveBalanceLimit` if `total_supply` exceeds the mercat balance limit. This is imposed by the MERCAT lib.
        /// - `UnknownConfidentialAsset` The ticker is not part of the set of confidential assets.
        /// - `InvalidAccountMintProof` if the proofs of ticker name and total supply are incorrect.
        #[weight = <T as Config>::WeightInfo::mint_confidential_asset()]
        pub fn mint_confidential_asset(
            origin,
            ticker: Ticker,
            total_supply: Balance,
            asset_mint_proof: InitializedAssetTxWrapper,
        ) -> DispatchResult {
            let owner_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_mint_confidential_asset(owner_did, ticker, total_supply, asset_mint_proof)
        }

        /// Applies any incoming balance to the mercat account balance.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `account` - the mercat account (mercat public key) of the `origin`.
        /// * `ticker` - Ticker of mercat account.
        ///
        /// # Errors
        /// - `BadOrigin` if not signed.
        #[weight = <T as Config>::WeightInfo::apply_incoming_balance()]
        pub fn apply_incoming_balance(
            origin,
            account: MercatAccount,
            ticker: Ticker,
        ) -> DispatchResult {
            let caller_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_apply_incoming_balance(caller_did, account, ticker)
        }

        /// Adds a new transaction.
        ///
        #[weight = <T as Config>::WeightInfo::add_transaction()]
        pub fn add_transaction(
            origin,
            venue_id: VenueId,
            legs: Vec<TransactionLeg>,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_add_transaction(did, venue_id, legs)?;
        }

        /// Affirm a transaction.
        #[weight = <T as Config>::WeightInfo::affirm_transaction(&affirm)]
        pub fn affirm_transaction(
            origin,
            transaction_id: TransactionId,
            affirm: AffirmLeg,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_affirm_transaction(did, transaction_id, affirm.leg_id, affirm.parity)?;
        }

        /// Execute transaction.
        #[weight = <T as Config>::WeightInfo::execute_transaction()]
        pub fn execute_transaction(
            origin,
            transaction_id: TransactionId,
            leg_count: u32,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_execute_transaction(did, transaction_id, leg_count as usize)?;
        }

        /// Revert pending transaction.
        #[weight = <T as Config>::WeightInfo::execute_transaction()]
        pub fn revert_transaction(
            origin,
            _transaction_id: TransactionId,
            _leg_count: u32,
        ) {
            let _did = Identity::<T>::ensure_perms(origin)?;
            // TODO:
            //Self::base_revert_transaction(did, transaction_id, leg_count as usize)?;
        }
    }
}

impl<T: Config> Module<T> {
    fn base_validate_mercat_account(
        caller_did: IdentityId,
        ticker: Ticker,
        tx: PubAccountTxWrapper,
    ) -> DispatchResult {
        let tx: PubAccountTx = tx.into();

        let account: MercatAccount = tx.pub_account.clone().into();
        // Ensure the mercat account's balance hasn't been initialized.
        ensure!(
            !MercatAccountBalance::contains_key(&account, ticker),
            Error::<T>::MercatAccountAlreadyInitialized
        );
        // Ensure the mercat account doesn't exist, or is already linked to the caller's identity.
        MercatAccountDid::try_mutate(&account, |account_did| -> DispatchResult {
            match account_did {
                Some(account_did) => {
                    // Ensure the caller's identity is the same.
                    ensure!(
                        *account_did == caller_did,
                        Error::<T>::MercatAccountAlreadyCreated
                    );
                }
                None => {
                    // Link the mercat account to the caller's identity.
                    *account_did = Some(caller_did);
                }
            }
            Ok(())
        })?;

        // Validate the balance ininitalization proof.
        AccountValidator
            .verify(&tx)
            .map_err(|_| Error::<T>::InvalidAccountCreationProof)?;

        // Initialize the mercat account balance.
        let wrapped_enc_balance = EncryptedAmountWrapper::from(tx.initial_balance);
        MercatAccountBalance::insert(&account, ticker, wrapped_enc_balance.clone());

        Self::deposit_event(Event::AccountCreated(
            caller_did,
            account,
            ticker,
            wrapped_enc_balance,
        ));
        Ok(())
    }

    fn base_add_mediator_mercat_account(
        caller_did: IdentityId,
        account: MercatAccount,
    ) -> DispatchResult {
        ensure!(account.is_valid(), Error::<T>::InvalidMercatAccount);

        MediatorMercatAccounts::insert(&caller_did, &account);

        Self::deposit_event(Event::MediatorAccountCreated(caller_did, account));
        Ok(())
    }

    fn base_create_confidential_asset(
        owner_did: IdentityId,
        _name: AssetName,
        ticker: Ticker,
        asset_type: AssetType,
    ) -> DispatchResult {
        // TODO: store asset name?

        // Ensure the asset hasn't been created yet.
        ensure!(
            !Details::contains_key(ticker),
            Error::<T>::AssetAlreadyCreated
        );

        let details = ConfidentialAssetDetails {
            total_supply: Zero::zero(),
            owner_did,
            asset_type,
        };
        Details::insert(ticker, details);

        Self::deposit_event(Event::ConfidentialAssetCreated(
            owner_did,
            ticker,
            Zero::zero(),
            true,
            asset_type,
            owner_did,
        ));
        Ok(())
    }

    fn base_mint_confidential_asset(
        owner_did: IdentityId,
        ticker: Ticker,
        total_supply: Balance,
        asset_mint_proof: InitializedAssetTxWrapper,
    ) -> DispatchResult {
        let mut details =
            Self::confidential_asset_details(ticker).ok_or(Error::<T>::UnknownConfidentialAsset)?;

        // Only the owner of the asset can change its total supply.
        ensure!(details.owner_did == owner_did, Error::<T>::Unauthorized);

        // Current total supply must be zero.
        // TODO: Allow increasing the total supply?
        ensure!(
            details.total_supply == Zero::zero(),
            Error::<T>::CanSetTotalSupplyOnlyOnce
        );

        // New total supply must be positive.
        ensure!(
            total_supply != Zero::zero(),
            Error::<T>::TotalSupplyMustBePositive
        );

        ensure!(
            Details::contains_key(ticker),
            Error::<T>::UnknownConfidentialAsset
        );

        // At the moment, mercat lib imposes that balances can be at most 32/64 bits.
        let max_balance_mercat = MercatBalance::MAX.saturated_into::<Balance>();
        ensure!(
            total_supply <= max_balance_mercat,
            Error::<T>::TotalSupplyAboveBalanceLimit
        );

        let account: MercatAccount = asset_mint_proof.account.clone().into();
        // Ensure the mercat account's balance has been initialized.
        let old_balance = MercatAccountBalance::try_get(&account, ticker)
            .map_err(|_| Error::<T>::MercatAccountMissing)?;

        let new_encrypted_balance = AssetValidator
            .verify_asset_transaction(
                total_supply.saturated_into::<MercatBalance>(),
                &asset_mint_proof,
                &asset_mint_proof.account,
                &old_balance.into(),
                &[],
            )
            .map_err(|_| Error::<T>::InvalidAccountMintProof)?;

        // Set the total supply (both encrypted and plain).
        MercatAccountBalance::insert(
            &account,
            ticker,
            EncryptedAmountWrapper::from(new_encrypted_balance),
        );

        // This will emit the total supply changed event.
        details.total_supply = total_supply;
        Details::insert(ticker, details);
        // TODO: emit Issue event.

        Ok(())
    }

    fn base_apply_incoming_balance(
        caller_did: IdentityId,
        account: MercatAccount,
        ticker: Ticker,
    ) -> DispatchResult {
        let account_did = Self::get_mercat_account_did(&account)?;
        // Ensure the caller is the owner of the mercat account.
        ensure!(account_did == caller_did, Error::<T>::Unauthorized);

        // Take the incoming balance.
        match IncomingBalance::take(&account, ticker) {
            Some(incoming_balance) => {
                // If there is an incoming balance, apply it to the mercat account balance.
                MercatAccountBalance::try_mutate(&account, ticker, |balance| -> DispatchResult {
                    if let Some(ref mut balance) = balance {
                        *balance += *incoming_balance;
                        Ok(())
                    } else {
                        Err(Error::<T>::MercatAccountMissing.into())
                    }
                })?;
            }
            None => (),
        }

        Ok(())
    }

    pub fn get_mercat_account_did(account: &MercatAccount) -> Result<IdentityId, DispatchError> {
        Self::mercat_account_did(account).ok_or(Error::<T>::MercatAccountMissing.into())
    }

    /// Add the `amount` to the mercat account's `IncomingBalance` accumulator.
    pub fn mercat_account_deposit_amount(
        account: &MercatAccount,
        ticker: Ticker,
        amount: EncryptedAmount,
    ) {
        IncomingBalance::mutate(account, ticker, |incoming_balance| match incoming_balance {
            Some(previous_balance) => {
                *previous_balance += amount;
            }
            None => {
                *incoming_balance = Some(amount.into());
            }
        });
    }

    /// Subtract the `amount` from the mercat account balance.
    pub fn mercat_account_withdraw_amount(
        account: &MercatAccount,
        ticker: Ticker,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        MercatAccountBalance::try_mutate(account, ticker, |balance| -> DispatchResult {
            if let Some(ref mut balance) = balance {
                *balance -= amount;
                Ok(())
            } else {
                Err(Error::<T>::MercatAccountMissing.into())
            }
        })
    }

    pub fn base_add_transaction(
        did: IdentityId,
        venue_id: VenueId,
        legs: Vec<TransactionLeg>,
    ) -> Result<TransactionId, DispatchError> {
        // TODO: Ensure transaction does not have too many legs.

        // TODO: Ensure venue exists & sender is its creator.
        //Self::venue_for_management(venue_id, did)?;

        // Create a list of unique counter parties involved in the transaction.
        // TODO: Counter parties? or just the number of missing proofs?

        // TODO: Check if the venue has required permissions from token owners.

        // Advance and get next `transaction_id`.
        let transaction_id = TransactionCounter::try_mutate(try_next_post::<T, _>)?;

        let pending_affirms = (legs.len() * 3) as u32;
        PendingAffirms::insert(transaction_id, pending_affirms);

        for (i, leg) in legs.iter().enumerate() {
            ensure!(leg.verify_accounts(), Error::<T>::InvalidMercatAccount);
            let leg_id = TransactionLegId(i as u64);
            let sender_did = Self::get_mercat_account_did(&leg.sender)?;
            let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
            UserAffirmations::insert(sender_did, (transaction_id, leg_id), false);
            UserAffirmations::insert(receiver_did, (transaction_id, leg_id), false);
            UserAffirmations::insert(&leg.mediator, (transaction_id, leg_id), false);
            TransactionLegs::insert(transaction_id, leg_id, leg.clone());
        }

        /*
        // TODO: Record transaction details: venue id, status, etc...
        let transaction = Transaction {
            transaction_id,
            venue_id,
            status: TransactionStatus::Pending,
        };
        <TransactionDetails<T>>::insert(transaction_id, transaction);
        */

        Self::deposit_event(Event::TransactionCreated(
            did,
            venue_id,
            transaction_id,
            legs,
        ));

        Ok(transaction_id)
    }

    fn base_execute_transaction(
        _did: IdentityId,
        id: TransactionId,
        leg_count: usize,
    ) -> DispatchResult {
        // Take transaction legs.
        let legs = TransactionLegs::drain_prefix(id).collect::<Vec<_>>();
        ensure!(legs.len() <= leg_count, Error::<T>::LegCountTooSmall);

        // Take pending affirms count and ensure that the transaction has been affirmed.
        let pending_affirms = PendingAffirms::take(id);
        ensure!(
            pending_affirms == Some(0),
            Error::<T>::InstructionNotAffirmed
        );

        for (leg_id, leg) in legs {
            Self::base_confidential_transfer(id, leg_id, leg)?;
        }

        Ok(())
    }

    fn base_affirm_transaction(
        caller_did: IdentityId,
        id: TransactionId,
        leg_id: TransactionLegId,
        parity: ParityAffirmLeg,
    ) -> DispatchResult {
        let leg = TransactionLegs::get(id, leg_id).ok_or(Error::<T>::UnknownInstructionLeg)?;

        // Ensure the caller hasn't already affirmed this leg.
        let caller_affirm = UserAffirmations::take(caller_did, (id, leg_id));
        ensure!(
            caller_affirm == Some(false),
            Error::<T>::InstructionAlreadyAffirmed
        );

        match parity {
            ParityAffirmLeg::SenderProof(init_tx) => {
                let sender_did = Self::mercat_account_did(&leg.sender);
                ensure!(Some(caller_did) == sender_did, Error::<T>::Unauthorized);

                // Ensure the sender/receiver accounts match with the transaction leg.
                let sender_account = &init_tx.memo.sender_account;
                ensure!(
                    MercatAccount::from(sender_account) == leg.sender,
                    Error::<T>::InvalidMercatTransferProof
                );
                let receiver_account = &init_tx.memo.receiver_account;
                ensure!(
                    MercatAccount::from(receiver_account) == leg.receiver,
                    Error::<T>::InvalidMercatTransferProof
                );

                // Get the sender's current balance.
                let from_current_balance = *Self::mercat_account_balance(&leg.sender, leg.ticker)
                    .ok_or(Error::<T>::MercatAccountMissing)?;

                // Verify the sender's proof.
                let mut rng = Self::get_rng();
                verify_initialized_transaction(
                    &init_tx,
                    sender_account,
                    &from_current_balance,
                    receiver_account,
                    &[],
                    &mut rng,
                )
                .map_err(|_| Error::<T>::InvalidMercatTransferProof)?;

                // Withdraw the transaction amount when the sender affirms.
                Self::mercat_account_withdraw_amount(
                    &leg.sender,
                    leg.ticker,
                    init_tx.memo.enc_amount_using_sender,
                )?;

                // Store the pending state for this transaction leg.
                TxPendingState::insert(
                    &(id, leg_id),
                    LegPendingState {
                        sender_init_balance: from_current_balance.into(),
                        sender_amount: init_tx.memo.enc_amount_using_sender.into(),
                        receiver_amount: init_tx.memo.enc_amount_using_receiver.into(),
                    },
                );

                // Store the sender's proof.
                SenderProofs::insert(id, leg_id, SenderProof(init_tx.into()));
            }
            ParityAffirmLeg::ReceiverAffirm => {
                let receiver_did = Self::mercat_account_did(&leg.receiver);
                ensure!(Some(caller_did) == receiver_did, Error::<T>::Unauthorized);
            }
            ParityAffirmLeg::MediatorAffirm => {
                ensure!(caller_did == leg.mediator, Error::<T>::Unauthorized);
            }
        }
        // Update affirmations.
        UserAffirmations::insert(caller_did, (id, leg_id), true);
        PendingAffirms::try_mutate(id, |pending| -> DispatchResult {
            if let Some(ref mut pending) = pending {
                *pending = pending.saturating_sub(1);
                Ok(())
            } else {
                Err(Error::<T>::UnknownInstruction.into())
            }
        })?;

        Ok(())
    }

    /// Transfers an asset from one identity's portfolio to another.
    pub fn base_confidential_transfer(
        instruction_id: TransactionId,
        leg_id: TransactionLegId,
        leg: TransactionLeg,
    ) -> DispatchResult {
        let ticker = leg.ticker;

        // Check affirmations and remove them.
        let sender_did = Self::get_mercat_account_did(&leg.sender)?;
        let sender_affirm = UserAffirmations::take(sender_did, (instruction_id, leg_id));
        ensure!(
            sender_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );
        let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
        let receiver_affirm = UserAffirmations::take(receiver_did, (instruction_id, leg_id));
        ensure!(
            receiver_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );
        let mediator_affirm = UserAffirmations::take(leg.mediator, (instruction_id, leg_id));
        ensure!(
            mediator_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );

        // Take the transaction leg's pending state.
        let pending_state = TxPendingState::take((instruction_id, leg_id))
            .ok_or(Error::<T>::InstructionNotAffirmed)?;

        // Remove the sender proof.
        SenderProofs::remove(instruction_id, leg_id);

        // Deposit the transaction amount into the receiver's account.
        Self::mercat_account_deposit_amount(&leg.receiver, ticker, *pending_state.receiver_amount);

        Ok(())
    }

    fn get_rng() -> Rng {
        // Increase the nonce each time.
        let nonce = RngNonce::get();
        RngNonce::put(nonce.wrapping_add(1));
        // Use the `nonce` and chain randomness to generate a new seed.
        let (random_hash, _) = T::Randomness::random(&(b"ConfidentialAsset", nonce).encode());
        let s = random_hash.as_ref();
        let mut seed = [0u8; 32];
        let len = seed.len().min(s.len());
        seed[..len].copy_from_slice(&s[..len]);
        Rng::from_seed(seed)
    }
}

decl_event! {
    pub enum Event {
        /// Event for creation of a Mediator Mercat account.
        /// caller DID/ owner DID and encryption public key
        MediatorAccountCreated(IdentityId, MercatAccount),

        /// Event for creation of a Mercat account.
        /// caller DID/ owner DID, mercat account id (which is equal to encrypted asset ID), encrypted balance
        AccountCreated(IdentityId, MercatAccount, Ticker, EncryptedAmountWrapper),

        /// Event for creation of a confidential asset.
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type, beneficiary DID
        ConfidentialAssetCreated(IdentityId, Ticker, Balance, bool, AssetType, IdentityId),

        /// Event for resetting the ordering state.
        /// caller DID/ owner DID, mercat account id, current encrypted account balance
        ResetConfidentialAccountOrderingState(IdentityId, MercatAccount, Ticker, EncryptedAmountWrapper),

        /// A new transaction has been created
        /// (did, venue_id, transaction_id, legs)
        TransactionCreated(
            IdentityId,
            VenueId,
            TransactionId,
            Vec<TransactionLeg>,
        ),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The MERCAT account creation proofs are invalid.
        InvalidAccountCreationProof,

        /// Mercat account hasn't been created yet.
        MercatAccountMissing,

        /// Mercat account already created.
        MercatAccountAlreadyCreated,

        /// Mercat account's balance already initialized.
        MercatAccountAlreadyInitialized,

        /// The MERCAT asset issuance proofs are invalid.
        InvalidAccountMintProof,

        /// The provided total supply of a confidential asset is invalid.
        InvalidTotalSupply,

        /// Mercat account isn't a valid CompressedEncryptionPubKey.
        InvalidMercatAccount,

        /// The balance values does not fit a mercat balance.
        TotalSupplyAboveBalanceLimit,

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

        /// We only support one confidential transfer per instruction at the moment.
        MoreThanOneConfidentialLeg,
        /// Certain transfer modes are not yet supported in confidential modes.
        ConfidentialModeNotSupportedYet,
        /// Transaction proof failed to verify.
        /// Failed to maintain confidential transaction's ordering state.
        InvalidMercatOrderingState,
        /// Undefined leg type.
        UndefinedLegKind,
        /// Confidential legs do not have assets or amounts.
        ConfidentialLegHasNoAssetOrAmount,
        /// Only `LegKind::NonConfidential` has receipt functionality.
        InvalidLegKind,
        /// The MERCAT transfer proof is invalid.
        InvalidMercatTransferProof,
        /// Need sender's proof to verify receiver's proof.
        MissingMercatInitializedTransferProof,

        /// The token has already been created.
        AssetAlreadyCreated,

        /// Venue does not exist.
        InvalidVenue,
        /// No pending affirmation for the provided instruction.
        NoPendingAffirm,
        /// Instruction has not been affirmed.
        InstructionNotAffirmed,
        /// Instruction has already been affirmed.
        InstructionAlreadyAffirmed,
        /// Provided instruction is not pending execution.
        InstructionNotPending,
        /// Provided instruction is not failing execution.
        InstructionNotFailed,
        /// Provided leg is not pending execution.
        LegNotPending,
        /// Signer is not authorized by the venue.
        UnauthorizedSigner,
        /// Receipt already used.
        ReceiptAlreadyClaimed,
        /// Receipt not used yet.
        ReceiptNotClaimed,
        /// Venue does not have required permissions.
        UnauthorizedVenue,
        /// While affirming the transfer, system failed to lock the assets involved.
        FailedToLockTokens,
        /// Instruction failed to execute.
        InstructionFailed,
        /// Instruction has invalid dates
        InstructionDatesInvalid,
        /// Instruction's target settle block reached.
        InstructionSettleBlockPassed,
        /// Offchain signature is invalid.
        InvalidSignature,
        /// Sender and receiver are the same.
        SameSenderReceiver,
        /// The provided settlement block number is in the past and cannot be used by the scheduler.
        SettleOnPastBlock,
        /// The current instruction affirmation status does not support the requested action.
        UnexpectedAffirmationStatus,
        /// Scheduling of an instruction fails.
        FailedToSchedule,
        /// Legs count should matches with the total number of legs in which given portfolio act as `from_portfolio`.
        LegCountTooSmall,
        /// Instruction is unknown.
        UnknownInstruction,
        /// Instruction leg is unknown.
        UnknownInstructionLeg,
        /// Maximum legs that can be in a single instruction.
        InstructionHasTooManyLegs,
        /// Signer is already added to venue.
        SignerAlreadyExists,
        /// Signer is not added to venue.
        SignerDoesNotExist,
        /// Instruction leg amount can't be zero
        ZeroAmount,
    }
}
