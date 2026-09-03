#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[cfg(feature = "std")]
use sp_version::NativeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use codec::Encode;
use frame_support::parameter_types;
use frame_support::traits::{ConstBool, KeyOwnerProofSystem};
use frame_support::weights::Weight;
pub use frame_support::StorageValue;
pub use frame_system::Call as SystemCall;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, NumberFor, StaticLookup, Verify};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_runtime::Cow;
use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

use pallet_asset::checkpoint as pallet_checkpoint;
pub use pallet_balances::Call as BalancesCall;
use pallet_corporate_actions::ballot as pallet_corporate_ballot;
use pallet_corporate_actions::distribution as pallet_capital_distribution;
use pallet_session::historical as pallet_session_historical;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_transaction_payment::{Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment};
use polymesh_primitives::constants::currency::*;
use polymesh_primitives::constants::ENSURED_MAX_LEN;
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::settlement::Leg;
use polymesh_primitives::{AccountId, Balance, BlockNumber, Moment};
use polymesh_runtime_common::impls::Author;
use polymesh_runtime_common::merge_active_and_inactive;
use polymesh_runtime_common::runtime::{GovernanceCommittee, BENCHMARK_MAX_INCREASE, VMO};
use polymesh_runtime_common::{AvailableBlockRatio, MaximumBlockWeight};

use crate::constants::time::*;

/// 100% goes to the block author.
pub type DealWithFees = Author<Runtime>;
pub type TxFeeHandler = polymesh_runtime_common::fee_details::TxFeeHandler<Runtime>;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("polymesh_mainnet"),
    impl_name: Cow::Borrowed("polymesh_mainnet"),
    authoring_version: 1,
    // `spec_version: aaa_bbb_ccd` should match node version v`aaa.bbb.cc`
    // N.B. `d` is unpinned from the binary version
    spec_version: 8_001_000,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 8,
    system_version: 1,
};

parameter_types! {
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const Version: RuntimeVersion = VERSION;

    // Frame:
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
    pub const SS58Prefix: u8 = 12;

    // Revive/EVM
    pub const EvmChainId: u64 = 1_641_820;

    // Base:
    pub const MaxLen: u32 = ENSURED_MAX_LEN;

    // Indices:
    pub const IndexDeposit: Balance = DOLLARS;

    // Balances:
    pub const ExistentialDeposit: Balance = 1u128;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
    pub const MaxHolds: u32 = 32;
    pub const MaxFreezes: u32 = 32;

    // Timestamp:
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;

    // Settlement:
    pub const MaxNumberOfOffChainAssets: u32 = 10;
    pub const MaxNumberOfFungibleAssets: u32 = 10;
    pub const MaxNumberOfNFTsPerLeg: u32 = 10;
    pub const MaxNumberOfNFTs: u32 = 100;
    pub const MaxNumberOfAssetHolders: u32 = (10 + 100) * 2;
    pub const MaxNumberOfVenueSigners: u32 = 50;
    pub const MaxInstructionMediators: u32 = 4;
    pub const MaximumLockPeriod: Moment = 86_400_000; // 24 hours
    pub const RelockCooldown: Moment = 14_400_000; // 4 hours
    pub const MaxRelockCount: u32 = 3;

    // Multisig
    pub const MaxMultiSigSigners: u32 = 50;

    // I'm online:
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();

    pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
    pub const MaxAuthorities: u32 = 100_000;
    pub const MaxKeys: u32 = 10_000;
    pub const MaxPeerInHeartbeats: u32 = 10_000;

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
    pub const MaxAuthRetries: u8 = 10;

    // NFT:
    pub const MaxNumberOfCollectionKeys: u8 = u8::MAX;

    // Portfolio:
    pub const MaxNumberOfFungibleMoves: u32 = 10;
    pub const MaxNumberOfNFTsMoves: u32 = 100;

    // PIPs
    pub const MaxRefundsAndVotesPruned: u32 = 128;

    // Staking
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: sp_staking::EraIndex = 28;
    pub const SlashDeferDuration: sp_staking::EraIndex = 14; // 1/2 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;

    // Validators
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * ONE_POLY;
    pub const FixedYearlyReward: Balance = 140_000_000 * ONE_POLY;
    pub const MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    pub MaxPayoutWeight: Weight = Perbill::from_percent(20) * MaximumBlockWeight::get();

    // Babe
    pub const MaxNominatorRewardedPerValidator: u32 = 1_024;
    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();

    // Election Provider Multi Phase
    pub const UnsignedPhase: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
}

// Precompiles
type Precompiles = (
    pallet_precompiles::FungibleAssetInterface<Runtime>,
    pallet_precompiles::PolymeshRuntimeInterface<Runtime>,
);

