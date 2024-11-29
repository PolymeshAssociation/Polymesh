#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[cfg(feature = "std")]
use sp_version::NativeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use codec::Encode;
use core::convert::TryFrom;
use frame_support::dispatch::DispatchResult;
use frame_support::traits::KeyOwnerProofSystem;
use frame_support::weights::Weight;
use frame_support::{construct_runtime, parameter_types};
use sp_runtime::create_runtime_str;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, Extrinsic, NumberFor, StaticLookup, Verify,
};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

use pallet_asset::checkpoint as pallet_checkpoint;
use pallet_corporate_actions::ballot as pallet_corporate_ballot;
use pallet_corporate_actions::distribution as pallet_capital_distribution;
use pallet_session::historical as pallet_session_historical;
pub use pallet_transaction_payment::{Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment};
use polymesh_common_utilities::constants::currency::*;
use polymesh_common_utilities::constants::ENSURED_MAX_LEN;
use polymesh_common_utilities::protocol_fee::ProtocolOp;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::settlement::Leg;
use polymesh_primitives::{AccountId, Balance, BlockNumber, Moment};
use polymesh_runtime_common::impls::Author;
use polymesh_runtime_common::merge_active_and_inactive;
use polymesh_runtime_common::runtime::{GovernanceCommittee, BENCHMARK_MAX_INCREASE, VMO};
use polymesh_runtime_common::{AvailableBlockRatio, MaximumBlockWeight};

use crate::constants::time::*;

pub use frame_support::StorageValue;
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("polymesh_dev"),
    impl_name: create_runtime_str!("polymesh_dev"),
    authoring_version: 1,
    // `spec_version: aaa_bbb_ccd` should match node version v`aaa.bbb.cc`
    // N.B. `d` is unpinned from the binary version
    spec_version: 7_000_005,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 7,
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

    // Settlement:
    pub const MaxNumberOfOffChainAssets: u32 = 10;
    pub const MaxNumberOfFungibleAssets: u32 = 10;
    pub const MaxNumberOfNFTsPerLeg: u32 = 10;
    pub const MaxNumberOfNFTs: u32 = 100;
    pub const MaxNumberOfPortfolios: u32 = (10 + 100) * 2;
    pub const MaxNumberOfVenueSigners: u32 = 50;
    pub const MaxInstructionMediators: u32 = 4;

    // Multisig
    pub const MaxMultiSigSigners: u32 = 50;

    // I'm online:
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();

    pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
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
    pub const MaxAssetMediators: u32 = 4;

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

    // Identity:
    pub const InitialPOLYX: Balance = 0;
    pub const MaxGivenAuths: u32 = 1024;

    // Contracts:
    pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
    pub DeletionWeightLimit: Weight = Weight::from_ref_time(500_000_000_000);
    pub DeletionQueueDepth: u32 = 1024;
    pub MaxInLen: u32 = 8 * 1024;
    pub MaxOutLen: u32 = 8 * 1024;

    // NFT:
    pub const MaxNumberOfCollectionKeys: u8 = u8::MAX;

    // Portfolio:
    pub const MaxNumberOfFungibleMoves: u32 = 10;
    pub const MaxNumberOfNFTsMoves: u32 = 100;

    // State trie Migration
    pub const MigrationSignedDepositPerItem: Balance = 1_000;
    pub const MigrationSignedDepositBase: Balance = 1_000_000;
    pub const MaxKeyLen: u32 = 2048;
}

/// 100% goes to the block author.
pub type DealWithFees = Author<Runtime>;

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
    pub const SessionsPerEra: sp_staking::SessionIndex = 3;
    pub const BondingDuration: sp_staking::EraIndex = 7;
    pub const SlashDeferDuration: sp_staking::EraIndex = 4;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 1_024;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const UnsignedPhase: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    pub const MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational(5u32, 10_000);
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * ONE_POLY;
    pub const FixedYearlyReward: Balance = 140_000_000 * ONE_POLY;
    pub const MinimumBond: Balance = ONE_POLY;
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;

    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

polymesh_runtime_common::misc_pallet_impls!();

pub type CddHandler = polymesh_runtime_common::fee_details::DevCddHandler<Runtime>;

