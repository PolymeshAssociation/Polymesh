// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities

use std::collections::BTreeMap;

use frame_election_provider_support::{onchain, SequentialPhragmen};
use frame_support::dispatch::{DispatchInfo, DispatchResult, Weight};
use frame_support::traits::{
    ConstU32, Currency, EitherOfDiverse, FindAuthor, GenesisBuild, Get, Hooks, Imbalance,
    KeyOwnerProofSystem, OnUnbalanced, OneSessionHandler,
};
use frame_support::weights::constants::RocksDbWeight;
use frame_support::{
    assert_ok, ord_parameter_types, parameter_types, StorageDoubleMap, StorageMap,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::testing::{Header, TestXt, UintAuthorityId};
use sp_runtime::traits::{IdentityLookup, Zero};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};
use sp_runtime::{KeyTypeId, Perbill};
use sp_staking::offence::{DisableStrategy, OffenceDetails, OnOffenceHandler};
use sp_staking::{EraIndex, SessionIndex};

use pallet_staking::types::SlashingSwitch;
use pallet_staking::{self as pallet_staking, *};
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_common_utilities::traits::balances::{AccountData, CheckCdd};
use polymesh_common_utilities::traits::group::{GroupTrait, InactiveMember};
use polymesh_common_utilities::traits::multisig::MultiSigSubTrait;
use polymesh_common_utilities::traits::portfolio::PortfolioSubTrait;
use polymesh_common_utilities::traits::relayer::SubsidiserTrait;
use polymesh_common_utilities::traits::CommonConfig;
use polymesh_common_utilities::transaction_payment::ChargeTxFee;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::identity_id::GenesisIdentityRecord;
use polymesh_primitives::{
    Authorization, AuthorizationData, Claim, IdentityId, Moment, NFTId, Permissions, PortfolioId,
    SecondaryKey, Signatory,
};

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 1_000;

/// The AccountId alias in this test module.
pub(crate) type AccountId = u64;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

/// Another session handler struct to test on_disabled.
pub struct OtherSessionHandler;
impl OneSessionHandler<AccountId> for OtherSessionHandler {
    type Key = UintAuthorityId;

    fn on_genesis_session<'a, I: 'a>(_: I)
    where
        I: Iterator<Item = (&'a AccountId, Self::Key)>,
        AccountId: 'a,
    {
    }

    fn on_new_session<'a, I: 'a>(_: bool, _: I, _: I)
    where
        I: Iterator<Item = (&'a AccountId, Self::Key)>,
        AccountId: 'a,
    {
    }

    fn on_disabled(_validator_index: u32) {}
}

impl sp_runtime::BoundToRuntimeAppPublic for OtherSessionHandler {
    type Public = UintAuthorityId;
}

pub fn is_disabled(controller: AccountId) -> bool {
    let stash = Staking::ledger(&controller).unwrap().stash;
    let validator_index = match Session::validators().iter().position(|v| *v == stash) {
        Some(index) => index as u32,
        None => return false,
    };

    Session::disabled_validators().contains(&validator_index)
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type Call = RuntimeCall;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Authorship: pallet_authorship,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        Historical: pallet_session::historical::{Pallet},
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>, Config<T>},
        CddServiceProviders: pallet_group::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>},
        ProtocolFee: pallet_protocol_fee::{Pallet, Call, Storage, Event<T>, Config},
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        Treasury: pallet_treasury::{Pallet, Call, Event<T>},
        PolymeshCommittee: pallet_committee::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        Pips: pallet_pips::{Pallet, Call, Storage, Event<T>, Config<T>},
        TestUtils: pallet_test_utils::{Pallet, Call, Storage, Event<T>},
        Base: pallet_base::{Pallet, Call, Event},
    }
);

/// Author of block is always 11
pub struct Author11;
impl FindAuthor<AccountId> for Author11 {
    fn find_author<'a, I>(_digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
    {
        Some(11)
    }
}

parameter_types! {
    pub static SessionsPerEra: sp_staking::SessionIndex = 3;
    pub static ExistentialDeposit: Balance = 1;
    pub static SlashDeferDuration: EraIndex = 0;
    pub static Period: BlockNumber = 5;
    pub static Offset: BlockNumber = 0;

    pub const MaxLen: u32 = 256;
    pub const MaxLocks: u32 = 1024;
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(
            Weight::from_ref_time(frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND * 2)
        );
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
    type RuntimeOrigin = RuntimeOrigin;
    type Index = AccountIndex;
    type BlockNumber = BlockNumber;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = AccountData;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_base::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxLen = MaxLen;
}

impl CommonConfig for Test {
    type BlockRewardsReserve = pallet_balances::Module<Test>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;

    type CddChecker = Test;
    type MaxLocks = MaxLocks;
    type WeightInfo = polymesh_weights::pallet_balances::SubstrateWeight;
}

sp_runtime::impl_opaque_keys! {
    pub struct SessionKeys {
        pub other: OtherSessionHandler,
    }
}

impl pallet_session::Config for Test {
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
    type Keys = SessionKeys;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionHandler = (OtherSessionHandler,);
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Test>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type WeightInfo = ();
}

