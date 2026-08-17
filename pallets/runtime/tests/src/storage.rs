#[cfg(feature = "std")]
use sp_version::NativeVersion;

use std::cell::RefCell;
use std::convert::From;

use codec::Encode;
use frame_support::traits::tokens::{fungible::Credit, imbalance::OnUnbalanced};
use frame_support::traits::{ConstBool, Currency, Imbalance, KeyOwnerProofSystem};
use frame_support::traits::{OnInitialize, TryCollect};
use frame_support::weights::RuntimeDbWeight;
use frame_support::weights::Weight;
use frame_support::{assert_ok, parameter_types, BoundedBTreeSet};
use sp_core::crypto::{key_types, Pair as PairTrait};
use sp_core::sr25519::Pair;
use sp_core::H256;
use sp_keyring::Sr25519Keyring;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::generic::Era;
use sp_runtime::testing::UintAuthorityId;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, IdentityLookup};
use sp_runtime::traits::{NumberFor, OpaqueKeys, StaticLookup, Verify};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionPriority};
use sp_runtime::{AnySignature, Cow, KeyTypeId, Perbill, Permill};
use sp_staking::{EraIndex, SessionIndex};
use sp_std::collections::btree_set::BTreeSet;
use sp_version::RuntimeVersion;

use frame_system::{EnsureRoot, RawOrigin};
use lazy_static::lazy_static;
use pallet_transaction_payment::RuntimeDispatchInfo;
use pallet_utility;
use polymesh_primitives::constants::currency::{DOLLARS, POLY};
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::settlement::Leg;
use polymesh_primitives::traits::{group::GroupTrait, CurrentFeePayer};
use polymesh_primitives::transaction_payment::CallPaymentInfo;
use polymesh_primitives::{AccountId, Authorization, AuthorizationData, BlockNumber};
use polymesh_primitives::{Moment, Permissions as AuthPermissions};
use polymesh_primitives::{PortfolioNumber, Scope, SecondaryKey, TrustedFor, TrustedIssuer};
use polymesh_runtime_common::merge_active_and_inactive;
use polymesh_runtime_common::runtime::{BENCHMARK_MAX_INCREASE, VMO};
use polymesh_runtime_common::{AvailableBlockRatio, MaximumBlockWeight};
use polymesh_runtime_develop::constants::time::{EPOCH_DURATION_IN_BLOCKS, MILLISECS_PER_BLOCK};

use pallet_asset::checkpoint as pallet_checkpoint;
use pallet_balances as balances;
use pallet_committee as committee;
use pallet_corporate_actions as corporate_actions;
use pallet_corporate_actions::ballot as pallet_corporate_ballot;
use pallet_corporate_actions::distribution as pallet_capital_distribution;
use pallet_group as group;
use pallet_identity as identity;
use pallet_protocol_fee as protocol_fee;
use pallet_session::historical as pallet_session_historical;

use super::ext_builder::{EXTRINSIC_BASE_WEIGHT, TRANSACTION_BYTE_FEE};

type Runtime = TestStorage;

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
    spec_name: Cow::Borrowed("test-storage"),
    impl_name: Cow::Borrowed("test-storage"),
    authoring_version: 1,
    // Per convention: if the runtime behavior changes, increment spec_version
    // and set impl_version to 0. If only runtime
    // implementation changes and behavior does not, then leave spec_version as
    // is and increment impl_version.
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 8,
    system_version: 1,
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