impl pallet_precompiles::Config for Runtime {
    type RuntimeCall = RuntimeCall;
}

// Staking
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

impl pallet_identity::Config for Runtime {
    type Proposal = RuntimeCall;
    type DidRegistrars = DidRegistrars;
    type Balances = pallet_balances::Pallet<Runtime>;
    type TxFeeHandler = TxFeeHandler;
    type Public = <MultiSignature as Verify>::Signer;
    type OffChainSignature = MultiSignature;
    type ProtocolFee = pallet_protocol_fee::Pallet<Runtime>;
    type GCVotingMajorityOrigin = VMO<GovernanceCommittee>;
    type WeightInfo = polymesh_weights::pallet_identity::SubstrateWeight;
    type IdentityFn = pallet_identity::Pallet<Runtime>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MaxGivenAuths = MaxGivenAuths;
    type MaxAuthRetries = MaxAuthRetries;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
}

impl pallet_committee::Config<GovernanceCommittee> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = VMO<GovernanceCommittee>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

/// PolymeshCommittee as an instance of group
impl pallet_group::Config<pallet_group::Instance1> for Runtime {
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
            type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
        }
        impl pallet_group::Config<pallet_group::$instance> for Runtime {
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
    type WeightInfo = polymesh_weights::pallet_pips::SubstrateWeight;
    type Scheduler = Scheduler;
    type SchedulerCall = RuntimeCall;
    type MaxRefundsAndVotesPruned = MaxRefundsAndVotesPruned;
    type SchedulerPreimage = Preimage;
}