impl pallet_committee::Config<pallet_committee::Instance1> for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_committee::SubstrateWeight;
}

impl pallet_session::historical::Config for Test {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Test>;
}

impl pallet_pips::Config for Test {
    type Currency = pallet_balances::Module<Self>;
    type VotingMajorityOrigin = frame_system::EnsureRoot<AccountId>;
    type GovernanceCommittee = crate::storage::Committee;
    type TechnicalCommitteeVMO = frame_system::EnsureRoot<AccountId>;
    type UpgradeCommitteeVMO = frame_system::EnsureRoot<AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_pips::SubstrateWeight;
    type Scheduler = Scheduler;
    type SchedulerCall = RuntimeCall;
}

impl pallet_treasury::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = pallet_balances::Module<Self>;
    type WeightInfo = polymesh_weights::pallet_treasury::SubstrateWeight;
}

impl pallet_authorship::Config for Test {
    type FindAuthor = Author11;
    type EventHandler = pallet_staking::Pallet<Test>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_group::Config<pallet_group::Instance2> for Test {
    type RuntimeEvent = RuntimeEvent;
    type LimitOrigin = frame_system::EnsureRoot<AccountId>;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type WeightInfo = polymesh_weights::pallet_group::SubstrateWeight;
}

impl pallet_protocol_fee::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type OnProtocolFeePayment = ();
    type WeightInfo = polymesh_weights::pallet_protocol_fee::SubstrateWeight;
    type Subsidiser = Test;
}

impl polymesh_common_utilities::traits::identity::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Proposal = RuntimeCall;
    type MultiSig = Test;
    type Portfolio = Test;
    type CddServiceProviders = pallet_group::Module<Test, pallet_group::Instance2>;
    type Balances = Balances;
    type ChargeTxFeeTarget = Test;
    type CddHandler = Test;
    type Public = UintAuthorityId;
    type OffChainSignature = sp_runtime::testing::TestSignature;
    type ProtocolFee = pallet_protocol_fee::Module<Test>;
    type GCVotingMajorityOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = polymesh_weights::pallet_identity::SubstrateWeight;
    type IdentityFn = pallet_identity::Module<Test>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MaxGivenAuths = MaxGivenAuths;
}

parameter_types! {
    pub const InitialPOLYX: Balance = 0;
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const MaxGivenAuths: u32 = 1024;
}

impl pallet_scheduler::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
    type OriginPrivilegeCmp = frame_support::traits::EqualPrivilegeOnly;
    type Preimages = Preimage;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = polymesh_runtime_common::deposit(2, 64);
    pub const PreimageByteDeposit: Balance = polymesh_runtime_common::deposit(0, 1);
}

impl pallet_preimage::Config for Test {
    type WeightInfo = polymesh_weights::pallet_preimage::SubstrateWeight;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

impl pallet_test_utils::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = polymesh_weights::pallet_test_utils::SubstrateWeight;
}

impl polymesh_common_utilities::transaction_payment::CddAndFeeDetails<AccountId, Call> for Test {
    fn get_valid_payer(
        _: &Call,
        _: &AccountId,
    ) -> Result<Option<AccountId>, sp_runtime::transaction_validity::InvalidTransaction> {
        Ok(None)
    }
    fn clear_context() {}
    fn set_payer_context(_: Option<AccountId>) {}
    fn get_payer_from_context() -> Option<AccountId> {
        None
    }
}

impl SubsidiserTrait<AccountId, RuntimeCall> for Test {
    fn check_subsidy(
        _: &AccountId,
        _: Balance,
        _: Option<&RuntimeCall>,
    ) -> Result<Option<AccountId>, InvalidTransaction> {
        Ok(None)
    }
    fn debit_subsidy(_: &AccountId, _: Balance) -> Result<Option<AccountId>, InvalidTransaction> {
        Ok(None)
    }
}

impl ChargeTxFee for Test {
    fn charge_fee(_len: u32, _info: DispatchInfo) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }
}

impl GroupTrait<Moment> for Test {
    fn get_members() -> Vec<IdentityId> {
        return Group::active_members();
    }

    fn get_inactive_members() -> Vec<InactiveMember<Moment>> {
        vec![]
    }

    fn disable_member(
        _who: IdentityId,
        _expiry: Option<Moment>,
        _at: Option<Moment>,
    ) -> DispatchResult {
        unimplemented!();
    }

    fn add_member(_who: IdentityId) -> DispatchResult {
        unimplemented!()
    }

    fn get_active_members() -> Vec<IdentityId> {
        Self::get_members()
    }

    /// Current set size
    fn member_count() -> usize {
        Self::get_members().len()
    }

    fn is_member(member_id: &IdentityId) -> bool {
        Self::get_members().contains(member_id)
    }

    /// It returns the current "active members" and any "inactive member" which its
    /// expiration time-stamp is greater than `moment`.
    fn get_valid_members_at(moment: Moment) -> Vec<IdentityId> {
        Self::get_active_members()
            .into_iter()
            .chain(
                Self::get_inactive_members()
                    .into_iter()
                    .filter(|m| !Self::is_member_expired(&m, moment))
                    .map(|m| m.id),
            )
            .collect::<Vec<_>>()
    }

