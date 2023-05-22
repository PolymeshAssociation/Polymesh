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
    impl_checked_inc,
    settlement::VenueId,
    Balance, IdentityId, Memo, Ticker,
};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, SaturatedConversion};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{convert::From, prelude::*};

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

type Identity<T> = identity::Module<T>;
type System<T> = frame_system::Pallet<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub trait WeightInfo {
    fn validate_mercat_account() -> Weight;
    fn add_mediator_mercat_account() -> Weight;
    fn create_confidential_asset() -> Weight;
    fn mint_confidential_asset() -> Weight;
    fn apply_incoming_balance() -> Weight;
    fn create_venue() -> Weight;
    fn set_venue_filtering() -> Weight;
    fn allow_venues(l: u32) -> Weight;
    fn disallow_venues(l: u32) -> Weight;
    fn add_transaction() -> Weight;
    fn sender_affirm_transaction() -> Weight;
    fn receiver_affirm_transaction() -> Weight;
    fn mediator_affirm_transaction() -> Weight;
    fn sender_unaffirm_transaction() -> Weight;
    fn receiver_unaffirm_transaction() -> Weight;
    fn mediator_unaffirm_transaction() -> Weight;
    fn execute_transaction(l: u32) -> Weight;
    fn revert_transaction(l: u32) -> Weight;

    fn affirm_transaction(affirm: &AffirmLeg) -> Weight {
        match affirm.party {
            AffirmParty::Sender(_) => Self::sender_affirm_transaction(),
            AffirmParty::Receiver => Self::receiver_affirm_transaction(),
            AffirmParty::Mediator => Self::mediator_affirm_transaction(),
        }
    }

    fn unaffirm_transaction(unaffirm: &UnaffirmLeg) -> Weight {
        match unaffirm.party {
            UnaffirmParty::Sender => Self::sender_unaffirm_transaction(),
            UnaffirmParty::Receiver => Self::receiver_unaffirm_transaction(),
            UnaffirmParty::Mediator => Self::mediator_unaffirm_transaction(),
        }
    }
}

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

/// Mercat types are uploaded as bytes (hex).
/// This also makes it easier to copy paste the proofs from CLI tools.
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

impl_wrapper!(MercatPubAccountTx, PubAccountTx);
impl_wrapper!(MercatMintAssetTx, InitializedAssetTx);

/// A global and unique confidential transaction ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct TransactionId(#[codec(compact)] pub u64);
impl_checked_inc!(TransactionId);

/// Transaction leg ID.
///
/// The leg ID is it's index position (i.e. the first leg is 0).
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct TransactionLegId(#[codec(compact)] pub u64);

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

/// Mercat sender proof.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct SenderProof(Vec<u8>);

impl SenderProof {
    pub fn into_tx(&self) -> Option<InitializedTransferTx> {
        InitializedTransferTx::decode(&mut self.0.as_slice()).ok()
    }
}

/// Who is affirming the transaction leg.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum AffirmParty {
    Sender(SenderProof),
    Receiver,
    Mediator,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct AffirmLeg {
    leg_id: TransactionLegId,
    party: AffirmParty,
}

impl AffirmLeg {
    pub fn sender(leg_id: TransactionLegId, tx: InitializedTransferTx) -> Self {
        Self {
            leg_id,
            party: AffirmParty::Sender(SenderProof(tx.encode())),
        }
    }

    pub fn receiver(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: AffirmParty::Receiver,
        }
    }

    pub fn mediator(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: AffirmParty::Mediator,
        }
    }
}

/// Who is unaffirming the transaction leg.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum UnaffirmParty {
    Sender,
    Receiver,
    Mediator,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct UnaffirmLeg {
    leg_id: TransactionLegId,
    party: UnaffirmParty,
}

impl UnaffirmLeg {
    pub fn sender(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: UnaffirmParty::Sender,
        }
    }

    pub fn receiver(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: UnaffirmParty::Receiver,
        }
    }

    pub fn mediator(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: UnaffirmParty::Mediator,
        }
    }
}

/// Confidential asset details.
#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Debug)]
pub struct ConfidentialAssetDetails {
    /// Confidential asset name.
    pub name: AssetName,
    /// Total supply of the asset.
    pub total_supply: Balance,
    /// Asset's owner DID.
    pub owner_did: IdentityId,
    /// Type of the asset.
    pub asset_type: AssetType,
}

