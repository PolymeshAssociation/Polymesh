#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::{
    asset, bridge, contracts_wrapper, dividend, exemption, general_tm,
    impls::{Author, CurrencyToVoteHandler, LinearWeightToFee, TargetedFeeAdjustment},
    multisig, percentage_tm, simple_token, statistics, sto_capped,
    update_did_signed_extension::UpdateDid,
    utils, voting,
};

use pallet_committee as committee;
use polymesh_primitives::{
    AccountId, AccountIndex, AccountKey, Balance, BlockNumber, Hash, IdentityId, Index, Moment,
    Signature, SigningItem, Ticker,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::{
    constants::{currency::*, fee::*, time::*},
    CommonTrait,
};
use polymesh_runtime_group as group;
use polymesh_runtime_identity as identity;

use frame_support::{
    construct_runtime, parameter_types,
    traits::{Currency, Randomness, SplitTwoWays},
    weights::Weight,
};
use pallet_elections::VoteIndex;
use sp_api::impl_runtime_apis;
use sp_core::u32_trait::{_1, _2, _4};
use sp_offchain;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::transaction_validity::TransactionValidity;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys, ApplyExtrinsicResult, Perbill, Permill,
};
use sp_runtime::{
    traits::{BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys, StaticLookup, Verify},
    MultiSignature,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

use frame_system::offchain::TransactionSubmitter;
use pallet_contracts_rpc_runtime_api::ContractExecResult;
use pallet_grandpa::{fg_primitives, AuthorityList as GrandpaAuthorityList};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
use polymesh_runtime_identity_rpc_runtime_api::{AssetDidResult, CddStatus, DidRecords};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe;
use sp_core::OpaqueMetadata;
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

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("polymesh"),
    impl_name: create_runtime_str!("polymath-polymesh"),
    authoring_version: 1,
    spec_version: 1002,
    impl_version: 1002,
    apis: RUNTIME_API_VERSIONS,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

pub type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000_000;
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for Runtime {
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

impl pallet_indices::Trait for Runtime {
    type IsDeadAccount = Balances;
    type AccountIndex = AccountIndex;
    type ResolveHint = pallet_indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    type Event = Event;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 0u128.into();
    pub const TransferFee: Balance = 1 * CENTS;
    pub const CreationFee: Balance = 1 * CENTS;
}

/// Splits fees 80/20 between treasury and block author.
pub type DealWithFees = SplitTwoWays<
    Balance,
    NegativeImbalance,
    _4,
    Treasury, // 4 parts (80%) goes to the treasury.
    _1,
    Author, // 1 part (20%) goes to the block author.
>;

impl CommonTrait for Runtime {
    type Balance = Balance;
    type CreationFee = CreationFee;
    type AcceptTransferTarget = Asset;
    type BlockRewardsReserve = balances::Module<Runtime>;
}

impl balances::Trait for Runtime {
    type OnFreeBalanceZero = ((Staking, Contracts), Session);
    type OnNewAccount = Indices;
    type Event = Event;
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type Identity = Identity;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 1 * CENTS;
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
    // setting this to zero will disable the weight fee.
    pub const WeightFeeCoefficient: Balance = 1_000;
    // for a sane configuration, this should always be less than `AvailableBlockRatio`.
    pub const TargetBlockFullness: Perbill = TARGET_BLOCK_FULLNESS;
}

impl pallet_transaction_payment::Trait for Runtime {
    type Currency = Balances;
    type OnTransactionPayment = DealWithFees;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = LinearWeightToFee<WeightFeeCoefficient>;
    type FeeMultiplierUpdate = TargetedFeeAdjustment<TargetBlockFullness>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

// TODO: substrate#2986 implement this properly
impl pallet_authorship::Trait for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = Staking;
}

parameter_types! {
    pub const Period: BlockNumber = 10 * MINUTES;
    pub const Offset: BlockNumber = 0;
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
    type OnSessionEnding = Staking;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type ShouldEndSession = Babe;
    type Event = Event;
    type Keys = SessionKeys;
    type ValidatorId = AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type SelectInitialValidators = Staking;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl pallet_session::historical::Trait for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Self>;
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_500_000,
        max_inflation: 1_000_000,
        ideal_stake: 0_100_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    // Six sessions in an era (24 hours).
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    // 28 eras for unbonding (28 days).
    pub const BondingDuration: pallet_staking::EraIndex = 28;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
}

impl pallet_staking::Trait for Runtime {
    type Currency = Balances;
    type Time = Timestamp;
    type CurrencyToVote = CurrencyToVoteHandler;
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
    type RequiredAddOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredRemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredComplianceOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredCommissionOrigin = frame_system::EnsureRoot<AccountId>;
}

type GovernanceCommittee = committee::Instance1;
impl committee::Trait<GovernanceCommittee> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type Event = Event;
}

/// PolymeshCommittee as an instance of group
impl group::Trait<group::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = PolymeshCommittee;
    type MembershipChanged = PolymeshCommittee;
}