    fn is_member_expired(_member: &InactiveMember<Moment>, _now: Moment) -> bool {
        false
    }
}

impl MultiSigSubTrait<AccountId> for Test {
    fn is_multisig(_account: &AccountId) -> bool {
        false
    }
}

impl PortfolioSubTrait<AccountId> for Test {
    fn ensure_portfolio_custody(_: PortfolioId, _: IdentityId) -> DispatchResult {
        unimplemented!()
    }

    fn ensure_portfolio_validity(_: &PortfolioId) -> DispatchResult {
        unimplemented!()
    }

    fn lock_tokens(_: &PortfolioId, _: &AssetId, _: Balance) -> DispatchResult {
        unimplemented!()
    }

    fn unlock_tokens(_: &PortfolioId, _: &AssetId, _: Balance) -> DispatchResult {
        unimplemented!()
    }

    fn ensure_portfolio_custody_and_permission(
        _: PortfolioId,
        _: IdentityId,
        _: Option<&SecondaryKey<AccountId>>,
    ) -> DispatchResult {
        unimplemented!()
    }

    fn lock_nft(_: &PortfolioId, _: &AssetId, _: &NFTId) -> DispatchResult {
        unimplemented!()
    }

    fn unlock_nft(_: &PortfolioId, _: &AssetId, _: &NFTId) -> DispatchResult {
        unimplemented!()
    }

    fn skip_portfolio_affirmation(_: &PortfolioId, _: &AssetId) -> bool {
        unimplemented!()
    }
}

impl CheckCdd<AccountId> for Test {
    fn check_key_cdd(_key: &AccountId) -> bool {
        true
    }
    fn get_key_cdd_did(_key: &AccountId) -> Option<IdentityId> {
        None
    }
}

impl polymesh_common_utilities::traits::permissions::Config for Test {
    type Checker = Identity;
}

parameter_types! {
    pub const EpochDuration: u64 = 10;
    pub const ExpectedBlockTime: u64 = 1;
    pub const MaxAuthorities: u32 = 100;
}

impl pallet_babe::Config for Test {
    type WeightInfo = ();
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;

    type KeyOwnerProofSystem = ();
    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::Proof;
    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::IdentificationTuple;
    type HandleEquivocation =
        pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, (), ()>;

    type MaxAuthorities = MaxAuthorities;
}

pallet_staking_reward_curve::build! {
    const I_NPOS: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}
parameter_types! {
    pub const BondingDuration: EraIndex = 3;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &I_NPOS;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(75);

    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * POLY;
    pub const FixedYearlyReward: Balance = 200_000_000 * POLY;
}

parameter_types! {
    pub static RewardRemainderUnbalanced: u128 = 0;
}

pub struct RewardRemainderMock;

impl OnUnbalanced<NegativeImbalanceOf<Test>> for RewardRemainderMock {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
        RewardRemainderUnbalanced::mutate(|v| {
            *v += amount.peek();
        });
        drop(amount);
    }
}

const THRESHOLDS: [sp_npos_elections::VoteWeight; 9] =
    [10, 20, 30, 40, 50, 60, 1_000, 2_000, 10_000];

parameter_types! {
    pub static BagThresholds: &'static [sp_npos_elections::VoteWeight] = &THRESHOLDS;
    pub static MaxNominations: u32 = 16;
    pub static HistoryDepth: u32 = 80;
    pub static MaxUnlockingChunks: u32 = 32;
    pub static RewardOnUnbalanceWasCalled: bool = false;
    pub static LedgerSlashPerEra: (BalanceOf<Test>, BTreeMap<EraIndex, BalanceOf<Test>>) = (Zero::zero(), BTreeMap::new());
    pub static MaxWinners: u32 = 100;
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
    type System = Test;
    type Solver = SequentialPhragmen<AccountId, Perbill>;
    type DataProvider = Staking;
    type WeightInfo = ();
    type MaxWinners = MaxWinners;
    type VotersBound = ConstU32<{ u32::MAX }>;
    type TargetsBound = ConstU32<{ u32::MAX }>;
}

pub struct MockReward {}

impl OnUnbalanced<PositiveImbalanceOf<Test>> for MockReward {
    fn on_unbalanced(_: PositiveImbalanceOf<Test>) {
        RewardOnUnbalanceWasCalled::set(true);
    }
}

pub struct OnStakerSlashMock<T: Config>(core::marker::PhantomData<T>);
impl<T: Config> sp_staking::OnStakerSlash<AccountId, Balance> for OnStakerSlashMock<T> {
    fn on_slash(
        _pool_account: &AccountId,
        slashed_bonded: Balance,
        slashed_chunks: &BTreeMap<EraIndex, Balance>,
    ) {
        LedgerSlashPerEra::set((slashed_bonded, slashed_chunks.clone()));
    }
}

