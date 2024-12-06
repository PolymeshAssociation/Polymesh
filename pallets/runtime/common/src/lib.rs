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
#![recursion_limit = "256"]

pub mod cdd_check;
pub mod fee_details;
pub mod impls;
pub mod runtime;

pub use frame_support::{
    dispatch::{DispatchClass, GetDispatchInfo, Weight},
    parameter_types,
    traits::{Currency, Get},
    weights::{
        constants::{
            WEIGHT_REF_TIME_PER_MICROS, WEIGHT_REF_TIME_PER_MILLIS, WEIGHT_REF_TIME_PER_NANOS,
            WEIGHT_REF_TIME_PER_SECOND,
        },
        RuntimeDbWeight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use smallvec::smallvec;
pub use sp_runtime::transaction_validity::TransactionPriority;
pub use sp_runtime::{Perbill, Permill};

use pallet_balances as balances;
use polymesh_common_utilities::constants::currency::*;
use polymesh_primitives::{Balance, BlockNumber, IdentityId, Moment};

pub use cdd_check::CddChecker;
pub use impls::{Author, CurrencyToVoteHandler};

pub type NegativeImbalance<T> =
    <balances::Module<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

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
    Weight::from_ref_time(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2)).set_proof_size(u64::MAX);

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
const MINER_MAX_ITERATIONS: u32 = 10;

// TODO (miguel) Remove unused constants.
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
    pub const BlockExecutionWeight: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_MILLIS.saturating_mul(20));
    /// 0.65 ms is needed to process an empty extrinsic.
    pub const ExtrinsicBaseWeight: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_MICROS.saturating_mul(650));
    /// When the read/writes are cached/buffered, they take 25/100 microseconds on NVMe disks.
    /// When they are uncached, they take 250/450 microseconds on NVMe disks.
    /// Most read will be cached and writes will be buffered in production.
    /// We are taking a number slightly higher than what cached suggest to allow for some extra breathing room.
    pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 50 * WEIGHT_REF_TIME_PER_MICROS,   // ~50 µs @ 100,000 items
        write: 200 * WEIGHT_REF_TIME_PER_MICROS, // ~200 µs @ 100,000 items
    };
    /// This implies a 100 POLYX fee per MB of transaction length
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
    /// We want the noop transaction to cost 0.03 POLYX
    pub const PolyXBaseFee: Balance = 3 * CENTS;
    /// The maximum weight of the pips extrinsic `enact_snapshot_results` which equals to
    /// `MaximumBlockWeight * AvailableBlockRatio`.
    pub const PipsEnactSnapshotMaximumWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_mul(75).saturating_div(100);
    /// Number of block delay an extrinsic claim surcharge has.
    pub const SignedClaimHandicap: u32 = 2;
    /// The balance every contract needs to deposit to stay alive indefinitely.
    pub const DepositPerContract: u128 = 10 * CENTS;
    /// The balance a contract needs to deposit per storage item to stay alive indefinitely.
    pub const DepositPerItem: u128 = deposit(1, 0);
    /// The balance a contract needs to deposit per storage byte to stay alive indefinitely.
    pub const DepositPerByte: u128 = deposit(0, 1);
    /// The maximum nesting level of a call/instantiate stack.
    pub const ContractsMaxDepth: u32 = 32;
    /// The maximum size of a storage value and event payload in bytes.
    pub const ContractsMaxValueSize: u32 = 16 * 1024;
    /// Max length of (instrumented) contract code in bytes.
    pub const ContractsMaxCodeSize: u32 = 100 * 1024;

    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(10 * 1024 * 1024, NORMAL_DISPATCH_RATIO);

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
    pub MaxNominations: u32 = 16;
    pub HistoryDepth:u32 = 84;
    pub MaxUnlockingChunks: u32 = 32;
    pub MaxValidatorPerIdentity: Permill = Permill::from_percent(33);

    // Multi-phase election parameters
    // Signed phase
    pub const SignedPhase: u32 = 0;
    pub const SignedPhaseBench: u32 = 1_200;
    pub const SignedMaxSubmissions: u32 = 0;
    pub const SignedMaxWeight: Weight = Weight::zero();
    pub const SignedMaxRefunds: u32 = 0;
    pub const SignedRewardBase: Balance = 0;
    pub const SignedDepositBase: Balance = 0;
    pub const SignedDepositByte: Balance = 0;
    // Unsigned phase
    pub BetterUnsignedThreshold: Perbill = Perbill::from_rational(1u32, 10_000);
    pub const MultiPhaseUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2 - 1u64;
    // Fallback parameters
    pub MaxOnChainElectingVoters: u32 = 5000;
    pub MaxOnChainElectableTargets: u16 = 1250;
    // Other config parameters
    pub OffChainRepeat: BlockNumber = 5;
    pub MaxElectingVoters: u32 = 40_000;
    pub MaxElectableTargets: u16 = 10_000;
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
}

frame_election_provider_support::generate_solution_type!(
    #[compact]
    pub struct NposSolution16::<
        VoterIndex = u32,
        TargetIndex = u16,
        Accuracy = sp_runtime::PerU16,
        MaxVoters = MaxElectingVoters,
    >(16)
);

/// Converts Weight to Fee
pub struct WeightToFee;

impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    /// We want a 0.03 POLYX fee per ExtrinsicBaseWeight.
    /// 650_000_000 weight = 30_000 fee => 21_666 weight = 1 fee.
    /// Hence, 1 fee = 0 + 1/21_666 weight.
    /// This implies, coeff_integer = 0 and coeff_frac = 1/21_666.
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            coeff_frac: Perbill::from_rational(
                PolyXBaseFee::get(),
                ExtrinsicBaseWeight::get().ref_time() as u128
            ),
            coeff_integer: 0u128, // Coefficient is zero.
            negative: false,
        }]
    }
}

impl Get<Vec<WeightToFeeCoefficient<Balance>>> for WeightToFee {
    fn get() -> Vec<WeightToFeeCoefficient<Balance>> {
        Self::polynomial().to_vec()
    }
}

use pallet_group_rpc_runtime_api::Member;
use polymesh_common_utilities::traits::group::InactiveMember;
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
