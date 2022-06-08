#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::constants::time::*;
use codec::Encode;

#[cfg(feature = "migration-dry-run")]
use frame_support::traits::OnRuntimeUpgrade;

use core::convert::TryFrom;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{tokens::imbalance::SplitTwoWays, KeyOwnerProofSystem},
    weights::Weight,
};
use pallet_asset::checkpoint as pallet_checkpoint;
use pallet_corporate_actions::ballot as pallet_corporate_ballot;
use pallet_corporate_actions::distribution as pallet_capital_distribution;
use pallet_session::historical as pallet_session_historical;
pub use pallet_transaction_payment::{Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment};
use polymesh_common_utilities::{
    constants::currency::*, constants::ENSURED_MAX_LEN, protocol_fee::ProtocolOp,
};
use polymesh_primitives::{Balance, BlockNumber, Moment};
use polymesh_runtime_common::{
    impls::Author,
    merge_active_and_inactive,
    runtime::{GovernanceCommittee, BENCHMARK_MAX_INCREASE, VMO},
    AvailableBlockRatio, MaximumBlockWeight, NegativeImbalance,
};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    traits::{BlakeTwo256, Block as BlockT, Extrinsic, NumberFor, StaticLookup, Verify},
    Perbill, Permill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_support::StorageValue;
pub use frame_system::{limits::BlockWeights, Call as SystemCall};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("polymesh_testnet"),
    impl_name: create_runtime_str!("polymesh_testnet"),
    authoring_version: 1,
    // `spec_version: aaa_bbb_ccc` should match node version v`aaa.bbb.ccc`
    spec_version: 5_000_001,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 3,
    state_version: 1,
};

parameter_types! {
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();

    pub const Version: RuntimeVersion = VERSION;

    // Frame:
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
    pub const SS58Prefix: u8 = 42;

    // Base:
    pub const MaxLen: u32 = ENSURED_MAX_LEN;

    // Indices:
    pub const IndexDeposit: Balance = DOLLARS;

    // Balances:
    pub const ExistentialDeposit: Balance = 0u128;
    pub const MaxLocks: u32 = 50;

    // Timestamp:
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;

    // Authorship:
    pub const UncleGenerations: BlockNumber = 0;

    // Settlement:
    pub const MaxLegsInInstruction: u32 = 10;

    // I'm online:
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();

    pub const MaxAuthorities: u32 = 100_000;
    pub const MaxKeys: u32 = 10_000;
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const MaxPeerDataEncodingSize: u32 = 1_000;

    // Assets:
    pub const AssetNameMaxLength: u32 = 128;
    pub const FundingRoundNameMaxLength: u32 = 128;
    pub const AssetMetadataNameMaxLength: u32 = 256;
    pub const AssetMetadataValueMaxLength: u32 = 8 * 1024;
    pub const AssetMetadataTypeDefMaxLength: u32 = 8 * 1024;

    // Compliance manager:
    pub const MaxConditionComplexity: u32 = 50;

    // Corporate Actions:
    pub const MaxTargetIds: u32 = 1000;
    pub const MaxDidWhts: u32 = 1000;

    // Statistics:
    pub const MaxStatsPerAsset: u32 = 10 + BENCHMARK_MAX_INCREASE;
    pub const MaxTransferConditionsPerAsset: u32 = 4 + BENCHMARK_MAX_INCREASE;

    // Scheduler:
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(10);

    // Identity:
    pub const InitialPOLYX: Balance = 100_000 * POLY;

    // Contracts:
    pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
    pub DeletionWeightLimit: Weight = 500_000_000_000;
    pub DeletionQueueDepth: u32 = 1024;
    pub MaxInLen: u32 = 8 * 1024;
}

/// Splits fees 80/20 between treasury and block author.
pub type DealWithFees = SplitTwoWays<
    Balance,
    NegativeImbalance<Runtime>,
    Treasury,        // 4 parts (80%) goes to the treasury.
    Author<Runtime>, // 1 part (20%) goes to the block author.
    4,
    1,
>;

// Staking:
pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'_> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_140_000,
        ideal_stake: 0_700_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}
parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: pallet_staking::EraIndex = 28;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 14; // 1/2 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 2048;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    pub const MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational(5u32, 10_000);
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * POLY;
    pub const FixedYearlyReward: Balance = 140_000_000 * POLY;
    pub const MinimumBond: Balance = 1 * POLY;
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;

    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

polymesh_runtime_common::misc_pallet_impls!();

type CddHandler = polymesh_runtime_common::fee_details::DevCddHandler<Runtime>;

