#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::{
    constants::{fee::*, time::*},
    fee_details::CddHandler,
};
use codec::Encode;
use pallet_asset as asset;
use pallet_balances as balances;
use pallet_basic_sto as sto;
use pallet_committee as committee;
use pallet_compliance_manager::{self as compliance_manager, AssetTransferRulesResult};
use pallet_confidential as confidential;
use pallet_group as group;
use pallet_identity::{
    self as identity,
    types::{AssetDidResult, CddStatus, DidRecords, DidStatus},
};
use pallet_multisig as multisig;
use pallet_pips::{HistoricalVotingByAddress, HistoricalVotingById, Vote, VoteCount};
use pallet_portfolio as portfolio;
use pallet_protocol_fee as protocol_fee;
use pallet_settlement as settlement;
use pallet_statistics as statistics;
pub use pallet_transaction_payment::{Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment};
use pallet_treasury as treasury;
use pallet_utility as utility;
use polymesh_common_utilities::{
    constants::currency::*,
    protocol_fee::ProtocolOp,
    traits::{balances::AccountData, identity::Trait as IdentityTrait},
    CommonTrait,
};
use polymesh_primitives::{
    AccountId, AccountIndex, Authorization, AuthorizationType, Balance, BlockNumber, Hash,
    IdentityId, Index, Moment, PortfolioId, SecondaryKey, Signatory, Signature, Ticker,
};
use polymesh_runtime_common::{
    bridge,
    cdd_check::CddChecker,
    contracts_wrapper, dividend, exemption,
    impls::{Author, CurrencyToVoteHandler},
    merge_active_and_inactive, sto_capped, voting, AvailableBlockRatio, BlockHashCount,
    MaximumBlockLength, MaximumBlockWeight, NegativeImbalance,
};

use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2, _4},
    OpaqueMetadata,
};
use sp_runtime::transaction_validity::{
    TransactionPriority, TransactionSource, TransactionValidity,
};
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic, impl_opaque_keys,
    traits::{
        BlakeTwo256, Block as BlockT, Extrinsic, NumberFor, OpaqueKeys, SaturatedConversion,
        Saturating, StaticLookup, Verify,
    },
    ApplyExtrinsicResult, MultiSignature, Perbill,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

// Comment in the favour of not using the Offchain worker
//use pallet_cdd_offchain_worker::crypto::SignerId as CddOffchainWorkerId;
use frame_support::{
    construct_runtime, debug, parameter_types,
    traits::{KeyOwnerProofSystem, Randomness, SplitTwoWays},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
        Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use pallet_contracts_rpc_runtime_api::ContractExecResult;

use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_protocol_fee_rpc_runtime_api::CappedFee;
use pallet_session::historical as pallet_session_historical;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_inherents::{CheckInherentsResult, InherentData};
#[cfg(feature = "std")]
use sp_version::NativeVersion;

pub use balances::Call as BalancesCall;
pub use frame_support::StorageValue;
pub use pallet_contracts::Gas;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use smallvec::smallvec;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("polymesh"),
    impl_name: create_runtime_str!("polymath-polymesh"),
    authoring_version: 1,
    // Per convention: if the runtime behavior changes, increment spec_version
    // and set impl_version to 0. If only runtime
    // implementation changes and behavior does not, then leave spec_version as
    // is and increment impl_version.
    spec_version: 2000,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const Version: RuntimeVersion = VERSION;
}

impl frame_system::Trait for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The weight of the overhead invoked on the block import process, independent of the
    /// extrinsics included in that block.
    type BlockExecutionWeight = BlockExecutionWeight;
    /// The base weight of any extrinsic processed by the runtime, independent of the
    /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
    /// idependent of the logic of that extrinsics. (Roughly max block weight - average on
    /// initialize cost).
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type ModuleToIndex = ModuleToIndex;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = AccountData<Balance>;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl pallet_babe::Trait for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
}

parameter_types! {
    pub const IndexDeposit: Balance = DOLLARS;
}

impl pallet_indices::Trait for Runtime {
    type AccountIndex = AccountIndex;
    type Currency = Balances;
    type Deposit = IndexDeposit;
    type Event = Event;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 0u128;
}