impl pallet_mips::Trait for Runtime {
    type Currency = Balances;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type VotingMajorityOrigin =
        committee::EnsureProportionAtLeast<_1, _2, AccountId, GovernanceCommittee>;
    type GovernanceCommittee = PolymeshCommittee;
    type Event = Event;
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
    pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
    pub const EmergencyVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
    pub const MinimumDeposit: Balance = 100 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
    pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
}

parameter_types! {
    pub const ContractTransferFee: Balance = 9999999999 * DOLLARS;
    pub const ContractCreationFee: Balance = 1 * CENTS;
    pub const ContractTransactionBaseFee: Balance = 1 * CENTS;
    pub const ContractTransactionByteFee: Balance = 10 * MILLICENTS;
    pub const ContractFee: Balance = 1 * CENTS;
    pub const TombstoneDeposit: Balance = 1 * DOLLARS;
    pub const RentByteFee: Balance = 1 * DOLLARS;
    pub const RentDepositOffset: Balance = 1000 * DOLLARS;
    pub const SurchargeReward: Balance = 150 * DOLLARS;
}

impl pallet_contracts::Trait for Runtime {
    type Currency = Balances;
    type Time = Timestamp;
    type Randomness = RandomnessCollectiveFlip;
    type Call = Call;
    type Event = Event;
    type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminator<Runtime>;
    type ComputeDispatchFee = pallet_contracts::DefaultDispatchFeeComputor<Runtime>;
    type TrieIdGenerator = pallet_contracts::TrieIdFromParentCounter<Runtime>;
    type GasPayment = ();
    type RentPayment = ();
    type SignedClaimHandicap = pallet_contracts::DefaultSignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type StorageSizeOffset = pallet_contracts::DefaultStorageSizeOffset;
    type RentByteFee = RentByteFee;
    type RentDepositOffset = RentDepositOffset;
    type SurchargeReward = SurchargeReward;
    type TransferFee = ContractTransferFee;
    type CreationFee = ContractCreationFee;
    type TransactionBaseFee = ContractTransactionBaseFee;
    type TransactionByteFee = ContractTransactionByteFee;
    type ContractFee = ContractFee;
    type CallBaseFee = pallet_contracts::DefaultCallBaseFee;
    type InstantiateBaseFee = pallet_contracts::DefaultInstantiateBaseFee;
    type MaxDepth = pallet_contracts::DefaultMaxDepth;
    type MaxValueSize = pallet_contracts::DefaultMaxValueSize;
    type BlockGasLimit = pallet_contracts::DefaultBlockGasLimit;
}

