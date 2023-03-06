// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities

use crate::asset_test::set_timestamp;
use crate::storage::create_cdd_id;
use chrono::prelude::Utc;
use frame_election_provider_support::NposSolution;
use frame_support::{
    assert_ok,
    dispatch::DispatchResult,
    parameter_types,
    traits::{
        Contains, Currency, FindAuthor, GenesisBuild as _, Get, Imbalance, KeyOwnerProofSystem,
        OnFinalize, OnInitialize, OnUnbalanced, OneSessionHandler, SortedMembers,
    },
    weights::{constants::RocksDbWeight, DispatchInfo, Weight},
    IterableStorageMap, StorageDoubleMap, StorageMap, StorageValue,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use pallet_group as group;
use pallet_identity as identity;
use pallet_protocol_fee as protocol_fee;
use pallet_staking::{self as staking, *};
use polymesh_common_utilities::{
    constants::currency::*,
    traits::{
        asset::AssetSubTrait,
        balances::{AccountData, CheckCdd},
        group::{GroupTrait, InactiveMember},
        multisig::MultiSigSubTrait,
        portfolio::PortfolioSubTrait,
        relayer::SubsidiserTrait,
        transaction_payment::{CddAndFeeDetails, ChargeTxFee},
        CommonConfig,
    },
};
use polymesh_primitives::{
    identity_id::GenesisIdentityRecord, Authorization, AuthorizationData, CddId, Claim, IdentityId,
    InvestorUid, Moment, NFTId, Permissions, PortfolioId, ScopeId, SecondaryKey, Signatory, Ticker,
};
use sp_core::H256;
use sp_npos_elections::{
    reduce, to_supports, ElectionScore, EvaluateSupport, ExtendedBalance, StakedAssignment,
};
use sp_runtime::{
    curve::PiecewiseLinear,
    testing::{Header, TestSignature, TestXt, UintAuthorityId},
    traits::{IdentityLookup, Zero},
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    KeyTypeId, Perbill, Permill,
};
use sp_staking::{
    offence::{DisableStrategy, OffenceDetails, OnOffenceHandler},
    SessionIndex,
};
use std::{cell::RefCell, collections::BTreeMap};

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 1000;

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

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Staking: staking::{Pallet, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
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
    pub const BlockHashCount: u64 = 250;
    pub const MaxLen: u32 = 256;
    pub const MaxLocks: u32 = 1024;
    pub const MaximumBlockWeight: Weight = 1024;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(
            frame_support::weights::constants::WEIGHT_PER_SECOND * 2
        );
    pub static SessionsPerEra: SessionIndex = 3;
    pub static ExistentialDeposit: Balance = 1;
    pub static SlashDeferDuration: EraIndex = 0;
    pub static ElectionLookahead: BlockNumber = 0;
    pub static Period: BlockNumber = 5;
    pub static Offset: BlockNumber = 0;
    pub static MaxIterations: u32 = 0;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
    type Origin = Origin;
    type Index = AccountIndex;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = AccountData;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type OnSetCode = ();
    type SS58Prefix = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_base::Config for Test {
    type Event = Event;
    type MaxLen = MaxLen;
}

impl CommonConfig for Test {
    type AssetSubTraitTarget = Test;
    type BlockRewardsReserve = pallet_balances::Module<Test>;
}

impl pallet_balances::Config for Test {
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type CddChecker = Test;
    type WeightInfo = polymesh_weights::pallet_balances::WeightInfo;
    type MaxLocks = MaxLocks;
}

parameter_types! {
    pub const UncleGenerations: u64 = 0;
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
    type Event = Event;
    type ValidatorId = AccountId;
    type ValidatorIdOf = StashOf<Test>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type WeightInfo = ();
}

impl pallet_committee::Config<pallet_committee::Instance1> for Test {
    type Origin = Origin;
    type Proposal = Call;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type VoteThresholdOrigin = Self::CommitteeOrigin;
    type Event = Event;
    type WeightInfo = polymesh_weights::pallet_committee::WeightInfo;
}

impl pallet_session::historical::Config for Test {
    type FullIdentification = Exposure<AccountId, Balance>;
    type FullIdentificationOf = ExposureOf<Test>;
}

impl pallet_pips::Config for Test {
    type Currency = pallet_balances::Module<Self>;
    type VotingMajorityOrigin = frame_system::EnsureRoot<AccountId>;
    type GovernanceCommittee = crate::storage::Committee;
    type TechnicalCommitteeVMO = frame_system::EnsureRoot<AccountId>;
    type UpgradeCommitteeVMO = frame_system::EnsureRoot<AccountId>;
    type Event = Event;
    type WeightInfo = polymesh_weights::pallet_pips::WeightInfo;
    type Scheduler = Scheduler;
    type SchedulerCall = Call;
}

impl pallet_treasury::Config for Test {
    type Event = Event;
    type Currency = pallet_balances::Module<Self>;
    type WeightInfo = polymesh_weights::pallet_treasury::WeightInfo;
}

impl pallet_authorship::Config for Test {
    type FindAuthor = Author11;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = pallet_staking::Module<Test>;
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

impl group::Config<group::Instance2> for Test {
    type Event = Event;
    type LimitOrigin = frame_system::EnsureRoot<AccountId>;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type WeightInfo = polymesh_weights::pallet_group::WeightInfo;
}

impl protocol_fee::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type OnProtocolFeePayment = ();
    type WeightInfo = polymesh_weights::pallet_protocol_fee::WeightInfo;
    type Subsidiser = Test;
}

impl polymesh_common_utilities::traits::identity::Config for Test {
    type Event = Event;
    type Proposal = Call;
    type MultiSig = Test;
    type Portfolio = Test;
    type CddServiceProviders = group::Module<Test, group::Instance2>;
    type Balances = Balances;
    type ChargeTxFeeTarget = Test;
    type CddHandler = Test;
    type Public = UintAuthorityId;
    type OffChainSignature = TestSignature;
    type ProtocolFee = protocol_fee::Module<Test>;
    type GCVotingMajorityOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = polymesh_weights::pallet_identity::WeightInfo;
    type IdentityFn = identity::Module<Test>;
    type SchedulerOrigin = OriginCaller;
    type InitialPOLYX = InitialPOLYX;
    type MultiSigBalanceLimit = polymesh_runtime_common::MultiSigBalanceLimit;
}

parameter_types! {
    pub const InitialPOLYX: Balance = 0;
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u64> = Some(10);
}

impl pallet_scheduler::Config for Test {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
    type OriginPrivilegeCmp = frame_support::traits::EqualPrivilegeOnly;
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = polymesh_runtime_common::deposit(2, 64);
    pub const PreimageByteDeposit: Balance = polymesh_runtime_common::deposit(0, 1);
}

impl pallet_preimage::Config for Test {
    type WeightInfo = polymesh_weights::pallet_preimage::WeightInfo;
    type Event = Event;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

impl pallet_test_utils::Config for Test {
    type Event = Event;
    type WeightInfo = polymesh_weights::pallet_test_utils::WeightInfo;
}

impl CddAndFeeDetails<AccountId, Call> for Test {
    fn get_valid_payer(_: &Call, _: &AccountId) -> Result<Option<AccountId>, InvalidTransaction> {
        Ok(None)
    }
    fn clear_context() {}
    fn set_payer_context(_: Option<AccountId>) {}
    fn get_payer_from_context() -> Option<AccountId> {
        None
    }
    fn set_current_identity(_: &IdentityId) {}
}

impl SubsidiserTrait<AccountId> for Test {
    fn check_subsidy(
        _: &AccountId,
        _: Balance,
        _: Option<&[u8]>,
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

impl AssetSubTrait for Test {
    fn update_balance_of_scope_id(_: ScopeId, _: IdentityId, _: Ticker) {}
    fn balance_of_at_scope(_: &ScopeId, _: &IdentityId) -> Balance {
        0
    }
    fn scope_id(_: &Ticker, _: &IdentityId) -> ScopeId {
        ScopeId::from(0u128)
    }
    fn ensure_investor_uniqueness_claims_allowed(_: &Ticker) -> DispatchResult {
        Ok(())
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

    fn lock_tokens(_: &PortfolioId, _: &Ticker, _: Balance) -> DispatchResult {
        unimplemented!()
    }

    fn unlock_tokens(_: &PortfolioId, _: &Ticker, _: Balance) -> DispatchResult {
        unimplemented!()
    }

    fn ensure_portfolio_custody_and_permission(
        _: PortfolioId,
        _: IdentityId,
        _: Option<&SecondaryKey<AccountId>>,
    ) -> DispatchResult {
        unimplemented!()
    }

    fn lock_nft(_: &PortfolioId, _: &Ticker, _: &NFTId) -> DispatchResult {
        unimplemented!()
    }

    fn unlock_nft(_: &PortfolioId, _: &Ticker, _: &NFTId) -> DispatchResult {
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
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const UnsignedPriority: u64 = 1 << 20;
    pub const MinSolutionScoreBump: Perbill = Perbill::zero();
    pub const MaxValidatorPerIdentity: Permill = Permill::from_percent(33);
    pub const MaxVariableInflationTotalIssuance: Balance = 1_000_000_000 * POLY;
    pub const FixedYearlyReward: Balance = 200_000_000 * POLY;
    pub const MinimumBond: Balance = 10u128;
}

thread_local! {
    pub static REWARD_REMAINDER_UNBALANCED: RefCell<u128> = RefCell::new(0);
}

pub struct RewardRemainderMock;

impl OnUnbalanced<NegativeImbalanceOf<Test>> for RewardRemainderMock {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
        REWARD_REMAINDER_UNBALANCED.with(|v| {
            *v.borrow_mut() += amount.peek();
        });
        drop(amount);
    }
}

parameter_types! {
    pub const TwoThousand: AccountId = 2000;
    pub const ThreeThousand: AccountId = 3000;
    pub const FourThousand: AccountId = 4000;
    pub const FiveThousand: AccountId = 5000;
}

impl Contains<u64> for TwoThousand {
    fn contains(t: &u64) -> bool {
        TwoThousand::get() == *t
    }
}

impl SortedMembers<u64> for TwoThousand {
    fn sorted_members() -> Vec<u64> {
        vec![TwoThousand::get()]
    }
}

impl Config for Test {
    const MAX_NOMINATIONS: u32 = pallet_staking::MAX_NOMINATIONS;
    type Currency = Balances;
    type UnixTime = Timestamp;
    type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
    type RewardRemainder = RewardRemainderMock;
    type Event = Event;
    type Slash = ();
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type BondingDuration = BondingDuration;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
    type UnsignedPriority = UnsignedPriority;
    type OffchainSolutionWeightLimit = polymesh_runtime_common::OffchainSolutionWeightLimit;
    type WeightInfo = polymesh_weights::pallet_staking::WeightInfo;
    type RequiredAddOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredRemoveOrigin = EnsureSignedBy<TwoThousand, Self::AccountId>;
    type RequiredCommissionOrigin = frame_system::EnsureRoot<AccountId>;
    type RewardScheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxValidatorPerIdentity = MaxValidatorPerIdentity;
    type MaxVariableInflationTotalIssuance = MaxVariableInflationTotalIssuance;
    type FixedYearlyReward = FixedYearlyReward;
    type MinimumBond = MinimumBond;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

pub type Extrinsic = TestXt<Call, ()>;

pub struct ExtBuilder {
    validator_pool: bool,
    nominate: bool,
    validator_count: u32,
    minimum_validator_count: u32,
    fair: bool,
    num_validators: Option<u32>,
    invulnerables: Vec<AccountId>,
    has_stakers: bool,
    initialize_first_session: bool,
    max_offchain_iterations: u32,
    slashing_allowed_for: SlashingSwitch,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            validator_pool: false,
            nominate: true,
            validator_count: 2,
            minimum_validator_count: 0,
            fair: true,
            num_validators: None,
            invulnerables: vec![],
            has_stakers: true,
            initialize_first_session: true,
            max_offchain_iterations: 0,
            slashing_allowed_for: SlashingSwitch::Validator,
        }
    }
}

impl ExtBuilder {
    pub fn existential_deposit(self, existential_deposit: Balance) -> Self {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = existential_deposit);
        self
    }
    pub fn validator_pool(mut self, validator_pool: bool) -> Self {
        self.validator_pool = validator_pool;
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
    pub fn fair(mut self, is_fair: bool) -> Self {
        self.fair = is_fair;
        self
    }
    pub fn num_validators(mut self, num_validators: u32) -> Self {
        self.num_validators = Some(num_validators);
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
    pub fn election_lookahead(self, look: BlockNumber) -> Self {
        ELECTION_LOOKAHEAD.with(|v| *v.borrow_mut() = look);
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
    pub fn max_offchain_iterations(self, iterations: u32) -> Self {
        MAX_ITERATIONS.with(|v| *v.borrow_mut() = iterations);
        self
    }
    pub fn offchain_election_ext(self) -> Self {
        self.session_per_era(4).period(5).election_lookahead(3)
    }
    pub fn initialize_first_session(mut self, init: bool) -> Self {
        self.initialize_first_session = init;
        self
    }
    pub fn offset(self, offset: BlockNumber) -> Self {
        OFFSET.with(|v| *v.borrow_mut() = offset);
        self
    }
    pub fn slashing_allowed_for(mut self, status: SlashingSwitch) -> Self {
        self.slashing_allowed_for = status;
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        sp_tracing::try_init_simple();
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        let balance_factor = if ExistentialDeposit::get() > 1 {
            256
        } else {
            1
        };

        let num_validators = self.num_validators.unwrap_or(self.validator_count);
        let validators = (0..num_validators)
            .map(|x| ((x + 1) * 10 + 1) as AccountId)
            .collect::<Vec<_>>();

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 10 * balance_factor),
                (2, 20 * balance_factor),
                (3, 300 * balance_factor),
                (4, 400 * balance_factor),
                (10, balance_factor),
                (11, balance_factor * 1000),
                (20, balance_factor),
                (21, balance_factor * 2000),
                (30, balance_factor),
                (31, balance_factor * 2000),
                (40, balance_factor),
                (41, balance_factor * 2000),
                (50, balance_factor),
                (51, balance_factor * 2000),
                (60, balance_factor),
                (61, balance_factor * 2000),
                (70, balance_factor),
                (71, balance_factor * 2000),
                (80, balance_factor),
                (81, balance_factor * 2000),
                (100, 2000 * balance_factor),
                (101, 2000 * balance_factor),
                // This allows us to have a total_payout different from 0.
                (999, 1_000_000_000_000),
            ],
        }
        .assimilate_storage(&mut storage);

        let _ = group::GenesisConfig::<Test, group::Instance2> {
            active_members_limit: u32::MAX,
            active_members: vec![IdentityId::from(1), IdentityId::from(2)],
            phantom: Default::default(),
        }
        .assimilate_storage(&mut storage);

        let _ = identity::GenesisConfig::<Test> {
            identities: vec![
                // (primary_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                // Provide Identity
                GenesisIdentityRecord {
                    primary_key: 1005,
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(1),
                    investor: InvestorUid::from(b"uid1".as_ref()),
                    ..Default::default()
                },
                GenesisIdentityRecord {
                    primary_key: 11,
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(11),
                    investor: InvestorUid::from(b"uid11".as_ref()),
                    ..Default::default()
                },
                GenesisIdentityRecord {
                    primary_key: 21,
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(21),
                    investor: InvestorUid::from(b"uid21".as_ref()),
                    ..Default::default()
                },
                GenesisIdentityRecord {
                    primary_key: 31,
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(31),
                    investor: InvestorUid::from(b"uid31".as_ref()),
                    ..Default::default()
                },
                GenesisIdentityRecord {
                    primary_key: 41,
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(41),
                    investor: InvestorUid::from(b"uid41".as_ref()),
                    ..Default::default()
                },
                GenesisIdentityRecord {
                    primary_key: 101,
                    issuers: vec![IdentityId::from(1)],
                    did: IdentityId::from(101),
                    investor: InvestorUid::from(b"uid101".as_ref()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
        .assimilate_storage(&mut storage);

        let mut stakers = vec![];
        if self.has_stakers {
            let stake_21 = if self.fair { 1000 } else { 2000 };
            let stake_31 = if self.validator_pool {
                balance_factor * 1000
            } else {
                MinimumBond::get()
            };
            let status_41 = if self.validator_pool {
                StakerStatus::<AccountId>::Validator
            } else {
                StakerStatus::<AccountId>::Idle
            };
            let nominated = if self.nominate { vec![11, 21] } else { vec![] };
            stakers = vec![
                // (IdentityId, stash, controller, staked_amount, status)
                (
                    IdentityId::from(11),
                    11,
                    10,
                    balance_factor * 1000,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    IdentityId::from(21),
                    21,
                    20,
                    stake_21,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    IdentityId::from(31),
                    31,
                    30,
                    stake_31,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    IdentityId::from(41),
                    41,
                    40,
                    balance_factor * 1000,
                    status_41,
                ),
                // nominator
                (
                    IdentityId::from(101),
                    101,
                    100,
                    balance_factor * 500,
                    StakerStatus::<AccountId>::Nominator(nominated),
                ),
            ];
        }
        let _ = pallet_staking::GenesisConfig::<Test> {
            stakers: stakers.clone(),
            validator_count: self.validator_count,
            minimum_validator_count: self.minimum_validator_count,
            invulnerables: self.invulnerables,
            slash_reward_fraction: Perbill::from_percent(10),
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
                validators
                    .into_iter()
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
                Staking::on_initialize(1);
                set_timestamp(INIT_TIMESTAMP);
            });
        }

        ext
    }
    pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
        let mut ext = self.build();
        ext.execute_with(test);
        ext.execute_with(post_conditions);
    }
}

pub type Group = group::Module<Test, group::Instance2>;

fn post_conditions() {
    check_nominators();
    check_exposures();
    check_ledgers();
}

pub(crate) fn active_era() -> EraIndex {
    Staking::active_era().unwrap().index
}

pub fn provide_did_to_user(account: AccountId) -> bool {
    if <identity::KeyRecords<Test>>::contains_key(&account) {
        return false;
    }
    let cdd_account_id = 1005;
    let cdd = Origin::signed(cdd_account_id);
    assert!(
        <identity::KeyRecords<Test>>::contains_key(&cdd_account_id),
        "CDD provider account not mapped to identity"
    );
    let cdd_did = Identity::get_identity(&cdd_account_id).expect("CDD provider missing identity");
    assert!(
        <identity::DidRecords<Test>>::contains_key(&cdd_did),
        "CDD provider identity has no DID record"
    );
    let cdd_did_record = <identity::DidRecords<Test>>::get(&cdd_did).unwrap_or_default();
    assert!(
        cdd_did_record.primary_key == Some(cdd_account_id),
        "CDD identity primary key mismatch"
    );
    assert!(
        Identity::cdd_register_did(cdd.clone(), account, vec![]).is_ok(),
        "Error in registering the DID"
    );
    let did = Identity::get_identity(&account).expect("DID not find in the storage");
    let (cdd_id, _) = create_cdd_id_and_investor_uid(did);
    assert!(
        Identity::add_claim(cdd.clone(), did, Claim::CustomerDueDiligence(cdd_id), None).is_ok(),
        "Error CDD Claim cannot be added to DID"
    );
    true
}

pub fn add_secondary_key(stash_key: AccountId, to_secondary_key: AccountId) {
    if !get_identity(to_secondary_key) {
        let _did = Identity::get_identity(&stash_key).unwrap();
        assert!(
            Identity::add_authorization(
                Origin::signed(stash_key),
                Signatory::Account(to_secondary_key),
                AuthorizationData::JoinIdentity(Permissions::default()),
                None
            )
            .is_ok(),
            "Error in providing the authorization"
        );
        let auth_id = get_last_auth_id(&Signatory::Account(to_secondary_key));
        assert_ok!(Identity::join_identity_as_key(
            Origin::signed(to_secondary_key),
            auth_id
        ));
    }
}

pub fn get_identity(key: AccountId) -> bool {
    <identity::KeyRecords<Test>>::contains_key(&key)
}

fn check_ledgers() {
    // check the ledger of all stakers.
    Bonded::<Test>::iter().for_each(|(_, ctrl)| assert_ledger_consistent(ctrl))
}

fn check_exposures() {
    // a check per validator to ensure the exposure struct is always sane.
    let era = active_era();
    ErasStakers::<Test>::iter_prefix_values(era).for_each(|expo| {
        assert_eq!(
            expo.total as u128,
            expo.own as u128 + expo.others.iter().map(|e| e.value as u128).sum::<u128>(),
            "wrong total exposure.",
        );
    })
}

fn check_nominators() {
    // a check per nominator to ensure their entire stake is correctly distributed. Will only kick-
    // in if the nomination was submitted before the current era.
    let era = active_era();
    <Nominators<Test>>::iter()
        .filter_map(|(nominator, nomination)| {
            if nomination.submitted_in > era {
                Some(nominator)
            } else {
                None
            }
        })
        .for_each(|nominator| {
            // must be bonded.
            assert_is_stash(nominator);
            let mut sum = 0;
            Session::validators()
                .iter()
                .map(|v| Staking::eras_stakers(era, v))
                .for_each(|e| {
                    let individual = e
                        .others
                        .iter()
                        .filter(|e| e.who == nominator)
                        .collect::<Vec<_>>();
                    let len = individual.len();
                    match len {
                        0 => { /* not supporting this validator at all. */ }
                        1 => sum += individual[0].value,
                        _ => panic!("nominator cannot back a validator more than once."),
                    };
                });

            let nominator_stake = Staking::slashable_balance_of(&nominator);
            // a nominator cannot over-spend.
            assert!(
                nominator_stake >= sum,
                "failed: Nominator({}) stake({}) >= sum divided({})",
                nominator,
                nominator_stake,
                sum,
            );

            let diff = nominator_stake - sum;
            assert!(diff < 100);
        });
}

fn assert_is_stash(acc: AccountId) {
    assert!(Staking::bonded(&acc).is_some(), "Not a stash.");
}

fn assert_ledger_consistent(ctrl: AccountId) {
    // ensures ledger.total == ledger.active + sum(ledger.unlocking).
    let ledger = Staking::ledger(ctrl).expect("Not a controller.");
    let real_total: Balance = ledger
        .unlocking
        .iter()
        .fold(ledger.active, |a, c| a + c.value);
    assert_eq!(real_total, ledger.total);
}

pub(crate) fn current_era() -> EraIndex {
    Staking::current_era().unwrap()
}

pub fn bond_validator(stash: AccountId, ctrl: AccountId, val: Balance) {
    bond_validator_with_intended_count(stash, ctrl, val, None)
}

pub fn bond_validator_with_intended_count(
    stash: AccountId,
    ctrl: AccountId,
    val: Balance,
    i_count: Option<u32>,
) {
    bond(stash, ctrl, val);
    let entity_id = Identity::get_identity(&stash).unwrap();
    if Staking::permissioned_identity(entity_id).is_none() {
        assert_ok!(Staking::add_permissioned_validator(
            frame_system::RawOrigin::Root.into(),
            entity_id,
            i_count
        ));
    }
    assert_ok!(Staking::validate(
        Origin::signed(ctrl),
        ValidatorPrefs::default()
    ));
    assert_ok!(Session::set_keys(
        Origin::signed(ctrl),
        SessionKeys { other: ctrl.into() },
        vec![]
    ));
}

pub fn bond(stash: AccountId, ctrl: AccountId, val: Balance) {
    let _ = Balances::make_free_balance_be(&stash, val);
    let _ = Balances::make_free_balance_be(&ctrl, val);
    provide_did_to_user(stash);
    add_secondary_key(stash, ctrl);
    if Staking::bonded(&stash).is_none() {
        assert_ok!(Staking::bond(
            Origin::signed(stash),
            ctrl,
            val,
            RewardDestination::Controller,
        ));
    }
}

pub(crate) fn bond_nominator(
    stash: AccountId,
    ctrl: AccountId,
    val: Balance,
    target: Vec<AccountId>,
) {
    let _ = Balances::make_free_balance_be(&stash, val);
    let _ = Balances::make_free_balance_be(&ctrl, val);
    assert_ok!(Staking::bond(
        Origin::signed(stash),
        ctrl,
        val,
        RewardDestination::Controller,
    ));
    assert_ok!(Staking::nominate(Origin::signed(ctrl), target));
}

pub fn bond_nominator_cdd(stash: AccountId, ctrl: AccountId, val: Balance, target: Vec<AccountId>) {
    provide_did_to_user(stash);
    add_secondary_key(stash, ctrl);
    bond_nominator(stash, ctrl, val, target);
}

/// Progress to the given block, triggering session and era changes as we progress.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    Staking::on_finalize(System::block_number());
    for b in System::block_number() + 1..=n {
        System::set_block_number(b);
        Session::on_initialize(b);
        Staking::on_initialize(b);
        set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);
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
        <Test as Config>::RewardCurve::get(),
        Staking::eras_total_stake(active_era()),
        Balances::total_issuance(),
        duration,
        MaxVariableInflationTotalIssuance::get(),
        FixedYearlyReward::get(),
    )
    .0;

    assert!(reward > 0);
    reward
}

pub(crate) fn maximum_payout_for_duration(duration: u64) -> Balance {
    inflation::compute_total_payout(
        <Test as Config>::RewardCurve::get(),
        0,
        Balances::total_issuance(),
        duration,
        MaxVariableInflationTotalIssuance::get(),
        FixedYearlyReward::get(),
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

    <Module<Test>>::reward_by_ids(rewards)
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
) {
    let bonded_eras = staking::BondedEras::get();
    for &(bonded_era, start_session) in bonded_eras.iter() {
        if bonded_era == era {
            let _ = Staking::on_offence(
                offenders,
                slash_fraction,
                start_session,
                DisableStrategy::WhenSlashed,
            );
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
            DisableStrategy::WhenSlashed,
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
    on_offence_in_era(offenders, slash_fraction, now)
}

pub(crate) fn add_slash(who: &AccountId) {
    on_offence_now(
        &[OffenceDetails {
            offender: (
                who.clone(),
                Staking::eras_stakers(Staking::active_era().unwrap().index, who.clone()),
            ),
            reporters: vec![],
        }],
        &[Perbill::from_percent(10)],
    );
}

// winners will be chosen by simply their unweighted total backing stake. Nominator stake is
// distributed evenly.
pub(crate) fn horrible_phragmen_with_post_processing(
    do_reduce: bool,
) -> (CompactAssignments, Vec<ValidatorIndex>, ElectionScore) {
    let mut backing_stake_of: BTreeMap<AccountId, Balance> = BTreeMap::new();

    // self stake
    <Validators<Test>>::iter().for_each(|(who, _p)| {
        *backing_stake_of.entry(who).or_insert(Zero::zero()) += Staking::slashable_balance_of(&who)
    });

    // add nominator stuff
    <Nominators<Test>>::iter().for_each(|(who, nomination)| {
        nomination.targets.iter().for_each(|v| {
            *backing_stake_of.entry(*v).or_insert(Zero::zero()) +=
                Staking::slashable_balance_of(&who)
        })
    });

    // elect winners
    let mut sorted: Vec<AccountId> = backing_stake_of.keys().cloned().collect();
    sorted.sort_by_key(|x| backing_stake_of.get(x).unwrap());
    let winners: Vec<AccountId> = sorted
        .iter()
        .cloned()
        .take(Staking::validator_count() as usize)
        .collect();

    // create assignments
    let mut staked_assignment: Vec<StakedAssignment<AccountId>> = Vec::new();
    <Nominators<Test>>::iter().for_each(|(who, nomination)| {
        let mut dist: Vec<(AccountId, ExtendedBalance)> = Vec::new();
        nomination.targets.iter().for_each(|v| {
            if winners.iter().find(|w| *w == v).is_some() {
                dist.push((*v, ExtendedBalance::zero()));
            }
        });

        if dist.len() == 0 {
            return;
        }

        // assign real stakes. just split the stake.
        let stake = Staking::slashable_balance_of(&who) as ExtendedBalance;
        let mut sum: ExtendedBalance = Zero::zero();
        let dist_len = dist.len();
        {
            dist.iter_mut().for_each(|(_, w)| {
                let partial = stake / (dist_len as ExtendedBalance);
                *w = partial;
                sum += partial;
            });
        }

        // assign the leftover to last.
        {
            let leftover = stake - sum;
            let last = dist.last_mut().unwrap();
            last.1 += leftover;
        }

        staked_assignment.push(StakedAssignment {
            who,
            distribution: dist,
        });
    });

    // Ensure that this result is worse than seq-phragmen. Otherwise, it should not have been used
    // for testing.
    let score = {
        let (_, _, better_score) = prepare_submission_with(true, true, 0, |_| {});

        let support = to_supports::<AccountId>(&staked_assignment);
        let score = support.evaluate();

        assert!(better_score.strict_threshold_better(score, MinSolutionScoreBump::get()));

        score
    };

    if do_reduce {
        reduce(&mut staked_assignment);
    }

    let snapshot_validators = Staking::snapshot_validators().unwrap();
    let snapshot_nominators = Staking::snapshot_nominators().unwrap();
    let nominator_index = |a: &AccountId| -> Option<NominatorIndex> {
        snapshot_nominators
            .iter()
            .position(|x| x == a)
            .map(|i| i as NominatorIndex)
    };
    let validator_index = |a: &AccountId| -> Option<ValidatorIndex> {
        snapshot_validators
            .iter()
            .position(|x| x == a)
            .map(|i| i as ValidatorIndex)
    };

    // convert back to ratio assignment. This takes less space.
    let assignments_reduced = sp_npos_elections::assignment_staked_to_ratio::<
        AccountId,
        OffchainAccuracy,
    >(staked_assignment);

    let compact =
        CompactAssignments::from_assignment(&assignments_reduced, nominator_index, validator_index)
            .unwrap();

    // winner ids to index
    let winners = winners
        .into_iter()
        .map(|w| validator_index(&w).unwrap())
        .collect::<Vec<_>>();

    (compact, winners, score)
}

// Note: this should always logically reproduce [`offchain_election::prepare_submission`], yet we
// cannot do it since we want to have `tweak` injected into the process.
pub(crate) fn prepare_submission_with(
    compute_real_score: bool,
    do_reduce: bool,
    iterations: usize,
    tweak: impl FnOnce(&mut Vec<StakedAssignment<AccountId>>),
) -> (CompactAssignments, Vec<ValidatorIndex>, ElectionScore) {
    // run election on the default stuff.
    let sp_npos_elections::ElectionResult {
        winners,
        assignments,
    } = Staking::do_phragmen::<OffchainAccuracy>(iterations).unwrap();
    let winners = winners.into_iter().map(|(who, _)| who).collect::<Vec<_>>();

    let mut staked = sp_npos_elections::assignment_ratio_to_staked(
        assignments,
        Staking::slashable_balance_of_fn(),
    );

    // apply custom tweaks. awesome for testing.
    tweak(&mut staked);

    if do_reduce {
        reduce(&mut staked);
    }

    // convert back to ratio assignment. This takes less space.
    let snapshot_validators = Staking::snapshot_validators().expect("snapshot not created.");
    let snapshot_nominators = Staking::snapshot_nominators().expect("snapshot not created.");
    let nominator_index = |a: &AccountId| -> Option<NominatorIndex> {
        snapshot_nominators.iter().position(|x| x == a).map_or_else(
            || {
                println!("unable to find nominator index for {:?}", a);
                None
            },
            |i| Some(i as NominatorIndex),
        )
    };
    let validator_index = |a: &AccountId| -> Option<ValidatorIndex> {
        snapshot_validators.iter().position(|x| x == a).map_or_else(
            || {
                println!("unable to find validator index for {:?}", a);
                None
            },
            |i| Some(i as ValidatorIndex),
        )
    };

    let assignments_reduced = sp_npos_elections::assignment_staked_to_ratio(staked);

    // re-compute score by converting, yet again, into staked type
    let score = if compute_real_score {
        let staked = sp_npos_elections::assignment_ratio_to_staked(
            assignments_reduced.clone(),
            Staking::slashable_balance_of_fn(),
        );

        let support_map = to_supports::<AccountId>(staked.as_slice());
        support_map.evaluate()
    } else {
        Default::default()
    };

    let compact =
        CompactAssignments::from_assignment(&assignments_reduced, nominator_index, validator_index)
            .expect("Failed to create compact");

    // winner ids to index
    let winners = winners
        .into_iter()
        .map(|w| validator_index(&w).unwrap())
        .collect::<Vec<_>>();

    (compact, winners, score)
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
            Origin::signed(1337),
            ledger.stash,
            era
        ));
    }
}

pub(crate) fn staking_events() -> Vec<staking::Event<Test>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::Staking(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn balances(who: &AccountId) -> (Balance, Balance) {
    (Balances::free_balance(who), Balances::reserved_balance(who))
}

fn get_primary_key(target: IdentityId) -> AccountId {
    Identity::get_primary_key(target).unwrap_or_default()
}

pub fn make_account_with_uid(
    id: AccountId,
) -> Result<(<Test as frame_system::Config>::Origin, IdentityId), &'static str> {
    make_account_with_balance(id, 1_000_000, None)
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    id: AccountId,
    balance: Balance,
    expiry: Option<Moment>,
) -> Result<(<Test as frame_system::Config>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id.clone());
    Balances::make_free_balance_be(&id, balance);
    let uid = create_investor_uid(id);
    // If we have CDD providers, first of them executes the registration.
    let cdd_providers = Group::get_members();
    let did = match cdd_providers.into_iter().nth(0) {
        Some(cdd_provider) => {
            let cdd_acc = get_primary_key(cdd_provider);
            let _ = Identity::cdd_register_did(Origin::signed(cdd_acc), id, vec![])
                .map_err(|_| "CDD register DID failed")?;
            let did = Identity::get_identity(&id).unwrap();
            let (cdd_id, _) = create_cdd_id(did, Ticker::default(), uid);
            let cdd_claim = Claim::CustomerDueDiligence(cdd_id);
            Identity::add_claim(Origin::signed(cdd_acc), did, cdd_claim, expiry)
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

pub fn add_trusted_cdd_provider(cdd_sp: IdentityId) {
    let root = Origin::from(frame_system::RawOrigin::Root);
    assert_ok!(Group::add_member(root, cdd_sp));
}

pub fn add_nominator_claim_with_expiry(
    _claim_issuer: IdentityId,
    identity_id: IdentityId,
    claim_issuer_account_id: AccountId,
    expiry: u64,
) {
    let signed_claim_issuer_id = Origin::signed(claim_issuer_account_id);
    let (cdd_id, _) = create_cdd_id_and_investor_uid(identity_id);
    assert_ok!(Identity::add_claim(
        signed_claim_issuer_id,
        identity_id,
        Claim::CustomerDueDiligence(cdd_id),
        Some(expiry.into()),
    ));
}

pub fn create_cdd_id_and_investor_uid(identity_id: IdentityId) -> (CddId, InvestorUid) {
    let uid = create_investor_uid(get_primary_key(identity_id));
    let (cdd_id, _) = create_cdd_id(identity_id, Ticker::default(), uid);
    (cdd_id, uid)
}

pub fn create_investor_uid(acc: AccountId) -> InvestorUid {
    InvestorUid::from(format!("{}", acc).as_str())
}

pub fn add_nominator_claim(
    _claim_issuer: IdentityId,
    identity_id: IdentityId,
    claim_issuer_account_id: AccountId,
) {
    let signed_claim_issuer_id = Origin::signed(claim_issuer_account_id);
    let now = Utc::now();
    let (cdd_id, _) = create_cdd_id_and_investor_uid(identity_id);
    assert_ok!(Identity::add_claim(
        signed_claim_issuer_id,
        identity_id,
        Claim::CustomerDueDiligence(cdd_id),
        Some((now.timestamp() as u64 + 10000_u64).into()),
    ));
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
    let (cdd_id, _) = create_cdd_id_and_investor_uid(did);
    assert_ok!(Identity::add_claim(
        Origin::signed(1005),
        did,
        Claim::CustomerDueDiligence(cdd_id),
        Some(expiry.into())
    ));
}

// `iter_prefix_values` has no guarantee that it will iterate in a sequential
// order. However, we need the latest `auth_id`. Which is why we search for the claim
// with the highest `auth_id`.
pub fn get_last_auth(signatory: &Signatory<AccountId>) -> Authorization<AccountId, u64> {
    <identity::Authorizations<Test>>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
}

pub fn get_last_auth_id(signatory: &Signatory<AccountId>) -> u64 {
    get_last_auth(signatory).auth_id
}

pub fn root() -> Origin {
    Origin::from(frame_system::RawOrigin::Root)
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
