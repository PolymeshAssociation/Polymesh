// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh

//! # Confidential assets Pallet
//!
//! The Confidential Assets pallet provides sender, receiver, asset and value confidentiality.
//!
//! ## Overview
//!
//! These pallets call out to the [Polymesh DART library](https://github.com/PolymeshAssociation/polymesh-dart)
//! which implements the ZK-proofs for DART.
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Compact, Decode, Encode};
use frame_support::pallet_prelude::DispatchError;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::{
        fungible::{Inspect, Mutate},
        tokens::Preservation::Expendable,
        Get,
    },
    weights::{Weight, WeightToFee},
    BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use polymesh_primitives::{
    erc20::{Name, Symbol, MAX_DECIMALS, MAX_NAME_LEN, MAX_SYMBOL_LEN},
    Balance, IdentityId,
};
use scale_info::TypeInfo;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{BoundedBTreeMap, BoundedBTreeSet};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::From;
use sp_std::vec::Vec;

use polymesh_dart::{
    curve_tree::WrappedCurveTreeParameters, AccountPublicKey, AccountRegistrationProof,
    AccountStateCommitment, AccountStateNullifier, AccountStateUpdate,
    AssetId as ConfidentialAssetId, AssetKeys, AssetMintingProof, AssetState,
    BatchedAccountAssetRegistrationProof, BatchedFeeAccountRegistrationProof,
    BatchedFeeAccountTopupProof, BatchedProof, BatchedProofs, BatchedSettlementProof, DartLimits,
    EncryptionKeyRegistrationProof, EncryptionPublicKey, Error as DartError,
    FeeAccountPaymentProof, FeeAccountStateCommitment, FeeAccountStateNullifier,
    FeePaymentWithBatchedProofs, InstantReceiverAffirmationProof, InstantSenderAffirmationProof,
    InstantSettlementProof, LeafIndex, LegEncrypted, LegId, LegRef, MediatorAffirmationProof,
    PolymeshPrivateLimits, ProofHash, ReceiverAffirmationProof, ReceiverClaimProof,
    SenderAffirmationProof, SenderCounterUpdateProof, SenderReversalProof, SettlementCounts,
    SettlementProof, SettlementRef, ACCOUNT_TREE_HEIGHT, ASSET_TREE_HEIGHT,
    FEE_ACCOUNT_TREE_HEIGHT, FEE_ASSET_ID,
};

use polymesh_worker_extension::{native_polymesh_worker, BackendKind, WorkerSessionId};
use polymesh_worker_protocol_dart_v0::{
    UpdateAssetStateRequest, VerifyDartAssetRequest, PROTOCOL as DART_PROTOCOL,
};

pub type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type AuditorKeys =
    BoundedBTreeSet<EncryptionPublicKey, <PolymeshPrivateLimits as DartLimits>::MaxAssetAuditors>;
pub type MediatorKeys = BoundedBTreeMap<
    AccountPublicKey,
    EncryptionPublicKey,
    <PolymeshPrivateLimits as DartLimits>::MaxAssetMediators,
>;

type PalletIdentity<T> = pallet_identity::Pallet<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(feature = "testing")]
pub mod testing;

pub mod weights;

mod curve_tree;
pub use curve_tree::*;

mod settlement;
pub use settlement::*;

/// Pallet ID for Confidential Assets pallet.
///
/// This is used to hold the POLYX used in private fee payments.
pub const CONFIDENTIAL_ASSETS_FEE_ID: PalletId = PalletId(*b"pm/dartf");

pub trait WeightInfo {
    fn update_account_curve_tree_root(l: u32) -> Weight;
    fn update_fee_account_curve_tree_root(l: u32) -> Weight;

    fn register_fee_accounts(k: u32) -> Weight;
    fn topup_fee_accounts(k: u32) -> Weight;
    fn verify_fee_payment() -> Weight;

    fn register_accounts(k: u32) -> Weight;

    fn register_encryption_keys(k: u32) -> Weight;

    fn create_asset() -> Weight;
    fn create_settlement(l: u32) -> Weight;
    fn mediator_affirmation() -> Weight;

    // Account state transition weights.
    fn register_account_assets(p: u32) -> Weight;
    fn mint_asset() -> Weight;
    fn sender_affirmation() -> Weight;
    fn receiver_affirmation() -> Weight;
    fn instant_sender_affirmation() -> Weight;
    fn instant_receiver_affirmation() -> Weight;
    fn sender_update_counter() -> Weight;
    fn sender_revert() -> Weight;
    fn receiver_claim() -> Weight;

    fn batched_settlement(counts: SettlementCounts) -> Weight {
        Self::create_settlement(counts.leg_count)
            .saturating_add(
                Self::sender_affirmation_with_leaf().saturating_mul(counts.sender_count),
            )
            .saturating_add(
                Self::receiver_affirmation_with_leaf().saturating_mul(counts.receiver_count),
            )
            .saturating_add(Self::mediator_affirmation().saturating_mul(counts.mediator_count))
    }

    fn execute_instant_settlement(counts: SettlementCounts) -> Weight {
        Self::create_settlement(counts.leg_count)
            .saturating_add(
                Self::instant_sender_affirmation_with_leaf().saturating_mul(counts.sender_count),
            )
            .saturating_add(
                Self::instant_receiver_affirmation_with_leaf()
                    .saturating_mul(counts.receiver_count),
            )
            .saturating_add(Self::mediator_affirmation().saturating_mul(counts.mediator_count))
    }

    fn batched_proofs(batch: &BatchedProofs<PolymeshPrivateLimits>) -> Weight {
        let mut weight = Weight::zero();
        for proof in &batch.proofs {
            match proof {
                BatchedProof::CreateSettlement(proof) => {
                    weight =
                        weight.saturating_add(Self::create_settlement(proof.legs.len() as u32));
                }
                BatchedProof::SenderAffirmation(_proof) => {
                    weight = weight.saturating_add(Self::sender_affirmation_with_leaf());
                }
                BatchedProof::ReceiverAffirmation(_proof) => {
                    weight = weight.saturating_add(Self::receiver_affirmation_with_leaf());
                }
                BatchedProof::MediatorAffirmation(_proof) => {
                    weight = weight.saturating_add(Self::mediator_affirmation());
                }
                BatchedProof::SenderCounterUpdate(_proof) => {
                    weight = weight.saturating_add(Self::sender_update_counter_with_leaf());
                }
                BatchedProof::SenderReversal(_proof) => {
                    weight = weight.saturating_add(Self::sender_revert_with_leaf());
                }
                BatchedProof::ReceiverClaim(_proof) => {
                    weight = weight.saturating_add(Self::receiver_claim_with_leaf());
                }
                BatchedProof::ExecuteInstantSettlement(proof) => {
                    weight = weight.saturating_add(Self::execute_instant_settlement(
                        proof.count_leg_affirmations(),
                    ))
                }
                BatchedProof::InstantSenderAffirmation(_proof) => {
                    weight = weight.saturating_add(Self::instant_sender_affirmation_with_leaf());
                }
                BatchedProof::InstantReceiverAffirmation(_proof) => {
                    weight = weight.saturating_add(Self::instant_receiver_affirmation_with_leaf());
                }
            }
        }
        weight
    }

    fn relayer_submit_batched_proofs(
        batch: &FeePaymentWithBatchedProofs<PolymeshPrivateLimits>,
    ) -> Weight {
        Self::verify_fee_payment_with_leaf()
            .saturating_add(Self::batched_proofs(&batch.batched_proofs))
    }

    fn on_init() -> Weight {
        Self::update_account_curve_tree_root(0)
            .saturating_add(Self::update_fee_account_curve_tree_root(0))
    }

    fn insert_account_leaf(l: u32) -> Weight {
        Self::update_account_curve_tree_root(l)
            .saturating_sub(Self::update_account_curve_tree_root(0))
    }

    fn insert_fee_account_leaf(l: u32) -> Weight {
        Self::update_fee_account_curve_tree_root(l)
            .saturating_sub(Self::update_fee_account_curve_tree_root(0))
    }

    fn register_fee_accounts_with_leaf(k: u32) -> Weight {
        Self::register_fee_accounts(k).saturating_add(Self::insert_fee_account_leaf(k))
    }

    fn topup_fee_accounts_with_leaf(k: u32) -> Weight {
        Self::topup_fee_accounts(k).saturating_add(Self::insert_fee_account_leaf(k))
    }

    fn verify_fee_payment_with_leaf() -> Weight {
        Self::verify_fee_payment().saturating_add(Self::insert_fee_account_leaf(1))
    }

    fn register_account_assets_with_leaf(p: u32) -> Weight {
        Self::register_account_assets(p).saturating_add(Self::insert_account_leaf(p))
    }

    fn mint_asset_with_leaf() -> Weight {
        Self::mint_asset().saturating_add(Self::insert_account_leaf(1))
    }

    fn sender_affirmation_with_leaf() -> Weight {
        Self::sender_affirmation().saturating_add(Self::insert_account_leaf(1))
    }

    fn receiver_affirmation_with_leaf() -> Weight {
        Self::receiver_affirmation().saturating_add(Self::insert_account_leaf(1))
    }

    fn instant_sender_affirmation_with_leaf() -> Weight {
        Self::instant_sender_affirmation().saturating_add(Self::insert_account_leaf(1))
    }

    fn instant_receiver_affirmation_with_leaf() -> Weight {
        Self::instant_receiver_affirmation().saturating_add(Self::insert_account_leaf(1))
    }

    fn sender_update_counter_with_leaf() -> Weight {
        Self::sender_update_counter().saturating_add(Self::insert_account_leaf(1))
    }

    fn sender_revert_with_leaf() -> Weight {
        Self::sender_revert().saturating_add(Self::insert_account_leaf(1))
    }

    fn receiver_claim_with_leaf() -> Weight {
        Self::receiver_claim().saturating_add(Self::insert_account_leaf(1))
    }
}