parameter_types! {
    pub const CandidacyBond: Balance = 10 * DOLLARS;
    pub const VotingBond: Balance = 1 * DOLLARS;
    pub const VotingFee: Balance = 2 * DOLLARS;
    pub const MinimumVotingLock: Balance = 1 * DOLLARS;
    pub const PresentSlashPerVoter: Balance = 1 * CENTS;
    pub const CarryCount: u32 = 6;
    // one additional vote should go by before an inactive voter can be reaped.
    pub const InactiveGracePeriod: VoteIndex = 1;
    pub const ElectionsVotingPeriod: BlockNumber = 2 * DAYS;
    pub const DecayRatio: u32 = 0;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 100 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 24 * DAYS;
    pub const Burn: Permill = Permill::from_percent(5);
}

impl pallet_treasury::Trait for Runtime {
    type Currency = Balances;
    type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
    type RejectOrigin = frame_system::EnsureRoot<AccountId>;
    type Event = Event;
    type ProposalRejection = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
}

impl pallet_offences::Trait for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

type SubmitTransaction = TransactionSubmitter<ImOnlineId, Runtime, UncheckedExtrinsic>;

parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_BLOCKS as _;
}

impl pallet_im_online::Trait for Runtime {
    type AuthorityId = ImOnlineId;
    type Call = Call;
    type Event = Event;
    type SubmitTransaction = SubmitTransaction;
    type ReportUnresponsiveness = Offences;
    type SessionDuration = SessionDuration;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
}

impl pallet_grandpa::Trait for Runtime {
    type Event = Event;
}

impl pallet_authority_discovery::Trait for Runtime {}

parameter_types! {
    pub const WindowSize: BlockNumber = pallet_finality_tracker::DEFAULT_WINDOW_SIZE.into();
    pub const ReportLatency: BlockNumber = pallet_finality_tracker::DEFAULT_REPORT_LATENCY.into();
}

impl pallet_finality_tracker::Trait for Runtime {
    type OnFinalizationStalled = ();
    type WindowSize = WindowSize;
    type ReportLatency = ReportLatency;
}

parameter_types! {
    pub const Prefix: &'static [u8] = b"Pay POLY to the Polymesh account:";
}

impl pallet_sudo::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
}

impl multisig::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    pub const MaxTimelockedTxsPerBlock: u32 = 10;
    pub const BlockRangeForTimelock: BlockNumber = 1000;
}

impl bridge::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
    type MaxTimelockedTxsPerBlock = MaxTimelockedTxsPerBlock;
    type BlockRangeForTimelock = BlockRangeForTimelock;
}

impl asset::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
}

impl utils::Trait for Runtime {
    type Public = <MultiSignature as Verify>::Signer;
    type OffChainSignature = MultiSignature;
    fn validator_id_to_account_id(
        v: <Self as pallet_session::Trait>::ValidatorId,
    ) -> Self::AccountId {
        v
    }
}

impl simple_token::Trait for Runtime {
    type Event = Event;
}

impl general_tm::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl voting::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl sto_capped::Trait for Runtime {
    type Event = Event;
    type SimpleTokenTrait = SimpleToken;
}

impl percentage_tm::Trait for Runtime {
    type Event = Event;
}

impl identity::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
    type AddSignerMultiSigTarget = MultiSig;
    type CddServiceProviders = CddServiceProviders;
    type Balances = balances::Module<Runtime>;
}

impl contracts_wrapper::Trait for Runtime {}

impl exemption::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl dividend::Trait for Runtime {
    type Event = Event;
}

/// CddProviders asan instance of group
impl group::Trait<group::Instance2> for Runtime {
    type Event = Event;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
}

