use super::ext_builder::{EXTRINSIC_BASE_WEIGHT, TRANSACTION_BYTE_FEE, WEIGHT_TO_FEE};
use codec::Encode;
use frame_support::{
    assert_ok,
    dispatch::{DispatchInfo, DispatchResult, Weight},
    parameter_types,
    traits::{Currency, Imbalance, KeyOwnerProofSystem, OnInitialize, OnUnbalanced},
    weights::{
        RuntimeDbWeight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    StorageDoubleMap,
};
use frame_system::{EnsureRoot, RawOrigin};
use lazy_static::lazy_static;
use pallet_asset::checkpoint as pallet_checkpoint;
use pallet_balances as balances;
use pallet_committee as committee;
use pallet_corporate_actions as corporate_actions;
use pallet_corporate_actions::ballot as corporate_ballots;
use pallet_corporate_actions::distribution as capital_distributions;
use pallet_group as group;
use pallet_identity as identity;
use pallet_multisig as multisig;
use pallet_pips as pips;
use pallet_portfolio as portfolio;
use pallet_protocol_fee as protocol_fee;
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::RuntimeDispatchInfo;
use pallet_utility;
use polymesh_common_utilities::{
    constants::currency::{DOLLARS, POLY},
    protocol_fee::ProtocolOp,
    traits::{
        group::GroupTrait,
        transaction_payment::{CddAndFeeDetails, ChargeTxFee},
    },
    Context,
};
use polymesh_primitives::{
    investor_zkproof_data::v1::InvestorZKProofData, AccountId, Authorization, AuthorizationData,
    BlockNumber, CddId, Claim, InvestorUid, Moment, Permissions as AuthPermissions,
    PortfolioNumber, Scope, ScopeId, SecondaryKey, TrustedFor, TrustedIssuer,
};
use polymesh_runtime_common::{
    merge_active_and_inactive,
    runtime::{BENCHMARK_MAX_INCREASE, VMO},
    AvailableBlockRatio, MaximumBlockWeight,
};
use polymesh_runtime_develop::constants::time::{EPOCH_DURATION_IN_BLOCKS, MILLISECS_PER_BLOCK};
use smallvec::smallvec;
use sp_core::{
    crypto::{key_types, Pair as PairTrait},
    sr25519::Pair,
    H256,
};
use sp_runtime::generic::Era;
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    testing::UintAuthorityId,
    traits::{
        BlakeTwo256, Block as BlockT, Extrinsic, IdentityLookup, NumberFor, OpaqueKeys,
        StaticLookup, Verify,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionValidity, ValidTransaction,
    },
    AnySignature, KeyTypeId, Perbill, Permill,
};
use sp_std::{collections::btree_set::BTreeSet, iter};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use std::cell::RefCell;
use std::convert::From;
use test_client::AccountKeyring;

lazy_static! {
    pub static ref INTEGRATION_TEST: bool = std::env::var("INTEGRATION_TEST")
        .map(|var| var.parse().unwrap_or(false))
        .unwrap_or(false);
}

#[macro_export]
macro_rules! exec_ok {
    ( $x:expr $(,)? ) => {
        frame_support::assert_ok!(polymesh_exec_macro::exec!($x))
    };
    ( $x:expr, $y:expr $(,)? ) => {
        frame_support::assert_ok!(polymesh_exec_macro::exec!($x), $y)
    };
}

#[macro_export]
macro_rules! exec_noop {
    (
		$x:expr,
		$y:expr $(,)?
	) => {
        // Use `assert_err` when running with `INTEGRATION_TEST`.
        // `assert_noop` returns false positives when using full extrinsic execution.
        if *crate::storage::INTEGRATION_TEST {
            frame_support::assert_err!(polymesh_exec_macro::exec!($x), $y);
        } else {
            frame_support::assert_noop!(polymesh_exec_macro::exec!($x), $y);
        }
    };
}

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
const GENESIS_HASH: [u8; 32] = [69u8; 32];
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("test-storage"),
    impl_name: create_runtime_str!("test-storage"),
    authoring_version: 1,
    // Per convention: if the runtime behavior changes, increment spec_version
    // and set impl_version to 0. If only runtime
    // implementation changes and behavior does not, then leave spec_version as
    // is and increment impl_version.
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 7,
    state_version: 1,
};

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

impl From<UintAuthorityId> for MockSessionKeys {
    fn from(dummy: UintAuthorityId) -> Self {
        Self { dummy }
    }
}

