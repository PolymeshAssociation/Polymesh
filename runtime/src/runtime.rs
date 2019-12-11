use crate::{
    asset, balances,
    constants::{currency::*, time::*},
    contracts_wrapper, dividend, exemption, general_tm, identity,
    impls::{CurrencyToVoteHandler, ToAuthor, WeightMultiplierUpdateHandler, WeightToFee},
    percentage_tm, registry, simple_token, staking, sto_capped,
    update_did_signed_extension::UpdateDid,
    utils, voting,
};
use primitives::{AccountId, AccountIndex, Balance, BlockNumber, Hash, Moment, Nonce, Signature};

use authority_discovery_primitives::{
    AuthorityId as EncodedAuthorityId, Signature as EncodedSignature,
};
use babe_primitives::AuthorityId as BabeId;
use client::{
    block_builder::api::{self as block_builder_api, CheckInherentsResult, InherentData},
    impl_runtime_apis, runtime_api as client_api,
};
use codec::{Decode, Encode};
pub use contracts::Gas;
use elections::VoteIndex;
use grandpa::{fg_primitives, AuthorityId as GrandpaId};
use im_online::sr25519::{AuthorityId as ImOnlineId, AuthoritySignature as ImOnlineSignature};
use rstd::prelude::*;
use sr_primitives::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic, impl_opaque_keys, key_types,
    traits::{BlakeTwo256, Block as BlockT, StaticLookup},
    transaction_validity::TransactionValidity,
    weights::Weight,
    AnySignature, ApplyResult,
};
use sr_staking_primitives::SessionIndex;
use srml_support::{
    construct_runtime, parameter_types,
    traits::{Currency, SplitTwoWays},
};
use substrate_primitives::{
    u32_trait::{_1, _2, _3, _4},
    OpaqueMetadata,
};

#[cfg(any(feature = "std", test))]
use version::NativeVersion;
use version::RuntimeVersion;

pub use balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use sr_primitives::BuildStorage;
pub use sr_primitives::{Perbill, Permill};
pub use srml_support::StorageValue;
use system::offchain::TransactionSubmitter;
pub use timestamp::Call as TimestampCall;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("polymesh"),
    impl_name: create_runtime_str!("polymath-polymesh"),
    authoring_version: 1,
    spec_version: 1002,
    impl_version: 0,
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
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
    type Origin = Origin;
    type Call = Call;
    type Index = Nonce;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = Indices;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type WeightMultiplierUpdate = WeightMultiplierUpdateHandler;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = Version;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl babe::Trait for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
}

impl indices::Trait for Runtime {
    type IsDeadAccount = Balances;
    type AccountIndex = AccountIndex;
    type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    type Event = Event;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 10 * CENTS;
    pub const TransferFee: Balance = 1 * CENTS;
    pub const CreationFee: Balance = 1 * CENTS;
    pub const TransactionBaseFee: Balance = 1 * CENTS;
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
}

/// Splits fees 80/20 between treasury and block author.
pub type DealWithFees = SplitTwoWays<
    Balance,
    NegativeImbalance,
    _4,
    Treasury, // 4 parts (80%) goes to the treasury.
    _1,
    ToAuthor, // 1 part (20%) goes to the block author.
>;