// example module to test behaviors.
#[frame_support::pallet(dev_mode)]
pub mod example {
    use frame_support::{dispatch::WithPostDispatchInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(*weight)]
        pub fn noop(_origin: OriginFor<T>, weight: Weight) -> DispatchResult {
            let _ = weight;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(*_start_weight)]
        pub fn foobar(
            origin: OriginFor<T>,
            err: bool,
            _start_weight: Weight,
            end_weight: Option<Weight>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;
            if err {
                let error: DispatchError = "The cake is a lie.".into();
                if let Some(weight) = end_weight {
                    Err(error.with_weight(weight))
                } else {
                    Err(error)?
                }
            } else {
                Ok(end_weight.into())
            }
        }

        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn big_variant(_origin: OriginFor<T>, _arg: [u8; 400]) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        pub fn noop2(_origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }
}

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
    pub const BondingDuration: EraIndex = 7;
    pub const SlashDeferDuration: EraIndex = 4;
    pub const UnsignedPhase: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational(5u32, 10_000);
    pub const MaxNominatorRewardedPerValidator: u32 = 1_024;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const IndexDeposit: Balance = DOLLARS;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
    pub const MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * POLY;
    pub const FixedYearlyReward: Balance = 140_000_000 * POLY;
    pub const MaxPayoutWeight: Weight = Weight::from_parts(1_000_000_000, 0);
    pub const MinimumBond: Balance = 1 * POLY;
    pub const MaxNumberOfFungibleAssets: u32 = 100;
    pub const MaxNumberOfNFTsPerLeg: u32 = 10;
    pub const MaxNumberOfNFTs: u32 = 100;
    pub const MaximumLockPeriod: Moment = 2;
    pub const RelockCooldown: Moment = 1;
    pub const MaxRelockCount: u32 = 3;

    // Multisig
    pub const MaxMultiSigSigners: u32 = 50;

    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
    pub const MaxAuthorities: u32 = 100_000;
    pub const MaxKeys: u32 = 10_000;
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
    pub const MaxNumberOfCollectionKeys: u8 = u8::MAX;
    pub const MaxNumberOfFungibleMoves: u32 = 10;
    pub const MaxNumberOfNFTsMoves: u32 = 100;
    pub const MaxNumberOfOffChainAssets: u32 = 10;
    pub const MaxNumberOfAssetHolders: u32 = (10 + 100) * 2;
    pub const MaxNumberOfVenueSigners: u32 = 50;
    pub const MaxInstructionMediators: u32 = 4;
    pub const MaxAssetMediators: u32 = 4;
    pub const MaxGivenAuths: u32 = 1024;
    pub const MaxAuthRetries: u8 = 10;
    pub const MigrationSignedDepositPerItem: Balance = 0;
    pub const MigrationSignedDepositBase: Balance = 0;
    pub const MaxKeyLen: u32 = 2048;

    // PIPs
    pub const MaxRefundsAndVotesPruned: u32 = 2;
}

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
    pub struct TestStorage;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Runtime>;

    #[runtime::pallet_index(1)]
    pub type Babe = pallet_babe::Pallet<Runtime>;

    #[runtime::pallet_index(2)]
    pub type Timestamp = pallet_timestamp::Pallet<Runtime>;

    #[runtime::pallet_index(3)]
    pub type Indices = pallet_indices::Pallet<Runtime>;

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

    #[runtime::pallet_index(25)]
    pub type Sudo = pallet_sudo::Pallet<Runtime>;

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

    #[runtime::pallet_index(80)]
    pub type Revive = pallet_revive::Pallet<Runtime>;

    #[runtime::pallet_index(200)]
    pub type Example = example::Pallet<Runtime>;
}

polymesh_runtime_common::runtime_apis! {}

// Precompiles
type Precompiles = (pallet_precompiles::FungibleAssetInterface<Runtime>,);

impl pallet_precompiles::Config for Runtime {
    type RuntimeCall = RuntimeCall;
}

#[derive(Copy, Clone)]
pub struct User {
    /// The `ring` of the `User` used to derive account related data,
    /// e.g., origins, keys, and balances.
    pub ring: Sr25519Keyring,
    /// The DID of the `User`.
    /// The `ring` need not be the primary key of this DID.
    pub did: IdentityId,
}

impl User {
    /// Creates a `User` provided a `did` and a `ring`.
    ///
    /// The function is useful when `ring` refers to a secondary key.
    /// At the time of calling, nothing is asserted about `did`'s registration.
    pub const fn new_with(did: IdentityId, ring: Sr25519Keyring) -> Self {
        User { ring, did }
    }