type Runtime = TestStorage;

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
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const Version: RuntimeVersion = VERSION;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
    pub const SessionsPerEra: sp_staking::SessionIndex = 3;
    pub const BondingDuration: pallet_staking::EraIndex = 7;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 4;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational(5u32, 10_000);
    pub const MaxNominatorRewardedPerValidator: u32 = 2048;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const IndexDeposit: Balance = DOLLARS;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
    pub const MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * POLY;
    pub const FixedYearlyReward: Balance = 140_000_000 * POLY;
    pub const MinimumBond: Balance = 1 * POLY;
    pub const MaxNumberOfFungibleAssets: u32 = 100;
    pub const MaxNumberOfNFTsPerLeg: u32 = 10;
    pub const MaxNumberOfNFTs: u32 = 100;
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
    pub const MaxAuthorities: u32 = 100_000;
    pub const MaxKeys: u32 = 10_000;
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const MaxPeerDataEncodingSize: u32 = 1_000;
    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
    pub const MaxNumberOfCollectionKeys: u8 = u8::MAX;
    pub const MaxNumberOfFungibleMoves: u32 = 10;
    pub const MaxNumberOfNFTsMoves: u32 = 100;
}

frame_support::construct_runtime!(
    pub enum TestStorage where
    Block = Block,
    NodeBlock = polymesh_primitives::Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
{
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 1,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
        // Authorship: pallet_authorship = 4,

        // Balance: Genesis config dependencies: System.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,

        // TransactionPayment: Genesis config dependencies: Balance.
        TransactionPayment: pallet_transaction_payment::{Pallet, Event<T>, Storage} = 6,

        // Identity: Genesis config deps: Timestamp.
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>, Config<T>} = 7,

        // Polymesh Committees

        // CddServiceProviders: Genesis config deps: Identity
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
        // UpgradeCommitteeMembership: Genesis config deps: UpgradeCommittee
        UpgradeCommitteeMembership: pallet_group::<Instance4>::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,

        MultiSig: pallet_multisig::{Pallet, Call, Config, Storage, Event<T>} = 15,
        // Bridge: Genesis config deps: Multisig, Identity,
        Bridge: pallet_bridge::{Pallet, Call, Storage, Config<T>, Event<T>} = 16,

        // Staking: Genesis config deps: Balances, Indices, Identity, Babe, Timestamp, CddServiceProviders.
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>, ValidateUnsigned} = 17,
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
        CapitalDistribution: capital_distributions::{Pallet, Call, Storage, Event} = 27,
        Checkpoint: pallet_checkpoint::{Pallet, Call, Storage, Event, Config} = 28,
        ComplianceManager: pallet_compliance_manager::{Pallet, Call, Storage, Event} = 29,
        CorporateAction: pallet_corporate_actions::{Pallet, Call, Storage, Event, Config} = 30,
        CorporateBallot: corporate_ballots::{Pallet, Call, Storage, Event} = 31,
        Permissions: pallet_permissions::{Pallet, Storage} = 32,
        Pips: pallet_pips::{Pallet, Call, Storage, Event<T>, Config<T>} = 33,
        Portfolio: pallet_portfolio::{Pallet, Call, Storage, Event} = 34,
        ProtocolFee: pallet_protocol_fee::{Pallet, Call, Storage, Event<T>, Config} = 35,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 36,
        Settlement: pallet_settlement::{Pallet, Call, Storage, Event<T>, Config} = 37,
        Statistics: pallet_statistics::{Pallet, Call, Storage, Event} = 38,
        Sto: pallet_sto::{Pallet, Call, Storage, Event<T>} = 39,
        Treasury: pallet_treasury::{Pallet, Call, Event<T>} = 40,
        Utility: pallet_utility::{Pallet, Call, Storage, Event} = 41,
        Base: pallet_base::{Pallet, Call, Event} = 42,
        ExternalAgents: pallet_external_agents::{Pallet, Call, Storage, Event} = 43,
        Relayer: pallet_relayer::{Pallet, Call, Storage, Event<T>} = 44,
        Rewards: pallet_rewards::{Pallet, Call, Storage, Event<T>, Config<T>} = 45,

        // Contracts
        Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>} = 46,
        PolymeshContracts: polymesh_contracts::{Pallet, Call, Storage, Event, Config} = 47,

        // Preimage register.  Used by `pallet_scheduler`.
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 48,

        TestUtils: pallet_test_utils::{Pallet, Call, Storage, Event<T> } = 50,

        Nft: pallet_nft::{Pallet, Call, Storage, Event} = 51,
    }
);

