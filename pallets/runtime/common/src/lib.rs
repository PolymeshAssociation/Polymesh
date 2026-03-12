// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

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
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "1024"]

pub mod fee_details;
pub mod impls;
pub mod migration;
pub mod runtime;

use frame_election_provider_support::bounds::{ElectionBounds, ElectionBoundsBuilder};
pub use frame_support::dispatch::{DispatchClass, GetDispatchInfo};
use frame_support::pallet_prelude::One;
pub use frame_support::parameter_types;
pub use frame_support::traits::{Currency, Get};
pub use frame_support::weights::constants::{
    WEIGHT_REF_TIME_PER_MICROS, WEIGHT_REF_TIME_PER_MILLIS, WEIGHT_REF_TIME_PER_NANOS,
    WEIGHT_REF_TIME_PER_SECOND,
};
pub use frame_support::weights::{
    RuntimeDbWeight, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::Multiplier;
pub use sp_runtime::transaction_validity::TransactionPriority;
pub use sp_runtime::{FixedU128, Perbill, Percent, Permill, SaturatedConversion, Saturating};

pub use impls::Author;
use polymesh_primitives::constants::currency::*;
pub use polymesh_primitives::RocksDbWeight;
use polymesh_primitives::{Balance, BlockNumber, IdentityId, Moment};

pub const fn deposit(items: u32, bytes: u32) -> Balance {
    items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
const MINER_MAX_ITERATIONS: u32 = 10;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 4096;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    ///
    /// If this is updated, `PipsEnactSnapshotMaximumWeight` needs to be updated accordingly.
    pub const MaximumBlockWeight: Weight = MAXIMUM_BLOCK_WEIGHT;
    /// Portion of the block available to normal class of dispatches.
    ///
    /// If this is updated, `PipsEnactSnapshotMaximumWeight` needs to be updated accordingly.
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Blocks can be of upto 10 MB in size.
    pub const MaximumBlockLength: u32 = 10 * 1024 * 1024;
    /// 20 ms is needed to create a block.
    pub const BlockExecutionWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_MILLIS.saturating_mul(20), 0);
    /// 0.65 ms is needed to process an empty extrinsic.
    pub const ExtrinsicBaseWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_MICROS.saturating_mul(650), 0);
    /// This implies a 100 POLYX fee per MB of transaction length
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
    /// We want the noop transaction to cost 0.03 POLYX
    pub const PolyXBaseFee: Balance = 3 * CENTS;
    /// The multiplier for operational fees.
    pub const OperationalFeeMultiplier: u8 = 5;
    /// The fee multiplier to use for adjusting fees.
    pub FeeMultiplier: Multiplier = Multiplier::one();
    /// The maximum weight of the pips extrinsic `enact_snapshot_results` which equals to
    /// `MaximumBlockWeight * AvailableBlockRatio`.
    pub const PipsEnactSnapshotMaximumWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_mul(75).saturating_div(100);
    /// Number of block delay an extrinsic claim surcharge has.
    pub const SignedClaimHandicap: u32 = 2;
    /// The balance every contract needs to deposit to stay alive indefinitely.
    pub const DepositPerContract: Balance = 10 * CENTS;
    /// The balance a contract needs to deposit per storage item to stay alive indefinitely.
    pub const DepositPerItem: Balance = deposit(1, 0);
    /// The balance a contract needs to deposit per child trie item to stay alive indefinitely.
    pub const DepositPerChildTrieItem: Balance = deposit(1, 0) / 100;
    /// The balance a contract needs to deposit per storage byte to stay alive indefinitely.
    pub const DepositPerByte: Balance = deposit(0, 1);
    /// The code hash lookup deposit.
    pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
    /// The default deposit limit.
    pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
    /// The maximum ratio of eth extrinsic weight to block weight.
    pub const MaxEthExtrinsicWeight: FixedU128 = FixedU128::from_rational(9, 10);
    /// The maximum nesting level of a call/instantiate stack.
    pub const ContractsMaxDepth: u32 = 32;
    /// The maximum size of a storage value and event payload in bytes.
    pub const ContractsMaxValueSize: u32 = 16 * 1024;
    /// Max length of (instrumented) contract code in bytes.
    pub const ContractsMaxCodeSize: u32 = 100 * 1024;

    pub RuntimeBlockLength: BlockLength = BlockLength::builder()
        .max_length(10 * 1024 * 1024)
        .modify_max_length_for_class(DispatchClass::Normal, |m| *m = NORMAL_DISPATCH_RATIO * *m)
        .build();

    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
    .for_class(DispatchClass::Normal, |weights| {
        weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
    })
    .for_class(DispatchClass::Operational, |weights| {
        weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
        // Operational transactions have some extra reserved space, so that they
        // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
        weights.reserved = Some(
            MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
    })
    .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();

    pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
        .saturating_sub(BlockExecutionWeight::get());

    // Staking constants
    pub MaxExposurePageSize: u32 = 64;
    pub MaxNominations: u32 = 16;
    pub HistoryDepth:u32 = 84;
    pub MaxUnlockingChunks: u32 = 32;
    pub MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    pub MaxControllersInDeprecationBatch: u32 = 100;
    pub MaxTransientStorageSize: u32 = 1_048_576; // 1 MiB

    // Multi-phase election parameters
    // Signed phase
    pub const SignedPhase: u32 = 0;
    pub const SignedPhaseBench: u32 = 1_200;
    pub const SignedMaxSubmissions: u32 = 0;
    pub const SignedMaxWeight: Weight = Weight::zero();
    pub const SignedMaxRefunds: u32 = 0;
    pub const SignedRewardBase: Balance = 0;
    pub const SignedFixedDeposit: Balance = 0;
    pub const SignedDepositIncreaseFactor: Percent = Percent::from_percent(0);
    pub const SignedDepositByte: Balance = 0;
    pub const MultiPhaseUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2 - 1u64;
    // Fallback parameters
    pub MaxOnChainElectingVoters: u32 = 5000;
    pub MaxOnChainElectableTargets: u16 = 1250;
    // Other config parameters
    pub OffChainRepeat: BlockNumber = 5;
    pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
        .voters_count(10_000.into()).targets_count(1_500.into()).build();
    pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
        .voters_count(5_000.into()).targets_count(1_250.into()).build();
    pub MaxActiveValidators: u32 = 1_000;
    // Miner Config parameters
    pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
        *RuntimeBlockLength::get()
        .max
        .get(DispatchClass::Normal);
    pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
        .saturating_sub(BlockExecutionWeight::get());
    pub MaxElectingVotersSolution: u32 = 40_000;
}