/// Splits fees 80/20 between treasury and block author.
pub type DealWithFees = SplitTwoWays<
    Balance,
    NegativeImbalance<Runtime>,
    _4,
    Treasury, // 4 parts (80%) goes to the treasury.
    _1,
    Author<Runtime>, // 1 part (20%) goes to the block author.
>;

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            coeff_frac: Perbill::from_percent(10),
            coeff_integer: 0u128, // Coefficient is zero
            negative: false,
        }]
    }
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
    // for a sane configuration, this should always be less than `AvailableBlockRatio`.
    pub const TargetBlockFullness: Perbill = TARGET_BLOCK_FULLNESS;
}

impl pallet_transaction_payment::Trait for Runtime {
    type Currency = Balances;
    type OnTransactionPayment = DealWithFees;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = WeightToFee;
    type FeeMultiplierUpdate = ();
    type CddHandler = CddHandler;
}

impl CommonTrait for Runtime {
    type Balance = Balance;
    type AcceptTransferTarget = Asset;
    type BlockRewardsReserve = balances::Module<Runtime>;
}

impl balances::Trait for Runtime {
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Runtime>;
    type Identity = Identity;
    type CddChecker = CddChecker<Runtime>;
}

impl protocol_fee::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type OnProtocolFeePayment = DealWithFees;
}

parameter_types! {
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
    type Moment = Moment;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 0;
}

// TODO: substrate#2986 implement this properly
impl pallet_authorship::Trait for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (Staking, ImOnline);
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

// NOTE: `SessionHandler` and `SessionKeys` are co-dependent: One key will be used for each handler.
// The number and order of items in `SessionHandler` *MUST* be the same number and order of keys in
// `SessionKeys`.
// TODO: Introduce some structure to tie these together to make it a bit less of a footgun. This
// should be easy, since OneSessionHandler trait provides the `Key` as an associated type. #2858
parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Trait for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Trait>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl pallet_session::historical::Trait for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_200_000,
        ideal_stake: 0_700_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 3;
    pub const BondingDuration: pallet_staking::EraIndex = 7;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 4; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
}

impl pallet_staking::Trait for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type CurrencyToVote = CurrencyToVoteHandler<Self>;
    type RewardRemainder = Treasury;
    type Event = Event;
    type Slash = Treasury; // send the slashed funds to the treasury.
    type Reward = (); // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = frame_system::EnsureRoot<AccountId>;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = StakingUnsignedPriority;
    type RequiredAddOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredRemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredComplianceOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredCommissionOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredChangeHistoryDepthOrigin = frame_system::EnsureRoot<AccountId>;
}

parameter_types! {
    pub const MotionDuration: BlockNumber = 0;
}

/// Voting majority origin for `Instance`.
type VMO<Instance> = committee::EnsureProportionAtLeast<_1, _2, AccountId, Instance>;

type GovernanceCommittee = committee::Instance1;
impl committee::Trait<GovernanceCommittee> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type CommitteeOrigin = VMO<GovernanceCommittee>;
    type Event = Event;
    type MotionDuration = MotionDuration;
}
/// PolymeshCommittee as an instance of group
impl group::Trait<group::Instance1> for Runtime {
    type Event = Event;
    type LimitOrigin = frame_system::EnsureRoot<AccountId>;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = PolymeshCommittee;
    type MembershipChanged = PolymeshCommittee;
}

macro_rules! committee_config {
    ($committee:ident, $instance:ident) => {
        impl committee::Trait<committee::$instance> for Runtime {
            type Origin = Origin;
            type Proposal = Call;
            // Can act upon itself.
            type CommitteeOrigin = VMO<committee::$instance>;
            type Event = Event;
            type MotionDuration = MotionDuration;
        }
        impl group::Trait<group::$instance> for Runtime {
            type Event = Event;
            // Committee cannot alter its own active membership limit.
            type LimitOrigin = frame_system::EnsureRoot<AccountId>;
            // Can manage its own addition, deletion, and swapping of membership...
            type AddOrigin = VMO<committee::$instance>;
            type RemoveOrigin = VMO<committee::$instance>;
            type SwapOrigin = VMO<committee::$instance>;
            // ...but it cannot reset its own membership; GC needs to do that.
            type ResetOrigin = VMO<GovernanceCommittee>;
            type MembershipInitialized = $committee;
            type MembershipChanged = $committee;
        }
    };
}

