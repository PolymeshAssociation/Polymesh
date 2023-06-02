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
    traits::{Get, Randomness},
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
            LegParty::Sender => Self::sender_unaffirm_transaction(),
            LegParty::Receiver => Self::receiver_unaffirm_transaction(),
            LegParty::Mediator => Self::mediator_unaffirm_transaction(),
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

    /// Maximum total supply.
    type MaxTotalSupply: Get<Balance>;

    /// Maximum number of legs in a confidential transaction.
    type MaxNumberOfLegs: Get<u32>;
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

    pub fn leg_party(&self) -> LegParty {
        match self.party {
            AffirmParty::Sender(_) => LegParty::Sender,
            AffirmParty::Receiver => LegParty::Receiver,
            AffirmParty::Mediator => LegParty::Mediator,
        }
    }
}

/// Which party of the transaction leg.
#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq)]
pub enum LegParty {
    Sender,
    Receiver,
    Mediator,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct UnaffirmLeg {
    leg_id: TransactionLegId,
    party: LegParty,
}

impl UnaffirmLeg {
    pub fn sender(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: LegParty::Sender,
        }
    }

    pub fn receiver(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: LegParty::Receiver,
        }
    }

    pub fn mediator(leg_id: TransactionLegId) -> Self {
        Self {
            leg_id,
            party: LegParty::Mediator,
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

        /// Venues that are allowed to create transactions involving a particular ticker.
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
        /// party identity -> (transaction_id, leg_id, leg_party) -> Option<bool>
        UserAffirmations get(fn user_affirmations):
            double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) (TransactionId, TransactionLegId, LegParty) => Option<bool>;

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
        /// - `TotalSupplyAboveMercatBalanceLimit` if `total_supply` exceeds the mercat balance limit. This is imposed by the MERCAT lib.
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
            Error::<T>::ConfidentialAssetAlreadyCreated
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
            asset_type,
        ));
        Ok(())
    }

    fn base_mint_confidential_asset(
        owner_did: IdentityId,
        ticker: Ticker,
        amount: Balance,
        asset_mint_proof: MercatMintAssetTx,
    ) -> DispatchResult {
        // Ensure the caller is the asset owner and get the asset details.
        let mut details = Self::ensure_asset_owner(ticker, owner_did)?;

        // The mint amount must be positive.
        ensure!(
            amount != Zero::zero(),
            Error::<T>::TotalSupplyMustBePositive
        );

        // Ensure the total supply doesn't go above `T::MaxTotalSupply`.
        details.total_supply = details.total_supply.saturating_add(amount);
        ensure!(
            details.total_supply < T::MaxTotalSupply::get(),
            Error::<T>::TotalSupplyOverLimit
        );

        // At the moment, mercat lib imposes that balances can be at most 32/64 bits.
        let max_balance_mercat = MercatBalance::MAX.saturated_into::<Balance>();
        ensure!(
            details.total_supply <= max_balance_mercat,
            Error::<T>::TotalSupplyAboveMercatBalanceLimit
        );

        ensure!(
            Details::contains_key(ticker),
            Error::<T>::UnknownConfidentialAsset
        );

        let account: MercatAccount = asset_mint_proof.account.clone().into();
        // Ensure the mercat account's balance has been initialized.
        ensure!(
            MercatAccountBalance::contains_key(&account, ticker),
            Error::<T>::MercatAccountMissing
        );

        let new_encrypted_balance = AssetValidator
            .verify_asset_transaction(
                amount.saturated_into::<MercatBalance>(),
                &asset_mint_proof,
                &asset_mint_proof.account,
                &[],
            )
            .map_err(|_| Error::<T>::InvalidAccountMintProof)?;

        // Deposit the minted assets into the issuer's mercat account.
        Self::mercat_account_deposit_amount(
            &account,
            ticker,
            asset_mint_proof.memo.enc_issued_amount,
        )?;

        // Emit Issue event with new `total_supply`.
        Self::deposit_event(Event::Issued(
            owner_did,
            ticker,
            amount,
            details.total_supply,
        ));

        // Update `total_supply`.
        Details::insert(ticker, details);
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
                // If there is an incoming balance, deposit it into the mercat account balance.
                Self::mercat_account_deposit_amount(&account, ticker, incoming_balance)?;
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

    /// If `tickers` doesn't contain the given `ticker`, ensures that venue_id is in the allowed list
    fn ensure_venue_filtering(
        tickers: &mut BTreeSet<Ticker>,
        ticker: Ticker,
        venue_id: &VenueId,
    ) -> DispatchResult {
        if tickers.insert(ticker) {
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
        Self::deposit_event(Event::VenueCreated(did, venue_id));
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
            Self::deposit_event(Event::VenuesAllowed(did, ticker, venues));
        } else {
            for venue in &venues {
                VenueAllowList::remove(&ticker, venue);
            }
            Self::deposit_event(Event::VenuesBlocked(did, ticker, venues));
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
        ensure!(
            legs.len() <= T::MaxNumberOfLegs::get() as usize,
            Error::<T>::TransactionHasTooManyLegs
        );

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
            UserAffirmations::insert(
                sender_did,
                (transaction_id, leg_id, LegParty::Sender),
                false,
            );
            UserAffirmations::insert(
                receiver_did,
                (transaction_id, leg_id, LegParty::Receiver),
                false,
            );
            UserAffirmations::insert(
                &leg.mediator,
                (transaction_id, leg_id, LegParty::Mediator),
                false,
            );
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
        let leg = TransactionLegs::get(id, leg_id).ok_or(Error::<T>::UnknownTransactionLeg)?;

        // Ensure the caller hasn't already affirmed this leg.
        let party = affirm.leg_party();
        let caller_affirm = UserAffirmations::get(caller_did, (id, leg_id, party));
        ensure!(
            caller_affirm == Some(false),
            Error::<T>::TransactionAlreadyAffirmed
        );

        match affirm.party {
            AffirmParty::Sender(proof) => {
                let init_tx = proof
                    .into_tx()
                    .ok_or(Error::<T>::InvalidMercatSenderProof)?;
                let sender_did = Self::mercat_account_did(&leg.sender);
                ensure!(Some(caller_did) == sender_did, Error::<T>::Unauthorized);

                // Get sender/receiver accounts from the leg.
                let sender_account = leg
                    .sender_account()
                    .ok_or(Error::<T>::InvalidMercatAccount)?;
                let receiver_account = leg
                    .receiver_account()
                    .ok_or(Error::<T>::InvalidMercatAccount)?;

                // Get the sender's current balance.
                let from_current_balance = Self::mercat_account_balance(&leg.sender, leg.ticker)
                    .ok_or(Error::<T>::MercatAccountMissing)?;

                // Verify the sender's proof.
                let mut rng = Self::get_rng();
                verify_initialized_transaction(
                    &init_tx,
                    &sender_account,
                    &from_current_balance,
                    &receiver_account,
                    &[],
                    &mut rng,
                )
                .map_err(|_| Error::<T>::InvalidMercatSenderProof)?;

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
        UserAffirmations::insert(caller_did, (id, leg_id, party), true);
        PendingAffirms::try_mutate(id, |pending| -> DispatchResult {
            if let Some(ref mut pending) = pending {
                *pending = pending.saturating_sub(1);
                Ok(())
            } else {
                Err(Error::<T>::UnknownTransaction.into())
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
        let caller_affirm = UserAffirmations::get(caller_did, (id, leg_id, unaffirm.party));
        ensure!(
            caller_affirm == Some(true),
            Error::<T>::TransactionNotAffirmed
        );

        let leg = TransactionLegs::get(id, leg_id).ok_or(Error::<T>::UnknownTransactionLeg)?;
        let pending_affirms = match unaffirm.party {
            LegParty::Sender => {
                let sender_did = Self::mercat_account_did(&leg.sender);
                ensure!(Some(caller_did) == sender_did, Error::<T>::Unauthorized);

                let mut pending_affirms = 1;
                // If the receiver has affirmed the leg, then we need to invalid their affirmation.
                let receiver_did = Self::mercat_account_did(&leg.receiver)
                    .ok_or(Error::<T>::MercatAccountMissing)?;
                UserAffirmations::mutate(
                    receiver_did,
                    (id, leg_id, LegParty::Receiver),
                    |affirmed| {
                        if *affirmed == Some(true) {
                            pending_affirms += 1;
                        }
                        *affirmed = Some(false)
                    },
                );
                // If the mediator has affirmed the leg, then we need to invalid their affirmation.
                UserAffirmations::mutate(
                    leg.mediator,
                    (id, leg_id, LegParty::Mediator),
                    |affirmed| {
                        if *affirmed == Some(true) {
                            pending_affirms += 1;
                        }
                        *affirmed = Some(false)
                    },
                );

                // Take the transaction leg's pending state.
                TxLegSenderBalance::remove((id, leg_id));
                let sender_amount = TxLegSenderAmount::take((id, leg_id))
                    .ok_or(Error::<T>::TransactionNotAffirmed)?;
                TxLegReceiverAmount::remove((id, leg_id));

                // Remove the sender proof.
                SenderProofs::remove(id, leg_id);

                // Deposit the transaction amount back into the sender's account.
                Self::mercat_account_deposit_amount(&leg.sender, leg.ticker, sender_amount)?;

                pending_affirms
            }
            LegParty::Receiver => {
                let receiver_did = Self::mercat_account_did(&leg.receiver);
                ensure!(Some(caller_did) == receiver_did, Error::<T>::Unauthorized);

                1
            }
            LegParty::Mediator => {
                ensure!(caller_did == leg.mediator, Error::<T>::Unauthorized);

                1
            }
        };
        // Update affirmations.
        UserAffirmations::insert(caller_did, (id, leg_id, unaffirm.party), false);
        PendingAffirms::try_mutate(id, |pending| -> DispatchResult {
            if let Some(ref mut pending) = pending {
                *pending = pending.saturating_add(pending_affirms);
                Ok(())
            } else {
                Err(Error::<T>::UnknownTransaction.into())
            }
        })?;

        Ok(())
    }

    fn base_execute_transaction(
        caller_did: IdentityId,
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
            Error::<T>::TransactionNotAffirmed
        );

        // Remove transaction details.
        let details =
            <Transactions<T>>::take(transaction_id).ok_or(Error::<T>::UnknownTransactionLeg)?;

        // Execute transaction legs.
        for (leg_id, leg) in legs {
            Self::execute_leg(transaction_id, leg_id, leg)?;
        }

        // Update status.
        let block = System::<T>::block_number();
        <TransactionStatuses<T>>::insert(transaction_id, TransactionStatus::Executed(block));

        Self::deposit_event(Event::TransactionExecuted(
            caller_did,
            transaction_id,
            details.memo,
        ));
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
        let sender_affirm =
            UserAffirmations::take(sender_did, (transaction_id, leg_id, LegParty::Sender));
        ensure!(
            sender_affirm == Some(true),
            Error::<T>::TransactionNotAffirmed
        );
        let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
        let receiver_affirm =
            UserAffirmations::take(receiver_did, (transaction_id, leg_id, LegParty::Receiver));
        ensure!(
            receiver_affirm == Some(true),
            Error::<T>::TransactionNotAffirmed
        );
        let mediator_affirm =
            UserAffirmations::take(leg.mediator, (transaction_id, leg_id, LegParty::Mediator));
        ensure!(
            mediator_affirm == Some(true),
            Error::<T>::TransactionNotAffirmed
        );

        // Take the transaction leg's pending state.
        TxLegSenderBalance::remove((transaction_id, leg_id));
        TxLegSenderAmount::remove((transaction_id, leg_id));
        let receiver_amount = TxLegReceiverAmount::take((transaction_id, leg_id))
            .ok_or(Error::<T>::TransactionNotAffirmed)?;

        // Remove the sender proof.
        SenderProofs::remove(transaction_id, leg_id);

        // Deposit the transaction amount into the receiver's account.
        Self::mercat_account_deposit_amount_incoming(&leg.receiver, ticker, receiver_amount);
        Ok(())
    }

    fn base_revert_transaction(
        caller_did: IdentityId,
        transaction_id: TransactionId,
        leg_count: usize,
    ) -> DispatchResult {
        // Take transaction legs.
        let legs = TransactionLegs::drain_prefix(transaction_id).collect::<Vec<_>>();
        ensure!(legs.len() <= leg_count, Error::<T>::LegCountTooSmall);

        // Remove the pending affirmation count.
        PendingAffirms::remove(transaction_id);

        // Remove transaction details.
        let details =
            <Transactions<T>>::take(transaction_id).ok_or(Error::<T>::UnknownTransactionLeg)?;

        // Revert transaction legs.
        for (leg_id, leg) in legs {
            Self::revert_leg(transaction_id, leg_id, leg)?;
        }

        // Update status.
        let block = System::<T>::block_number();
        <TransactionStatuses<T>>::insert(transaction_id, TransactionStatus::Reverted(block));

        Self::deposit_event(Event::TransactionReverted(
            caller_did,
            transaction_id,
            details.memo,
        ));
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
        let sender_affirm =
            UserAffirmations::take(sender_did, (transaction_id, leg_id, LegParty::Sender));
        let receiver_did = Self::get_mercat_account_did(&leg.receiver)?;
        UserAffirmations::remove(receiver_did, (transaction_id, leg_id, LegParty::Receiver));
        UserAffirmations::remove(leg.mediator, (transaction_id, leg_id, LegParty::Mediator));

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
        let balance = MercatAccountBalance::try_mutate(
            account,
            ticker,
            |balance| -> Result<EncryptedAmount, DispatchError> {
                if let Some(ref mut balance) = balance {
                    *balance -= amount;
                    Ok(*balance)
                } else {
                    Err(Error::<T>::MercatAccountMissing.into())
                }
            },
        )?;
        Self::deposit_event(Event::AccountWithdraw(account.clone(), ticker, balance));
        Ok(())
    }

    /// Add the `amount` to the mercat account's balance.
    fn mercat_account_deposit_amount(
        account: &MercatAccount,
        ticker: Ticker,
        amount: EncryptedAmount,
    ) -> DispatchResult {
        let balance = MercatAccountBalance::try_mutate(
            account,
            ticker,
            |balance| -> Result<EncryptedAmount, DispatchError> {
                if let Some(ref mut balance) = balance {
                    *balance += amount;
                    Ok(*balance)
                } else {
                    Err(Error::<T>::MercatAccountMissing.into())
                }
            },
        )?;
        Self::deposit_event(Event::AccountDeposit(account.clone(), ticker, balance));
        Ok(())
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
        Self::deposit_event(Event::AccountDepositIncoming(
            account.clone(),
            ticker,
            amount,
        ));
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
        /// caller DID, Mediator mercat account (public key)
        MediatorAccountCreated(IdentityId, MercatAccount),
        /// Event for creation of a Mercat account.
        /// caller DID, mercat account (public key), ticker, encrypted balance
        AccountCreated(IdentityId, MercatAccount, Ticker, EncryptedAmount),
        /// Event for creation of a confidential asset.
        /// (caller DID, ticker, total supply, asset type)
        ConfidentialAssetCreated(IdentityId, Ticker, Balance, AssetType),
        /// Issued confidential assets.
        /// (caller DID, ticker, amount issued, total_supply)
        Issued(IdentityId, Ticker, Balance, Balance),
        /// A new venue has been created.
        /// (caller DID, venue_id)
        VenueCreated(IdentityId, VenueId),
        /// Venues added to allow list.
        /// (caller DID, ticker, Vec<VenueId>)
        VenuesAllowed(IdentityId, Ticker, Vec<VenueId>),
        /// Venues removed from the allow list.
        /// (caller DID, ticker, Vec<VenueId>)
        VenuesBlocked(IdentityId, Ticker, Vec<VenueId>),
        /// A new transaction has been created
        /// (caller DID, venue_id, transaction_id, legs, memo)
        TransactionCreated(
            IdentityId,
            VenueId,
            TransactionId,
            Vec<TransactionLeg>,
            Option<Memo>,
        ),
        /// Confidential transaction executed.
        /// (caller DID, transaction_id, memo)
        TransactionExecuted(
            IdentityId,
            TransactionId,
            Option<Memo>,
        ),
        /// Confidential transaction reverted.
        /// (caller DID, transaction_id, memo)
        TransactionReverted(
            IdentityId,
            TransactionId,
            Option<Memo>,
        ),
        /// Mercat account balance decreased.
        /// This happens when the sender affirms the transaction.
        /// (mercat account, ticker, new encrypted balance)
        AccountWithdraw(MercatAccount, Ticker, EncryptedAmount),
        /// Mercat account balance increased.
        /// This happens when the sender unaffirms a transaction or
        /// when the receiver calls `apply_incoming_balance`.
        /// (mercat account, ticker, new encrypted balance)
        AccountDeposit(MercatAccount, Ticker, EncryptedAmount),
        /// Mercat account has an incoming amount.
        /// This happens when a transaction executes.
        /// (mercat account, ticker, encrypted amount)
        AccountDepositIncoming(MercatAccount, Ticker, EncryptedAmount),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The MERCAT account creation proofs are invalid.
        InvalidAccountCreationProof,
        /// The MERCAT asset issuance proofs are invalid.
        InvalidAccountMintProof,
        /// Mercat account hasn't been created yet.
        MercatAccountMissing,
        /// Mercat account already created.
        MercatAccountAlreadyCreated,
        /// Mercat account's balance already initialized.
        MercatAccountAlreadyInitialized,
        /// Mercat account isn't a valid CompressedEncryptionPubKey.
        InvalidMercatAccount,
        /// The balance values does not fit a mercat balance.
        TotalSupplyAboveMercatBalanceLimit,
        /// The user is not authorized.
        Unauthorized,
        /// The provided asset is not among the set of valid asset ids.
        UnknownConfidentialAsset,
        /// The confidential asset has already been created.
        ConfidentialAssetAlreadyCreated,
        /// A confidential asset's total supply can't go above `T::MaxTotalSupply`.
        TotalSupplyOverLimit,
        /// A confidential asset's total supply must be positive.
        TotalSupplyMustBePositive,
        /// Insufficient mercat authorizations are provided.
        InsufficientMercatAuthorizations,
        /// Confidential transfer's proofs are invalid.
        ConfidentialTransferValidationFailure,
        /// The MERCAT sender proof is invalid.
        InvalidMercatSenderProof,
        /// Venue does not exist.
        InvalidVenue,
        /// Transaction has not been affirmed.
        TransactionNotAffirmed,
        /// Transaction has already been affirmed.
        TransactionAlreadyAffirmed,
        /// Venue does not have required permissions.
        UnauthorizedVenue,
        /// Transaction failed to execute.
        TransactionFailed,
        /// Legs count should matches with the total number of legs in the transaction.
        LegCountTooSmall,
        /// Transaction is unknown.
        UnknownTransaction,
        /// Transaction leg is unknown.
        UnknownTransactionLeg,
        /// Maximum legs that can be in a single instruction.
        TransactionHasTooManyLegs,
    }
}