    /// Creates and registers a `User` for the given `ring` which will act as the primary key.
    pub fn new(ring: Sr25519Keyring) -> Self {
        Self::new_with(register_keyring_account(ring).unwrap(), ring)
    }

    /// Creates a `User` for an already registered DID with `ring` as its primary key.
    pub fn existing(ring: Sr25519Keyring) -> Self {
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
pub(crate) type OffChainSignature = AnySignature;
type AuthorityId = <AnySignature as Verify>::Signer;
pub(crate) type Balance = u128;

parameter_types! {
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const BlockExecutionWeight: Weight = Weight::from_parts(10, 0);
    pub TransactionByteFee: Balance = TRANSACTION_BYTE_FEE.with(|v| *v.borrow());
    pub ExtrinsicBaseWeight: Weight = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow());
    pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 10,
        write: 100,
    };
    pub FeeCollector: AccountId = account_from(5000);
}

pub struct DealWithFees;

impl OnUnbalanced<Credit<AccountId, Balances>> for DealWithFees {
    fn on_nonzero_unbalanced(credit: Credit<AccountId, Balances>) {
        // Use a fixed account to receive transactino and protocol fees in tests.
        let target = account_from(5000);
        let _ = Balances::deposit_creating(&target, credit.peek());
    }
}

parameter_types! {
    pub const SS58Prefix: u8 = 12;
    pub const EvmChainId: u64 = 1_641_818;
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxHolds: u32 = 32;
    pub const MaxFreezes: u32 = 32;
    pub const MaxReserves: u32 = 50;
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
}

thread_local! {
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<BlockNumber> = RefCell::new(2);
}

pub type NegativeImbalance<T> =
    <balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub(crate) type TxFeeHandler = polymesh_runtime_common::fee_details::TxFeeHandler<TestStorage>;

impl CurrentFeePayer<AccountId, RuntimeCall> for TestStorage {
    fn call_payment_info(
        call: &RuntimeCall,
        caller: AccountId,
    ) -> Result<CallPaymentInfo<AccountId>, InvalidTransaction> {
        TxFeeHandler::call_payment_info(call, caller)
    }
    fn set_payer_context(payer: Option<AccountId>) {
        TxFeeHandler::set_payer_context(payer)
    }
    fn get_payer_from_context() -> Option<AccountId> {
        TxFeeHandler::get_payer_from_context()
    }
    fn decrease_authorization_count(call_payment_info: &CallPaymentInfo<AccountId>) {
        TxFeeHandler::decrease_authorization_count(call_payment_info);
    }
}