impl pallet_staking::Config for Test {
    type Currency = Balances;
    type CurrencyBalance = Balance;
    type UnixTime = Timestamp;
    type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
    type ElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type GenesisElectionProvider = Self::ElectionProvider;
    type MaxNominations = MaxNominations;
    type HistoryDepth = HistoryDepth;
    type RewardRemainder = RewardRemainderMock;
    type RuntimeEvent = RuntimeEvent;
    type Slash = ();
    type Reward = MockReward;
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type AdminOrigin = frame_system::EnsureRoot<AccountId>;
    type SessionInterface = Self;
    type EraPayout = ConvertCurve<RewardCurve>;
    type NextNewSession = Session;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
    type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Self>;
    type TargetList = pallet_staking::UseValidatorsMap<Self>;
    type MaxUnlockingChunks = MaxUnlockingChunks;
    type OnStakerSlash = OnStakerSlashMock<Self>;
    type BenchmarkingConfig = pallet_staking::SampleBenchmarkingConfig;
    type WeightInfo = polymesh_weights::pallet_staking::SubstrateWeight;
    type MaxValidatorPerIdentity = polymesh_runtime_common::MaxValidatorPerIdentity;
    type MaxVariableInflationTotalIssuance = MaxVariableInflationTotalIssuance;
    type FixedYearlyReward = FixedYearlyReward;
    type Call = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type RewardScheduler = Scheduler;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = RuntimeCall;
    type Extrinsic = Extrinsic;
}

pub type Extrinsic = TestXt<Call, ()>;

pub(crate) type StakingCall = pallet_staking::Call<Test>;
pub(crate) type TestCall = <Test as frame_system::Config>::RuntimeCall;

pub struct ExtBuilder {
    nominate: bool,
    validator_count: u32,
    minimum_validator_count: u32,
    invulnerables: Vec<AccountId>,
    has_stakers: bool,
    initialize_first_session: bool,
    pub min_nominator_bond: Balance,
    min_validator_bond: Balance,
    balance_factor: Balance,
    status: BTreeMap<AccountId, StakerStatus<AccountId>>,
    stakes: BTreeMap<AccountId, Balance>,
    stakers: Vec<(
        IdentityId,
        AccountId,
        AccountId,
        Balance,
        StakerStatus<AccountId>,
    )>,
    slashing_allowed_for: SlashingSwitch,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            nominate: true,
            validator_count: 2,
            minimum_validator_count: 0,
            balance_factor: 1,
            invulnerables: vec![],
            has_stakers: true,
            initialize_first_session: true,
            min_nominator_bond: ExistentialDeposit::get(),
            min_validator_bond: ExistentialDeposit::get(),
            status: Default::default(),
            stakes: Default::default(),
            stakers: Default::default(),
            slashing_allowed_for: SlashingSwitch::Validator,
        }
    }
}