polymesh_runtime_common::runtime_apis! {}

#[derive(Copy, Clone)]
pub struct User {
    /// The `ring` of the `User` used to derive account related data,
    /// e.g., origins, keys, and balances.
    pub ring: AccountKeyring,
    /// The DID of the `User`.
    /// The `ring` need not be the primary key of this DID.
    pub did: IdentityId,
}

impl User {
    /// Creates a `User` provided a `did` and a `ring`.
    ///
    /// The function is useful when `ring` refers to a secondary key.
    /// At the time of calling, nothing is asserted about `did`'s registration.
    pub const fn new_with(did: IdentityId, ring: AccountKeyring) -> Self {
        User { ring, did }
    }

    /// Creates and registers a `User` for the given `ring` which will act as the primary key.
    pub fn new(ring: AccountKeyring) -> Self {
        Self::new_with(register_keyring_account(ring).unwrap(), ring)
    }

    /// Creates a `User` for an already registered DID with `ring` as its primary key.
    pub fn existing(ring: AccountKeyring) -> Self {
        Self::new_with(get_identity_id(ring).unwrap(), ring)
    }

    /// Returns the current balance of `self`'s account.
    pub fn balance(self, balance: u128) -> Self {
        use frame_support::traits::Currency as _;
        Balances::make_free_balance_be(&self.acc(), balance);
        self
    }

    /// Returns `self`'s `AccountId`. This is based on the `ring`.
    pub fn acc(&self) -> AccountId {
        self.ring.to_account_id()
    }

    /// Returns an account based signatory for `self`.
    pub fn signatory_acc(&self) -> Signatory<AccountId> {
        Signatory::Account(self.acc())
    }

    /// Returns an account based signatory for `self`.
    pub const fn signatory_did(&self) -> Signatory<AccountId> {
        Signatory::Identity(self.did)
    }

    /// Returns an `Origin` that can be used to execute extrinsics.
    pub fn origin(&self) -> RuntimeOrigin {
        RuntimeOrigin::signed(self.acc())
    }

    pub fn uid(&self) -> InvestorUid {
        create_investor_uid(self.acc())
    }

    pub fn make_scope_claim(&self, ticker: Ticker, cdd_provider: &AccountId) -> (ScopeId, CddId) {
        provide_scope_claim(self.did, ticker, self.uid(), cdd_provider.clone(), None)
    }

    /// Create a `Scope::Identity` from a User
    pub fn scope(&self) -> Scope {
        Scope::Identity(self.did)
    }

    /// Create a `TrustedIssuer` trusted from a User
    pub fn trusted_issuer_for(&self, trusted_for: TrustedFor) -> TrustedIssuer {
        TrustedIssuer {
            issuer: self.did,
            trusted_for,
        }
    }

    /// Create a `TrustedIssuer` trusted from a User
    pub fn issuer(&self) -> TrustedIssuer {
        self.did.into()
    }
}

pub type EventTest = RuntimeEvent;

type Hash = H256;
type Hashing = BlakeTwo256;
type Lookup = IdentityLookup<AccountId>;
type OffChainSignature = AnySignature;
type SessionIndex = u32;
type AuthorityId = <AnySignature as Verify>::Signer;
pub(crate) type Balance = u128;

parameter_types! {
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const BlockExecutionWeight: Weight = Weight::from_ref_time(10);
    pub TransactionByteFee: Balance = TRANSACTION_BYTE_FEE.with(|v| *v.borrow());
    pub ExtrinsicBaseWeight: Weight = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow());
    pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 10,
        write: 100,
    };
    pub FeeCollector: AccountId = account_from(5000);
}

pub struct DealWithFees;

impl OnUnbalanced<NegativeImbalance<TestStorage>> for DealWithFees {
    fn on_nonzero_unbalanced(amount: NegativeImbalance<TestStorage>) {
        let target = account_from(5000);
        let positive_imbalance = Balances::deposit_creating(&target, amount.peek());
        let _ = amount.offset(positive_imbalance).same().map_err(|_| 4); // random value mapped for error
    }
}