/// Confidential asset details.
#[derive(Clone, Encode, Decode, Debug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct AssetDetails<T: Config> {
    /// Total supply of the asset.
    pub total_supply: Balance,
    /// Asset's owner DID.
    pub owner_did: IdentityId,
    /// Asset data.
    pub data: BoundedVec<u8, T::MaxAssetDataLength>,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_identity::Config + pallet_transaction_payment::Config
    {
        /// Confidential asset pallet weights.
        type WeightInfo: WeightInfo;

        /// Currency used for fee payments.
        type Currency: Mutate<Self::AccountId, Balance = <<Self as pallet_transaction_payment::Config>::WeightToFee as WeightToFee>::Balance>;

        /// Maximum total supply.
        #[pallet::constant]
        type MaxTotalSupply: Get<Balance>;

        /// Maximum asset data length.
        #[pallet::constant]
        type MaxAssetDataLength: Get<u32>;

        /// The maximum number of keys in an account registration proof.
        #[pallet::constant]
        type MaxKeysPerRegProof: Get<u32>;

        /// The maximum number of proofs in a single batched proof.
        #[pallet::constant]
        type MaxBatchedProofs: Get<u32>;

        /// The maximum number of fee account registration proofs in a single transaction.
        #[pallet::constant]
        type MaxFeeAccountRegProofs: Get<u32>;

        /// The maximum number of fee account topup proofs in a single transaction.
        #[pallet::constant]
        type MaxFeeAccountTopupProofs: Get<u32>;

        /// The maximum number of account asset registration proofs in a single transaction.
        #[pallet::constant]
        type MaxAccountAssetRegProofs: Get<u32>;

        /// The maximum number of legs in a settlement.
        #[pallet::constant]
        type MaxSettlementLegs: Get<u32>;

        /// The maximum settlement memo length.
        #[pallet::constant]
        type MaxSettlementMemoLength: Get<u32>;

        /// The maximum number of asset auditors.
        #[pallet::constant]
        type MaxAssetAuditors: Get<u32>;

        /// The maximum number of asset mediators.
        #[pallet::constant]
        type MaxAssetMediators: Get<u32>;

        /// The maximum number of asset encryption keys (mediators + auditors).
        #[pallet::constant]
        type MaxAssetEncryptionKeys: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Confidential account has been registered.
        AccountRegistered {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Confidential account (public key)
            account: AccountPublicKey,
            /// Confidential account (encryuption key)
            encryption_key: EncryptionPublicKey,
        },
        /// An encryption key has been registered.
        EncryptionKeyRegistered {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Encryption key.
            encryption_key: EncryptionPublicKey,
        },
        /// A new Confidential asset has been created.
        AssetCreated {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Asset ID.
            asset_id: ConfidentialAssetId,
            /// Mediators.
            mediators: MediatorKeys,
            /// Auditors.
            auditors: AuditorKeys,
            /// Asset name.
            name: Name,
            /// Asset symbol.
            symbol: Symbol,
            /// Asset decimals.
            decimals: u8,
            /// Extra asset metadata.
            data: BoundedVec<u8, T::MaxAssetDataLength>,
        },
        /// A Confidential asset has been updated.
        AssetUpdated {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Asset ID.
            asset_id: ConfidentialAssetId,
            /// Mediators.
            mediators: MediatorKeys,
            /// Auditors.
            auditors: AuditorKeys,
        },
        /// Account asset registered.
        AccountAssetRegistered {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Confidential account (public key)
            account: AccountPublicKey,
            /// Confidential asset ID.
            asset_id: ConfidentialAssetId,
        },
        /// Minted Confidential asset.
        AssetMinted {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Confidential asset ID.
            asset_id: ConfidentialAssetId,
            /// Amount minted.
            amount: Balance,
            /// Total supply after minting.
            total_supply: Balance,
            /// Confidential account (public key)
            account: AccountPublicKey,
        },
        /// Fee account updated.
        ///
        /// This event is emitted for both registration and top-up of fee accounts.
        FeeAccountUpdated {
            /// Caller's identity.
            caller_did: IdentityId,
            /// Confidential account (public key)
            account: AccountPublicKey,
            /// Is registration or top-up.
            is_registration: bool,
            /// If this was a registration then `amount` is the initial top-up amount.
            amount: BalanceOf<T>,
        },
        /// Settlement created.
        SettlementCreated {
            /// Settlement reference.
            settlement_ref: SettlementRef,
            /// Settlement memo.
            memo: BoundedVec<u8, <PolymeshPrivateLimits as DartLimits>::MaxSettlementMemoLength>,
            /// Asset CurveTree root_block.
            asset_root_block: BlockNumberFor<T>,
            /// Legs in the settlement.
            legs:
                BoundedVec<LegEncrypted, <PolymeshPrivateLimits as DartLimits>::MaxSettlementLegs>,
        },
        /// Sender has affirmed a leg.
        SenderAffirmed {
            /// Settlement Leg reference.
            leg_ref: LegRef,
        },
        /// Receiver has affirmed a leg.
        ReceiverAffirmed {
            /// Settlement Leg reference.
            leg_ref: LegRef,
        },
        /// Mediator has affirmed a leg.
        MediatorAffirmed {
            /// Settlement Leg reference.
            leg_ref: LegRef,
            /// Mediator key index.
            key_index: u8,
        },
        /// Mediator has rejected a leg.
        MediatorRejected {
            /// Settlement Leg reference.
            leg_ref: LegRef,
            /// Mediator key index.
            key_index: u8,
        },
        /// Sender updated counter.
        SenderCounterUpdated {
            /// Settlement Leg reference.
            leg_ref: LegRef,
        },
        /// Sender has reverted a leg.
        SenderReverted {
            /// Settlement Leg reference.
            leg_ref: LegRef,
        },
        /// Receiver has claimed assets.
        ReceiverClaimed {
            /// Settlement Leg reference.
            leg_ref: LegRef,
        },
        /// Settlement status updated.
        SettlementStatusUpdated {
            /// Settlement reference.
            settlement_ref: SettlementRef,
            /// New status of the settlement.
            status: SettlementStatus,
        },
        /// Account curve tree leaf inserted.
        ///
        /// This curve tree is append-only, so we only store the new leaf.
        AccountStateLeafInserted {
            /// The new leaf index.
            leaf_index: LeafIndex,
            /// The new account state commitment.
            account_commitment: AccountStateCommitment,
        },
        /// Fee account curve tree leaf inserted.
        ///
        /// This curve tree is append-only, so we only store the new leaf.
        FeeAccountStateLeafInserted {
            /// The new leaf index.
            leaf_index: LeafIndex,
            /// The new fee account state commitment.
            fee_account_commitment: FeeAccountStateCommitment,
        },
        /// An asset state leaf has been updated.
        ///
        /// This curve tree is mutable, so we can update existing leaves.
        AssetStateLeafUpdated {
            /// The new leaf index.
            leaf_index: LeafIndex,
            /// The new asset leaf.
            asset_leaf: AssetLeaf,
        },
        /// Asset curve tree root updated.
        AssetCurveTreeRootUpdated {
            /// The new root.
            root: AssetTreeRoot,
        },
        /// Account curve tree root updated.
        AccountCurveTreeRootUpdated {
            /// The new root.
            root: AccountTreeRoot,
        },
        /// Fee account curve tree root updated.
        FeeAccountCurveTreeRootUpdated {
            /// The new root.
            root: FeeAccountTreeRoot,
        },
        /// POLYX deposited into the fee account.
        FeeAccountDeposited {
            /// Sender account.
            sender: T::AccountId,
            /// Amount deposited.
            amount: BalanceOf<T>,
        },
        /// POLYX withdrawn from the fee account.
        FeeAccountWithdrawn {
            /// Receiver account.
            receiver: T::AccountId,
            /// Amount withdrawn.
            amount: BalanceOf<T>,
        },
        /// Relayer submitted batched proofs including fee payment.
        RelayerBatchedProofs {
            /// Relayer account.
            relayer: T::AccountId,
            /// Fee amount.
            amount: BalanceOf<T>,
            /// Batch proofs context hash.
            batch_hash: ProofHash,
            /// Batch results.
            batch_result: DispatchResult,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Confidential account hasn't been registered yet.
        AccountMissing,
        /// Confidential account already exists.
        AccountAlreadyCreated,
        /// Confidential account has already registered that asset.
        AccountAssetAlreadyRegistered,
        /// Encryption key already registered.
        EncryptionKeyAlreadyRegistered,
        /// Confidential fee account hasn't been registered yet.
        FeeAccountMissing,
        /// Confidential fee account already registered.
        FeeAccountAlreadyRegistered,
        /// Insufficient fee payment amount.
        InsufficientFeePaymentAmount,
        /// Invalid Fee Payment proof.
        InvalidFeePaymentProof,
        /// Batch of proofs can't be empty.
        EmptyBatchedProofs,
        /// Invalid fee asset id.
        InvalidFeeAssetId,
        /// Amount overflow.
        AmountOverflow,
        /// CurveTree error.
        CurveTreeError,
        /// CurveTree root not found.
        CurveTreeRootNotFound,
        /// Leaf not found in the curve tree.
        LeafNotFound,
        /// Invalid proof provided.
        InvalidProof,
        /// Asset state is invalid.
        AssetStateInvalid,
        /// Confidential asset hasn't been registered yet.
        AssetMissing,
        /// The caller is not the owner of the Confidential account.
        NotAccountOwner,
        /// The caller is not the owner of the Confidential asset.
        NotAssetOwner,
        /// The asset total supply cannot exceed the maximum total supply.
        MaxTotalSupplyExceeded,
        /// The nullifier for the account state commitment has already been used.
        NullifierAlreadyUsed,
        /// Encryption key for the Confidential account is missing.
        EncryptionKeyMissing,
        /// Settlement already exists.
        SettlementAlreadyExists,
        /// Settlement is missing legs.
        SettlementMissingLegs,
        /// Settlement has too many legs.
        SettlementTooManyLegs,
        /// Batched settlement has invalid leg references.
        BatchedSettlementInvalidLegRefs,
        /// Already affirmed.
        AlreadyAffirmed,
        /// Already rejected.
        AlreadyRejected,
        /// Already finalized.
        AlreadyFinalized,
        /// Too many mediators for this leg.
        TooManyMediators,
        /// Wrong mediator id for this leg.
        WrongMediatorId,
        /// No pending affirmations for this settlement.
        NoPendingAffirmations,
        /// Settlement not found.
        SettlementNotFound,
        /// Leg not found in the settlement.
        LegNotFound,
        /// Settlement not pending.
        SettlementNotPending,
        /// Settlement not executed.
        SettlementNotExecuted,
        /// Settlement not rejected.
        SettlementNotRejected,
        /// CurveTree parameters not set.
        CurveTreeParametersNotSet,
        /// No current worker session.
        NoCurrentWorkerSession,
        /// Confidential assets require at least one mediator or auditor.
        NoAuditorsOrMediators,
        /// Not the last pending affirmation for the settlement.
        NotLastPendingAffirmation,
        /// Too many decimals for the asset.
        TooManyDecimals,
        /// Name too long for the asset.
        NameTooLong,
        /// Symbol too long for the asset.
        SymbolTooLong,
        /// Invalid asset name.
        InvalidAssetName,
        /// Invalid affirmation status transition.
        InvalidAffirmationStatusTransition,
    }

    impl<T: Config> From<DartError> for Error<T> {
        fn from(_error: DartError) -> Self {
            Self::CurveTreeError
        }
    }

    /// Cache wrapped asset curve tree parameters.
    #[pallet::storage]
    pub(crate) type CachedAssetCurveTreeParameters<T: Config> =
        StorageValue<_, WrappedCurveTreeParameters, OptionQuery>;

    /// Cache wrapped account curve tree parameters.
    #[pallet::storage]
    pub(crate) type CachedAccountCurveTreeParameters<T: Config> =
        StorageValue<_, WrappedCurveTreeParameters, OptionQuery>;

    /// Next Asset ID to be used for Confidential assets.
    #[pallet::storage]
    pub(crate) type NextAssetId<T: Config> = StorageValue<_, ConfidentialAssetId, ValueQuery>;

    /// Mapping of Confidential Asset ID to its details.
    #[pallet::storage]
    pub(super) type Details<T: Config> =
        StorageMap<_, Twox64Concat, ConfidentialAssetId, AssetDetails<T>, OptionQuery>;

    /// Mapping of Confidential Asset ID to its auditor and mediator keys.
    #[pallet::storage]
    pub(super) type Keys<T: Config> =
        StorageMap<_, Twox64Concat, ConfidentialAssetId, AssetKeys, OptionQuery>;

    /// A Confidential assets token name.
    #[pallet::storage]
    pub(super) type Names<T: Config> =
        StorageMap<_, Twox64Concat, ConfidentialAssetId, Name, OptionQuery>;

    /// A Confidential assets token symbol.
    #[pallet::storage]
    pub(super) type Symbols<T: Config> =
        StorageMap<_, Twox64Concat, ConfidentialAssetId, Symbol, OptionQuery>;

    /// A Confidential assets token decimals.
    #[pallet::storage]
    pub(super) type Decimals<T: Config> =
        StorageMap<_, Twox64Concat, ConfidentialAssetId, u8, OptionQuery>;

    /// Mapping of asset owner to their assets.
    #[pallet::storage]
    pub(super) type OwnerAssets<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, IdentityId>,
            NMapKey<Twox64Concat, ConfidentialAssetId>,
        ),
        (),
        OptionQuery,
    >;

    /// Encryption key to identity mapping.
    ///
    /// This is used for the auditor and mediator encryption keys.
    #[pallet::storage]
    pub(super) type EncryptionKeyDid<T: Config> =
        StorageMap<_, Twox64Concat, EncryptionPublicKey, IdentityId, OptionQuery>;

    /// Confidential account to identity mapping.
    #[pallet::storage]
    pub(super) type AccountDid<T: Config> =
        StorageMap<_, Twox64Concat, AccountPublicKey, IdentityId, OptionQuery>;

    /// Mapping of identity to their Confidential accounts.
    #[pallet::storage]
    pub(super) type DidAccounts<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, IdentityId>,
            NMapKey<Twox64Concat, AccountPublicKey>,
        ),
        (),
        OptionQuery,
    >;

    /// Confidential fee account to identity mapping.
    #[pallet::storage]
    pub(super) type FeeAccountDid<T: Config> =
        StorageMap<_, Twox64Concat, AccountPublicKey, IdentityId, OptionQuery>;

    /// Mapping of Confidential account public keys to their encryption keys.
    #[pallet::storage]
    pub(super) type AccountEncryptionKey<T: Config> =
        StorageMap<_, Twox64Concat, AccountPublicKey, EncryptionPublicKey, OptionQuery>;

    /// Mapping of Confidential encryption keys to their public keys.
    #[pallet::storage]
    pub(super) type EncryptionKeyAccount<T: Config> =
        StorageMap<_, Twox64Concat, EncryptionPublicKey, AccountPublicKey, OptionQuery>;

    /// Confidential account asset registrations.
    ///
    /// The chain must prevent the same account from registering the same asset multiple times.
    ///
    /// This is a double map where the first key is the account public key and the second key is the asset ID.
    #[pallet::storage]
    pub(super) type AccountAssetRegistrations<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, AccountPublicKey>,
            NMapKey<Twox64Concat, ConfidentialAssetId>,
        ),
        bool,
        ValueQuery,
    >;

    /// Leaf storage for Confidential assets curve tree.
    ///
    /// A counted map is used since we need to support updating the leaves in the tree.
    #[pallet::storage]
    pub(super) type AssetLeaves<T: Config> =
        CountedStorageMap<_, Twox64Concat, LeafIndex, AssetLeaf, OptionQuery>;

    /// Inner node storage for Confidential assets curve tree.
    #[pallet::storage]
    pub(super) type AssetInnerNodes<T: Config> =
        StorageMap<_, Twox64Concat, AssetNodeLocation, AssetInnerNode, OptionQuery>;

    /// The current CurveTree Root for Confidential assets curve tree.
    #[pallet::storage]
    pub(super) type AssetCurveTreeCurrentRoot<T: Config> =
        StorageValue<_, AssetTreeRoot, OptionQuery>;

    /// CurveTree Roots for Confidential assets curve tree.
    ///
    /// At the end of each block we will store the root of the assets curve tree.
    /// The map key is the block number and the value is the root of the assets curve tree.
    #[pallet::storage]
    pub(super) type AssetCurveTreeRoots<T: Config> =
        StorageMap<_, Identity, BlockNumberFor<T>, AssetTreeRoot, OptionQuery>;

    /// The block number of the last asset curve tree root update.
    ///
    /// This is used to track the last time the asset curve tree was updated.
    #[pallet::storage]
    pub(crate) type AssetCurveTreeLastUpdate<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Leaf storage for Confidential accounts curve tree.
    ///
    /// The leaves are immutable, so we use a simple storage map.
    #[pallet::storage]
    pub(super) type AccountLeaves<T: Config> =
        StorageMap<_, Twox64Concat, LeafIndex, AccountStateCommitment, OptionQuery>;

    /// Next leaf index for Confidential accounts curve tree.
    ///
    /// This is used to allocate new leaves in the tree.
    #[pallet::storage]
    pub(super) type NextAccountLeafIndex<T: Config> = StorageValue<_, LeafIndex, ValueQuery>;

    /// The last committed leaf index for Confidential accounts curve tree.
    ///
    /// This is used to do batched inserts into the tree.
    #[pallet::storage]
    pub(super) type LastCommittedAccountLeafIndex<T: Config> =
        StorageValue<_, LeafIndex, ValueQuery>;

    /// Inner node storage for Confidential accounts curve tree.
    #[pallet::storage]
    pub(super) type AccountInnerNodes<T: Config> =
        StorageMap<_, Twox64Concat, AccountNodeLocation, AccountInnerNode, OptionQuery>;

    /// The current CurveTree Root for Confidential accounts curve tree.
    #[pallet::storage]
    pub(super) type AccountCurveTreeCurrentRoot<T: Config> =
        StorageValue<_, AccountTreeRoot, OptionQuery>;

    /// CurveTree Roots for Confidential accounts curve tree.
    ///
    /// At the end of each block we will store the root of the accounts curve tree.
    /// The map key is the block number and the value is the root of the accounts curve tree.
    #[pallet::storage]
    pub(super) type AccountCurveTreeRoots<T: Config> =
        StorageMap<_, Identity, BlockNumberFor<T>, AccountTreeRoot, OptionQuery>;

    /// The block number of the last account curve tree root update.
    ///
    /// This is used to track the last time the account curve tree was updated.
    #[pallet::storage]
    pub(crate) type AccountCurveTreeLastUpdate<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Nullifiers for Confidential account state commitments.
    ///
    /// This is used to ensure that the same account state commitment cannot be used twice.
    #[pallet::storage]
    pub(super) type AccountStateCommitmentNullifiers<T: Config> =
        StorageMap<_, Identity, AccountStateNullifier, (), OptionQuery>;

    /// Confidential fee account egistrations.
    ///
    /// The chain must prevent the same account from registering the multiple times.
    #[pallet::storage]
    pub(super) type FeeAccountRegistrations<T: Config> =
        StorageMap<_, Twox64Concat, AccountPublicKey, bool, ValueQuery>;

    /// Leaf storage for Confidential fee accounts curve tree.
    ///
    /// The leaves are immutable, so we use a simple storage map.
    #[pallet::storage]
    pub(super) type FeeAccountLeaves<T: Config> =
        StorageMap<_, Twox64Concat, LeafIndex, FeeAccountStateCommitment, OptionQuery>;

    /// Next leaf index for Confidential fee accounts curve tree.
    ///
    /// This is used to allocate new leaves in the tree.
    #[pallet::storage]
    pub(super) type NextFeeAccountLeafIndex<T: Config> = StorageValue<_, LeafIndex, ValueQuery>;

    /// The last committed leaf index for Confidential fee accounts curve tree.
    ///
    /// This is used to do batched inserts into the tree.
    #[pallet::storage]
    pub(super) type LastCommittedFeeAccountLeafIndex<T: Config> =
        StorageValue<_, LeafIndex, ValueQuery>;

    /// Inner node storage for Confidential fee accounts curve tree.
    #[pallet::storage]
    pub(super) type FeeAccountInnerNodes<T: Config> =
        StorageMap<_, Twox64Concat, FeeAccountNodeLocation, FeeAccountInnerNode, OptionQuery>;

    /// The current CurveTree Root for Confidential fee accounts curve tree.
    #[pallet::storage]
    pub(super) type FeeAccountCurveTreeCurrentRoot<T: Config> =
        StorageValue<_, FeeAccountTreeRoot, OptionQuery>;

    /// CurveTree Roots for Confidential fee accounts curve tree.
    ///
    /// At the end of each block we will store the root of the fee accounts curve tree.
    /// The map key is the block number and the value is the root of the fee accounts curve tree.
    #[pallet::storage]
    pub(super) type FeeAccountCurveTreeRoots<T: Config> =
        StorageMap<_, Identity, BlockNumberFor<T>, FeeAccountTreeRoot, OptionQuery>;

    /// The block number of the last fee account curve tree root update.
    ///
    /// This is used to track the last time the fee account curve tree was updated.
    #[pallet::storage]
    pub(crate) type FeeAccountCurveTreeLastUpdate<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Nullifiers for Confidential fee account state commitments.
    ///
    /// This is used to ensure that the same fee account state commitment cannot be used twice.
    #[pallet::storage]
    pub(super) type FeeAccountStateCommitmentNullifiers<T: Config> =
        StorageMap<_, Identity, FeeAccountStateNullifier, (), OptionQuery>;

    /// The settlement status.
    #[pallet::storage]
    pub(crate) type SettlementState<T: Config> =
        StorageMap<_, Identity, SettlementRef, SettlementStatus, OptionQuery>;

    /// The settlement memo.
    #[pallet::storage]
    pub(crate) type SettlementMemo<T: Config> = StorageMap<
        _,
        Identity,
        SettlementRef,
        BoundedVec<u8, <PolymeshPrivateLimits as DartLimits>::MaxSettlementMemoLength>,
        OptionQuery,
    >;

    /// The settlement legs.
    ///
    /// This is a double map where the first key is the settlement ID and the second key is the leg ID.
    /// The value is the DartSettlementLeg.
    #[pallet::storage]
    pub(crate) type SettlementLegs<T: Config> = StorageNMap<
        _,
        (NMapKey<Identity, SettlementRef>, NMapKey<Identity, LegId>),
        LegEncrypted,
        OptionQuery,
    >;

    /// The number of legs in each settlement.
    #[pallet::storage]
    pub(crate) type SettlementLegCount<T: Config> =
        StorageMap<_, Identity, SettlementRef, Compact<u32>, OptionQuery>;

    /// The number of pending affirmations for a settlement.
    /// This is used to track when a settlement can be executed.
    #[pallet::storage]
    pub(crate) type SettlementPendingAffirmations<T: Config> =
        StorageMap<_, Identity, SettlementRef, u32, ValueQuery>;

    /// The number of pending finalizations for a settlement.
    /// This is used to track when a settlement can be finalized and all storage can be cleaned up.
    #[pallet::storage]
    pub(crate) type SettlementPendingFinalizations<T: Config> =
        StorageMap<_, Identity, SettlementRef, u32, ValueQuery>;

    /// The affirmation status of each party in a settlement leg.
    /// This is a triple map where the first key is the settlement ID, the second key is the leg ID, and the third key is the party (sender, receiver, mediator).
    #[pallet::storage]
    pub(crate) type LegAffirmationStatus<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Identity, SettlementRef>,
            NMapKey<Identity, LegId>,
            NMapKey<Identity, LegAffirmParty>,
        ),
        AffirmationStatus,
        OptionQuery,
    >;

    /// The WorkerSessionId for the current block.
    #[pallet::storage]
    pub(crate) type CurrentWorkerSessionId<T: Config> =
        StorageValue<_, WorkerSessionId, OptionQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Generate the asset curve tree parameters.
            let params = WrappedCurveTreeParameters::new::<AssetTreeConfig>()
                .expect("Asset curve tree parameters should be valid; qed");
            // Store the asset curve tree parameters in the storage.
            CachedAssetCurveTreeParameters::<T>::put(params);

            // Generate the account curve tree parameters.
            let params = WrappedCurveTreeParameters::new::<AccountTreeConfig>()
                .expect("Account curve tree parameters should be valid; qed");
            // Store the account curve tree parameters in the storage.
            CachedAccountCurveTreeParameters::<T>::put(params);

            // ============================= TODO: This might not be needed.
            //
            // The host functions are not available in the genesis build, so we cannot initialize the curve trees here.
            // // Initialize the Asset Curve Tree.
            // {
            //     let mut tree = Pallet::<T>::get_asset_curve_tree()
            //         .expect("Asset curve tree should be initialized; qed");
            //     tree.init_root()
            //         .expect("Asset curve tree should be able to init root; qed");
            // }
            // // Initialize the Account Curve Tree.
            // {
            //     let mut tree = Pallet::<T>::get_account_curve_tree()
            //         .expect("Account curve tree should be initialized; qed");
            //     tree.init_root()
            //         .expect("Account curve tree should be able to init root; qed");
            // }
            // // Initialize the Fee Account Curve Tree.
            // {
            //     let mut tree = Pallet::<T>::get_fee_account_curve_tree()
            //         .expect("Fee account curve tree should be initialized; qed");
            //     tree.init_root()
            //         .expect("Fee account curve tree should be able to init root; qed");
            // }
            // =============================
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Self::init_block()
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            Self::finalize_block();
        }
    }

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        /// Get the Confidential Assets fee pallet id.
        pub fn pallet_fee_id() -> PalletId {
            CONFIDENTIAL_ASSETS_FEE_ID
        }

        /// Get the Confidential Assets fee account id.
        pub fn fee_account_id() -> T::AccountId {
            Self::pallet_fee_id().into_account_truncating()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a dart account.
        ///
        /// # Arguments
        /// * `account` the dart account to register.
        /// * `encryption_key` the encryption key for the dart account.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `AccountAlreadyCreated` if the dart account or encryption key is already registered.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_accounts(proof.len() as u32))]
        pub fn register_accounts(
            origin: OriginFor<T>,
            proof: AccountRegistrationProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            let caller_did = PalletIdentity::<T>::ensure_perms(origin)?;

            // Ensure the accounts haven't already been registered.
            for account in &proof.accounts {
                // Ensure the dart account doesn't exist.
                ensure!(
                    !AccountDid::<T>::contains_key(&account.acct),
                    Error::<T>::AccountAlreadyCreated
                );
                // Ensure the encryption key doesn't exist.
                ensure!(
                    !EncryptionKeyDid::<T>::contains_key(&account.enc),
                    Error::<T>::EncryptionKeyAlreadyRegistered,
                );
                // Link the dart account to the caller's identity.
                AccountDid::<T>::insert(&account.acct, caller_did);
                // Link the caller's identity to the dart account.
                DidAccounts::<T>::insert((caller_did, &account.acct), ());
                // Link the encryption key to the caller's identity.
                EncryptionKeyDid::<T>::insert(&account.enc, caller_did);
                // Link the encryption key to the dart account.
                AccountEncryptionKey::<T>::insert(&account.acct, &account.enc);
                // Link the dart account to the encryption key.
                EncryptionKeyAccount::<T>::insert(&account.enc, &account.acct);

                Self::deposit_event(Event::<T>::AccountRegistered {
                    caller_did,
                    account: account.acct,
                    encryption_key: account.enc,
                });
            }

            // Verify the proof.
            Self::submit_and_wait(VerifyDartAssetRequest::AccountRegistration {
                did: caller_did.into(),
                proof,
            })?;

            Ok(())
        }

        /// Create a new Confidential Asset.
        ///
        /// # Arguments
        /// * `auditor_or_mediator` - The auditor or mediator public key.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `AccountMissing` if the auditor or mediator is not registered.
        /// * `EncryptionKeyMissing` if the encryption key of the auditor or mediator is not registered.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::create_asset())]
        pub fn create_asset(
            origin: OriginFor<T>,
            name: Name,
            symbol: Symbol,
            decimals: u8,
            mediators: MediatorKeys,
            auditors: AuditorKeys,
            data: BoundedVec<u8, T::MaxAssetDataLength>,
        ) -> DispatchResult {
            let issuer = PalletIdentity::<T>::ensure_perms(origin)?;

            Self::base_create_asset(issuer, name, symbol, decimals, mediators, auditors, data)?;
            Ok(())
        }

        /// Batch register multiple accounts and assets.
        ///
        /// This is used to initialize the first account state commitment of the Confidential asset for the Confidential account.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.  They must be the owner of the Confidential account.
        /// * `proof` - The Batched Account asset registration proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `AccountMissing` if the Confidential account is not registered.
        /// * `AssetMissing` if the Confidential asset is not registered.
        /// * `AccountAssetAlreadyRegistered` if the Confidential account has already registered the Confidential asset.
        /// * `NotAccountOwner` if the caller is not the owner of the Confidential account.
        /// * `InvalidProof` if the proof is invalid.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::register_account_assets_with_leaf(proof.len() as u32))]
        pub fn register_account_assets(
            origin: OriginFor<T>,
            proof: BatchedAccountAssetRegistrationProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            // Ensure the caller is allowed to make this call.
            let caller_did = PalletIdentity::<T>::ensure_perms(origin)?;

            let mut seen_account = BTreeSet::new();
            let mut seen_asset = BTreeSet::new();
            let mut registrations = Vec::with_capacity(proof.proofs.len());
            for p in &proof.proofs {
                if !seen_account.contains(&p.account.acct) {
                    seen_account.insert(p.account.acct.clone());
                    // Ensure the Confidential account is registered to the caller's identity.
                    Self::ensure_dart_account_owner(caller_did, &p.account.acct)?;
                }
                if !seen_asset.contains(&p.asset_id) {
                    seen_asset.insert(p.asset_id);
                    // Ensure the Confidential asset exists.
                    Self::ensure_dart_asset_exists(p.asset_id)?;
                }

                // Ensure the Confidential account hasn't already registered the Confidential asset.
                ensure!(
                    !AccountAssetRegistrations::<T>::get((&p.account.acct, &p.asset_id)),
                    Error::<T>::AccountAssetAlreadyRegistered
                );
                AccountAssetRegistrations::<T>::insert((&p.account.acct, &p.asset_id), true);

                registrations.push((p.account, p.asset_id, p.account_state_commitment));
            }

            // Verify the proof.
            Self::submit_and_wait(VerifyDartAssetRequest::BatchedAccountAssetRegistration {
                did: caller_did.into(),
                proof,
            })?;

            // Process each registration.
            for (account, asset_id, account_state_commitment) in registrations {
                // Insert the new account state commitment from the proof into the account curve tree.
                Self::insert_account_leaf(account_state_commitment, None)?;

                // Emit an event for the account asset registration.
                Self::deposit_event(Event::<T>::AccountAssetRegistered {
                    caller_did,
                    asset_id,
                    account: account.acct,
                });
            }

            Ok(())
        }

        /// Mint a Confidential asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call. They must be the owner of the Confidential asset and Confidential account.
        /// * `proof` - The minting proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `AccountMissing` if the Confidential account is not registered.
        /// * `AssetMissing` if the Confidential asset is not registered.
        /// * `NotAccountOwner` if the caller is not the owner of the Confidential account.
        /// * `InvalidProof` if the proof is invalid.
        /// * `NotAssetOwner` if the caller is not the owner of the Confidential asset.
        /// * `MaxTotalSupplyExceeded` if the total supply of the Confidential asset exceeds the maximum total supply.
        /// * `NullifierAlreadyUsed` if the nullifier for the account state commitment has already been used.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::mint_asset_with_leaf())]
        pub fn mint_asset(origin: OriginFor<T>, proof: AssetMintingProof) -> DispatchResult {
            // Ensure the caller has the required permissions to mint the asset.
            let caller_did = Self::ensure_dart_account_permissions(origin, &proof.pk)?;

            // Ensure the caller is the owner of the Confidential asset.
            let mut asset_details = Self::ensure_dart_asset_owner(caller_did, proof.asset_id)?;

            // Update the total supply of the Confidential asset and ensure it does not exceed the maximum total supply.
            let amount = proof.amount as Balance;
            asset_details.total_supply = asset_details
                .total_supply
                .checked_add(amount)
                .ok_or(Error::<T>::MaxTotalSupplyExceeded)?;
            ensure!(
                asset_details.total_supply <= T::MaxTotalSupply::get(),
                Error::<T>::MaxTotalSupplyExceeded
            );

            let asset_id = proof.asset_id;
            let account = proof.pk;
            // Handle the account state update proof and verify the nullifier.
            Self::handle_account_state_update_proof(proof, |proof, root| {
                // Verify the proof.
                Self::submit_and_wait(VerifyDartAssetRequest::MintAsset {
                    did: caller_did.into(),
                    root,
                    proof,
                })
            })?;

            // Store the updated asset details.
            Details::<T>::insert(asset_id, &asset_details);

            // Emit an event for the asset minting.
            Self::deposit_event(Event::<T>::AssetMinted {
                caller_did,
                asset_id,
                amount,
                total_supply: asset_details.total_supply,
                account,
            });

            Ok(())
        }

        /// Create a new settlement.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The settlement proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `InvalidProof` if the proof is invalid.
        /// * `SettlementMissingLegs` if the settlement has no legs.
        /// * `SettlementTooManyLegs` if the settlement has more legs than the maximum allowed.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::create_settlement(proof.legs.len() as u32))]
        pub fn create_settlement(
            origin: OriginFor<T>,
            proof: SettlementProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_create_settlement(proof)
        }

        /// Sender affirms a settlement leg.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The sender affirmation proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotPending` if the settlement is not pending.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        /// * `AlreadyAffirmed` if the leg has already been affirmed by the sender.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::sender_affirmation_with_leaf())]
        pub fn sender_affirmation(
            origin: OriginFor<T>,
            proof: SenderAffirmationProof,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_sender_affirmation(proof)
        }

        /// Receiver affirms a settlement leg.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The receiver affirmation proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotPending` if the settlement is not pending.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        /// * `AlreadyAffirmed` if the leg has already been affirmed by the receiver.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::receiver_affirmation_with_leaf())]
        pub fn receiver_affirmation(
            origin: OriginFor<T>,
            proof: ReceiverAffirmationProof,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_receiver_affirmation(proof)
        }

        /// Mediator affirms a settlement leg.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The mediator affirmation proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotPending` if the settlement is not pending.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        /// * `AlreadyAffirmed` if the leg has already been affirmed by the mediator.
        /// * `WrongMediatorId` if the mediator ID does not match the expected mediator for the leg.
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::mediator_affirmation())]
        pub fn mediator_affirmation(
            origin: OriginFor<T>,
            proof: MediatorAffirmationProof,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_mediator_affirmation(proof)
        }

        /// Sender updates their counter after a settlement has been executed.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The sender update proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotExecuted` if the settlement has not been executed.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::sender_update_counter_with_leaf())]
        pub fn sender_update_counter(
            origin: OriginFor<T>,
            proof: SenderCounterUpdateProof,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_sender_update_counter(proof)
        }

        /// Sender reverts their affirmation after a settlement has been rejected.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The sender revert proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotRejected` if the settlement has not been rejected.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::sender_revert_with_leaf())]
        pub fn sender_revert(origin: OriginFor<T>, proof: SenderReversalProof) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_sender_revert(proof)
        }

        /// Receiver claims their assets after a settlement has been executed.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The receiver claim proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotExecuted` if the settlement has not been executed.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::receiver_claim_with_leaf())]
        pub fn receiver_claim(origin: OriginFor<T>, proof: ReceiverClaimProof) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_receiver_claim(proof)
        }

        /// Create a settlement with batched leg affirmations.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The settlement proof with batched leg affirmations.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `InvalidProof` if the proof is invalid.
        /// * `SettlementMissingLegs` if the settlement has no legs.
        /// * `SettlementTooManyLegs` if the settlement has more legs than the maximum allowed.
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::batched_settlement(proof.count_leg_affirmations()))]
        pub fn batched_settlement(
            origin: OriginFor<T>,
            proof: BatchedSettlementProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_batched_settlement(proof)
        }

        /// Batch register multiple fee accounts.
        ///
        /// This is used to register fee accounts for Confidential private fee payments.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The batched fee account registration proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `FeeAccountAlreadyRegistered` if the fee account or encryption key is already registered.
        /// * `NullifierAlreadyUsed` if the nullifier for the fee account state commitment has already been used.
        /// * `InvalidFeeAssetId` if the fee asset ID is invalid.
        /// * `InvalidProof` if the proof is invalid.
        /// * `InsufficientBalance` if the caller has insufficient balance to pay the deposit.
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::register_fee_accounts_with_leaf(proof.len() as u32))]
        pub fn register_fee_accounts(
            origin: OriginFor<T>,
            proof: BatchedFeeAccountRegistrationProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            let pallet_identity::PermissionedCallOriginData {
                sender,
                primary_did: caller_did,
                ..
            } = PalletIdentity::<T>::ensure_origin_call_permissions(origin)?;

            // Deposit the total POLYX amount into the fee pool.
            let amount = Self::amount_to_balance(proof.total_amount(FEE_ASSET_ID))?;
            Self::fee_account_deposit(sender, amount)?;

            // Ensure the fee accounts haven't already been registered.
            let mut registrations = Vec::with_capacity(proof.proofs.len());
            for p in &proof.proofs {
                // Ensure the fee account doesn't exist.
                ensure!(
                    !FeeAccountDid::<T>::contains_key(&p.account),
                    Error::<T>::FeeAccountAlreadyRegistered
                );
                // Link the fee account to the caller's identity.
                FeeAccountDid::<T>::insert(&p.account, caller_did);

                // Ensure the fee asset id is valid.  Only one is supported now.
                ensure!(p.asset_id == FEE_ASSET_ID, Error::<T>::InvalidFeeAssetId);

                let amount = Self::amount_to_balance(p.amount)?;
                registrations.push((p.account, amount, p.account_state_commitment));
            }

            // Verify the proof.
            Self::submit_and_wait(VerifyDartAssetRequest::BatchedFeeAccountRegistration {
                did: caller_did.into(),
                proof,
            })?;

            // Process each registration.
            for (account, amount, account_state_commitment) in registrations {
                // Insert the new fee account state commitment from the proof into the fee account curve tree.
                Self::insert_fee_account_leaf(account_state_commitment, None)?;

                // Emit an event for the fee account registration.
                Self::deposit_event(Event::<T>::FeeAccountUpdated {
                    caller_did,
                    account,
                    is_registration: true,
                    amount,
                });
            }

            Ok(())
        }

        /// Toup a batch of fee accounts.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The batched fee account topup proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `FeeAccountMissing` if the fee account is not registered.
        /// * `InvalidFeeAssetId` if the fee asset ID is invalid.
        /// * `InvalidProof` if the proof is invalid.
        /// * `InsufficientBalance` if the caller has insufficient balance to pay the deposit.
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::topup_fee_accounts_with_leaf(proof.len() as u32))]
        pub fn topup_fee_accounts(
            origin: OriginFor<T>,
            proof: BatchedFeeAccountTopupProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            let pallet_identity::PermissionedCallOriginData {
                sender,
                primary_did: caller_did,
                ..
            } = PalletIdentity::<T>::ensure_origin_call_permissions(origin)?;

            // Deposit the total POLYX amount into the fee pool.
            let amount = Self::amount_to_balance(proof.total_amount(FEE_ASSET_ID))?;
            Self::fee_account_deposit(sender, amount)?;

            let mut seen_nullifier = BTreeSet::new();
            let mut topups = Vec::with_capacity(proof.proofs.len());
            for p in &proof.proofs {
                // Ensure the fee account exists.
                ensure!(
                    FeeAccountDid::<T>::contains_key(&p.account),
                    Error::<T>::FeeAccountMissing
                );
                // Ensure the fee asset id is valid.  Only one is supported now.
                ensure!(p.asset_id == FEE_ASSET_ID, Error::<T>::InvalidFeeAssetId);

                // Ensure the nullifier is unique.
                if seen_nullifier.contains(&p.nullifier) {
                    return Err(Error::<T>::NullifierAlreadyUsed.into());
                } else {
                    seen_nullifier.insert(p.nullifier);
                }
                // Ensure the nullifier is unique in storage.
                Self::ensure_fee_account_state_nullifier_unique(&p.nullifier)?;

                let amount = Self::amount_to_balance(p.amount)?;
                topups.push((
                    p.account,
                    amount,
                    p.updated_account_state_commitment,
                    p.nullifier,
                ));
            }

            // Get the root block and curve tree root.
            let root_block: BlockNumberFor<T> = proof.root_block.into();
            let root = FeeAccountCurveTreeRoots::<T>::get(root_block)
                .ok_or(Error::<T>::CurveTreeRootNotFound)?;

            // Verify the proof.
            Self::submit_and_wait(VerifyDartAssetRequest::BatchedFeeAccountTopup {
                did: caller_did.into(),
                proof,
                root,
            })?;

            // Process each topup.
            for (account, amount, account_state_commitment, nullifier) in topups {
                // Insert the new fee account state commitment from the proof into the fee account curve tree.
                Self::insert_fee_account_leaf(account_state_commitment, Some(nullifier))?;

                // Emit an event for the fee account topup.
                Self::deposit_event(Event::<T>::FeeAccountUpdated {
                    caller_did,
                    account,
                    is_registration: false,
                    amount,
                });
            }

            Ok(())
        }

        /// Submit a batch of proofs.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The batched proofs.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `InvalidProof` if any of the proofs are invalid.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::batched_proofs(&proof))]
        pub fn submit_batched_proofs(
            origin: OriginFor<T>,
            proof: BatchedProofs<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::process_batched_proofs(proof)
        }

        /// Relayer submit a batch of proofs paid using a private fee payment.
        ///
        /// Users can use a Relayer service to submit their Confidential proofs for privacy (i.e., the origin is not the user).
        /// The Relayer is paid/reimbursed using a private fee payment from the user's Confidential fee account.
        ///
        /// Relayers can charge a commission fee on top of the transaction fee (i.e. `commission fee + transaction fee = fee amount`).
        ///
        /// Relayers should verify that the fee payment proof is valid before submitting the batched Confidential proofs.  They are not required
        /// to verify the batched Confidential proofs.  If the user's Confidential proofs are invalid, the user is still responsible for paying the fee to the relayer.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.  This is the relayer.
        /// * `proof` - The fee payment proof and batched Confidential proofs.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `InvalidFeePaymentProof` if the fee payment proof is invalid.
        /// * `InsufficientFeePayment` if the fee payment is insufficient to cover the relayer fee.
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::relayer_submit_batched_proofs(&proof))]
        pub fn relayer_submit_batched_proofs(
            origin: OriginFor<T>,
            proof: FeePaymentWithBatchedProofs<PolymeshPrivateLimits>,
        ) -> DispatchResultWithPostInfo {
            let relayer = ensure_signed(origin)?;

            Self::base_relayer_submit_batched_proofs(relayer, proof)
        }

        /// Register encryption keys for auditors/mediators.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The auditor/mediator encryption registration proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `EncryptionKeyAlreadyRegistered` if the encryption key is already registered.
        /// * `InvalidProof` if the proof is invalid.
        #[pallet::call_index(16)]
        #[pallet::weight(<T as Config>::WeightInfo::register_encryption_keys(proof.len() as u32))]
        pub fn register_encryption_keys(
            origin: OriginFor<T>,
            proof: EncryptionKeyRegistrationProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            let caller_did = PalletIdentity::<T>::ensure_perms(origin)?;

            for encryption_key in &proof.keys {
                // Ensure the encryption key doesn't exist.
                ensure!(
                    !EncryptionKeyDid::<T>::contains_key(&encryption_key),
                    Error::<T>::EncryptionKeyAlreadyRegistered
                );
                EncryptionKeyDid::<T>::insert(&encryption_key, caller_did);

                Self::deposit_event(Event::<T>::EncryptionKeyRegistered {
                    caller_did,
                    encryption_key: *encryption_key,
                });
            }

            // Verify the proof.
            Self::submit_and_wait(VerifyDartAssetRequest::EncryptionKeyRegistration {
                did: caller_did.into(),
                proof,
            })?;

            Ok(())
        }

        /// Create and execute an instant settlement.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The settlement proof with batched leg affirmations.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `InvalidProof` if the proof is invalid.
        /// * `SettlementMissingLegs` if the settlement has no legs.
        /// * `SettlementTooManyLegs` if the settlement has more legs than the maximum allowed.
        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::execute_instant_settlement(proof.count_leg_affirmations()))]
        pub fn execute_instant_settlement(
            origin: OriginFor<T>,
            proof: InstantSettlementProof<PolymeshPrivateLimits>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::base_execute_instant_settlement(proof)
        }

        /// Sender affirms a settlement leg as the last pending affirmation.
        ///
        /// This can only be used when the sender affirmation is the last pending affirmation for the settlement.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The instant sender affirmation proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotPending` if the settlement is not pending.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        /// * `AlreadyAffirmed` if the leg has already been affirmed by the sender.
        /// * `NotLastPendingAffirmation` if the sender affirmation is not the last pending affirmation for the settlement.
        #[pallet::call_index(18)]
        #[pallet::weight(<T as Config>::WeightInfo::instant_sender_affirmation_with_leaf())]
        pub fn instant_sender_affirmation(
            origin: OriginFor<T>,
            proof: InstantSenderAffirmationProof,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            // Perform the base instant sender affirmation.
            Self::base_instant_sender_affirmation(proof, true)
        }

        /// Receiver affirms a settlement leg as the last pending affirmation.
        ///
        /// This can only be used when the receiver affirmation is the last pending affirmation for the settlement.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proof` - The instant receiver affirmation proof.
        ///
        /// # Errors
        /// * `BadOrigin` if `origin` isn't signed.
        /// * `SettlementNotPending` if the settlement is not pending.
        /// * `SettlementNotFound` if the settlement is not found.
        /// * `LegNotFound` if the leg is not found in the settlement.
        /// * `AlreadyAffirmed` if the leg has already been affirmed by the receiver.
        /// * `NotLastPendingAffirmation` if the receiver affirmation is not the last pending affirmation for the settlement.
        #[pallet::call_index(19)]
        #[pallet::weight(<T as Config>::WeightInfo::instant_receiver_affirmation_with_leaf())]
        pub fn instant_receiver_affirmation(
            origin: OriginFor<T>,
            proof: InstantReceiverAffirmationProof,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            // Perform the base instant receiver affirmation.
            Self::base_instant_receiver_affirmation(proof, true)
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Create a new Confidential asset.
    pub fn base_create_asset(
        owner_did: IdentityId,
        name: Name,
        symbol: Symbol,
        decimals: u8,
        mediators: MediatorKeys,
        auditors: AuditorKeys,
        data: BoundedVec<u8, T::MaxAssetDataLength>,
    ) -> Result<ConfidentialAssetId, DispatchError> {
        // Ensure the name is valid.
        ensure!(!name.is_empty(), Error::<T>::InvalidAssetName);
        ensure!(name.len() <= MAX_NAME_LEN, Error::<T>::NameTooLong);
        // Ensure the symbol is valid.
        ensure!(symbol.len() <= MAX_SYMBOL_LEN, Error::<T>::SymbolTooLong);

        // Ensure `decimals` is valid.
        ensure!(decimals <= MAX_DECIMALS, Error::<T>::TooManyDecimals);

        // Ensure the auditor or mediator is registered.
        Self::ensure_mediators_registered(&mediators)?;
        Self::ensure_auditors_registered(&auditors)?;

        // Allocate a new asset ID.
        let asset_id = NextAssetId::<T>::get();
        // Increment the next asset ID for the next call.
        NextAssetId::<T>::put(asset_id + 1);

        // Create the asset details.
        let asset_detail = AssetDetails {
            total_supply: 0,
            owner_did,
            data: data.clone(),
        };
        // Store the asset details.
        Details::<T>::insert(asset_id, asset_detail);

        // Store the asset name.
        Names::<T>::insert(asset_id, &name);
        // Store the asset symbol.
        Symbols::<T>::insert(asset_id, &symbol);
        // Store the asset decimals.
        Decimals::<T>::insert(asset_id, decimals);

        // Add the asset ID to the owner's list of assets.
        OwnerAssets::<T>::insert((owner_did, asset_id), ());

        // Insert the asset state into the asset curve tree.
        Self::update_asset_leaf(owner_did, asset_id, &mediators, &auditors, true)?;

        // Emit the event for asset creation.
        Self::deposit_event(Event::<T>::AssetCreated {
            caller_did: owner_did,
            asset_id,
            mediators,
            auditors,
            name,
            symbol,
            decimals,
            data,
        });

        Ok(asset_id)
    }

    pub fn base_create_settlement(proof: SettlementProof<PolymeshPrivateLimits>) -> DispatchResult {
        let settlement_ref = proof.settlement_ref();
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            // Ensure the settlement has at least one leg.
            ensure!(proof.legs.len() > 0, Error::<T>::SettlementMissingLegs);
        }

        // Ensure that the settlement does not exist.  This also prevents replay of the same settlement proof.
        ensure!(
            !SettlementState::<T>::contains_key(settlement_ref),
            Error::<T>::SettlementAlreadyExists
        );

        // Get details of the settlement.
        let memo = proof.memo.clone();
        // TODO: Only copy needed data from the legs (leg_enc and mediator count).
        let proof_legs = proof.legs.clone();
        let root_block: BlockNumberFor<T> = proof.root_block.into();

        // Verify the settlement proof.
        let root =
            AssetCurveTreeRoots::<T>::get(root_block).ok_or(Error::<T>::CurveTreeRootNotFound)?;
        Self::submit_and_wait(VerifyDartAssetRequest::CreateSettlement {
            root,
            asset_lookup: Default::default(),
            proof,
        })?;

        // Set the settlement state to pending.
        SettlementState::<T>::insert(settlement_ref, SettlementStatus::Pending);

        // Initialize all affirmation statuses to Pending.
        let mut pending_affirmations = 0;
        for (leg_idx, leg) in proof_legs.iter().enumerate() {
            let leg_idx = leg_idx as LegId;
            let mediators = leg.mediator_count().map_err(Error::<T>::from)? as u32;

            pending_affirmations += 2 + mediators; // Sender + Receiver + Mediators
            LegAffirmationStatus::<T>::insert(
                (settlement_ref, leg_idx, LegAffirmParty::Sender),
                AffirmationStatus::Pending,
            );
            LegAffirmationStatus::<T>::insert(
                (settlement_ref, leg_idx, LegAffirmParty::Receiver),
                AffirmationStatus::Pending,
            );
            for idx in 0..mediators {
                LegAffirmationStatus::<T>::insert(
                    (settlement_ref, leg_idx, LegAffirmParty::Mediator(idx as u8)),
                    AffirmationStatus::Pending,
                );
            }
        }
        // Store the number of pending affirmations for the settlement.
        SettlementPendingAffirmations::<T>::insert(settlement_ref, pending_affirmations);

        // Store the settlement memo.
        SettlementMemo::<T>::insert(settlement_ref, &memo);

        // Store the number of legs for the settlement.
        let leg_count = Compact(proof_legs.len() as u32);
        SettlementLegCount::<T>::insert(settlement_ref, leg_count);

        // Store the settlement legs.
        let mut legs = BoundedVec::new();
        for (leg_id, leg) in proof_legs.into_iter().enumerate() {
            let leg_id = leg_id as LegId;
            legs.force_push(leg.leg_enc().clone());

            SettlementLegs::<T>::insert((settlement_ref, leg_id), leg.leg_enc());
        }

        // Emit an event for the settlement creation.
        Self::deposit_event(Event::<T>::SettlementCreated {
            settlement_ref,
            asset_root_block: root_block,
            memo,
            legs,
        });

        Ok(())
    }

    pub fn base_batched_settlement(
        proof: BatchedSettlementProof<PolymeshPrivateLimits>,
    ) -> DispatchResult {
        // Ensure that the all the leg affirmations have the same settlement reference.
        ensure!(
            proof.check_leg_references(),
            Error::<T>::BatchedSettlementInvalidLegRefs
        );

        // Create the settlement with the provided proof.
        Self::base_create_settlement(proof.settlement)?;

        // Handle the leg affirmations in the proof.
        for leg_affirmation in proof.leg_affirmations {
            if let Some(sender_proof) = leg_affirmation.sender {
                Self::base_sender_affirmation(sender_proof)?;
            }
            if let Some(receiver_proof) = leg_affirmation.receiver {
                Self::base_receiver_affirmation(receiver_proof)?;
            }
        }

        Ok(())
    }

    pub fn base_execute_instant_settlement(
        proof: InstantSettlementProof<PolymeshPrivateLimits>,
    ) -> DispatchResult {
        // Ensure that the all the leg affirmations have the same settlement reference.
        ensure!(
            proof.check_leg_references(),
            Error::<T>::BatchedSettlementInvalidLegRefs
        );
        let settlement_ref = proof.settlement.settlement_ref();

        // Create the settlement with the provided proof.
        Self::base_create_settlement(proof.settlement)?;

        // Handle the leg affirmations in the proof.
        for leg_affirmation in proof.leg_affirmations {
            Self::base_instant_sender_affirmation(leg_affirmation.sender, false)?;
            Self::base_instant_receiver_affirmation(leg_affirmation.receiver, false)?;
            for mediator in leg_affirmation.mediators {
                Self::base_mediator_affirmation(mediator)?;
            }
        }

        // Ensure that the settlement has executed and finalized.
        Self::ensure_settlement_finalized(settlement_ref)?;

        Ok(())
    }

    pub fn base_sender_affirmation(proof: SenderAffirmationProof) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the sender affirmation proof and update the settlement and leg status.
            update_settlement.sender_affirmation(proof, root)
        })?;

        // Emit an event for the sender affirmation.
        Self::deposit_event(Event::<T>::SenderAffirmed { leg_ref });

        Ok(())
    }

    pub fn base_receiver_affirmation(proof: ReceiverAffirmationProof) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the receiver affirmation proof and update the settlement and leg status.
            update_settlement.receiver_affirmation(proof, root)
        })?;

        // Emit an event for the receiver affirmation.
        Self::deposit_event(Event::<T>::ReceiverAffirmed { leg_ref });

        Ok(())
    }

    pub fn base_instant_sender_affirmation(
        proof: InstantSenderAffirmationProof,
        normal_settlement: bool,
    ) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        let settlement_ref = leg_ref.settlement_ref();

        // If this is a normal settlement, ensure this is the last pending affirmation.
        if normal_settlement {
            // Ensure the sender affirmation is the last pending affirmation for the settlement.
            Self::ensure_last_pending_affirmation(settlement_ref, 1)?;
        }

        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the instant sender affirmation proof and update the settlement and leg status.
            update_settlement.instant_sender_affirmation(proof, root)
        })?;

        // If this is a normal settlement, ensure the settlement has executed.
        if normal_settlement {
            // Ensure the settlement executed after the instant sender affirmation.
            Self::ensure_settlement_executed(settlement_ref)?;
        }

        // Emit an event for the sender affirmation.
        Self::deposit_event(Event::<T>::SenderAffirmed { leg_ref });
        Self::deposit_event(Event::<T>::SenderCounterUpdated { leg_ref });

        Ok(())
    }

    pub fn base_instant_receiver_affirmation(
        proof: InstantReceiverAffirmationProof,
        normal_settlement: bool,
    ) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        let settlement_ref = leg_ref.settlement_ref();

        // For normal settlements, ensure this is the last pending affirmation.
        if normal_settlement {
            // Ensure the receiver affirmation is the last pending affirmation for the settlement.
            Self::ensure_last_pending_affirmation(settlement_ref, 1)?;
        }

        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the instant receiver affirmation proof and update the settlement and leg status.
            update_settlement.instant_receiver_affirmation(proof, root)
        })?;

        // For normal settlements, ensure the settlement has executed.
        if normal_settlement {
            // Ensure the settlement executed after the instant receiver affirmation.
            Self::ensure_settlement_executed(settlement_ref)?;
        }

        // Emit an event for the receiver affirmation.
        Self::deposit_event(Event::<T>::ReceiverAffirmed { leg_ref });
        Self::deposit_event(Event::<T>::ReceiverClaimed { leg_ref });

        Ok(())
    }

    /// Ensure that the settlement has the expected number of pending affirmations.
    pub fn ensure_last_pending_affirmation(
        settlement_ref: SettlementRef,
        expected_pending: u32,
    ) -> Result<(), DispatchError> {
        let pending_affirmations = SettlementPendingAffirmations::<T>::get(settlement_ref);
        ensure!(
            pending_affirmations == expected_pending,
            Error::<T>::NotLastPendingAffirmation
        );
        Ok(())
    }

    /// Ensure that the settlement has been finalized.
    pub fn ensure_settlement_finalized(settlement_ref: SettlementRef) -> Result<(), DispatchError> {
        let settlement_status =
            SettlementState::<T>::get(settlement_ref).ok_or(Error::<T>::SettlementNotFound)?;
        ensure!(
            settlement_status == SettlementStatus::Finalized,
            Error::<T>::SettlementNotExecuted
        );
        Ok(())
    }

    /// Ensure that the settlement has been executed.
    pub fn ensure_settlement_executed(settlement_ref: SettlementRef) -> Result<(), DispatchError> {
        let settlement_status =
            SettlementState::<T>::get(settlement_ref).ok_or(Error::<T>::SettlementNotFound)?;
        ensure!(
            settlement_status == SettlementStatus::Executed,
            Error::<T>::SettlementNotExecuted
        );
        Ok(())
    }

    pub fn base_mediator_affirmation(proof: MediatorAffirmationProof) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        let accept = proof.accept;
        let key_index = proof.key_index;
        // Create an update settlement status instance.
        let update_settlement = UpdateSettlementStatus::<T>::new(leg_ref)?;

        // Verify the mediator affirmation proof and update the settlement and leg status.
        update_settlement.mediator_affirmation(proof)?;

        // Emit an event for the mediator affirmation.
        if accept {
            Self::deposit_event(Event::<T>::MediatorAffirmed { leg_ref, key_index });
        } else {
            Self::deposit_event(Event::<T>::MediatorRejected { leg_ref, key_index });
        }

        Ok(())
    }

    pub fn base_sender_update_counter(proof: SenderCounterUpdateProof) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the sender update proof and update the settlement and leg status.
            update_settlement.sender_counter_update(proof, root)
        })?;

        // Emit an event for the sender update counter.
        Self::deposit_event(Event::<T>::SenderCounterUpdated { leg_ref });

        Ok(())
    }

    pub fn base_sender_revert(proof: SenderReversalProof) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the sender reversal proof and update the settlement and leg status.
            update_settlement.sender_reversal(proof, root)
        })?;

        // Emit an event for the sender revert.
        Self::deposit_event(Event::<T>::SenderReverted { leg_ref });

        Ok(())
    }

    pub fn base_receiver_claim(proof: ReceiverClaimProof) -> DispatchResult {
        let leg_ref = proof.leg_ref;
        // Handle the account state update proof and verify the nullifier.
        Self::handle_account_state_update_proof(proof, |proof, root| {
            // Create an update settlement status instance.
            let update_settlement = UpdateSettlementStatus::<T>::new(proof.leg_ref)?;

            // Verify the receiver claim proof and update the settlement and leg status.
            update_settlement.receiver_claim(proof, root)
        })?;

        // Emit an event for the receiver claim.
        Self::deposit_event(Event::<T>::ReceiverClaimed { leg_ref });

        Ok(())
    }

    pub fn base_relayer_submit_batched_proofs(
        relayer: T::AccountId,
        proof: FeePaymentWithBatchedProofs<PolymeshPrivateLimits>,
    ) -> DispatchResultWithPostInfo {
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            // Ensure the batched proofs is not empty.
            ensure!(
                !proof.batched_proofs.proofs.is_empty(),
                Error::<T>::EmptyBatchedProofs
            );
        }

        // Calculate the batch weight and corresponding tx fee.
        let batch_weight = <T as Config>::WeightInfo::relayer_submit_batched_proofs(&proof);
        let batch_tx_fee = T::WeightToFee::weight_to_fee(&batch_weight);

        // Verify the fee payment proof.
        let batch_hash = proof.fee_payment_ctx();
        let verify_res =
            Self::verify_fee_payment(relayer.clone(), batch_tx_fee, batch_hash, proof.fee_payment);

        // If the fee payment verification fails, return an error but still charge the relayer for the verification cost.
        let amount = match verify_res {
            Ok(amount) => amount,
            Err(error) => {
                // Only charge for the fee payment verification cost.
                return Err(DispatchErrorWithPostInfo {
                    post_info: Some(<T as Config>::WeightInfo::verify_fee_payment_with_leaf())
                        .into(),
                    error,
                });
            }
        };

        // Process the batched Confidential proofs.  The proofs are processed inside of a transaction so that
        // if any proof fails, the entire batch is reverted (but the relayer is still paid).
        Self::process_batched_proofs_atomic(proof.batched_proofs)?;

        // Emit an event for the relayer batched proofs submission.
        Self::deposit_event(Event::<T>::RelayerBatchedProofs {
            relayer,
            batch_hash,
            amount,
            batch_result: Ok(()),
        });

        Ok(().into())
    }

    pub fn verify_fee_payment(
        relayer: T::AccountId,
        batch_tx_fee: BalanceOf<T>,
        batch_hash: ProofHash,
        proof: FeeAccountPaymentProof,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let account_state_commitment = proof.updated_account_state_commitment;
        let nullifier = proof.nullifier;
        let amount = Self::amount_to_balance(proof.amount)?;

        // TODO: Put a cap on the maximum commission fee that a relayer can charge.
        ensure!(
            amount >= batch_tx_fee,
            Error::<T>::InsufficientFeePaymentAmount
        );

        // Ensure the fee asset id is valid.  Only one is supported now.
        ensure!(
            proof.asset_id == FEE_ASSET_ID,
            Error::<T>::InvalidFeeAssetId
        );

        // Ensure the nullifier is unique in storage.
        Self::ensure_fee_account_state_nullifier_unique(&nullifier)?;

        // Get the root block and curve tree root.
        let root_block: BlockNumberFor<T> = proof.root_block.into();
        let root = FeeAccountCurveTreeRoots::<T>::get(root_block)
            .ok_or(Error::<T>::CurveTreeRootNotFound)?;

        // Verify the proof.
        Self::submit_and_wait(VerifyDartAssetRequest::FeeAccountPayment {
            ctx: batch_hash,
            root,
            proof,
        })
        .map_err(|_| Error::<T>::InvalidFeePaymentProof)?;

        // Insert the new fee account state commitment from the proof into the fee account curve tree.
        Self::insert_fee_account_leaf(account_state_commitment, Some(nullifier))?;

        // Withdraw the fee amount from the fee pool.
        Self::fee_account_withdraw(relayer.clone(), amount)?;

        Ok(amount)
    }

    /// Process a batch of proofs inside a transaction.
    pub fn process_batched_proofs_atomic(
        proof: BatchedProofs<PolymeshPrivateLimits>,
    ) -> DispatchResult {
        use frame_support::storage::TransactionOutcome;
        frame_support::storage::with_transaction(|| {
            let res = Self::process_batched_proofs(proof);
            if res.is_ok() {
                TransactionOutcome::Commit(res)
            } else {
                TransactionOutcome::Rollback(res)
            }
        })
    }

    /// Process a batch of proofs, this should be called inside a transaction.
    pub fn process_batched_proofs(proof: BatchedProofs<PolymeshPrivateLimits>) -> DispatchResult {
        for proof in proof.proofs {
            match proof {
                BatchedProof::CreateSettlement(p) => Self::base_create_settlement(p)?,
                BatchedProof::SenderAffirmation(p) => Self::base_sender_affirmation(p)?,
                BatchedProof::ReceiverAffirmation(p) => Self::base_receiver_affirmation(p)?,
                BatchedProof::MediatorAffirmation(p) => Self::base_mediator_affirmation(p)?,
                BatchedProof::SenderCounterUpdate(p) => Self::base_sender_update_counter(p)?,
                BatchedProof::SenderReversal(p) => Self::base_sender_revert(p)?,
                BatchedProof::ReceiverClaim(p) => Self::base_receiver_claim(p)?,
                BatchedProof::ExecuteInstantSettlement(p) => {
                    Self::base_execute_instant_settlement(p)?
                }
                BatchedProof::InstantSenderAffirmation(p) => {
                    Self::base_instant_sender_affirmation(p, true)?
                }
                BatchedProof::InstantReceiverAffirmation(p) => {
                    Self::base_instant_receiver_affirmation(p, true)?
                }
            }
        }
        Ok(())
    }

    /// Handle account state update proof verification and insertion.
    pub fn handle_account_state_update_proof<P: AccountStateUpdate>(
        proof: P,
        verify: impl FnOnce(P, AccountTreeRoot) -> DispatchResult,
    ) -> DispatchResult {
        let nullifier = proof.nullifier();
        let account_commitment = proof.account_state_commitment();
        // Ensure the nullifier is unique.
        Self::ensure_account_state_nullifier_unique(&nullifier)?;

        // Verify the account state update proof.
        let root_block: BlockNumberFor<T> = proof.root_block().into();
        let root =
            AccountCurveTreeRoots::<T>::get(root_block).ok_or(Error::<T>::CurveTreeRootNotFound)?;
        verify(proof, root)?;

        // Insert the update account state commitment into the account curve tree.
        Self::insert_account_leaf(account_commitment, Some(nullifier))?;

        Ok(())
    }

    /// Get the asset curve tree interface.
    pub fn get_asset_curve_tree() -> Result<AssetCurveTree<T>, Error<T>> {
        Ok(AssetCurveTree::<T>::new(ASSET_TREE_HEIGHT)?)
    }

    /// Get the account curve tree interface.
    pub fn get_account_curve_tree() -> Result<AccountCurveTree<T>, Error<T>> {
        Ok(AccountCurveTree::<T>::new(ACCOUNT_TREE_HEIGHT)?)
    }

    /// Get the fee account curve tree interface.
    pub fn get_fee_account_curve_tree() -> Result<FeeAccountCurveTree<T>, Error<T>> {
        Ok(FeeAccountCurveTree::<T>::new(FEE_ACCOUNT_TREE_HEIGHT)?)
    }

    /// Update the asset leaf in the asset curve tree.
    fn update_asset_leaf(
        caller_did: IdentityId,
        asset_id: ConfidentialAssetId,
        mediators: &MediatorKeys,
        auditors: &AuditorKeys,
        is_create: bool,
    ) -> DispatchResult {
        // Require at least one auditor/mediator.
        ensure!(
            (mediators.len() + auditors.len()) > 0,
            Error::<T>::NoAuditorsOrMediators
        );

        // Create the Asset keys.
        let keys = AssetKeys::new_bounded::<PolymeshPrivateLimits>(mediators, auditors)
            .map_err(|_| Error::<T>::AssetStateInvalid)?;
        Keys::<T>::insert(asset_id, &keys);

        // Create the Asset State.
        let asset_state = AssetState { asset_id, keys };
        let req = UpdateAssetStateRequest::new(asset_state);
        let resp = req.update().map_err(|_| Error::<T>::AssetStateInvalid)?;
        let asset_leaf = resp.asset_leaf();

        // Update the asset curve tree with the new asset.
        let mut asset_curve_tree = Self::get_asset_curve_tree()?;
        let leaf_index = asset_id.into();
        asset_curve_tree.update_leaf(leaf_index, asset_leaf)?;

        // Emit an event for the asset state update.
        if !is_create {
            // Only emit the AssetUpdated event for updates, not for the initial creation.
            Self::deposit_event(Event::<T>::AssetUpdated {
                caller_did,
                asset_id,
                auditors: auditors.clone(),
                mediators: mediators.clone(),
            });
        }
        Self::deposit_event(Event::<T>::AssetStateLeafUpdated {
            leaf_index,
            asset_leaf,
        });

        Ok(())
    }

    /// Insert a new account state commitment into the account curve tree.
    fn insert_account_leaf(
        account_commitment: AccountStateCommitment,
        nullifier: Option<AccountStateNullifier>,
    ) -> Result<(), Error<T>> {
        if let Some(nullifier) = nullifier {
            // Burn the nullifier for the old account commitment to ensure it cannot be used again.
            AccountStateCommitmentNullifiers::<T>::try_mutate(nullifier, |maybe_val| {
                if maybe_val.is_some() {
                    return Err(Error::<T>::NullifierAlreadyUsed);
                }
                *maybe_val = Some(());
                Ok(())
            })?;
        }

        // Insert the new account leaf.
        let leaf_index = Self::next_account_leaf_index();
        AccountLeaves::<T>::set(leaf_index, Some(account_commitment));

        // Emit an event for the account curve tree update.
        Self::deposit_event(Event::<T>::AccountStateLeafInserted {
            leaf_index,
            account_commitment,
        });

        Ok(())
    }

    /// Insert a new fee account state commitment into the fee account curve tree.
    fn insert_fee_account_leaf(
        fee_account_commitment: FeeAccountStateCommitment,
        nullifier: Option<FeeAccountStateNullifier>,
    ) -> Result<(), Error<T>> {
        if let Some(nullifier) = nullifier {
            // Burn the nullifier for the old fee account commitment to ensure it cannot be used again.
            FeeAccountStateCommitmentNullifiers::<T>::try_mutate(nullifier, |maybe_val| {
                if maybe_val.is_some() {
                    return Err(Error::<T>::NullifierAlreadyUsed);
                }
                *maybe_val = Some(());
                Ok(())
            })?;
        }

        // Insert the new fee account leaf.
        let leaf_index = Self::next_fee_account_leaf_index();
        FeeAccountLeaves::<T>::set(leaf_index, Some(fee_account_commitment));

        // Emit an event for the fee account curve tree update.
        Self::deposit_event(Event::<T>::FeeAccountStateLeafInserted {
            leaf_index,
            fee_account_commitment,
        });
        Ok(())
    }

    /// Get the next account leaf index.
    pub fn next_account_leaf_index() -> LeafIndex {
        let leaf_index = NextAccountLeafIndex::<T>::get();
        // Increment the next leaf index for the next call.
        NextAccountLeafIndex::<T>::put(leaf_index + 1);
        leaf_index
    }

    /// Get the next fee account leaf index.
    pub fn next_fee_account_leaf_index() -> LeafIndex {
        let leaf_index = NextFeeAccountLeafIndex::<T>::get();
        // Increment the next leaf index for the next call.
        NextFeeAccountLeafIndex::<T>::put(leaf_index + 1);
        leaf_index
    }

    /// Ensure account state nullifier is unique.
    pub fn ensure_account_state_nullifier_unique(
        nullifier: &AccountStateNullifier,
    ) -> Result<(), Error<T>> {
        // Ensure the nullifier is not already used.
        ensure!(
            !AccountStateCommitmentNullifiers::<T>::contains_key(nullifier),
            Error::<T>::NullifierAlreadyUsed
        );
        Ok(())
    }

    /// Ensure fee account state nullifier is unique.
    pub fn ensure_fee_account_state_nullifier_unique(
        nullifier: &FeeAccountStateNullifier,
    ) -> Result<(), Error<T>> {
        // Ensure the nullifier is not already used.
        ensure!(
            !FeeAccountStateCommitmentNullifiers::<T>::contains_key(nullifier),
            Error::<T>::NullifierAlreadyUsed
        );
        Ok(())
    }

    /// Ensure Confidential account is registered.
    pub fn ensure_dart_account_registered(
        account: &AccountPublicKey,
    ) -> Result<IdentityId, Error<T>> {
        AccountDid::<T>::get(account).ok_or(Error::<T>::AccountMissing)
    }

    /// Ensure the caller's identity is the owner of the Confidential account.
    pub fn ensure_dart_account_owner(
        caller_did: IdentityId,
        account: &AccountPublicKey,
    ) -> Result<(), DispatchError> {
        let owner_did = Self::ensure_dart_account_registered(account)?;
        ensure!(owner_did == caller_did, Error::<T>::NotAccountOwner);
        Ok(())
    }

    /// Ensure the caller's identity is the owner of the Confidential asset.
    pub fn ensure_dart_asset_owner(
        caller_did: IdentityId,
        asset_id: ConfidentialAssetId,
    ) -> Result<AssetDetails<T>, DispatchError> {
        // Ensure the Confidential asset exists.
        let asset_detail = Self::ensure_dart_asset_exists(asset_id)?;
        // Ensure the caller's identity is the owner of the Confidential asset.
        ensure!(
            asset_detail.owner_did == caller_did,
            Error::<T>::NotAssetOwner
        );
        Ok(asset_detail)
    }

    /// Ensure the caller has the required permissions to access the Confidential account.
    pub fn ensure_dart_account_permissions(
        origin: OriginFor<T>,
        account: &AccountPublicKey,
    ) -> Result<IdentityId, DispatchError> {
        // Ensure the caller is allowed to make this call.
        let caller_did = PalletIdentity::<T>::ensure_perms(origin)?;
        // Ensure the Confidential account is registered to the caller's identity.
        Self::ensure_dart_account_owner(caller_did, account)?;
        // Return the caller's identity.
        Ok(caller_did)
    }

    /// Ensure that the Confidential asset exists.
    pub fn ensure_dart_asset_exists(
        asset_id: ConfidentialAssetId,
    ) -> Result<AssetDetails<T>, Error<T>> {
        Details::<T>::get(asset_id).ok_or(Error::<T>::AssetMissing)
    }

    /// Ensure encryption key is registered.
    pub fn ensure_encryption_key_registered(
        encryption_key: &EncryptionPublicKey,
    ) -> Result<IdentityId, Error<T>> {
        EncryptionKeyDid::<T>::get(encryption_key).ok_or(Error::<T>::EncryptionKeyMissing)
    }

    /// Ensure auditor encryption public keys are registered.
    pub fn ensure_auditors_registered(keys: &AuditorKeys) -> Result<(), Error<T>> {
        for key in keys {
            Self::ensure_encryption_key_registered(key)?;
        }
        Ok(())
    }

    /// Ensure mediator encryption public keys are registered.
    pub fn ensure_mediators_registered(keys: &MediatorKeys) -> Result<(), Error<T>> {
        for key in keys {
            Self::ensure_dart_account_registered(&key.0)?;
        }
        Ok(())
    }

    pub fn update_account_curve_tree_root() {
        // Get the account curve tree.
        let mut tree =
            Self::get_account_curve_tree().expect("Account curve tree should be initialized; qed");

        // Commit the delayed updates to the account curve tree.
        tree.commit_leaves_to_tree()
            .expect("Account curve tree should be able to commit leaves; qed");
    }

    pub fn update_fee_account_curve_tree_root() {
        // Get the fee account curve tree.
        let mut tree = Self::get_fee_account_curve_tree()
            .expect("Fee Account curve tree should be initialized; qed");

        // Commit the delayed updates to the account curve tree.
        tree.commit_leaves_to_tree()
            .expect("Fee Account curve tree should be able to commit leaves; qed");
    }

    pub fn init_block() -> Weight {
        // Start worker session.
        let backends = BackendKind::all_mask();
        let session_id =
            native_polymesh_worker::start_session(0, backends, DART_PROTOCOL.to_number());

        CurrentWorkerSessionId::<T>::put(session_id);

        // TODO: add missing writes to weight.
        <T as Config>::WeightInfo::on_init()
    }

    pub fn finalize_block() {
        Self::update_account_curve_tree_root();
        Self::update_fee_account_curve_tree_root();

        // Close the batch.
        if let Some(session_id) = CurrentWorkerSessionId::<T>::take() {
            native_polymesh_worker::end_session(session_id);
        }
    }

    pub fn submit_and_wait(req: VerifyDartAssetRequest) -> DispatchResult {
        req.submit_and_wait(Self::session_id()?)
            .map_err(|_| Error::<T>::InvalidProof)?;
        Ok(())
    }

    pub fn session_id() -> Result<WorkerSessionId, DispatchError> {
        CurrentWorkerSessionId::<T>::get().ok_or(Error::<T>::NoCurrentWorkerSession.into())
    }

    /// Transfer funds to the fee account.
    pub fn fee_account_deposit(
        sender: T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        // Transfer the amount to the fee account.
        T::Currency::transfer(&sender, &Self::fee_account_id(), amount, Expendable)?;

        // Emit an event for the fee account deposit.
        Self::deposit_event(Event::<T>::FeeAccountDeposited { sender, amount });

        Ok(amount)
    }

    /// Withdraw funds from the fee account.
    pub fn fee_account_withdraw(
        receiver: T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        // Transfer the amount from the fee account.
        T::Currency::transfer(&Self::fee_account_id(), &receiver, amount, Expendable)?;

        // Emit an event for the fee account withdrawal.
        Self::deposit_event(Event::<T>::FeeAccountWithdrawn { receiver, amount });

        Ok(())
    }

    pub fn amount_to_balance(amount: u64) -> Result<BalanceOf<T>, DispatchError> {
        amount
            .try_into()
            .map_err(|_| DispatchError::from(Error::<T>::AmountOverflow))
    }
}