impl ExtBuilder {
    pub fn existential_deposit(self, existential_deposit: Balance) -> Self {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = existential_deposit);
        self
    }
    pub fn nominate(mut self, nominate: bool) -> Self {
        self.nominate = nominate;
        self
    }
    pub fn validator_count(mut self, count: u32) -> Self {
        self.validator_count = count;
        self
    }
    pub fn minimum_validator_count(mut self, count: u32) -> Self {
        self.minimum_validator_count = count;
        self
    }
    pub fn slash_defer_duration(self, eras: EraIndex) -> Self {
        SLASH_DEFER_DURATION.with(|v| *v.borrow_mut() = eras);
        self
    }
    pub fn invulnerables(mut self, invulnerables: Vec<AccountId>) -> Self {
        self.invulnerables = invulnerables;
        self
    }
    pub fn session_per_era(self, length: SessionIndex) -> Self {
        SESSIONS_PER_ERA.with(|v| *v.borrow_mut() = length);
        self
    }
    pub fn period(self, length: BlockNumber) -> Self {
        PERIOD.with(|v| *v.borrow_mut() = length);
        self
    }
    pub fn has_stakers(mut self, has: bool) -> Self {
        self.has_stakers = has;
        self
    }
    pub fn initialize_first_session(mut self, init: bool) -> Self {
        self.initialize_first_session = init;
        self
    }
    pub fn offset(self, offset: BlockNumber) -> Self {
        OFFSET.with(|v| *v.borrow_mut() = offset);
        self
    }
    pub fn min_nominator_bond(mut self, amount: Balance) -> Self {
        self.min_nominator_bond = amount;
        self
    }
    pub fn min_validator_bond(mut self, amount: Balance) -> Self {
        self.min_validator_bond = amount;
        self
    }
    pub fn set_status(mut self, who: AccountId, status: StakerStatus<AccountId>) -> Self {
        self.status.insert(who, status);
        self
    }
    pub fn set_stake(mut self, who: AccountId, stake: Balance) -> Self {
        self.stakes.insert(who, stake);
        self
    }
    pub fn add_staker(
        mut self,
        identity: IdentityId,
        stash: AccountId,
        ctrl: AccountId,
        stake: Balance,
        status: StakerStatus<AccountId>,
    ) -> Self {
        self.stakers.push((identity, stash, ctrl, stake, status));
        self
    }
    pub fn balance_factor(mut self, factor: Balance) -> Self {
        self.balance_factor = factor;
        self
    }
    fn build(self) -> sp_io::TestExternalities {
        sp_tracing::try_init_simple();
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 10 * self.balance_factor),
                (2, 20 * self.balance_factor),
                (3, 300 * self.balance_factor),
                (4, 400 * self.balance_factor),
                // controllers
                (10, self.balance_factor),
                (20, self.balance_factor),
                (30, self.balance_factor),
                (40, self.balance_factor),
                (50, self.balance_factor),
                // stashes
                (11, self.balance_factor * 1000),
                (21, self.balance_factor * 2000),
                (31, self.balance_factor * 2000),
                (41, self.balance_factor * 2000),
                (51, self.balance_factor * 2000),
                // optional nominator
                (100, self.balance_factor * 2000),
                (101, self.balance_factor * 2000),
                // aux accounts
                (60, self.balance_factor),
                (61, self.balance_factor * 2000),
                (70, self.balance_factor),
                (71, self.balance_factor * 2000),
                (80, self.balance_factor),
                (81, self.balance_factor * 2000),
                // This allows us to have a total_payout different from 0.
                (999, 1_000_000_000_000),
            ],
        }
        .assimilate_storage(&mut storage);

        pallet_group::GenesisConfig::<Test, pallet_group::Instance2> {
            active_members_limit: u32::MAX,
            active_members: vec![IdentityId::from(1), IdentityId::from(2)],
            phantom: Default::default(),
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        pallet_identity::GenesisConfig::<Test> {
            identities: vec![
                // (primary_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                // Provide Identity
                GenesisIdentityRecord {
                    primary_key: Some(1005),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(1),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(11),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(11),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(21),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(21),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(31),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(31),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(41),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(41),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(101),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(101),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(61),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(61),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(71),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(71),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
                GenesisIdentityRecord {
                    primary_key: Some(81),
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(81),
                    secondary_keys: Default::default(),
                    cdd_claim_expiry: None,
                },
            ],
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let mut stakers = vec![];
        if self.has_stakers {
            stakers = vec![
                // (stash, ctrl, stake, status)
                // these two will be elected in the default test where we elect 2.
                (
                    IdentityId::from(11),
                    11,
                    10,
                    self.balance_factor * 1000,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    IdentityId::from(21),
                    21,
                    20,
                    self.balance_factor * 1000,
                    StakerStatus::<AccountId>::Validator,
                ),
                // a loser validator
                (
                    IdentityId::from(31),
                    31,
                    30,
                    self.balance_factor * 500,
                    StakerStatus::<AccountId>::Validator,
                ),
                // an idle validator
                (
                    IdentityId::from(41),
                    41,
                    40,
                    self.balance_factor * 1000,
                    StakerStatus::<AccountId>::Idle,
                ),
            ];
            // optionally add a nominator
            if self.nominate {
                stakers.push((
                    IdentityId::from(101),
                    101,
                    100,
                    self.balance_factor * 500,
                    StakerStatus::<AccountId>::Nominator(vec![11, 21]),
                ))
            }

            // replace any of the status if needed.
            self.status.into_iter().for_each(|(stash, status)| {
                let (_, _, _, _, ref mut prev_status) = stakers
                    .iter_mut()
                    .find(|s| s.1 == stash)
                    .expect("set_status staker should exist; qed");
                *prev_status = status;
            });

            // replaced any of the stakes if needed.
            self.stakes.into_iter().for_each(|(stash, stake)| {
                let (_, _, _, ref mut prev_stake, _) = stakers
                    .iter_mut()
                    .find(|s| s.1 == stash)
                    .expect("set_stake staker should exits; qed.");
                *prev_stake = stake;
            });
            // extend stakers if needed.
            stakers.extend(self.stakers)
        }

        let _ = pallet_staking::GenesisConfig::<Test> {
            stakers: stakers.clone(),
            validator_count: self.validator_count,
            minimum_validator_count: self.minimum_validator_count,
            invulnerables: self.invulnerables,
            slash_reward_fraction: Perbill::from_percent(10),
            min_nominator_bond: self.min_nominator_bond,
            min_validator_bond: self.min_validator_bond,
            slashing_allowed_for: self.slashing_allowed_for,
            ..Default::default()
        }
        .assimilate_storage(&mut storage);

        let _ = pallet_session::GenesisConfig::<Test> {
            keys: if self.has_stakers {
                // set the keys for the first session.
                stakers
                    .into_iter()
                    .map(|(_, id, ..)| (id, id, SessionKeys { other: id.into() }))
                    .collect()
            } else {
                // set some dummy validators in genesis.
                (0..self.validator_count as u64)
                    .map(|id| (id, id, SessionKeys { other: id.into() }))
                    .collect()
            },
        }
        .assimilate_storage(&mut storage);

        let mut ext = sp_io::TestExternalities::from(storage);

        if self.initialize_first_session {
            // We consider all test to start after timestamp is initialized This must be ensured by
            // having `timestamp::on_initialize` called before `staking::on_initialize`. Also, if
            // session length is 1, then it is already triggered.
            ext.execute_with(|| {
                System::set_block_number(1);
                Session::on_initialize(1);
                <Staking as Hooks<u64>>::on_initialize(1);
                Timestamp::set_timestamp(INIT_TIMESTAMP);
            });
        }

        ext
    }
    pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
        sp_tracing::try_init_simple();
        let mut ext = self.build();
        ext.execute_with(test);
        ext.execute_with(|| {
            Staking::do_try_state(System::block_number()).unwrap();
        });
    }
}