committee_config!(TechnicalCommittee, Instance3);
committee_config!(UpgradeCommittee, Instance4);

impl pallet_pips::Trait for Runtime {
    type Currency = Balances;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type VotingMajorityOrigin = VMO<GovernanceCommittee>;
    type GovernanceCommittee = PolymeshCommittee;
    type TechnicalCommitteeVMO = VMO<committee::Instance3>;
    type UpgradeCommitteeVMO = VMO<committee::Instance4>;
    type Treasury = Treasury;
    type Event = Event;
}

parameter_types! {
    pub const TombstoneDeposit: Balance = DOLLARS;
    pub const RentByteFee: Balance = DOLLARS;
    pub const RentDepositOffset: Balance = 300 * DOLLARS;
    pub const SurchargeReward: Balance = 150 * DOLLARS;
}

impl pallet_contracts::Trait for Runtime {
    type Time = Timestamp;
    type Randomness = RandomnessCollectiveFlip;
    type Currency = Balances;
    type Call = Call;
    type Event = Event;
    type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminer<Runtime>;
    type TrieIdGenerator = pallet_contracts::TrieIdFromParentCounter<Runtime>;
    type RentPayment = ();
    type SignedClaimHandicap = pallet_contracts::DefaultSignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type StorageSizeOffset = pallet_contracts::DefaultStorageSizeOffset;
    type RentByteFee = RentByteFee;
    type RentDepositOffset = RentDepositOffset;
    type SurchargeReward = SurchargeReward;
    type MaxDepth = pallet_contracts::DefaultMaxDepth;
    type MaxValueSize = pallet_contracts::DefaultMaxValueSize;
    type WeightPrice = pallet_transaction_payment::Module<Self>;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(Call, <UncheckedExtrinsic as Extrinsic>::SignaturePayload)> {
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let tip = 0;
        let extra: SignedExtra = (
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
            pallet_grandpa::ValidateEquivocationReport::<Runtime>::new(),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                debug::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = Indices::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl treasury::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
}

parameter_types! {
    pub const MaxScheduledInstructionLegsPerBlock: u32 = 500;
    pub const MaxLegsInAInstruction: u32 = 20;
}

impl settlement::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
    type MaxScheduledInstructionLegsPerBlock = MaxScheduledInstructionLegsPerBlock;
    type MaxLegsInAInstruction = MaxLegsInAInstruction;
}

impl sto::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * MaximumBlockWeight::get();
}

impl pallet_offences::Trait for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
    type WeightSoftLimit = OffencesWeightSoftLimit;
}

parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_SLOTS as _;
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

impl pallet_im_online::Trait for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type UnsignedPriority = ImOnlineUnsignedPriority;
    type ReportUnresponsiveness = Offences;
    type SessionDuration = SessionDuration;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
}

impl pallet_grandpa::Trait for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = pallet_grandpa::EquivocationHandler<
        Self::KeyOwnerIdentification,
        polymesh_primitives::report::ReporterAppCrypto,
        Runtime,
        Offences,
    >;
}

impl pallet_authority_discovery::Trait for Runtime {}

parameter_types! {
    pub const WindowSize: BlockNumber = pallet_finality_tracker::DEFAULT_WINDOW_SIZE;
    pub const ReportLatency: BlockNumber = pallet_finality_tracker::DEFAULT_REPORT_LATENCY;
}

impl pallet_finality_tracker::Trait for Runtime {
    type OnFinalizationStalled = ();
    type WindowSize = WindowSize;
    type ReportLatency = ReportLatency;
}

impl pallet_sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

impl multisig::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    pub const MaxTimelockedTxsPerBlock: u32 = 10;
}

impl bridge::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
    type MaxTimelockedTxsPerBlock = MaxTimelockedTxsPerBlock;
}