/// PolymeshCommittee as an instance of group
impl group::Config<group::Instance1> for TestStorage {
    type LimitOrigin = EnsureRoot<AccountId>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = committee::Pallet<TestStorage, committee::Instance1>;
    type MembershipChanged = committee::Pallet<TestStorage, committee::Instance1>;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl group::Config<group::Instance2> for TestStorage {
    type LimitOrigin = EnsureRoot<AccountId>;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = identity::Pallet<TestStorage>;
    type MembershipChanged = identity::Pallet<TestStorage>;
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl group::Config<group::Instance3> for TestStorage {
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
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl committee::Config<committee::Instance3> for TestStorage {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = EnsureRoot<AccountId>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl committee::Config<committee::Instance4> for TestStorage {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = EnsureRoot<AccountId>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl pallet_identity::Config for TestStorage {
    type Proposal = RuntimeCall;
    type DidRegistrars = DidRegistrar;
    type Balances = balances::Pallet<TestStorage>;
    type TxFeeHandler = polymesh_runtime_common::fee_details::TxFeeHandler<TestStorage>;
    type Public = <MultiSignature as Verify>::Signer;
    type OffChainSignature = MultiSignature;
    type ProtocolFee = protocol_fee::Pallet<TestStorage>;
    type GCVotingMajorityOrigin = VMO<committee::Instance1>;
    type WeightInfo = polymesh_weights::pallet_identity::SubstrateWeight;
    type IdentityFn = identity::Pallet<TestStorage>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MaxGivenAuths = MaxGivenAuths;
    type MaxAuthRetries = MaxAuthRetries;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<TestStorage>;
}

impl example::Config for TestStorage {}

pub struct TestBaseCallFilter;

impl frame_support::traits::Contains<RuntimeCall> for TestBaseCallFilter {
    fn contains(c: &RuntimeCall) -> bool {
        match *c {
            // Use `noop2` and `set_storage` for a call that doesn't pass the filter.
            RuntimeCall::Example(example::Call::noop2 {}) => false,
            RuntimeCall::System(frame_system::Call::set_storage { .. }) => false,
            _ => true,
        }
    }
}

type RuntimeBaseCallFilter = TestBaseCallFilter;

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

impl pallet_pips::Config for Runtime {
    type Currency = Balances;
    type VotingMajorityOrigin = VMO<pallet_committee::Instance1>;
    type GovernanceCommittee = Committee;
    type TechnicalCommitteeVMO = VMO<pallet_committee::Instance3>;
    type UpgradeCommitteeVMO = VMO<pallet_committee::Instance4>;
    type WeightInfo = polymesh_weights::pallet_pips::SubstrateWeight;
    type Scheduler = Scheduler;
    type SchedulerCall = RuntimeCall;
    type MaxRefundsAndVotesPruned = MaxRefundsAndVotesPruned;
    type SchedulerPreimage = Preimage;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

polymesh_runtime_common::misc_pallet_impls!();

pub type GovernanceCommittee = group::Pallet<TestStorage, group::Instance1>;
pub type DidRegistrar = group::Pallet<TestStorage, group::Instance2>;
pub type Committee = committee::Pallet<TestStorage, committee::Instance1>;
pub type CorporateActions = corporate_actions::Pallet<TestStorage>;

pub fn make_account(
    id: AccountId,
) -> Result<
    (
        <TestStorage as frame_system::Config>::RuntimeOrigin,
        IdentityId,
    ),
    &'static str,
> {
    make_account_with_balance(id, 1_000_000)
}

pub fn make_account_with_portfolio(ring: Sr25519Keyring) -> (User, PortfolioId) {
    let user = User::new(ring);
    let portfolio = PortfolioId::default_portfolio(user.did);
    (user, portfolio)
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    id: AccountId,
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

    // If we have DID registrars, first of them executes the registration.
    let did_registrars = DidRegistrar::get_members();
    let did = match did_registrars.into_iter().nth(0) {
        Some(did_registrar) => {
            let registrar_acc = get_primary_key(did_registrar);
            // Use the new register_did extrinsic (no secondary keys, no CDD claim)
            Identity::register_did(RuntimeOrigin::signed(registrar_acc), id.clone())
                .map_err(|_| "Register DID failed")?;
            Identity::get_identity(&id).unwrap()
        }
        _ => {
            let _ =
                Identity::testing_register_did(id.clone()).map_err(|_| "Register DID failed")?;
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
    let did = Identity::register_did_without_cdd(id.clone(), vec![], None).expect("did");
    Ok((signed_id, did))
}

pub fn register_keyring_account(acc: Sr25519Keyring) -> Result<IdentityId, &'static str> {
    register_keyring_account_with_balance(acc, 10_000_000)
}

pub fn register_keyring_account_with_balance(
    acc: Sr25519Keyring,
    balance: Balance,
) -> Result<IdentityId, &'static str> {
    let acc_id = acc.to_account_id();
    make_account_with_balance(acc_id, balance).map(|(_, id)| id)
}

pub fn get_primary_key(target: IdentityId) -> AccountId {
    Identity::get_primary_key(target).expect("Primary key")
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
    )
    .unwrap();
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

pub fn get_identity_id(acc: Sr25519Keyring) -> Option<IdentityId> {
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
pub fn default_asset_holder_set(
    did: IdentityId,
) -> BoundedBTreeSet<AssetHolder, MaxNumberOfAssetHolders> {
    let asset_holder = AssetHolder::from(PortfolioId::default_portfolio(did));
    BTreeSet::from([asset_holder]).try_into().unwrap()
}

/// Returns a Bounded btreeset from an unbounded Vec.
pub fn vec_to_btreeset(
    vec: Vec<PortfolioId>,
) -> BoundedBTreeSet<AssetHolder, MaxNumberOfAssetHolders> {
    vec.into_iter()
        .map(AssetHolder::from)
        .try_collect()
        .expect("Vec has too many items")
}

/// Returns a vector that contains default portfolio for the identity.
pub fn default_portfolio_vec(did: IdentityId) -> Vec<PortfolioId> {
    vec![PortfolioId::default_portfolio(did)]
}

/// Returns a btreeset that contains a portfolio for the identity.
pub fn user_asset_holder_set(
    did: IdentityId,
    num: PortfolioNumber,
) -> BoundedBTreeSet<AssetHolder, MaxNumberOfAssetHolders> {
    let asset_holder = AssetHolder::from(PortfolioId::user_portfolio(did, num));
    BTreeSet::from([asset_holder]).try_into().unwrap()
}

/// Returns a vector that contains a portfolio for the identity.
pub fn user_portfolio_vec(did: IdentityId, num: PortfolioNumber) -> Vec<PortfolioId> {
    vec![PortfolioId::user_portfolio(did, num)]
}

pub fn root() -> RuntimeOrigin {
    RuntimeOrigin::from(frame_system::RawOrigin::Root)
}

pub fn make_remark_proposal() -> RuntimeCall {
    RuntimeCall::System(frame_system::Call::remark {
        remark: vec![b'X'; 100],
    })
    .into()
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

    let format = {
        match origin.unwrap() {
            RawOrigin::Signed(acc) => {
                let info = frame_system::Account::<TestStorage>::get(&acc);
                generic::ExtrinsicFormat::Signed(acc, signed_extra(info.nonce))
            }
            _ => generic::ExtrinsicFormat::Bare,
        }
    };

    Executive::apply_extrinsic(sign(CheckedExtrinsic {
        format,
        function: call.into(),
    }))
    .unwrap()
}

/// Sign given `CheckedExtrinsic` returning an `UncheckedExtrinsic` usable for execution.
fn sign(checked_extrinsic: CheckedExtrinsic) -> UncheckedExtrinsic {
    let preamble = {
        match checked_extrinsic.format {
            generic::ExtrinsicFormat::Signed(acc, ext) => {
                let payload = (
                    &checked_extrinsic.function,
                    ext.clone(),
                    VERSION.spec_version,
                    VERSION.transaction_version,
                    GENESIS_HASH,
                    GENESIS_HASH,
                );
                let key = Sr25519Keyring::from_account_id(&acc).unwrap();
                let signature = payload
                    .using_encoded(|b| {
                        if b.len() > 256 {
                            key.sign(&sp_io::hashing::blake2_256(b))
                        } else {
                            key.sign(b)
                        }
                    })
                    .into();
                generic::Preamble::Signed(Address::Id(acc), signature, ext)
            }
            _ => generic::Preamble::Bare(0),
        }
    };

    let function = checked_extrinsic.function;
    generic::UncheckedExtrinsic {
        preamble,
        function,
        encoded_call: None,
    }
    .into()
}

/// Returns transaction extra.
fn signed_extra(nonce: Nonce) -> TxExtension {
    (
        (
            frame_system::AuthorizeCall::new(),
            frame_system::CheckNonZeroSender::new(),
            frame_system::CheckSpecVersion::new(),
            frame_system::CheckTxVersion::new(),
            frame_system::CheckGenesis::new(),
        ),
        frame_system::CheckEra::from(Era::mortal(256, 0)),
        frame_system::CheckNonce::from(nonce),
        frame_system::CheckWeight::new(),
        polymesh_transaction_payment::ChargeTransactionPayment::from(0),
        pallet_permissions::StoreCallMetadata::new(),
        frame_metadata_hash_extension::CheckMetadataHash::new(false),
        pallet_revive::evm::tx_extension::SetOrigin::default(),
        frame_system::WeightReclaim::new(),
    )
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