pub type Group = pallet_group::Module<Test, pallet_group::Instance2>;

pub fn provide_did_to_user(account: AccountId) -> bool {
    if pallet_identity::KeyRecords::<Test>::contains_key(&account) {
        return false;
    }

    let cdd_account_id = 1005;
    let cdd = RuntimeOrigin::signed(cdd_account_id);
    assert!(
        pallet_identity::KeyRecords::<Test>::contains_key(&cdd_account_id),
        "CDD provider account not mapped to identity"
    );

    let cdd_did = pallet_identity::Module::<Test>::get_identity(&cdd_account_id)
        .expect("CDD provider missing identity");
    assert!(
        pallet_identity::DidRecords::<Test>::contains_key(&cdd_did),
        "CDD provider identity has no DID record"
    );

    let cdd_did_record = pallet_identity::DidRecords::<Test>::get(&cdd_did).unwrap_or_default();
    assert!(
        cdd_did_record.primary_key == Some(cdd_account_id),
        "CDD identity primary key mismatch"
    );
    assert!(
        pallet_identity::Module::<Test>::cdd_register_did(cdd.clone(), account, vec![]).is_ok(),
        "Error in registering the DID"
    );

    let did = pallet_identity::Module::<Test>::get_identity(&account)
        .expect("DID not find in the storage");
    assert!(
        pallet_identity::Module::<Test>::add_claim(
            cdd.clone(),
            did,
            Claim::CustomerDueDiligence(Default::default()),
            None
        )
        .is_ok(),
        "Error CDD Claim cannot be added to DID"
    );
    true
}

pub fn add_secondary_key(stash_key: AccountId, to_secondary_key: AccountId) {
    if !get_identity(to_secondary_key) {
        pallet_identity::Module::<Test>::get_identity(&stash_key).unwrap();
        assert!(
            pallet_identity::Module::<Test>::add_authorization(
                RuntimeOrigin::signed(stash_key),
                Signatory::Account(to_secondary_key),
                AuthorizationData::JoinIdentity(Permissions::default()),
                None
            )
            .is_ok(),
            "Error in providing the authorization"
        );

        let auth_id = get_last_auth_id(&Signatory::Account(to_secondary_key));
        assert_ok!(pallet_identity::Module::<Test>::join_identity_as_key(
            RuntimeOrigin::signed(to_secondary_key),
            auth_id
        ));
    }
}

pub(crate) fn active_era() -> EraIndex {
    Staking::active_era().unwrap().index
}

pub(crate) fn current_era() -> EraIndex {
    Staking::current_era().unwrap()
}

pub(crate) fn bond(stash: AccountId, ctrl: AccountId, val: Balance) {
    let _ = Balances::make_free_balance_be(&stash, val);
    let _ = Balances::make_free_balance_be(&ctrl, val);

    provide_did_to_user(stash);
    add_secondary_key(stash, ctrl);

    if Staking::bonded(&stash).is_none() {
        assert_ok!(Staking::bond(
            RuntimeOrigin::signed(stash),
            ctrl,
            val,
            RewardDestination::Controller,
        ));
    }
}

pub(crate) fn bond_validator(stash: AccountId, ctrl: AccountId, val: Balance) {
    bond_validator_with_intended_count(stash, ctrl, val, None)
}

pub fn bond_validator_with_intended_count(
    stash: AccountId,
    ctrl: AccountId,
    val: Balance,
    i_count: Option<u32>,
) {
    bond(stash, ctrl, val);

    let stash_id = Identity::get_identity(&stash).unwrap();
    if Staking::permissioned_identity(stash_id).is_none() {
        assert_ok!(Staking::add_permissioned_validator(
            frame_system::RawOrigin::Root.into(),
            stash_id,
            i_count
        ));
    }

    assert_ok!(Staking::validate(
        RuntimeOrigin::signed(ctrl),
        ValidatorPrefs::default()
    ));
    assert_ok!(Session::set_keys(
        RuntimeOrigin::signed(ctrl),
        SessionKeys { other: ctrl.into() },
        vec![]
    ));
}

pub(crate) fn bond_nominator(
    stash: AccountId,
    ctrl: AccountId,
    val: Balance,
    target: Vec<AccountId>,
) {
    bond(stash, ctrl, val);
    assert_ok!(Staking::nominate(RuntimeOrigin::signed(ctrl), target));
}