impl balances::Trait for Runtime {
    type Balance = Balance;
    type OnFreeBalanceZero = ((Staking, Contracts), Session);
    type OnNewAccount = Indices;
    type Event = Event;
    type TransactionPayment = DealWithFees;
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = WeightToFee;
    type Identity = Identity;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

// TODO: substrate#2986 implement this properly
impl authorship::Trait for Runtime {
    type FindAuthor = session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = Staking;
}

parameter_types! {
    pub const Period: BlockNumber = 10 * MINUTES;
    pub const Offset: BlockNumber = 0;
}

type SessionHandlers = (Grandpa, Babe, ImOnline, AuthorityDiscovery);
impl_opaque_keys! {
    pub struct SessionKeys {
        #[id(key_types::GRANDPA)]
        pub grandpa: GrandpaId,
        #[id(key_types::BABE)]
        pub babe: BabeId,
        #[id(key_types::IM_ONLINE)]
        pub im_online: ImOnlineId,
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

impl session::Trait for Runtime {
    type OnSessionEnding = Staking;
    type SessionHandler = SessionHandlers;
    type ShouldEndSession = Babe;
    type Event = Event;
    type Keys = SessionKeys;
    type SelectInitialValidators = Staking;
    type ValidatorId = AccountId;
    type ValidatorIdOf = staking::StashOf<Self>;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl session::historical::Trait for Runtime {
    type FullIdentification = staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = staking::ExposureOf<Self>;
}

srml_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    // Six sessions in an era (24 hours).
    pub const SessionsPerEra: SessionIndex = 6;
    // 28 eras for unbonding (28 days).
    pub const BondingDuration: staking::EraIndex = 28;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
}

impl staking::Trait for Runtime {
    type OnRewardMinted = Treasury;
    type CurrencyToVote = CurrencyToVoteHandler;
    type Event = Event;
    type Currency = Balances;
    type Slash = Treasury;
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SessionInterface = Self;
    type Time = Timestamp;
    type RewardCurve = RewardCurve;
    type AddOrigin = collective::EnsureProportionMoreThan<_2, _3, AccountId, GovernanceCollective>;
    type RemoveOrigin =
        collective::EnsureProportionMoreThan<_2, _3, AccountId, GovernanceCollective>;
    type ComplianceOrigin =
        collective::EnsureProportionMoreThan<_2, _3, AccountId, GovernanceCollective>;
}

type GovernanceCollective = collective::Instance1;
impl collective::Trait<GovernanceCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
}

impl membership::Trait<membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, GovernanceCollective>;
    type RemoveOrigin =
        collective::EnsureProportionMoreThan<_1, _2, AccountId, GovernanceCollective>;
    type SwapOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, GovernanceCollective>;
    type ResetOrigin =
        collective::EnsureProportionMoreThan<_1, _2, AccountId, GovernanceCollective>;
    type MembershipInitialized = GovernanceCommittee;
    type MembershipChanged = GovernanceCommittee;
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

impl contracts::Trait for Runtime {
    type Currency = Balances;
    type Call = Call;
    type Event = Event;
    type DetermineContractAddress = contracts::SimpleAddressDeterminator<Runtime>;
    type ComputeDispatchFee = contracts::DefaultDispatchFeeComputor<Runtime>;
    type TrieIdGenerator = contracts::TrieIdFromParentCounter<Runtime>;
    type GasPayment = ();
    type SignedClaimHandicap = contracts::DefaultSignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type StorageSizeOffset = contracts::DefaultStorageSizeOffset;
    type RentByteFee = RentByteFee;
    type RentDepositOffset = RentDepositOffset;
    type SurchargeReward = SurchargeReward;
    type TransferFee = ContractTransferFee;
    type CreationFee = ContractCreationFee;
    type TransactionBaseFee = ContractTransactionBaseFee;
    type TransactionByteFee = ContractTransactionByteFee;
    type ContractFee = ContractFee;
    type CallBaseFee = contracts::DefaultCallBaseFee;
    type InstantiateBaseFee = contracts::DefaultInstantiateBaseFee;
    type MaxDepth = contracts::DefaultMaxDepth;
    type MaxValueSize = contracts::DefaultMaxValueSize;
    type BlockGasLimit = contracts::DefaultBlockGasLimit;
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

impl treasury::Trait for Runtime {
    type Currency = Balances;
    type ApproveOrigin =
        collective::EnsureProportionAtLeast<_2, _3, AccountId, GovernanceCollective>;
    type RejectOrigin =
        collective::EnsureProportionMoreThan<_1, _2, AccountId, GovernanceCollective>;
    type Event = Event;
    type MintedForSpending = ();
    type ProposalRejection = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
}

impl offences::Trait for Runtime {
    type Event = Event;
    type IdentificationTuple = session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

type SubmitTransaction = TransactionSubmitter<ImOnlineId, Runtime, UncheckedExtrinsic>;
impl im_online::Trait for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type Call = Call;
    type SubmitTransaction = SubmitTransaction;
    type ReportUnresponsiveness = ();
}

impl grandpa::Trait for Runtime {
    type Event = Event;
}

impl authority_discovery::Trait for Runtime {}

parameter_types! {
    pub const WindowSize: BlockNumber = finality_tracker::DEFAULT_WINDOW_SIZE.into();
    pub const ReportLatency: BlockNumber = finality_tracker::DEFAULT_REPORT_LATENCY.into();
}

impl finality_tracker::Trait for Runtime {
    type OnFinalizationStalled = ();
    type WindowSize = WindowSize;
    type ReportLatency = ReportLatency;
}

parameter_types! {
    pub const Prefix: &'static [u8] = b"Pay POLY to the Polymesh account:";
}

impl sudo::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
}

impl asset::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
}

impl utils::Trait for Runtime {
    type OffChainSignature = AnySignature;
    fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId {
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
}

impl contracts_wrapper::Trait for Runtime {}

impl exemption::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl dividend::Trait for Runtime {
    type Event = Event;
}

impl registry::Trait for Runtime {}

construct_runtime!(
    pub enum Runtime where
    Block = Block,
    NodeBlock = primitives::Block,
    UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff; balances is uncallable initially.
        System: system::{Module, Call, Storage, Config, Event},

        // Must be before session.
        Babe: babe::{Module, Call, Storage, Config, Inherent(Timestamp)},

        Timestamp: timestamp::{Module, Call, Storage, Inherent},
        Indices: indices,
        Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},