/// Transaction information.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Transaction<BlockNumber> {
    /// Id of the venue this instruction belongs to
    pub venue_id: VenueId,
    /// BlockNumber that the transaction was created.
    pub created_at: BlockNumber,
    /// Memo attached to the transaction.
    pub memo: Option<Memo>,
}

/// Status of a transaction.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionStatus<BlockNumber> {
    /// Pending affirmation and execution.
    Pending,
    /// Executed at block.
    Executed(BlockNumber),
    /// Reverted at block.
    Reverted(BlockNumber),
}

decl_storage! {
    trait Store for Module<T: Config> as ConfidentialAsset {
        /// Venue creator.
        ///
        /// venue_id -> Option<IdentityId>
        pub VenueCreator get(fn venue_creator):
            map hasher(twox_64_concat) VenueId => Option<IdentityId>;

        /// Track venues created by an identity.
        /// Only needed for the UI.
        ///
        /// creator_did -> venue_id -> ()
        pub IdentityVenues get(fn identity_venues):
            double_map hasher(twox_64_concat) IdentityId,
                       hasher(twox_64_concat) VenueId
                    => ();

        /// Transaction created by a venue.
        /// Only needed for the UI.
        ///
        /// venue_id -> transaction_id -> ()
        pub VenueTransactions get(fn venue_transactions):
            double_map hasher(twox_64_concat) VenueId,
                       hasher(twox_64_concat) TransactionId
                    => ();

        /// Tracks if a token has enabled filtering venues that can create transactions involving their token.
        ///
        /// ticker -> filtering_enabled
        VenueFiltering get(fn venue_filtering): map hasher(blake2_128_concat) Ticker => bool;

        /// Venues that are allowed to create transactions involving a particular ticker. Only used if filtering is enabled.
        ///
        /// ticker -> venue_id -> allowed
        VenueAllowList get(fn venue_allow_list): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) VenueId => bool;

        /// Number of venues in the system (It's one more than the actual number)
        VenueCounter get(fn venue_counter) build(|_| VenueId(1u64)): VenueId;

        /// Details of the confidential asset.
        ///
        /// ticker -> Option<ConfidentialAssetDetails>
        pub Details get(fn confidential_asset_details): map hasher(blake2_128_concat) Ticker => Option<ConfidentialAssetDetails>;

        /// Contains the encryption key for a mercat mediator.
        ///
        /// identity_id -> Option<MercatAccount>
        pub MediatorMercatAccounts get(fn mediator_mercat_accounts):
            map hasher(twox_64_concat) IdentityId => Option<MercatAccount>;

        /// Records the did for a mercat account.
        ///
        /// account -> Option<IdentityId>.
        pub MercatAccountDid get(fn mercat_account_did):
            map hasher(blake2_128_concat) MercatAccount
            => Option<IdentityId>;

        /// Contains the encrypted balance of a mercat account.
        ///
        /// account -> ticker -> Option<EncryptedAmount>
        pub MercatAccountBalance get(fn mercat_account_balance):
            double_map hasher(blake2_128_concat) MercatAccount,
            hasher(blake2_128_concat) Ticker
            => Option<EncryptedAmount>;

        /// Accumulates the encrypted incoming balance for a mercat account.
        ///
        /// account -> ticker -> Option<EncryptedAmount>
        IncomingBalance get(fn incoming_balance):
            double_map hasher(blake2_128_concat) MercatAccount,
            hasher(blake2_128_concat) Ticker
            => Option<EncryptedAmount>;

        /// Legs of a transaction.
        ///
        /// transaction_id -> leg_id -> Option<Leg>
        pub TransactionLegs get(fn transaction_legs):
            double_map hasher(twox_64_concat) TransactionId, hasher(twox_64_concat) TransactionLegId => Option<TransactionLeg>;

        /// Stores the sender's initial balance when they affirmed the transaction leg.
        ///
        /// This is needed to verify the sender's proof.  It is only stored
        /// for clients to use during off-chain proof verification.
        ///
        /// (transaction_id, leg_id) -> Option<EncryptedAmount>
        pub TxLegSenderBalance get(fn tx_leg_sender_balance):
            map hasher(blake2_128_concat) (TransactionId, TransactionLegId) => Option<EncryptedAmount>;

        /// Stores the transfer amount encrypted using the sender's public key.
        ///
        /// This is needed to revert the transaction.
        ///
        /// (transaction_id, leg_id) -> Option<EncryptedAmount>
        pub TxLegSenderAmount get(fn tx_leg_sender_amount):
            map hasher(blake2_128_concat) (TransactionId, TransactionLegId) => Option<EncryptedAmount>;

        /// Stores the transfer amount encrypted using the receiver's public key.
        ///
        /// This is needed to execute the transaction.
        ///
        /// (transaction_id, leg_id) -> Option<EncryptedAmount>
        pub TxLegReceiverAmount get(fn tx_leg_receiver_amount):
            map hasher(blake2_128_concat) (TransactionId, TransactionLegId) => Option<EncryptedAmount>;

        /// Number of affirmations pending before transaction is executed.
        ///
        /// transaction_id -> Option<affirms_pending>
        PendingAffirms get(fn affirms_pending): map hasher(twox_64_concat) TransactionId => Option<u32>;

        /// Track pending transaction affirmations.
        ///
        /// counter_party -> (transaction_id, leg_id) -> Option<bool>
        UserAffirmations get(fn user_affirmations):
            double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) (TransactionId, TransactionLegId) => Option<bool>;

        /// The storage for mercat sender proofs.
        ///
        /// transaction_id -> leg_id -> Option<SenderProof>
        SenderProofs get(fn sender_proofs):
            double_map hasher(twox_64_concat) TransactionId, hasher(twox_64_concat) TransactionLegId => Option<SenderProof>;

        /// Transaction statuses.
        ///
        /// transaction_id -> Option<TransactionStatus>
        TransactionStatuses get(fn transaction_status):
            map hasher(twox_64_concat) TransactionId => Option<TransactionStatus<T::BlockNumber>>;

        /// Details about an instruction.
        ///
        /// transaction_id -> transaction_details
        pub Transactions get(fn transactions):
            map hasher(twox_64_concat) TransactionId => Option<Transaction<T::BlockNumber>>;

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
            tx: MercatPubAccountTx,
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
            asset_mint_proof: MercatMintAssetTx,
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

        /// Registers a new venue.
        ///
        #[weight = <T as Config>::WeightInfo::create_venue()]
        pub fn create_venue(origin) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_create_venue(did)
        }

        /// Enables or disabled venue filtering for a token.
        ///
        /// # Arguments
        /// * `ticker` - Ticker of the token in question.
        /// * `enabled` - Boolean that decides if the filtering should be enabled.
        #[weight = <T as Config>::WeightInfo::set_venue_filtering()]
        pub fn set_venue_filtering(origin, ticker: Ticker, enabled: bool) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_set_venue_filtering(did, ticker, enabled)
        }

        /// Allows additional venues to create instructions involving an asset.
        ///
        /// * `ticker` - Ticker of the token in question.
        /// * `venues` - Array of venues that are allowed to create instructions for the token in question.
        #[weight = <T as Config>::WeightInfo::allow_venues(venues.len() as u32)]
        pub fn allow_venues(origin, ticker: Ticker, venues: Vec<VenueId>) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_update_venue_allow_list(did, ticker, venues, true)
        }

        /// Revokes permission given to venues for creating instructions involving a particular asset.
        ///
        /// * `ticker` - Ticker of the token in question.
        /// * `venues` - Array of venues that are no longer allowed to create instructions for the token in question.
        #[weight = <T as Config>::WeightInfo::disallow_venues(venues.len() as u32)]
        pub fn disallow_venues(origin, ticker: Ticker, venues: Vec<VenueId>) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_update_venue_allow_list(did, ticker, venues, false)
        }

        /// Adds a new transaction.
        ///
        #[weight = <T as Config>::WeightInfo::add_transaction()]
        pub fn add_transaction(
            origin,
            venue_id: VenueId,
            legs: Vec<TransactionLeg>,
            memo: Option<Memo>,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_add_transaction(did, venue_id, legs, memo)?;
        }

        /// Affirm a transaction.
        #[weight = <T as Config>::WeightInfo::affirm_transaction(&affirm)]
        pub fn affirm_transaction(
            origin,
            transaction_id: TransactionId,
            affirm: AffirmLeg,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_affirm_transaction(did, transaction_id, affirm)?;
        }

        /// Unaffirm a transaction.
        #[weight = <T as Config>::WeightInfo::unaffirm_transaction(&unaffirm)]
        pub fn unaffirm_transaction(
            origin,
            transaction_id: TransactionId,
            unaffirm: UnaffirmLeg,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_unaffirm_transaction(did, transaction_id, unaffirm)?;
        }

        /// Execute transaction.
        #[weight = <T as Config>::WeightInfo::execute_transaction(*leg_count)]
        pub fn execute_transaction(
            origin,
            transaction_id: TransactionId,
            leg_count: u32,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_execute_transaction(did, transaction_id, leg_count as usize)?;
        }

        /// Revert pending transaction.
        #[weight = <T as Config>::WeightInfo::execute_transaction(*leg_count)]
        pub fn revert_transaction(
            origin,
            transaction_id: TransactionId,
            leg_count: u32,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_revert_transaction(did, transaction_id, leg_count as usize)?;
        }
    }
}