/// DidRegistrars instance of group
impl pallet_group::Config<pallet_group::Instance2> for Runtime {
    type LimitOrigin = polymesh_primitives::EnsureRoot;
    type AddOrigin = polymesh_primitives::EnsureRoot;
    type RemoveOrigin = polymesh_primitives::EnsureRoot;
    type SwapOrigin = polymesh_primitives::EnsureRoot;
    type ResetOrigin = polymesh_primitives::EnsureRoot;
    type MembershipInitialized = Identity;
    type MembershipChanged = Identity;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

pub struct OnChainSeqPhragmen;

impl frame_election_provider_support::onchain::Config for OnChainSeqPhragmen {
    type Sort = ConstBool<true>;
    type System = Runtime;
    type Solver = frame_election_provider_support::SequentialPhragmen<
        polymesh_primitives::AccountId,
        pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
    >;
    type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
    type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
    type Bounds = polymesh_runtime_common::ElectionBoundsOnChain;
    type MaxBackersPerWinner = polymesh_runtime_common::MaxElectingVotersSolution;
    type MaxWinnersPerPage = polymesh_runtime_common::MaxActiveValidators;
}

polymesh_runtime_common::misc_pallet_impls!();

#[frame_support::runtime]
mod runtime {
    use super::*;

    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason
    )]
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Runtime>;

    #[runtime::pallet_index(1)]
    pub type Babe = pallet_babe::Pallet<Runtime>;

    #[runtime::pallet_index(2)]
    pub type Timestamp = pallet_timestamp::Pallet<Runtime>;

    #[runtime::pallet_index(3)]
    pub type Indices = pallet_indices::Pallet<Runtime>;

    #[runtime::pallet_index(4)]
    pub type Authorship = pallet_authorship::Pallet<Runtime>;

    #[runtime::pallet_index(5)]
    pub type Balances = pallet_balances::Pallet<Runtime>;

    #[runtime::pallet_index(6)]
    pub type TransactionPayment = pallet_transaction_payment::Pallet<Runtime>;

    #[runtime::pallet_index(51)]
    pub type PolymeshTransactionPayment = polymesh_transaction_payment::Pallet<Runtime>;

    #[runtime::pallet_index(7)]
    pub type Identity = pallet_identity::Pallet<Runtime>;

    #[runtime::pallet_index(8)]
    pub type DidRegistrars = pallet_group::Pallet<Runtime, Instance2>;

    #[runtime::pallet_index(9)]
    pub type PolymeshCommittee = pallet_committee::Pallet<Runtime, Instance1>;

    #[runtime::pallet_index(10)]
    pub type CommitteeMembership = pallet_group::Pallet<Runtime, Instance1>;

    #[runtime::pallet_index(11)]
    pub type TechnicalCommittee = pallet_committee::Pallet<Runtime, Instance3>;

    #[runtime::pallet_index(12)]
    pub type TechnicalCommitteeMembership = pallet_group::Pallet<Runtime, Instance3>;

    #[runtime::pallet_index(13)]
    pub type UpgradeCommittee = pallet_committee::Pallet<Runtime, Instance4>;

    #[runtime::pallet_index(14)]
    pub type UpgradeCommitteeMembership = pallet_group::Pallet<Runtime, Instance4>;

    #[runtime::pallet_index(15)]
    pub type MultiSig = pallet_multisig::Pallet<Runtime>;

    #[runtime::pallet_index(16)]
    pub type Validators = pallet_validators::Pallet<Runtime>;

    #[runtime::pallet_index(17)]
    pub type Staking = pallet_staking::Pallet<Runtime>;

    #[runtime::pallet_index(18)]
    pub type Offences = pallet_offences::Pallet<Runtime>;

    #[runtime::pallet_index(19)]
    pub type Session = pallet_session::Pallet<Runtime>;

    #[runtime::pallet_index(20)]
    pub type AuthorityDiscovery = pallet_authority_discovery::Pallet<Runtime>;

    #[runtime::pallet_index(21)]
    pub type Grandpa = pallet_grandpa::Pallet<Runtime>;

    #[runtime::pallet_index(22)]
    pub type Historical = pallet_session_historical::Pallet<Runtime>;

    #[runtime::pallet_index(23)]
    pub type ImOnline = pallet_im_online::Pallet<Runtime>;

    #[runtime::pallet_index(26)]
    pub type Asset = pallet_asset::Pallet<Runtime>;

    #[runtime::pallet_index(27)]
    pub type CapitalDistribution = pallet_capital_distribution::Pallet<Runtime>;

    #[runtime::pallet_index(28)]
    pub type Checkpoint = pallet_checkpoint::Pallet<Runtime>;

    #[runtime::pallet_index(29)]
    pub type ComplianceManager = pallet_compliance_manager::Pallet<Runtime>;

    #[runtime::pallet_index(30)]
    pub type CorporateAction = pallet_corporate_actions::Pallet<Runtime>;

    #[runtime::pallet_index(31)]
    pub type CorporateBallot = pallet_corporate_ballot::Pallet<Runtime>;

    #[runtime::pallet_index(32)]
    pub type Permissions = pallet_permissions::Pallet<Runtime>;

    #[runtime::pallet_index(33)]
    pub type Pips = pallet_pips::Pallet<Runtime>;

    #[runtime::pallet_index(34)]
    pub type Portfolio = pallet_portfolio::Pallet<Runtime>;

    #[runtime::pallet_index(35)]
    pub type ProtocolFee = pallet_protocol_fee::Pallet<Runtime>;

    #[runtime::pallet_index(36)]
    pub type Scheduler = pallet_scheduler::Pallet<Runtime>;

    #[runtime::pallet_index(37)]
    pub type Settlement = pallet_settlement::Pallet<Runtime>;

    #[runtime::pallet_index(38)]
    pub type Statistics = pallet_statistics::Pallet<Runtime>;

    #[runtime::pallet_index(39)]
    pub type Sto = pallet_sto::Pallet<Runtime>;

    #[runtime::pallet_index(40)]
    pub type Treasury = pallet_treasury::Pallet<Runtime>;

    #[runtime::pallet_index(41)]
    pub type Utility = pallet_utility::Pallet<Runtime>;

    #[runtime::pallet_index(42)]
    pub type Base = pallet_base::Pallet<Runtime>;

    #[runtime::pallet_index(43)]
    pub type ExternalAgents = pallet_external_agents::Pallet<Runtime>;

    #[runtime::pallet_index(44)]
    pub type Relayer = pallet_relayer::Pallet<Runtime>;

    #[runtime::pallet_index(48)]
    pub type Preimage = pallet_preimage::Pallet<Runtime>;

    #[runtime::pallet_index(49)]
    pub type Nft = pallet_nft::Pallet<Runtime>;

    #[runtime::pallet_index(50)]
    pub type ElectionProviderMultiPhase = pallet_election_provider_multi_phase::Pallet<Runtime>;

    #[runtime::pallet_index(52)]
    pub type Beefy = pallet_beefy::Pallet<Runtime>;

    // MMR leaf construction must be after session in order to have a leaf's next_auth_set
    // refer to block<N>. See issue polkadot-fellows/runtimes#160 for details.
    #[runtime::pallet_index(53)]
    pub type Mmr = pallet_mmr::Pallet<Runtime>;

    #[runtime::pallet_index(54)]
    pub type MmrLeaf = pallet_beefy_mmr::Pallet<Runtime>;

    #[runtime::pallet_index(55)]
    pub type MultiBlockMigrations = pallet_migrations::Pallet<Runtime>;

    #[runtime::pallet_index(60)]
    pub type WorkerModules = pallet_worker_modules::Pallet<Runtime>;

    #[runtime::pallet_index(80)]
    pub type Revive = pallet_revive::Pallet<Runtime>;
}

polymesh_runtime_common::runtime_apis! {}