/// Progress to the given block, triggering session and era changes as we progress.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    Staking::on_finalize(System::block_number());
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
        Session::on_initialize(b);
        <Staking as Hooks<u64>>::on_initialize(b);
        Timestamp::set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);
        if b != n {
            Staking::on_finalize(System::block_number());
        }
    }
}

/// Progresses from the current block number (whatever that may be) to the `P * session_index + 1`.
pub(crate) fn start_session(session_index: SessionIndex) {
    let end: u64 = if Offset::get().is_zero() {
        (session_index as u64) * Period::get()
    } else {
        Offset::get() + (session_index.saturating_sub(1) as u64) * Period::get()
    };
    run_to_block(end);
    // session must have progressed properly.
    assert_eq!(
        Session::current_index(),
        session_index,
        "current session index = {}, expected = {}",
        Session::current_index(),
        session_index,
    );
}

/// Go one session forward.
pub(crate) fn advance_session() {
    let current_index = Session::current_index();
    start_session(current_index + 1);
}

/// Progress until the given era.
pub(crate) fn start_active_era(era_index: EraIndex) {
    start_session((era_index * <SessionsPerEra as Get<u32>>::get()).into());
    assert_eq!(active_era(), era_index);
    // One way or another, current_era must have changed before the active era, so they must match
    // at this point.
    assert_eq!(current_era(), active_era());
}

pub(crate) fn current_total_payout_for_duration(duration: u64) -> Balance {
    let reward = inflation::compute_total_payout(
        &I_NPOS,
        Staking::eras_total_stake(active_era()),
        Balances::total_issuance(),
        duration,
        <Test as Config>::MaxVariableInflationTotalIssuance::get(),
        <Test as Config>::FixedYearlyReward::get(),
    )
    .0;
    assert!(reward > 0);
    reward
}

pub(crate) fn maximum_payout_for_duration(duration: u64) -> Balance {
    inflation::compute_total_payout(
        &I_NPOS,
        0,
        Balances::total_issuance(),
        duration,
        <Test as Config>::MaxVariableInflationTotalIssuance::get(),
        <Test as Config>::FixedYearlyReward::get(),
    )
    .1
}

/// Time it takes to finish a session.
///
/// Note, if you see `time_per_session() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_session() -> u64 {
    Period::get() * BLOCK_TIME
}

/// Time it takes to finish an era.
///
/// Note, if you see `time_per_era() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_era() -> u64 {
    time_per_session() * SessionsPerEra::get() as u64
}

/// Time that will be calculated for the reward per era.
pub(crate) fn reward_time_per_era() -> u64 {
    time_per_era() - BLOCK_TIME
}

pub(crate) fn reward_all_elected() {
    let rewards = <Test as Config>::SessionInterface::validators()
        .into_iter()
        .map(|v| (v, 1));

    <Pallet<Test>>::reward_by_ids(rewards)
}

pub(crate) fn validator_controllers() -> Vec<AccountId> {
    Session::validators()
        .into_iter()
        .map(|s| Staking::bonded(&s).expect("no controller for validator"))
        .collect()
}

pub(crate) fn on_offence_in_era(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_fraction: &[Perbill],
    era: EraIndex,
    disable_strategy: DisableStrategy,
) {
    let bonded_eras = pallet_staking::BondedEras::<Test>::get();
    for &(bonded_era, start_session) in bonded_eras.iter() {
        if bonded_era == era {
            let _ = Staking::on_offence(offenders, slash_fraction, start_session, disable_strategy);
            return;
        } else if bonded_era > era {
            break;
        }
    }

    if Staking::active_era().unwrap().index == era {
        let _ = Staking::on_offence(
            offenders,
            slash_fraction,
            Staking::eras_start_session_index(era).unwrap(),
            disable_strategy,
        );
    } else {
        panic!("cannot slash in era {}", era);
    }
}

pub(crate) fn on_offence_now(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_fraction: &[Perbill],
) {
    let now = Staking::active_era().unwrap().index;
    on_offence_in_era(offenders, slash_fraction, now, DisableStrategy::WhenSlashed)
}

pub(crate) fn add_slash(who: &AccountId) {
    on_offence_now(
        &[OffenceDetails {
            offender: (*who, Staking::eras_stakers(active_era(), *who)),
            reporters: vec![],
        }],
        &[Perbill::from_percent(10)],
    );
}

/// Make all validator and nominator request their payment
pub(crate) fn make_all_reward_payment(era: EraIndex) {
    let validators_with_reward = ErasRewardPoints::<Test>::get(era)
        .individual
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    // reward validators
    for validator_controller in validators_with_reward.iter().filter_map(Staking::bonded) {
        let ledger = <Ledger<Test>>::get(&validator_controller).unwrap();
        assert_ok!(Staking::payout_stakers(
            RuntimeOrigin::signed(1337),
            ledger.stash,
            era
        ));
    }
}

#[macro_export]
macro_rules! assert_session_era {
    ($session:expr, $era:expr) => {
        assert_eq!(
            Session::current_index(),
            $session,
            "wrong session {} != {}",
            Session::current_index(),
            $session,
        );
        assert_eq!(
            Staking::current_era().unwrap(),
            $era,
            "wrong current era {} != {}",
            Staking::current_era().unwrap(),
            $era,
        );
    };
}