parameter_types! {
    pub const SS58Prefix: u8 = 12;
    pub const ExistentialDeposit: u64 = 0;
    pub const MaxLocks: u32 = 50;
    pub const MaxLen: u32 = 256;
    pub const AssetNameMaxLength: u32 = 128;
    pub const FundingRoundNameMaxLength: u32 = 128;
    pub const AssetMetadataNameMaxLength: u32 = 256;
    pub const AssetMetadataValueMaxLength: u32 = 8 * 1024;
    pub const AssetMetadataTypeDefMaxLength: u32 = 8 * 1024;
    pub const BlockRangeForTimelock: BlockNumber = 1000;
    pub const MaxTargetIds: u32 = 10;
    pub const MaxDidWhts: u32 = 10;
    pub const MinimumPeriod: u64 = 3;

    pub const MaxStatsPerAsset: u32 = 10 + BENCHMARK_MAX_INCREASE;
    pub const MaxTransferConditionsPerAsset: u32 = 4 + BENCHMARK_MAX_INCREASE;

    pub const MaxConditionComplexity: u32 = 50;
    pub const MaxDefaultTrustedClaimIssuers: usize = 10;
    pub const MaxTrustedIssuerPerCondition: usize = 10;
    pub const MaxSenderConditionsPerCompliance: usize = 30;
    pub const MaxReceiverConditionsPerCompliance: usize = 30;
    pub const MaxCompliancePerRequirement: usize = 10;

    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    pub const MaxScheduledPerBlock: u32 = 50;

    pub const InitialPOLYX: Balance = 41;
    pub const SignedClaimHandicap: u64 = 2;
    pub const StorageSizeOffset: u32 = 8;
    pub const MaxDepth: u32 = 100;
    pub const MaxValueSize: u32 = 16_384;

    pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
    pub DeletionWeightLimit: Weight = Weight::from_ref_time(500_000_000_000);
    pub DeletionQueueDepth: u32 = 1024;
    pub MaxInLen: u32 = 8 * 1024;
    pub MaxOutLen: u32 = 8 * 1024;
}

thread_local! {
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<BlockNumber> = RefCell::new(2);
}

pub type NegativeImbalance<T> =
    <balances::Module<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

impl ChargeTxFee for TestStorage {
    fn charge_fee(_len: u32, _info: DispatchInfo) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }
}

type CddHandler = TestStorage;
impl CddAndFeeDetails<AccountId, RuntimeCall> for TestStorage {
    fn get_valid_payer(
        _: &RuntimeCall,
        caller: &AccountId,
    ) -> Result<Option<AccountId>, InvalidTransaction> {
        let caller: AccountId = caller.clone();
        Ok(Some(caller))
    }
    fn clear_context() {
        Context::set_current_identity::<Identity>(None);
        Context::set_current_payer::<Identity>(None);
    }
    fn set_payer_context(payer: Option<AccountId>) {
        Context::set_current_payer::<Identity>(payer);
    }
    fn get_payer_from_context() -> Option<AccountId> {
        Context::current_payer::<Identity>()
    }
    fn set_current_identity(did: &IdentityId) {
        Context::set_current_identity::<Identity>(Some(*did));
    }
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            coeff_frac: Perbill::zero(),
            coeff_integer: WEIGHT_TO_FEE.with(|v| *v.borrow()),
            negative: false,
        }]
    }
}