impl portfolio::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    pub const MaxNumberOfTMExtensionForAsset: u32 = 5;
}

impl asset::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type ComplianceManager = compliance_manager::Module<Runtime>;
    type MaxNumberOfTMExtensionForAsset = MaxNumberOfTMExtensionForAsset;
}

parameter_types! {
    pub const MaxRuleComplexity: u32 = 50;
}

impl compliance_manager::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
    type MaxRuleComplexity = MaxRuleComplexity;
}

impl voting::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl sto_capped::Trait for Runtime {
    type Event = Event;
}

impl IdentityTrait for Runtime {
    type Event = Event;
    type Proposal = Call;
    type MultiSig = MultiSig;
    type CddServiceProviders = CddServiceProviders;
    type Balances = balances::Module<Runtime>;
    type ChargeTxFeeTarget = TransactionPayment;
    type CddHandler = CddHandler;
    type Public = <MultiSignature as Verify>::Signer;
    type OffChainSignature = MultiSignature;
    type ProtocolFee = protocol_fee::Module<Runtime>;
}

impl contracts_wrapper::Trait for Runtime {}

impl exemption::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl dividend::Trait for Runtime {
    type Event = Event;
}

/// CddProviders instance of group
impl group::Trait<group::Instance2> for Runtime {
    type Event = Event;
    // Cannot alter its own active membership limit.
    type LimitOrigin = frame_system::EnsureRoot<AccountId>;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = Identity;
    type MembershipChanged = Identity;
}

impl statistics::Trait for Runtime {}

impl pallet_utility::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

impl confidential::Trait for Runtime {
    type Event = Event;
}

// / A runtime transaction submitter for the cdd_offchain_worker
// Comment it in the favour of Testnet v1 release
//type SubmitTransactionCdd = TransactionSubmitter<CddOffchainWorkerId, Runtime, UncheckedExtrinsic>;

// Comment it in the favour of Testnet v1 release
// parameter_types! {
//     pub const CoolingInterval: BlockNumber = 3;
//     pub const BufferInterval: BlockNumber = 5;
// }