frame_election_provider_support::generate_solution_type!(
    #[compact]
    pub struct NposSolution16::<
        VoterIndex = u32,
        TargetIndex = u16,
        Accuracy = sp_runtime::PerU16,
        MaxVoters = MaxElectingVotersSolution,
    >(16)
);

/// Converts Weight to Fee
pub struct WeightToFee;

impl frame_support::weights::WeightToFee for WeightToFee {
    type Balance = Balance;

    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        let weight_ref_time = weight.ref_time();

        let fees_per_base =
            weight_ref_time.saturating_div(ExtrinsicBaseWeight::get().ref_time()) as Balance;

        let fee = fees_per_base.saturating_mul(PolyXBaseFee::get());

        Self::Balance::saturated_from(fee)
    }
}

pub struct LengthToFee;

impl frame_support::weights::WeightToFee for LengthToFee {
    type Balance = Balance;

    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        let weight_ref_time = weight.ref_time();

        Self::Balance::saturated_from(weight_ref_time).saturating_mul(TransactionByteFee::get())
    }
}

use pallet_group_rpc_runtime_api::Member;
use polymesh_primitives::traits::group::InactiveMember;
use sp_std::{convert::From, prelude::*};

/// It merges actives and in-actives members.
pub fn merge_active_and_inactive<Block>(
    active: Vec<IdentityId>,
    inactive: Vec<InactiveMember<Moment>>,
) -> Vec<Member> {
    active
        .into_iter()
        .map(Member::from)
        .chain(inactive.into_iter().map(Member::from))
        .collect()
}