impl statistics::Trait for Runtime {}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = polymesh_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Storage, Config, Event},
        // Must be before session.
        Babe: pallet_babe::{Module, Call, Storage, Config, Inherent(Timestamp)},

        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Indices: pallet_indices,
        Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},

        // Consensus frame_support.
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},
        Staking: pallet_staking::{default, OfflineWorker},
        Offences: pallet_offences::{Module, Call, Storage, Event},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        FinalityTracker: pallet_finality_tracker::{Module, Call, Inherent},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
        ImOnline: pallet_im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
        AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        // Sudo. Usable initially.
        // RELEASE: remove this for release build.
        Sudo: pallet_sudo,

        MultiSig: multisig::{Module, Call, Storage, Event<T>},

        // Contracts
        Contracts: pallet_contracts,
        // ContractsWrapper: contracts_wrapper::{Module, Call, Storage},

        // Polymesh Governance Committees
        Treasury: pallet_treasury::{Module, Call, Storage, Event<T>},
        PolymeshCommittee: committee::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        CommitteeMembership: group::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        Mips: pallet_mips::{Module, Call, Storage, Event<T>, Config<T>},

        //Polymesh
        Asset: asset::{Module, Call, Storage, Config<T>, Event<T>},
        Bridge: bridge::{Module, Call, Storage, Config<T>, Event<T>},
        Dividend: dividend::{Module, Call, Storage, Event<T>},
        Identity: identity::{Module, Call, Storage, Event<T>, Config<T>},
        GeneralTM: general_tm::{Module, Call, Storage, Event},
        Voting: voting::{Module, Call, Storage, Event<T>},
        StoCapped: sto_capped::{Module, Call, Storage, Event<T>},
        PercentageTM: percentage_tm::{Module, Call, Storage, Event<T>},
        Exemption: exemption::{Module, Call, Storage, Event},
        SimpleToken: simple_token::{Module, Call, Storage, Event<T>, Config<T>},
        CddServiceProviders: group::<Instance2>::{Module, Call, Storage, Event<T>, Config<T>},
        Statistic: statistics::{Module, Call, Storage },
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
    frame_system::CheckVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    pallet_contracts::CheckBlockGasLimit<Runtime>,
    UpdateDid<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
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
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(number: NumberFor<Block>) {
            Executive::offchain_worker(number)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                secondary_slots: true,
            }
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

    impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance> for Runtime {
        fn call(
            origin: AccountId,
            dest: AccountId,
            value: Balance,
            gas_limit: u64,
            input_data: Vec<u8>,
        ) -> ContractExecResult {
            let exec_result = Contracts::bare_call(
                origin,
                dest.into(),
                value,
                gas_limit,
                input_data,
            );
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
        ) -> pallet_contracts_rpc_runtime_api::GetStorageResult {
            Contracts::get_storage(address, key).map_err(|rpc_err| {
                use pallet_contracts::GetStorageError;
                use pallet_contracts_rpc_runtime_api::{GetStorageError as RpcGetStorageError};
                /// Map the contract error into the RPC layer error.
                match rpc_err {
                    GetStorageError::ContractDoesntExist => RpcGetStorageError::ContractDoesntExist,
                    GetStorageError::IsTombstone => RpcGetStorageError::IsTombstone,
                }
            })
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
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
    }

    impl pallet_staking_rpc_runtime_api::StakingApi<Block> for Runtime {
        fn get_curve() -> Vec<(Perbill, Perbill)> {
            Staking::get_curve()
        }
    }

    impl pallet_mips_rpc_runtime_api::MipsApi<Block, AccountId, Balance> for Runtime {
        /// Get vote count for a given proposal index
        fn get_votes(index: u32) -> pallet_mips_rpc_runtime_api::VoteCount<Balance> {
            Mips::get_votes(index)
        }

        /// Proposals voted by `address`
        fn proposed_by(address: AccountId) -> Vec<u32> {
            Mips::proposed_by(address)
        }

        /// Proposals `address` voted on
        fn voted_on(address: AccountId) -> Vec<u32> {
            Mips::voted_on(address)
        }
    }

    impl
        polymesh_runtime_identity_rpc_runtime_api::IdentityApi<
            Block,
            IdentityId,
            Ticker,
            AccountKey,
            SigningItem,
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

        /// Retrieve master key and signing keys for a given IdentityId
        fn get_did_records(did: IdentityId) -> DidRecords<AccountKey, SigningItem> {
            Identity::get_did_records(did)
        }
    }
}