impl<'a> TryFrom<&'a RuntimeCall> for &'a pallet_test_utils::Call<Runtime> {
    type Error = ();
    fn try_from(call: &'a RuntimeCall) -> Result<&'a pallet_test_utils::Call<Runtime>, ()> {
        match call {
            RuntimeCall::TestUtils(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl polymesh_common_utilities::traits::identity::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Proposal = RuntimeCall;
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
    type WeightInfo = polymesh_weights::pallet_identity::SubstrateWeight;
    type IdentityFn = pallet_identity::Module<Runtime>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MaxGivenAuths = MaxGivenAuths;
}

impl pallet_committee::Config<GovernanceCommittee> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = VMO<GovernanceCommittee>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

/// PolymeshCommittee as an instance of group
impl pallet_group::Config<pallet_group::Instance1> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type LimitOrigin = polymesh_primitives::EnsureRoot;
    type AddOrigin = Self::LimitOrigin;
    type RemoveOrigin = Self::LimitOrigin;
    type SwapOrigin = Self::LimitOrigin;
    type ResetOrigin = Self::LimitOrigin;
    type MembershipInitialized = PolymeshCommittee;
    type MembershipChanged = PolymeshCommittee;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

macro_rules! committee_config {
    ($committee:ident, $instance:ident) => {
        impl pallet_committee::Config<pallet_committee::$instance> for Runtime {
            type RuntimeOrigin = RuntimeOrigin;
            type Proposal = RuntimeCall;
            // Can act upon itself.
            type CommitteeOrigin = VMO<pallet_committee::$instance>;
            type VoteThresholdOrigin = Self::CommitteeOrigin;
            type RuntimeEvent = RuntimeEvent;
            type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
        }
        impl pallet_group::Config<pallet_group::$instance> for Runtime {
            type RuntimeEvent = RuntimeEvent;
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
            type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
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
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_pips::SubstrateWeight;
    type Scheduler = Scheduler;
    type SchedulerCall = RuntimeCall;
}

/// CddProviders instance of group
impl pallet_group::Config<pallet_group::Instance2> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // Cannot alter its own active membership limit.
    type LimitOrigin = polymesh_primitives::EnsureRoot;
    type AddOrigin = polymesh_primitives::EnsureRoot;
    type RemoveOrigin = polymesh_primitives::EnsureRoot;
    type SwapOrigin = polymesh_primitives::EnsureRoot;
    type ResetOrigin = polymesh_primitives::EnsureRoot;
    type MembershipInitialized = Identity;
    type MembershipChanged = Identity;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl pallet_test_utils::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_test_utils::SubstrateWeight;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
}

/// NB It is needed by benchmarks, in order to use `UserBuilder`.
impl TestUtilsFn<AccountId> for Runtime {
    fn register_did(
        target: AccountId,
        secondary_keys: Vec<polymesh_primitives::secondary_key::SecondaryKey<AccountId>>,
    ) -> DispatchResult {
        <TestUtils as TestUtilsFn<AccountId>>::register_did(target, secondary_keys)
    }
}

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [pallet_asset, Asset]
        [pallet_balances, Balances]
        [pallet_identity, Identity]
        [pallet_pips, Pips]
        [pallet_multisig, MultiSig]
        [pallet_portfolio, Portfolio]
        [pallet_protocol_fee, ProtocolFee]
        [frame_system, SystemBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_settlement, Settlement]
        [pallet_sto, Sto]
        [pallet_checkpoint, Checkpoint]
        [pallet_compliance_manager, ComplianceManager]
        [pallet_corporate_actions, CorporateAction]
        [pallet_corporate_ballot, CorporateBallot]
        [pallet_capital_distribution, CapitalDistribution]
        [pallet_external_agents, ExternalAgents]
        [pallet_relayer, Relayer]
        [pallet_committee, PolymeshCommittee]
        [pallet_utility, Utility]
        [pallet_treasury, Treasury]
        [pallet_im_online, ImOnline]
        [pallet_group, CddServiceProviders]
        [pallet_statistics, Statistics]
        [pallet_permissions, Permissions]
        [pallet_preimage, Preimage]
        [pallet_babe, Babe]
        [pallet_indices, Indices]
        [pallet_session, SessionBench::<Runtime>]
        [pallet_grandpa, Grandpa]
        [pallet_scheduler, Scheduler]
        [pallet_staking, Staking]
        [pallet_test_utils, TestUtils]
        [polymesh_contracts, PolymeshContracts]
        [pallet_nft, Nft]
        [pallet_contracts, Contracts]
    );
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = polymesh_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 1,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
        Authorship: pallet_authorship = 4,

        // Balance: Genesis config dependencies: System.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,

        // TransactionPayment: Genesis config dependencies: Balance.
        TransactionPayment: pallet_transaction_payment::{Pallet, Call, Event<T>, Storage} = 6,

        // Identity: Genesis config deps: Timestamp.
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>, Config<T>} = 7,

        // Polymesh Committees

        // CddServiceProviders (group only): Genesis config deps: Identity
        CddServiceProviders: pallet_group::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 8,

        // Governance Council (committee)
        PolymeshCommittee: pallet_committee::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 9,
        // CommitteeMembership: Genesis config deps: PolymeshCommittee, Identity.
        CommitteeMembership: pallet_group::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 10,

        // Technical Committee
        TechnicalCommittee: pallet_committee::<Instance3>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 11,
        // TechnicalCommitteeMembership: Genesis config deps: TechnicalCommittee, Identity
        TechnicalCommitteeMembership: pallet_group::<Instance3>::{Pallet, Call, Storage, Event<T>, Config<T>} = 12,

        // Upgrade Committee
        UpgradeCommittee: pallet_committee::<Instance4>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 13,
        // UpgradeCommitteeMembership: Genesis config deps: UpgradeCommittee, Identity
        UpgradeCommitteeMembership: pallet_group::<Instance4>::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,

        MultiSig: pallet_multisig::{Pallet, Call, Config, Storage, Event<T>} = 15,

        Bridge: pallet_bridge::{Pallet, Storage} = 16,

        // Staking: Genesis config deps: Bridge, Balances, Indices, Identity, Babe, Timestamp, Committees
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>} = 17,

        Offences: pallet_offences::{Pallet, Storage, Event} = 18,

        // Session: Genesis config deps: System.
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 19,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 20,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event} = 21,
        Historical: pallet_session_historical::{Pallet} = 22,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 23,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage} = 24,

        // Sudo. Usable initially.
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 25,

        // Asset: Genesis config deps: Timestamp,
        Asset: pallet_asset::{Pallet, Call, Storage, Config<T>, Event<T>} = 26,
        CapitalDistribution: pallet_capital_distribution::{Pallet, Call, Storage, Event} = 27,
        Checkpoint: pallet_checkpoint::{Pallet, Call, Storage, Event, Config} = 28,
        ComplianceManager: pallet_compliance_manager::{Pallet, Call, Storage, Event} = 29,
        CorporateAction: pallet_corporate_actions::{Pallet, Call, Storage, Event, Config} = 30,
        CorporateBallot: pallet_corporate_ballot::{Pallet, Call, Storage, Event} = 31,
        Permissions: pallet_permissions::{Pallet} = 32,
        Pips: pallet_pips::{Pallet, Call, Storage, Event<T>, Config<T>} = 33,
        Portfolio: pallet_portfolio::{Pallet, Call, Storage, Event, Config} = 34,
        ProtocolFee: pallet_protocol_fee::{Pallet, Call, Storage, Event<T>, Config} = 35,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 36,
        Settlement: pallet_settlement::{Pallet, Call, Storage, Event<T>, Config} = 37,
        Statistics: pallet_statistics::{Pallet, Call, Storage, Event, Config} = 38,
        Sto: pallet_sto::{Pallet, Call, Storage, Event<T>} = 39,
        Treasury: pallet_treasury::{Pallet, Call, Event<T>} = 40,
        Utility: pallet_utility::{Pallet, Call, Storage, Event<T>} = 41,
        Base: pallet_base::{Pallet, Call, Event} = 42,
        ExternalAgents: pallet_external_agents::{Pallet, Call, Storage, Event} = 43,
        Relayer: pallet_relayer::{Pallet, Call, Storage, Event<T>} = 44,
        // Removed pallet_rewards = 45,

        // Contracts
        Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>} = 46,
        PolymeshContracts: polymesh_contracts::{Pallet, Call, Storage, Event<T>, Config<T>} = 47,

        // Preimage register.  Used by `pallet_scheduler`.
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 48,

        Nft: pallet_nft::{Pallet, Call, Storage, Event} = 49,

        ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 50,

        TestUtils: pallet_test_utils::{Pallet, Call, Storage, Event<T> } = 200,
    }
);

polymesh_runtime_common::runtime_apis! {
    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        #[allow(non_local_definitions)]
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch,  TrackedStorageKey};

            use crate::benchmarks::pallet_session::Pallet as SessionBench;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}
            impl crate::benchmarks::pallet_session::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
                // Treasury Account
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);
            Ok(batches)
        }

        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;

            use crate::benchmarks::pallet_session::Pallet as SessionBench;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            return (list, storage_info)
        }
    }
}

pub struct OnChainSeqPhragmen;

impl frame_election_provider_support::onchain::Config for OnChainSeqPhragmen {
    type System = Runtime;
    type Solver = frame_election_provider_support::SequentialPhragmen<
        polymesh_primitives::AccountId,
        pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
    >;
    type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
    type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
    type MaxWinners = <Runtime as pallet_election_provider_multi_phase::Config>::MaxWinners;
    type VotersBound = polymesh_runtime_common::MaxOnChainElectingVoters;
    type TargetsBound = polymesh_runtime_common::MaxOnChainElectableTargets;
}