impl<T: Config> Module<T> {
    fn base_validate_mercat_account(
        caller_did: IdentityId,
        ticker: Ticker,
        tx: MercatPubAccountTx,
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
        let wrapped_enc_balance = tx.initial_balance;
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
        name: AssetName,
        ticker: Ticker,
        asset_type: AssetType,
    ) -> DispatchResult {
        // Ensure the asset hasn't been created yet.
        ensure!(
            !Details::contains_key(ticker),
            Error::<T>::AssetAlreadyCreated
        );

        let details = ConfidentialAssetDetails {
            name,
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
        asset_mint_proof: MercatMintAssetTx,
    ) -> DispatchResult {
        // Ensure the caller is the asset owner and get the asset details.
        let mut details = Self::ensure_asset_owner(ticker, owner_did)?;

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
        MercatAccountBalance::insert(&account, ticker, new_encrypted_balance);

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
                        *balance += incoming_balance;
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

    // Ensure the caller is the asset owner.
    fn ensure_asset_owner(
        ticker: Ticker,
        did: IdentityId,
    ) -> Result<ConfidentialAssetDetails, DispatchError> {
        let details =
            Self::confidential_asset_details(ticker).ok_or(Error::<T>::UnknownConfidentialAsset)?;

        // Ensure that the caller is the asset owner.
        ensure!(details.owner_did == did, Error::<T>::Unauthorized);
        Ok(details)
    }

    // Ensure the caller is the venue creator.
    fn ensure_venue_creator(id: VenueId, did: IdentityId) -> Result<(), DispatchError> {
        // Get the venue creator.
        let creator_did = Self::venue_creator(id).ok_or(Error::<T>::InvalidVenue)?;
        ensure!(creator_did == did, Error::<T>::Unauthorized);
        Ok(())
    }

    /// If `tickers` doesn't contain the given `ticker` and venue_filtering is enabled, ensures that venue_id is in the allowed list
    fn ensure_venue_filtering(
        tickers: &mut BTreeSet<Ticker>,
        ticker: Ticker,
        venue_id: &VenueId,
    ) -> DispatchResult {
        if tickers.insert(ticker) && Self::venue_filtering(ticker) {
            ensure!(
                Self::venue_allow_list(ticker, venue_id),
                Error::<T>::UnauthorizedVenue
            );
        }
        Ok(())
    }

    fn base_create_venue(did: IdentityId) -> DispatchResult {
        // Advance venue counter.
        // NB: Venue counter starts with 1.
        let venue_id = VenueCounter::try_mutate(try_next_post::<T, _>)?;

        // Other commits to storage + emit event.
        VenueCreator::insert(venue_id, did);
        IdentityVenues::insert(did, venue_id, ());
        // TODO:
        //Self::deposit_event(RawEvent::VenueCreated(did, id, details, typ));
        Ok(())
    }

    fn base_set_venue_filtering(did: IdentityId, ticker: Ticker, enabled: bool) -> DispatchResult {
        // Ensure the caller is the asset owner.
        Self::ensure_asset_owner(ticker, did)?;
        if enabled {
            VenueFiltering::insert(ticker, enabled);
        } else {
            VenueFiltering::remove(ticker);
        }
        // TODO:
        //Self::deposit_event(RawEvent::VenueFiltering(did, ticker, enabled));
        Ok(())
    }

    fn base_update_venue_allow_list(
        did: IdentityId,
        ticker: Ticker,
        venues: Vec<VenueId>,
        allow: bool,
    ) -> DispatchResult {
        // Ensure the caller is the asset owner.
        Self::ensure_asset_owner(ticker, did)?;
        if allow {
            for venue in &venues {
                VenueAllowList::insert(&ticker, venue, true);
            }
            // TODO:
            //Self::deposit_event(RawEvent::VenuesAllowed(did, ticker, venues));
        } else {
            for venue in &venues {
                VenueAllowList::remove(&ticker, venue);
            }
            // TODO:
            //Self::deposit_event(RawEvent::VenuesBlocked(did, ticker, venues));
        }
        Ok(())
    }

    pub fn base_add_transaction(
        did: IdentityId,
        venue_id: VenueId,
        legs: Vec<TransactionLeg>,
        memo: Option<Memo>,
    ) -> Result<TransactionId, DispatchError> {
        // Ensure transaction does not have too many legs.
        // TODO: Add pallet constant for limit.
        ensure!(legs.len() <= 10, Error::<T>::InstructionHasTooManyLegs);

        // Ensure venue exists and the caller is its creator.
        Self::ensure_venue_creator(venue_id, did)?;

        // Advance and get next `transaction_id`.
        let transaction_id = TransactionCounter::try_mutate(try_next_post::<T, _>)?;
        VenueTransactions::insert(venue_id, transaction_id, ());

        let pending_affirms = (legs.len() * 3) as u32;
        PendingAffirms::insert(transaction_id, pending_affirms);

        let mut tickers: BTreeSet<Ticker> = BTreeSet::new();
        for (i, leg) in legs.iter().enumerate() {
            // Check if the venue has required permissions from asset owners.
            Self::ensure_venue_filtering(&mut tickers, leg.ticker, &venue_id)?;
            ensure!(leg.verify_accounts(), Error::<T>::InvalidMercatAccount);
            let leg_id = TransactionLegId(i as _);
            let sender_did = Self::get_mercat_account_did(&leg.sender)?;
            let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
            UserAffirmations::insert(sender_did, (transaction_id, leg_id), false);
            UserAffirmations::insert(receiver_did, (transaction_id, leg_id), false);
            UserAffirmations::insert(&leg.mediator, (transaction_id, leg_id), false);
            TransactionLegs::insert(transaction_id, leg_id, leg.clone());
        }

        // Record transaction details and status.
        <Transactions<T>>::insert(
            transaction_id,
            Transaction {
                venue_id,
                created_at: System::<T>::block_number(),
                memo: memo.clone(),
            },
        );
        <TransactionStatuses<T>>::insert(transaction_id, TransactionStatus::Pending);

        Self::deposit_event(Event::TransactionCreated(
            did,
            venue_id,
            transaction_id,
            legs,
            memo,
        ));

        Ok(transaction_id)
    }

    fn base_affirm_transaction(
        caller_did: IdentityId,
        id: TransactionId,
        affirm: AffirmLeg,
    ) -> DispatchResult {
        let leg_id = affirm.leg_id;
        let leg = TransactionLegs::get(id, leg_id).ok_or(Error::<T>::UnknownInstructionLeg)?;

        // Ensure the caller hasn't already affirmed this leg.
        let caller_affirm = UserAffirmations::get(caller_did, (id, leg_id));
        ensure!(
            caller_affirm == Some(false),
            Error::<T>::InstructionAlreadyAffirmed
        );

        match affirm.party {
            AffirmParty::Sender(proof) => {
                let init_tx = proof
                    .into_tx()
                    .ok_or(Error::<T>::InvalidMercatTransferProof)?;
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
                let from_current_balance = Self::mercat_account_balance(&leg.sender, leg.ticker)
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
                TxLegSenderBalance::insert(&(id, leg_id), from_current_balance);
                TxLegSenderAmount::insert(&(id, leg_id), init_tx.memo.enc_amount_using_sender);
                TxLegReceiverAmount::insert(&(id, leg_id), init_tx.memo.enc_amount_using_receiver);

                // Store the sender's proof.
                SenderProofs::insert(id, leg_id, proof);
            }
            AffirmParty::Receiver => {
                let receiver_did = Self::mercat_account_did(&leg.receiver);
                ensure!(Some(caller_did) == receiver_did, Error::<T>::Unauthorized);
            }
            AffirmParty::Mediator => {
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

    fn base_unaffirm_transaction(
        caller_did: IdentityId,
        id: TransactionId,
        unaffirm: UnaffirmLeg,
    ) -> DispatchResult {
        let leg_id = unaffirm.leg_id;

        // Ensure the caller has affirmed this leg.
        let caller_affirm = UserAffirmations::get(caller_did, (id, leg_id));
        ensure!(
            caller_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );

        let leg = TransactionLegs::get(id, leg_id).ok_or(Error::<T>::UnknownInstructionLeg)?;
        match unaffirm.party {
            UnaffirmParty::Sender => {
                let sender_did = Self::mercat_account_did(&leg.sender);
                ensure!(Some(caller_did) == sender_did, Error::<T>::Unauthorized);

                // Take the transaction leg's pending state.
                TxLegSenderBalance::remove((id, leg_id));
                let sender_amount = TxLegSenderAmount::take((id, leg_id))
                    .ok_or(Error::<T>::InstructionNotAffirmed)?;
                TxLegReceiverAmount::remove((id, leg_id));

                // Remove the sender proof.
                SenderProofs::remove(id, leg_id);

                // Deposit the transaction amount back into the sender's account.
                Self::mercat_account_deposit_amount(&leg.sender, leg.ticker, sender_amount)?;
            }
            UnaffirmParty::Receiver => {
                let receiver_did = Self::mercat_account_did(&leg.receiver);
                ensure!(Some(caller_did) == receiver_did, Error::<T>::Unauthorized);
            }
            UnaffirmParty::Mediator => {
                ensure!(caller_did == leg.mediator, Error::<T>::Unauthorized);
            }
        }
        // Update affirmations.
        UserAffirmations::insert(caller_did, (id, leg_id), false);
        PendingAffirms::try_mutate(id, |pending| -> DispatchResult {
            if let Some(ref mut pending) = pending {
                *pending = pending.saturating_add(1);
                Ok(())
            } else {
                Err(Error::<T>::UnknownInstruction.into())
            }
        })?;

        Ok(())
    }

    fn base_execute_transaction(
        _did: IdentityId,
        transaction_id: TransactionId,
        leg_count: usize,
    ) -> DispatchResult {
        // Take transaction legs.
        let legs = TransactionLegs::drain_prefix(transaction_id).collect::<Vec<_>>();
        ensure!(legs.len() <= leg_count, Error::<T>::LegCountTooSmall);

        // Take pending affirms count and ensure that the transaction has been affirmed.
        let pending_affirms = PendingAffirms::take(transaction_id);
        ensure!(
            pending_affirms == Some(0),
            Error::<T>::InstructionNotAffirmed
        );

        // Execute transaction legs.
        for (leg_id, leg) in legs {
            Self::execute_leg(transaction_id, leg_id, leg)?;
        }

        // Remove transaction details and update status.
        <Transactions<T>>::remove(transaction_id);
        let block = System::<T>::block_number();
        <TransactionStatuses<T>>::insert(transaction_id, TransactionStatus::Executed(block));

        // TODO: emit event.
        Ok(())
    }

    /// Transfer the confidential asset into the receiver's incoming account balance.
    fn execute_leg(
        transaction_id: TransactionId,
        leg_id: TransactionLegId,
        leg: TransactionLeg,
    ) -> DispatchResult {
        let ticker = leg.ticker;

        // Check affirmations and remove them.
        let sender_did = Self::get_mercat_account_did(&leg.sender)?;
        let sender_affirm = UserAffirmations::take(sender_did, (transaction_id, leg_id));
        ensure!(
            sender_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );
        let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
        let receiver_affirm = UserAffirmations::take(receiver_did, (transaction_id, leg_id));
        ensure!(
            receiver_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );
        let mediator_affirm = UserAffirmations::take(leg.mediator, (transaction_id, leg_id));
        ensure!(
            mediator_affirm == Some(true),
            Error::<T>::InstructionNotAffirmed
        );

        // Take the transaction leg's pending state.
        TxLegSenderBalance::remove((transaction_id, leg_id));
        TxLegSenderAmount::remove((transaction_id, leg_id));
        let receiver_amount = TxLegReceiverAmount::take((transaction_id, leg_id))
            .ok_or(Error::<T>::InstructionNotAffirmed)?;

        // Remove the sender proof.
        SenderProofs::remove(transaction_id, leg_id);

        // Deposit the transaction amount into the receiver's account.
        Self::mercat_account_deposit_amount_incoming(&leg.receiver, ticker, receiver_amount);

        Ok(())
    }

    fn base_revert_transaction(
        _did: IdentityId,
        transaction_id: TransactionId,
        leg_count: usize,
    ) -> DispatchResult {
        // Take transaction legs.
        let legs = TransactionLegs::drain_prefix(transaction_id).collect::<Vec<_>>();
        ensure!(legs.len() <= leg_count, Error::<T>::LegCountTooSmall);

        // Remove the pending affirmation count.
        PendingAffirms::remove(transaction_id);

        // Revert transaction legs.
        for (leg_id, leg) in legs {
            Self::revert_leg(transaction_id, leg_id, leg)?;
        }

        // Remove transaction details and update status.
        <Transactions<T>>::remove(transaction_id);
        let block = System::<T>::block_number();
        <TransactionStatuses<T>>::insert(transaction_id, TransactionStatus::Reverted(block));

        // TODO: emit event.
        Ok(())
    }

    /// Revert the leg by transfer the `amount` back to the sender.
    fn revert_leg(
        transaction_id: TransactionId,
        leg_id: TransactionLegId,
        leg: TransactionLeg,
    ) -> DispatchResult {
        // Remove user affirmations.
        let sender_did = Self::get_mercat_account_did(&leg.sender)?;
        let sender_affirm = UserAffirmations::take(sender_did, (transaction_id, leg_id));
        let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
        UserAffirmations::remove(receiver_did, (transaction_id, leg_id));
        UserAffirmations::remove(leg.mediator, (transaction_id, leg_id));

        if sender_affirm == Some(true) {
            // Take the transaction leg's pending state.
            match TxLegSenderAmount::take((transaction_id, leg_id)) {
                Some(sender_amount) => {
                    // Deposit the transaction amount back into the sender's incoming account.
                    Self::mercat_account_deposit_amount_incoming(
                        &leg.sender,
                        leg.ticker,
                        sender_amount,
                    );
                }
                None => (),
            }
            TxLegSenderBalance::remove((transaction_id, leg_id));
            TxLegReceiverAmount::remove((transaction_id, leg_id));

            // Remove the sender proof.
            SenderProofs::remove(transaction_id, leg_id);
        }

        Ok(())
    }

    pub fn get_mercat_account_did(account: &MercatAccount) -> Result<IdentityId, DispatchError> {
        Self::mercat_account_did(account).ok_or(Error::<T>::MercatAccountMissing.into())
    }

    /// Subtract the `amount` from the mercat account balance.
    fn mercat_account_withdraw_amount(
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

    /// Add the `amount` to the mercat account's balance.
    fn mercat_account_deposit_amount(
        account: &MercatAccount,
        ticker: Ticker,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        MercatAccountBalance::try_mutate(account, ticker, |balance| -> DispatchResult {
            if let Some(ref mut balance) = balance {
                *balance += amount;
                Ok(())
            } else {
                Err(Error::<T>::MercatAccountMissing.into())
            }
        })
    }

    /// Add the `amount` to the mercat account's `IncomingBalance` accumulator.
    fn mercat_account_deposit_amount_incoming(
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
        AccountCreated(IdentityId, MercatAccount, Ticker, EncryptedAmount),

        /// Event for creation of a confidential asset.
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type, beneficiary DID
        ConfidentialAssetCreated(IdentityId, Ticker, Balance, bool, AssetType, IdentityId),

        /// Event for resetting the ordering state.
        /// caller DID/ owner DID, mercat account id, current encrypted account balance
        ResetConfidentialAccountOrderingState(IdentityId, MercatAccount, Ticker, EncryptedAmount),

        /// A new transaction has been created
        /// (did, venue_id, transaction_id, legs, memo)
        TransactionCreated(
            IdentityId,
            VenueId,
            TransactionId,
            Vec<TransactionLeg>,
            Option<Memo>,
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
        /// Legs count should matches with the total number of legs in the transaction.
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