// impl pallet_cdd_offchain_worker::Trait for Runtime {
//     /// SignerId
//     type SignerId = CddOffchainWorkerId;
//     /// The overarching event type.
//     type Event = Event;
//     /// The overarching dispatch call type
//     type Call = Call;
//     /// No. of blocks delayed to execute the offchain worker
//     type CoolingInterval = CoolingInterval;
//     /// Buffer given to check the validity of the cdd claim. It is in block numbers.
//     type BufferInterval = BufferInterval;
//     /// The type submit transactions.
//     type SubmitUnsignedTransaction = SubmitTransactionCdd;
// }

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = polymesh_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        // Must be before session.
        Babe: pallet_babe::{Module, Call, Storage, Config, Inherent(Timestamp)},

        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>},
        Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},

        // Consensus frame_support.
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},
        Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
        Offences: pallet_offences::{Module, Call, Storage, Event},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        FinalityTracker: pallet_finality_tracker::{Module, Call, Inherent},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
        ImOnline: pallet_im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
        AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},

        Historical: pallet_session_historical::{Module},
        // Sudo. Usable initially.
        // RELEASE: remove this for release build.
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},

        MultiSig: multisig::{Module, Call, Storage, Event<T>},

        // Contracts
        Contracts: pallet_contracts::{Module, Call, Config, Storage, Event<T>},
        // ContractsWrapper: contracts_wrapper::{Module, Call, Storage},

        // Polymesh Governance Committees
        Treasury: treasury::{Module, Call, Event<T>},
        PolymeshCommittee: committee::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        CommitteeMembership: group::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        Pips: pallet_pips::{Module, Call, Storage, Event<T>, Config<T>},

        TechnicalCommittee: committee::<Instance3>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TechnicalCommitteeMembership: group::<Instance3>::{Module, Call, Storage, Event<T>, Config<T>},

        UpgradeCommittee: committee::<Instance4>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        UpgradeCommitteeMembership: group::<Instance4>::{Module, Call, Storage, Event<T>, Config<T>},

        //Polymesh
        Asset: asset::{Module, Call, Storage, Config<T>, Event<T>},
        Dividend: dividend::{Module, Call, Storage, Event<T>},
        Identity: identity::{Module, Call, Storage, Event<T>, Config<T>},
        Bridge: bridge::{Module, Call, Storage, Config<T>, Event<T>},
        ComplianceManager: compliance_manager::{Module, Call, Storage, Event},
        Voting: voting::{Module, Call, Storage, Event<T>},
        StoCapped: sto_capped::{Module, Call, Storage, Event<T>},
        Exemption: exemption::{Module, Call, Storage, Event},
        Settlement: settlement::{Module, Call, Storage, Event<T>, Config},
        Sto: sto::{Module, Call, Storage, Event<T>},
        CddServiceProviders: group::<Instance2>::{Module, Call, Storage, Event<T>, Config<T>},
        Statistic: statistics::{Module, Call, Storage},
        ProtocolFee: protocol_fee::{Module, Call, Storage, Event<T>, Config<T>},
        Utility: utility::{Module, Call, Storage, Event},
        // Comment it in the favour of Testnet v1 release
        // CddOffchainWorker: pallet_cdd_offchain_worker::{Module, Call, Storage, ValidateUnsigned, Event<T>}
        Portfolio: portfolio::{Module, Call, Storage, Event<T>},
        Confidential: confidential::{Module, Call, Storage, Event },
    }
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    pallet_grandpa::ValidateEquivocationReport<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = pallet_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_report_equivocation_extrinsic(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::SlotNumber {
            Babe::current_epoch_start()
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
        for Runtime
    {
        fn call(
            origin: AccountId,
            dest: AccountId,
            value: Balance,
            gas_limit: u64,
            input_data: Vec<u8>,
        ) -> ContractExecResult {
            let exec_result =
                Contracts::bare_call(origin, dest, value, gas_limit, input_data);
            match exec_result {
                Ok(v) => ContractExecResult::Success {
                    status: v.status,
                    data: v.data,
                },
                Err(_) => ContractExecResult::Error,
            }
        }

        fn get_storage(
            address: AccountId,
            key: [u8; 32],
        ) -> pallet_contracts_primitives::GetStorageResult {
            Contracts::get_storage(address, key)
        }

        fn rent_projection(
            address: AccountId,
        ) -> pallet_contracts_primitives::RentProjectionResult<BlockNumber> {
            Contracts::rent_projection(address)
        }
    }

    impl node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<
        Block,
        Balance,
        UncheckedExtrinsic,
    > for Runtime {
        fn query_info(uxt: UncheckedExtrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl pallet_staking_rpc_runtime_api::StakingApi<Block> for Runtime {
        fn get_curve() -> Vec<(Perbill, Perbill)> {
            Staking::get_curve()
        }
    }

    impl node_rpc_runtime_api::pips::PipsApi<Block, AccountId, Balance>
    for Runtime
    {
        /// Get vote count for a given proposal index
        fn get_votes(index: u32) -> VoteCount<Balance> {
            Pips::get_votes(index)
        }

        /// Proposals voted by `address`
        fn proposed_by(address: AccountId) -> Vec<u32> {
            Pips::proposed_by(pallet_pips::Proposer::Community(address))
        }

        /// Proposals `address` voted on
        fn voted_on(address: AccountId) -> Vec<u32> {
            Pips::voted_on(address)
        }

        /// Retrieve referendums voted on information by `address` account.
        fn voting_history_by_address(address: AccountId) -> HistoricalVotingByAddress<Vote<Balance>> {
            Pips::voting_history_by_address(address)

        }

        /// Retrieve referendums voted on information by `id` identity (and its secondary items).
        fn voting_history_by_id(id: IdentityId) -> HistoricalVotingById<AccountId, Vote<Balance>> {
            Pips::voting_history_by_id(id)
        }
    }

    impl pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<
        Block,
    > for Runtime {
        fn compute_fee(op: ProtocolOp) -> CappedFee {
            ProtocolFee::compute_fee(op).into()
        }
    }

    impl
        node_rpc_runtime_api::identity::IdentityApi<
            Block,
            IdentityId,
            Ticker,
            AccountId,
            SecondaryKey<AccountId>,
            Signatory<AccountId>,
            Moment
        > for Runtime
    {
        /// RPC call to know whether the given did has valid cdd claim or not
        fn is_identity_has_valid_cdd(did: IdentityId, leeway: Option<u64>) -> CddStatus {
            Identity::fetch_cdd(did, leeway.unwrap_or_default())
                .ok_or_else(|| "Either cdd claim is expired or not yet provided to give identity".into())
        }

        /// RPC call to query the given ticker did
        fn get_asset_did(ticker: Ticker) -> AssetDidResult {
            match Identity::get_asset_did(ticker) {
                Ok(did) => Ok(did),
                Err(_) => Err("Error in computing the given ticker error".into()),
            }
        }

        /// Retrieve primary key and secondary keys for a given IdentityId
        fn get_did_records(did: IdentityId) -> DidRecords<AccountId, SecondaryKey<AccountId>> {
            Identity::get_did_records(did)
        }

        /// Retrieve the status of the DIDs
        fn get_did_status(dids: Vec<IdentityId>) -> Vec<DidStatus> {
            Identity::get_did_status(dids)
        }

        /// Retrieve list of a authorization for a given signatory
        fn get_filtered_authorizations(
            signatory: Signatory<AccountId>,
            allow_expired: bool,
            auth_type: Option<AuthorizationType>
        ) -> Vec<Authorization<AccountId, Moment>> {
            Identity::get_filtered_authorizations(signatory, allow_expired, auth_type)
        }
    }

    impl node_rpc_runtime_api::asset::AssetApi<Block, AccountId> for Runtime {
        #[inline]
        fn can_transfer(
            sender: AccountId,
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            value: Balance) -> node_rpc_runtime_api::asset::CanTransferResult
        {
            Asset::unsafe_can_transfer(sender, ticker, from_did, to_did, value)
                .map_err(|msg| msg.as_bytes().to_vec())
        }
    }

    impl node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId, Balance>
        for Runtime
    {
        #[inline]
        fn can_transfer(
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            primary_issuance_agent: Option<IdentityId>,
        ) -> AssetTransferRulesResult
        {
            ComplianceManager::granular_verify_restriction(&ticker, from_did, to_did, primary_issuance_agent)
        }
    }

    impl pallet_group_rpc_runtime_api::GroupApi<Block> for Runtime {
        fn get_cdd_valid_members() -> Vec<pallet_group_rpc_runtime_api::Member> {
            merge_active_and_inactive::<Block>(
                CddServiceProviders::active_members(),
                CddServiceProviders::inactive_members())
        }

        fn get_gc_valid_members() -> Vec<pallet_group_rpc_runtime_api::Member> {
            merge_active_and_inactive::<Block>(
                CommitteeMembership::active_members(),
                CommitteeMembership::inactive_members())
        }
    }

    impl node_rpc_runtime_api::portfolio::PortfolioApi<Block, Balance> for Runtime {
        #[inline]
        fn get_portfolios(did: IdentityId) -> node_rpc_runtime_api::portfolio::GetPortfoliosResult {
            Ok(Portfolio::rpc_get_portfolios(did))
        }

        #[inline]
        fn get_portfolio_assets(portfolio_id: PortfolioId) ->
            node_rpc_runtime_api::portfolio::GetPortfolioAssetsResult<Balance>
        {
            Ok(Portfolio::rpc_get_portfolio_assets(portfolio_id))
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            pallet: Vec<u8>,
            benchmark: Vec<u8>,
            lowest_range_values: Vec<u32>,
            highest_range_values: Vec<u32>,
            steps: Vec<u32>,
            repeat: u32,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark};

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&pallet, &benchmark, &lowest_range_values, &highest_range_values, &steps, repeat);

            add_benchmark!(params, batches, b"asset", Asset);
            add_benchmark!(params, batches, b"identity", Identity);
            add_benchmark!(params, batches, b"im-online", ImOnline);
            add_benchmark!(params, batches, b"staking", Staking);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}