impl<'a> TryFrom<&'a Call> for &'a pallet_test_utils::Call<Runtime> {
    type Error = ();
    fn try_from(call: &'a Call) -> Result<&'a pallet_test_utils::Call<Runtime>, ()> {
        match call {
            Call::TestUtils(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl polymesh_common_utilities::traits::identity::Config for Runtime {
    type Event = Event;
    type Proposal = Call;
    type MultiSig = MultiSig;
    type Portfolio = Portfolio;
    type CddServiceProviders = CddServiceProviders;
    type Balances = pallet_balances::Module<Runtime>;
    type ChargeTxFeeTarget = TransactionPayment;
    type CddHandler = CddHandler;
    type Public = <MultiSignature as Verify>::Signer;
    type OffChainSignature = MultiSignature;
    type ProtocolFee = pallet_protocol_fee::Module<Runtime>;
    type GCVotingMajorityOrigin = VMO<GovernanceCommittee>;
    type WeightInfo = polymesh_weights::pallet_identity::WeightInfo;
    type IdentityFn = pallet_identity::Module<Runtime>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MultiSigBalanceLimit = polymesh_runtime_common::MultiSigBalanceLimit;
}

impl pallet_committee::Config<GovernanceCommittee> for Runtime {
    type CommitteeOrigin = VMO<GovernanceCommittee>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type Event = Event;
    type WeightInfo = polymesh_weights::pallet_committee::WeightInfo;
}

/// PolymeshCommittee as an instance of group
impl pallet_group::Config<pallet_group::Instance1> for Runtime {
    type Event = Event;
    type LimitOrigin = polymesh_primitives::EnsureRoot;
    type AddOrigin = Self::LimitOrigin;
    type RemoveOrigin = Self::LimitOrigin;
    type SwapOrigin = Self::LimitOrigin;
    type ResetOrigin = Self::LimitOrigin;
    type MembershipInitialized = PolymeshCommittee;
    type MembershipChanged = PolymeshCommittee;
    type WeightInfo = polymesh_weights::pallet_group::WeightInfo;
}

macro_rules! committee_config {
    ($committee:ident, $instance:ident) => {
        impl pallet_committee::Config<pallet_committee::$instance> for Runtime {
            // Can act upon itself.
            type CommitteeOrigin = VMO<pallet_committee::$instance>;
            type VoteThresholdOrigin = Self::CommitteeOrigin;
            type Event = Event;
            type WeightInfo = polymesh_weights::pallet_committee::WeightInfo;
        }
        impl pallet_group::Config<pallet_group::$instance> for Runtime {
            type Event = Event;
            // Committee cannot alter its own active membership limit.
            type LimitOrigin = polymesh_primitives::EnsureRoot;
            // Can manage its own addition, deletion, and swapping of membership...
            type AddOrigin = VMO<pallet_committee::$instance>;
            type RemoveOrigin = Self::AddOrigin;
            type SwapOrigin = Self::AddOrigin;
            // ...but it cannot reset its own membership; GC needs to do that.
            type ResetOrigin = VMO<GovernanceCommittee>;
            type MembershipInitialized = $committee;
            type MembershipChanged = $committee;
            type WeightInfo = polymesh_weights::pallet_group::WeightInfo;
        }
    };
}

committee_config!(TechnicalCommittee, Instance3);
committee_config!(UpgradeCommittee, Instance4);

impl pallet_pips::Config for Runtime {
    type Currency = Balances;
    type VotingMajorityOrigin = VMO<GovernanceCommittee>;
    type GovernanceCommittee = PolymeshCommittee;
    type TechnicalCommitteeVMO = VMO<pallet_committee::Instance3>;
    type UpgradeCommitteeVMO = VMO<pallet_committee::Instance4>;
    type Event = Event;
    type WeightInfo = polymesh_weights::pallet_pips::WeightInfo;
    type Scheduler = Scheduler;
}

/// CddProviders instance of group
impl pallet_group::Config<pallet_group::Instance2> for Runtime {
    type Event = Event;
    type LimitOrigin = polymesh_primitives::EnsureRoot;
    type AddOrigin = polymesh_primitives::EnsureRoot;
    type RemoveOrigin = polymesh_primitives::EnsureRoot;
    type SwapOrigin = polymesh_primitives::EnsureRoot;
    type ResetOrigin = polymesh_primitives::EnsureRoot;
    type MembershipInitialized = Identity;
    type MembershipChanged = Identity;
    type WeightInfo = polymesh_weights::pallet_group::WeightInfo;
}

impl pallet_test_utils::Config for Runtime {
    type Event = Event;
    type WeightInfo = polymesh_weights::pallet_test_utils::WeightInfo;
}

pub type AllModulesExported = AllPalletsWithSystem;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = polymesh_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {

        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>},
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},

        // Balance: Genesis config dependencies: System.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},

        // TransactionPayment: Genesis config dependencies: Balance.
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},

        // Identity: Genesis config deps: Timestamp.
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>, Config<T>},

        // Polymesh Committees

        // CddServiceProviders (group only): Genesis config deps: Identity
        CddServiceProviders: pallet_group::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>},

        // Governance Council (committee)
        PolymeshCommittee: pallet_committee::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // CommitteeMembership: Genesis config deps: PolymeshCommittee, Identity.
        CommitteeMembership: pallet_group::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},

        // Technical Committee
        TechnicalCommittee: pallet_committee::<Instance3>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // TechnicalCommitteeMembership: Genesis config deps: TechnicalCommittee, Identity
        TechnicalCommitteeMembership: pallet_group::<Instance3>::{Pallet, Call, Storage, Event<T>, Config<T>},

        // Upgrade Committee
        UpgradeCommittee: pallet_committee::<Instance4>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // UpgradeCommitteeMembership: Genesis config deps: UpgradeCommittee, Identity
        UpgradeCommitteeMembership: pallet_group::<Instance4>::{Pallet, Call, Storage, Event<T>, Config<T>},

        MultiSig: pallet_multisig::{Pallet, Call, Config, Storage, Event<T>},
        // Bridge: Genesis config deps: Multisig, Identity, Committees
        Bridge: pallet_bridge::{Pallet, Call, Storage, Config<T>, Event<T>},

        // Staking: Genesis config deps: Bridge, Balances, Indices, Identity, Babe, Timestamp, Committees
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
        Offences: pallet_offences::{Pallet, Storage, Event},

        // Session: Genesis config deps: System.
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config},
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event},
        Historical: pallet_session_historical::{Pallet},
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},

        // Sudo. Usable initially.
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},

        // Asset: Genesis config deps: Timestamp,
        Asset: pallet_asset::{Pallet, Call, Storage, Config<T>, Event<T>},
        CapitalDistribution: pallet_capital_distribution::{Pallet, Call, Storage, Event},
        Checkpoint: pallet_checkpoint::{Pallet, Call, Storage, Event, Config},
        ComplianceManager: pallet_compliance_manager::{Pallet, Call, Storage, Event},
        CorporateAction: pallet_corporate_actions::{Pallet, Call, Storage, Event, Config},
        CorporateBallot: pallet_corporate_ballot::{Pallet, Call, Storage, Event},
        Permissions: pallet_permissions::{Pallet},
        Pips: pallet_pips::{Pallet, Call, Storage, Event<T>, Config<T>},
        Portfolio: pallet_portfolio::{Pallet, Call, Storage, Event},
        ProtocolFee: pallet_protocol_fee::{Pallet, Call, Storage, Event<T>, Config},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        Settlement: pallet_settlement::{Pallet, Call, Storage, Event<T>, Config},
        Statistics: pallet_statistics::{Pallet, Call, Storage, Event},
        Sto: pallet_sto::{Pallet, Call, Storage, Event<T>},
        Treasury: pallet_treasury::{Pallet, Call, Event<T>},
        Utility: pallet_utility::{Pallet, Call, Storage, Event},
        Base: pallet_base::{Pallet, Call, Event},
        ExternalAgents: pallet_external_agents::{Pallet, Call, Storage, Event},
        Relayer: pallet_relayer::{Pallet, Call, Storage, Event<T>},
        Rewards: pallet_rewards::{Pallet, Call, Storage, Event<T>, Config<T>, ValidateUnsigned},

        // Contracts
        Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
        PolymeshContracts: polymesh_contracts::{Pallet, Call, Storage, Event},

        // Preimage register.  Used by `pallet_scheduler`.
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>},

        TestUtils: pallet_test_utils::{Pallet, Call, Storage, Event<T> } = 50,
    }
);

polymesh_runtime_common::runtime_apis! {}

/// Trait for testing storage migrations.
/// NB: Since this is defined outside the `impl_runtime_apis` macro, it is not callable in WASM.
#[cfg(feature = "migration-dry-run")]
pub trait DryRunRuntimeUpgrade {
    /// dry-run runtime upgrades, returning the total weight consumed.
    fn dry_run_runtime_upgrade() -> u64;
}

#[cfg(feature = "migration-dry-run")]
impl DryRunRuntimeUpgrade for Runtime {
    fn dry_run_runtime_upgrade() -> Weight {
        <AllPallets as OnRuntimeUpgrade>::on_runtime_upgrade()
    }
}