pub(crate) fn staking_events() -> Vec<pallet_staking::Event<Test>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let RuntimeEvent::Staking(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}

parameter_types! {
    static StakingEventsIndex: usize = 0;
}
ord_parameter_types! {
    pub const One: u64 = 1;
}

type EnsureOneOrRoot = EitherOfDiverse<EnsureRoot<AccountId>, EnsureSignedBy<One, AccountId>>;

pub(crate) fn staking_events_since_last_call() -> Vec<pallet_staking::Event<Test>> {
    let all: Vec<_> = System::events()
        .into_iter()
        .filter_map(|r| {
            if let RuntimeEvent::Staking(inner) = r.event {
                Some(inner)
            } else {
                None
            }
        })
        .collect();
    let seen = StakingEventsIndex::get();
    StakingEventsIndex::set(all.len());
    all.into_iter().skip(seen).collect()
}

pub(crate) fn balances(who: &AccountId) -> (Balance, Balance) {
    (Balances::free_balance(who), Balances::reserved_balance(who))
}

pub fn get_identity(key: AccountId) -> bool {
    pallet_identity::KeyRecords::<Test>::contains_key(&key)
}

pub fn bond_nominator_cdd(stash: AccountId, ctrl: AccountId, val: Balance, target: Vec<AccountId>) {
    provide_did_to_user(stash);
    add_secondary_key(stash, ctrl);
    bond_nominator(stash, ctrl, val, target);
}

// `iter_prefix_values` has no guarantee that it will iterate in a sequential
// order. However, we need the latest `auth_id`. Which is why we search for the claim
// with the highest `auth_id`.
pub fn get_last_auth(signatory: &Signatory<AccountId>) -> Authorization<AccountId, u64> {
    pallet_identity::Authorizations::<Test>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
}

pub fn get_last_auth_id(signatory: &Signatory<AccountId>) -> u64 {
    get_last_auth(signatory).auth_id
}

// Polymesh change
// -----------------------------------------------------------------

pub type Origin = <Test as frame_system::Config>::RuntimeOrigin;

fn get_primary_key(target: IdentityId) -> AccountId {
    Identity::get_primary_key(target).unwrap_or_default()
}

pub fn make_account(account_id: AccountId) -> (Origin, IdentityId) {
    make_account_with_balance(account_id, 1_000_000, None)
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    account_id: AccountId,
    balance: Balance,
    expiry: Option<Moment>,
) -> (Origin, IdentityId) {
    Balances::make_free_balance_be(&account_id, balance);

    let signed_account = Origin::signed(account_id.clone());

    let cdd_providers = Group::get_members();
    if let Some(cdd_provider_did) = cdd_providers.into_iter().nth(0) {
        let cdd_provider_acc = get_primary_key(cdd_provider_did);
        Identity::cdd_register_did(Origin::signed(cdd_provider_acc), account_id, vec![]).unwrap();
        let account_did = Identity::get_identity(&account_id).unwrap();
        Identity::add_claim(
            Origin::signed(cdd_provider_acc),
            account_did,
            Claim::CustomerDueDiligence(Default::default()),
            expiry,
        )
        .unwrap();
        return (signed_account, account_did);
    }

    TestUtils::register_did(signed_account.clone(), vec![]).unwrap();
    let account_did = Identity::get_identity(&account_id).unwrap();

    (signed_account, account_did)
}

pub fn add_trusted_cdd_provider(identity: IdentityId) {
    assert_ok!(Group::add_member(
        Origin::from(frame_system::RawOrigin::Root),
        identity
    ));
}

pub fn run_to_block_scheduler(n: u64) {
    while System::block_number() < n {
        Staking::on_finalize(System::block_number());
        Scheduler::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        Scheduler::on_initialize(System::block_number());
        Session::on_initialize(System::block_number());
        Staking::on_initialize(System::block_number());
        Staking::on_finalize(System::block_number());
    }
}

pub fn bond_nominator_with_expiry(acc: u64, val: u128, claim_expiry: u64, target: Vec<AccountId>) {
    // a = controller
    // a + 1 = stash
    let controller = acc;
    let stash = acc + 1;
    let _ = Balances::make_free_balance_be(&(stash), val);
    assert_ok!(Staking::bond(
        Origin::signed(stash),
        controller,
        val,
        RewardDestination::Controller
    ));
    create_did_and_add_claim_with_expiry(stash, claim_expiry);
    assert_ok!(Staking::nominate(Origin::signed(controller), target));
}

pub fn create_did_and_add_claim_with_expiry(stash: AccountId, expiry: u64) {
    Balances::make_free_balance_be(&1005, 1_000_000);
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(1005),
        stash,
        vec![]
    ));
    let did = Identity::get_identity(&stash).unwrap();
    assert_ok!(Identity::add_claim(
        Origin::signed(1005),
        did,
        Claim::CustomerDueDiligence(Default::default()),
        Some(expiry.into())
    ));
}

// -----------------------------------------------------------------