        // Consensus srml_support.
        Authorship: authorship::{Module, Call, Storage},
        Staking: staking::{default, OfflineWorker},
        Offences: offences::{Module, Call, Storage, Event},
        Session: session::{Module, Call, Storage, Event, Config<T>},
        FinalityTracker: finality_tracker::{Module, Call, Inherent},
        Grandpa: grandpa::{Module, Call, Storage, Config, Event},
        ImOnline: im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
        AuthorityDiscovery: authority_discovery::{Module, Call, Config<T>},

        // Sudo. Usable initially.
        // RELEASE: remove this for release build.
        Sudo: sudo,

        // Contracts
        Contracts: contracts::{Module, Call, Storage, Config<T>, Event<T>},
        // ContractsWrapper: contracts_wrapper::{Module, Call, Storage},

        // Polymesh Governance Committees
        Treasury: treasury::{Module, Call, Storage, Event<T>},
        GovernanceMembership: membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        GovernanceCommittee: collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},

        //Polymesh
        Asset: asset::{Module, Call, Storage, Config<T>, Event<T>},
        Dividend: dividend::{Module, Call, Storage, Event<T>},
        Registry: registry::{Module, Call, Storage},
        Identity: identity::{Module, Call, Storage, Event<T>, Config<T>},
        GeneralTM: general_tm::{Module, Call, Storage, Event},
        Voting: voting::{Module, Call, Storage, Event<T>},
        STOCapped: sto_capped::{Module, Call, Storage, Event<T>},
        PercentageTM: percentage_tm::{Module, Call, Storage, Event<T>},
        Exemption: exemption::{Module, Call, Storage, Event},
        SimpleToken: simple_token::{Module, Call, Storage, Event<T>, Config<T>},
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
    system::CheckVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    balances::TakeFees<Runtime>,
    contracts::CheckBlockGasLimit<Runtime>,
    UpdateDid<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Nonce, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
    impl client_api::Core<Block> for Runtime {
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

    impl client_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
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
            System::random_seed()
        }
    }

    impl client_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(number: sr_primitives::traits::NumberFor<Block>) {
            Executive::offchain_worker(number)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> Vec<(GrandpaId, u64)> {
            Grandpa::grandpa_authorities()
        }
    }

    impl babe_primitives::BabeApi<Block> for Runtime {
        fn configuration() -> babe_primitives::BabeConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            babe_primitives::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                secondary_slots: true,
            }
        }
    }

    impl authority_discovery_primitives::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<EncodedAuthorityId> {
            AuthorityDiscovery::authorities().into_iter()
                .map(|id| id.encode())
                .map(EncodedAuthorityId)
                .collect()
        }

        fn sign(payload: &Vec<u8>) -> Option<(EncodedSignature, EncodedAuthorityId)> {
            AuthorityDiscovery::sign(payload).map(|(sig, id)| {
                (EncodedSignature(sig.encode()), EncodedAuthorityId(id.encode()))
            })
        }

        fn verify(payload: &Vec<u8>, signature: &EncodedSignature, authority_id: &EncodedAuthorityId) -> bool {
            let signature = match ImOnlineSignature::decode(&mut &signature.0[..]) {
                Ok(s) => s,
                _ => return false,
            };

            let authority_id = match ImOnlineId::decode(&mut &authority_id.0[..]) {
                Ok(id) => id,
                _ => return false,
            };

            AuthorityDiscovery::verify(payload, signature, authority_id)
        }
    }

    impl substrate_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            let seed = seed.as_ref().map(|s| rstd::str::from_utf8(&s).expect("Seed is an utf8 string"));
            SessionKeys::generate(seed)
        }
    }
}