/// PolymeshCommittee as an instance of group
impl group::Config<group::Instance1> for TestStorage {
    type RuntimeEvent = RuntimeEvent;
    type LimitOrigin = EnsureRoot<AccountId>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = committee::Module<TestStorage, committee::Instance1>;
    type MembershipChanged = committee::Module<TestStorage, committee::Instance1>;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl group::Config<group::Instance2> for TestStorage {
    type RuntimeEvent = RuntimeEvent;
    type LimitOrigin = EnsureRoot<AccountId>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = identity::Module<TestStorage>;
    type MembershipChanged = identity::Module<TestStorage>;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl group::Config<group::Instance3> for TestStorage {
    type RuntimeEvent = RuntimeEvent;
    type LimitOrigin = EnsureRoot<AccountId>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl group::Config<group::Instance4> for TestStorage {
    type RuntimeEvent = RuntimeEvent;
    type LimitOrigin = EnsureRoot<AccountId>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = UpgradeCommittee;
    type MembershipChanged = UpgradeCommittee;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

pub type CommitteeOrigin<T, I> = committee::RawOrigin<<T as frame_system::Config>::AccountId, I>;

impl committee::Config<committee::Instance1> for TestStorage {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = VMO<committee::Instance1>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl committee::Config<committee::Instance3> for TestStorage {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = EnsureRoot<AccountId>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl committee::Config<committee::Instance4> for TestStorage {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = EnsureRoot<AccountId>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl polymesh_common_utilities::traits::identity::Config for TestStorage {
    type RuntimeEvent = RuntimeEvent;
    type Proposal = RuntimeCall;
    type MultiSig = multisig::Module<TestStorage>;
    type Portfolio = portfolio::Module<TestStorage>;
    type CddServiceProviders = CddServiceProvider;
    type Balances = balances::Module<TestStorage>;
    type ChargeTxFeeTarget = TestStorage;
    type CddHandler = TestStorage;
    type Public = <MultiSignature as Verify>::Signer;
    type OffChainSignature = MultiSignature;
    type ProtocolFee = protocol_fee::Module<TestStorage>;
    type GCVotingMajorityOrigin = VMO<committee::Instance1>;
    type WeightInfo = polymesh_weights::pallet_identity::SubstrateWeight;
    type IdentityFn = identity::Module<TestStorage>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MultiSigBalanceLimit = polymesh_runtime_common::MultiSigBalanceLimit;
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

    fn on_new_session<Ks: OpaqueKeys>(
        _changed: bool,
        _validators: &[(AuthorityId, Ks)],
        _queued_validators: &[(AuthorityId, Ks)],
    ) {
    }

    fn on_disabled(_validator_index: u32) {}

    fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}
    fn on_before_session_ending() {}
}

pub struct TestShouldEndSession;
impl pallet_session::ShouldEndSession<BlockNumber> for TestShouldEndSession {
    fn should_end_session(now: BlockNumber) -> bool {
        let l = SESSION_LENGTH.with(|l| *l.borrow());
        now % l == 0
            || FORCE_SESSION_END.with(|l| {
                let r = *l.borrow();
                *l.borrow_mut() = false;
                r
            })
    }
}

pub struct TestSessionManager;
impl pallet_session::SessionManager<AccountId> for TestSessionManager {
    fn end_session(_: SessionIndex) {}
    fn start_session(_: SessionIndex) {}
    fn new_session(_: SessionIndex) -> Option<Vec<AccountId>> {
        None
    }
}

impl pips::Config for TestStorage {
    type Currency = balances::Module<Self>;
    type VotingMajorityOrigin = VMO<committee::Instance1>;
    type GovernanceCommittee = Committee;
    type TechnicalCommitteeVMO = VMO<committee::Instance3>;
    type UpgradeCommitteeVMO = VMO<committee::Instance4>;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_pips::SubstrateWeight;
    type Scheduler = Scheduler;
    type SchedulerCall = RuntimeCall;
}

impl pallet_test_utils::Config for TestStorage {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_test_utils::SubstrateWeight;
}

polymesh_runtime_common::misc_pallet_impls!();

pub type GovernanceCommittee = group::Module<TestStorage, group::Instance1>;
pub type CddServiceProvider = group::Module<TestStorage, group::Instance2>;
pub type Committee = committee::Module<TestStorage, committee::Instance1>;
pub type DefaultCommittee = committee::Module<TestStorage, committee::DefaultInstance>;
//pub type WrapperContracts = polymesh_contracts::Module<TestStorage>;
pub type CorporateActions = corporate_actions::Module<TestStorage>;

pub fn make_account(
    id: AccountId,
) -> Result<
    (
        <TestStorage as frame_system::Config>::RuntimeOrigin,
        IdentityId,
    ),
    &'static str,
> {
    let uid = create_investor_uid(id.clone());
    make_account_with_uid(id, uid)
}

pub fn make_account_with_portfolio(ring: AccountKeyring) -> (User, PortfolioId) {
    let user = User::new(ring);
    let portfolio = PortfolioId::default_portfolio(user.did);
    (user, portfolio)
}

pub fn make_account_with_scope(
    id: AccountId,
    ticker: Ticker,
    cdd_provider: AccountId,
) -> Result<
    (
        <TestStorage as frame_system::Config>::RuntimeOrigin,
        IdentityId,
        ScopeId,
    ),
    &'static str,
> {
    let uid = create_investor_uid(id.clone());
    let (origin, did) = make_account_with_uid(id, uid.clone()).unwrap();
    let scope_id = provide_scope_claim(did, ticker, uid, cdd_provider, None).0;
    Ok((origin, did, scope_id))
}

pub fn make_account_with_uid(
    id: AccountId,
    uid: InvestorUid,
) -> Result<
    (
        <TestStorage as frame_system::Config>::RuntimeOrigin,
        IdentityId,
    ),
    &'static str,
> {
    make_account_with_balance(id, uid, 1_000_000)
}

/// It creates an Account and registers its DID and its InvestorUid.
pub fn make_account_with_balance(
    id: AccountId,
    uid: InvestorUid,
    balance: Balance,
) -> Result<
    (
        <TestStorage as frame_system::Config>::RuntimeOrigin,
        IdentityId,
    ),
    &'static str,
> {
    let signed_id = RuntimeOrigin::signed(id.clone());
    Balances::make_free_balance_be(&id, balance);

    // If we have CDD providers, first of them executes the registration.
    let cdd_providers = CddServiceProvider::get_members();
    let did = match cdd_providers.into_iter().nth(0) {
        Some(cdd_provider) => {
            let cdd_acc = get_primary_key(cdd_provider);
            let _ = Identity::cdd_register_did(
                RuntimeOrigin::signed(cdd_acc.clone()),
                id.clone(),
                vec![],
            )
            .map_err(|_| "CDD register DID failed")?;

            // Add CDD Claim
            let did = Identity::get_identity(&id).unwrap();
            let (cdd_id, _) = create_cdd_id(did, Ticker::default(), uid);
            let cdd_claim = Claim::CustomerDueDiligence(cdd_id);
            Identity::add_claim(RuntimeOrigin::signed(cdd_acc), did, cdd_claim, None)
                .map_err(|_| "CDD provider cannot add the CDD claim")?;
            did
        }
        _ => {
            let _ = TestUtils::register_did(signed_id.clone(), uid, vec![])
                .map_err(|_| "Register DID failed")?;
            Identity::get_identity(&id).unwrap()
        }
    };

    Ok((signed_id, did))
}

pub fn make_account_without_cdd(
    id: AccountId,
) -> Result<
    (
        <TestStorage as frame_system::Config>::RuntimeOrigin,
        IdentityId,
    ),
    &'static str,
> {
    let signed_id = RuntimeOrigin::signed(id.clone());
    Balances::make_free_balance_be(&id, 10_000_000);
    let did = Identity::_register_did(id.clone(), vec![], None).expect("did");
    Ok((signed_id, did))
}

pub fn register_keyring_account(acc: AccountKeyring) -> Result<IdentityId, &'static str> {
    register_keyring_account_with_balance(acc, 10_000_000)
}

pub fn register_keyring_account_with_balance(
    acc: AccountKeyring,
    balance: Balance,
) -> Result<IdentityId, &'static str> {
    let acc_id = acc.to_account_id();
    let uid = create_investor_uid(acc_id.clone());
    make_account_with_balance(acc_id, uid, balance).map(|(_, id)| id)
}

pub fn get_primary_key(target: IdentityId) -> AccountId {
    Identity::get_primary_key(target).unwrap_or_default()
}

pub fn get_secondary_keys(target: IdentityId) -> Vec<SecondaryKey<AccountId>> {
    match Identity::get_did_records(target) {
        RpcDidRecords::Success { secondary_keys, .. } => secondary_keys,
        _ => vec![],
    }
}

pub fn add_secondary_key_with_perms(did: IdentityId, acc: AccountId, perms: AuthPermissions) {
    let _primary_key = get_primary_key(did);
    let auth_id = Identity::add_auth(
        did.clone(),
        Signatory::Account(acc.clone()),
        AuthorizationData::JoinIdentity(perms),
        None,
    );
    assert_ok!(Identity::join_identity(RuntimeOrigin::signed(acc), auth_id));
}

pub fn add_secondary_key(did: IdentityId, acc: AccountId) {
    add_secondary_key_with_perms(did, acc, <_>::default())
}

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    let pk = *Pair::from_seed(&enc_id).public().as_array_ref();
    pk.into()
}

pub fn get_identity_id(acc: AccountKeyring) -> Option<IdentityId> {
    Identity::get_identity(&acc.to_account_id())
}

pub fn authorizations_to(to: &Signatory<AccountId>) -> Vec<Authorization<AccountId, u64>> {
    identity::Authorizations::<TestStorage>::iter_prefix_values(to).collect::<Vec<_>>()
}

/// Advances the system `block_number` and run any scheduled task.
pub fn next_block() -> Weight {
    let block_number = frame_system::Pallet::<TestStorage>::block_number() + 1;
    frame_system::Pallet::<TestStorage>::set_block_number(block_number);

    // Call the timelocked tx handler.
    pallet_scheduler::Pallet::<TestStorage>::on_initialize(block_number)
}

pub fn fast_forward_to_block(n: u32) {
    let i = System::block_number();
    for _ in i..=n {
        next_block();
    }
}

pub fn fast_forward_blocks(offset: u32) {
    fast_forward_to_block(offset + System::block_number())
}

// `iter_prefix_values` has no guarantee that it will iterate in a sequential
// order. However, we need the latest `auth_id`. Which is why we search for the claim
// with the highest `auth_id`.
pub fn get_last_auth(signatory: &Signatory<AccountId>) -> Authorization<AccountId, u64> {
    <identity::Authorizations<TestStorage>>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
}

pub fn get_last_auth_id(signatory: &Signatory<AccountId>) -> u64 {
    get_last_auth(signatory).auth_id
}

/// Returns a btreeset that contains default portfolio for the identity.
pub fn default_portfolio_btreeset(did: IdentityId) -> BTreeSet<PortfolioId> {
    iter::once(PortfolioId::default_portfolio(did)).collect::<BTreeSet<_>>()
}

/// Returns a vector that contains default portfolio for the identity.
pub fn default_portfolio_vec(did: IdentityId) -> Vec<PortfolioId> {
    vec![PortfolioId::default_portfolio(did)]
}

/// Returns a btreeset that contains a portfolio for the identity.
pub fn user_portfolio_btreeset(did: IdentityId, num: PortfolioNumber) -> BTreeSet<PortfolioId> {
    iter::once(PortfolioId::user_portfolio(did, num)).collect::<BTreeSet<_>>()
}

/// Returns a vector that contains a portfolio for the identity.
pub fn user_portfolio_vec(did: IdentityId, num: PortfolioNumber) -> Vec<PortfolioId> {
    vec![PortfolioId::user_portfolio(did, num)]
}

pub fn create_cdd_id(
    claim_to: IdentityId,
    scope: Ticker,
    investor_uid: InvestorUid,
) -> (CddId, InvestorZKProofData) {
    let proof: InvestorZKProofData = InvestorZKProofData::new(&claim_to, &investor_uid, &scope);
    let cdd_id = CddId::new_v1(claim_to, investor_uid);
    (cdd_id, proof)
}

pub fn create_investor_uid(acc: AccountId) -> InvestorUid {
    InvestorUid::from(format!("{}", acc).as_str())
}

pub fn add_cdd_claim(
    claim_to: IdentityId,
    scope: Ticker,
    investor_uid: InvestorUid,
    cdd_provider: AccountId,
    cdd_claim_expiry: Option<u64>,
) -> (ScopeId, CddId, InvestorZKProofData) {
    let (cdd_id, proof) = create_cdd_id(claim_to, scope, investor_uid);
    let scope_id = InvestorZKProofData::make_scope_id(&scope.as_slice(), &investor_uid);

    // Add cdd claim first
    assert_ok!(Identity::add_claim(
        RuntimeOrigin::signed(cdd_provider),
        claim_to,
        Claim::CustomerDueDiligence(cdd_id),
        cdd_claim_expiry,
    ));
    (scope_id, cdd_id, proof)
}

pub fn provide_scope_claim(
    claim_to: IdentityId,
    scope: Ticker,
    investor_uid: InvestorUid,
    cdd_provider: AccountId,
    cdd_claim_expiry: Option<u64>,
) -> (ScopeId, CddId) {
    // Add CDD claim, create scope_id and proof.
    let (scope_id, cdd_id, proof) = add_cdd_claim(
        claim_to,
        scope,
        investor_uid,
        cdd_provider,
        cdd_claim_expiry,
    );

    // Add the InvestorUniqueness claim.
    assert_ok!(add_investor_uniqueness_claim(
        claim_to, scope, scope_id, cdd_id, proof
    ));
    (claim_to.into(), cdd_id)
}

pub fn add_investor_uniqueness_claim(
    claim_to: IdentityId,
    scope: Ticker,
    scope_id: ScopeId,
    cdd_id: CddId,
    proof: InvestorZKProofData,
) -> DispatchResult {
    if Asset::disable_iu(scope) {
        return Ok(());
    }
    let signed_claim_to = RuntimeOrigin::signed(get_primary_key(claim_to));

    // Provide the InvestorUniqueness.
    Identity::add_investor_uniqueness_claim(
        signed_claim_to,
        claim_to,
        Claim::InvestorUniqueness(Scope::Ticker(scope), scope_id, cdd_id),
        proof,
        None,
    )
}

pub fn provide_scope_claim_to_multiple_parties<'a>(
    parties: impl IntoIterator<Item = &'a IdentityId>,
    ticker: Ticker,
    cdd_provider: AccountId,
) {
    parties.into_iter().for_each(|id| {
        let uid = create_investor_uid(get_primary_key(*id));
        provide_scope_claim(*id, ticker, uid, cdd_provider.clone(), None).0;
    });
}

pub fn root() -> RuntimeOrigin {
    RuntimeOrigin::from(frame_system::RawOrigin::Root)
}

pub fn create_cdd_id_and_investor_uid(identity_id: IdentityId) -> (CddId, InvestorUid) {
    let uid = create_investor_uid(get_primary_key(identity_id));
    let (cdd_id, _) = create_cdd_id(identity_id, Ticker::default(), uid);
    (cdd_id, uid)
}

pub fn make_remark_proposal() -> RuntimeCall {
    RuntimeCall::System(frame_system::Call::remark {
        remark: vec![b'X'; 100],
    })
    .into()
}

pub(crate) fn set_curr_did(did: Option<IdentityId>) {
    Context::set_current_identity::<Identity>(did);
}

#[macro_export]
macro_rules! assert_last_event {
    ($event:pat) => {
        assert_last_event!($event, true);
    };
    ($event:pat, $cond:expr) => {
        assert!(matches!(
            &*System::events(),
            [.., EventRecord {
                event: $event,
                ..
            }]
            if $cond
        ));
    };
}

#[macro_export]
macro_rules! assert_event_exists {
    ($event:pat) => {
        assert_event_exists!($event, true);
    };
    ($event:pat, $cond:expr) => {
        assert!(System::events().iter().any(|e| {
            matches!(
                e,
                EventRecord {
                    event: $event,
                    ..
                }
                if $cond
            )
        }));
    };
}

#[macro_export]
macro_rules! assert_event_doesnt_exist {
    ($event:pat) => {
        assert_event_doesnt_exist!($event, true);
    };
    ($event:pat, $cond:expr) => {
        assert!(System::events().iter().all(|e| {
            !matches!(
                e,
                EventRecord {
                    event: $event,
                    ..
                }
                if $cond
            )
        }));
    };
}

pub fn exec<C: Into<RuntimeCall>>(origin: RuntimeOrigin, call: C) -> DispatchResult {
    let origin: Result<RawOrigin<AccountId>, RuntimeOrigin> = origin.into();
    let signed = match origin.unwrap() {
        RawOrigin::Signed(acc) => {
            let info = frame_system::Account::<TestStorage>::get(&acc);
            Some((acc, signed_extra(info.nonce)))
        }
        _ => None,
    };
    Executive::apply_extrinsic(sign(CheckedExtrinsic {
        signed,
        function: call.into(),
    }))
    .unwrap()
}

/// Sign given `CheckedExtrinsic` returning an `UncheckedExtrinsic`
/// usable for execution.
fn sign(xt: CheckedExtrinsic) -> UncheckedExtrinsic {
    let CheckedExtrinsic {
        signed, function, ..
    } = xt;
    UncheckedExtrinsic {
        signature: signed.map(|(signed, extra)| {
            let payload = (
                &function,
                extra.clone(),
                VERSION.spec_version,
                VERSION.transaction_version,
                GENESIS_HASH,
                GENESIS_HASH,
            );
            let key = AccountKeyring::from_account_id(&signed).unwrap();
            let signature = payload
                .using_encoded(|b| {
                    if b.len() > 256 {
                        key.sign(&sp_io::hashing::blake2_256(b))
                    } else {
                        key.sign(b)
                    }
                })
                .into();
            (Address::Id(signed), signature, extra)
        }),
        function,
    }
}

/// Returns transaction extra.
fn signed_extra(nonce: Index) -> SignedExtra {
    (
        frame_system::CheckSpecVersion::new(),
        frame_system::CheckTxVersion::new(),
        frame_system::CheckGenesis::new(),
        frame_system::CheckEra::from(Era::mortal(256, 0)),
        frame_system::CheckNonce::from(nonce),
        polymesh_extensions::CheckWeight::new(),
        pallet_transaction_payment::ChargeTransactionPayment::from(0),
        pallet_permissions::StoreCallMetadata::new(),
    )
}
